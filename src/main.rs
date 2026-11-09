//! Punto de entrada: lee los argumentos, carga la MT y simula cada cadena.

use std::path::PathBuf;
use std::process::ExitCode;
use turing_sim::config;
use turing_sim::diagram;
use turing_sim::display::{self, Style};
use turing_sim::machine::TuringMachine;
use turing_sim::simulator::{self, DEFAULT_MAX_STEPS, SimulationResult};

const HELP: &str = "\
Simulador de Máquinas de Turing de una cinta

USO:
    turing_sim <archivo.yaml> [opciones]

OPCIONES:
    -i, --input <cadena>   Simula esta cadena en lugar de las del YAML
                           (se puede repetir; use \"\" para la cadena vacía)
    -m, --max-steps <n>    Límite de pasos por cadena (por defecto 10000)
    -q, --quiet            No muestra las IDs, solo el resultado de cada cadena
    -d, --diagrama <ruta>  Exporta el diagrama de estados: .dot (Graphviz)
                           o .md/.mmd (Mermaid)
        --no-color         Desactiva los colores
    -h, --help             Muestra esta ayuda

EJEMPLO:
    turing_sim maquinas/reconocedora_multiplicacion.yaml -i aabbcccc
";

/// Opciones de la línea de comandos.
struct Options {
    file: PathBuf,
    inputs: Vec<String>,
    max_steps: usize,
    quiet: bool,
    color: bool,
    diagram: Option<PathBuf>,
}

fn parse_args(args: &[String]) -> Result<Options, String> {
    let mut file = None;
    let mut inputs = Vec::new();
    let mut max_steps = DEFAULT_MAX_STEPS;
    let mut quiet = false;
    let mut diagram = None;
    let mut color = std::env::var_os("NO_COLOR").is_none();

    let mut it = args.iter();
    while let Some(arg) = it.next() {
        let mut value = |flag: &str| {
            it.next()
                .cloned()
                .ok_or_else(|| format!("falta el valor de {flag}"))
        };
        match arg.as_str() {
            "-i" | "--input" => inputs.push(value(arg)?),
            "-m" | "--max-steps" => {
                let v = value(arg)?;
                max_steps = v
                    .parse()
                    .map_err(|_| format!("`{v}` no es un número de pasos válido"))?;
            }
            "-q" | "--quiet" => quiet = true,
            "-d" | "--diagrama" => diagram = Some(PathBuf::from(value(arg)?)),
            "--no-color" => color = false,
            "-h" | "--help" => return Err(String::new()),
            other if other.starts_with('-') && other.len() > 1 => {
                return Err(format!("opción desconocida `{other}`"));
            }
            other => {
                if file.replace(PathBuf::from(other)).is_some() {
                    return Err("solo se puede indicar un archivo YAML".to_string());
                }
            }
        }
    }

    Ok(Options {
        file: file.ok_or("falta el archivo YAML con la MT")?,
        inputs,
        max_steps,
        quiet,
        color,
        diagram,
    })
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let opts = match parse_args(&args) {
        Ok(opts) => opts,
        Err(msg) if msg.is_empty() => {
            print!("{HELP}");
            return ExitCode::SUCCESS;
        }
        Err(msg) => {
            eprintln!("error: {msg}\n\n{HELP}");
            return ExitCode::FAILURE;
        }
    };
    let style = Style { color: opts.color };

    // 1. Parsing del YAML y construcción/validación de la MT.
    let machine = match config::load_file(&opts.file)
        .map_err(|e| e.to_string())
        .and_then(|raw| TuringMachine::from_raw(raw).map_err(|e| e.to_string()))
    {
        Ok(machine) => machine,
        Err(msg) => {
            eprintln!("{} {msg}", style.red("error:"));
            return ExitCode::FAILURE;
        }
    };

    println!("{}", display::machine_definition(&machine, &style));

    if let Some(path) = &opts.diagram {
        let is_mermaid = matches!(
            path.extension().and_then(|e| e.to_str()),
            Some("md" | "mmd")
        );
        let text = if is_mermaid {
            diagram::to_mermaid(&machine)
        } else {
            diagram::to_dot(&machine)
        };
        if let Err(e) = std::fs::write(path, text) {
            eprintln!(
                "{} no se pudo escribir {}: {e}",
                style.red("error:"),
                path.display()
            );
            return ExitCode::FAILURE;
        }
        println!("Diagrama de estados guardado en {}\n", path.display());
    }

    let inputs = if opts.inputs.is_empty() {
        machine.inputs.clone()
    } else {
        opts.inputs.clone()
    };
    if inputs.is_empty() {
        println!("No hay cadenas para simular: agréguelas en `inputs` o use --input.");
        return ExitCode::SUCCESS;
    }

    // 2. Simulación de cada cadena.
    let mut results: Vec<SimulationResult> = Vec::new();
    for input in &inputs {
        let result = simulator::run(&machine, input, opts.max_steps);
        println!("{}", display::input_header(&result, &style));
        if !opts.quiet {
            for id in &result.ids {
                println!("{}", display::id_line(&machine, id, &style));
            }
        }
        println!("{}", display::verdict(&machine, &result, &style));
        results.push(result);
    }

    // 3. Resumen.
    if results.len() > 1 {
        println!("{}", display::summary(&results, &style));
    }
    ExitCode::SUCCESS
}
