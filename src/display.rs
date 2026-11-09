//! Presentación en consola: definición formal de la MT, descripciones
//! instantáneas, resultado de cada cadena y tabla resumen.
//!
//! Todas las funciones devuelven `String`; quien decide cuándo imprimir es `main.rs`.

use crate::machine::TuringMachine;
use crate::simulator::{Halt, InstantaneousDescription, SimulationResult};
use crate::tape::join_symbols;

/// Colores ANSI opcionales (se apagan con `--no-color` o la variable `NO_COLOR`).
#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub color: bool,
}

impl Style {
    fn paint(&self, code: &str, text: &str) -> String {
        if self.color {
            format!("\x1b[{code}m{text}\x1b[0m")
        } else {
            text.to_string()
        }
    }
    pub fn bold(&self, t: &str) -> String {
        self.paint("1", t)
    }
    pub fn dim(&self, t: &str) -> String {
        self.paint("2", t)
    }
    pub fn green(&self, t: &str) -> String {
        self.paint("1;32", t)
    }
    pub fn red(&self, t: &str) -> String {
        self.paint("1;31", t)
    }
    pub fn yellow(&self, t: &str) -> String {
        self.paint("33", t)
    }
    pub fn cyan(&self, t: &str) -> String {
        self.paint("36", t)
    }
}

fn set(items: &[String]) -> String {
    format!("{{{}}}", items.join(", "))
}

fn rule(title: &str) -> String {
    let width = 72usize.saturating_sub(title.chars().count() + 4);
    format!("── {title} {}", "─".repeat(width))
}

/// Descripción formal M = (Q, Σ, Γ, δ, q0, B, F) con todas sus transiciones.
pub fn machine_definition(m: &TuringMachine, style: &Style) -> String {
    let mut accept: Vec<String> = m
        .states
        .iter()
        .filter(|s| m.is_accepting(s))
        .cloned()
        .collect();
    if accept.is_empty() {
        accept = m.accept_states.iter().cloned().collect();
    }

    let mut out = String::new();
    out += &style.bold(&rule(&m.name));
    out += "\n";
    if let Some(desc) = &m.description {
        for line in desc.trim().lines() {
            out += &format!("  {}\n", line.trim());
        }
        out += "\n";
    }
    out += &format!("  Q  (estados)           = {}\n", set(&m.states));
    out += &format!("  Σ  (alfabeto entrada)  = {}\n", set(&m.input_alphabet));
    out += &format!("  Γ  (alfabeto cinta)    = {}\n", set(&m.tape_alphabet));
    out += &format!("  q0 (estado inicial)    = {}\n", m.initial_state);
    out += &format!("  B  (blanco)            = {}\n", m.blank);
    out += &format!("  F  (aceptación)        = {}\n", set(&accept));
    out += &format!(
        "  Control finito         = [estado, caché]{}\n",
        if m.uses_cache() {
            ""
        } else {
            " (esta MT no usa la caché)"
        }
    );
    out += &format!("\n  δ  ({} transiciones):\n", m.transitions.len());
    let digits = m.transitions.len().to_string().len();
    for (i, t) in m.transitions.iter().enumerate() {
        out += &format!("    {} {}\n", style.dim(&format!("{:>digits$}.", i + 1)), t);
    }
    out
}

/// Encabezado de la simulación de una cadena.
pub fn input_header(result: &SimulationResult, style: &Style) -> String {
    let shown = if result.input.is_empty() {
        "ε (cadena vacía)".to_string()
    } else {
        format!("\"{}\"", result.input)
    };
    style.bold(&rule(&format!("Entrada: {shown}")))
}

/// Una línea con la ID, en notación α[q, c]β, y la transición que la produjo.
pub fn id_line(m: &TuringMachine, id: &InstantaneousDescription, style: &Style) -> String {
    let mut beta = vec![id.tape.head.clone()];
    beta.extend(id.tape.right.iter().cloned());
    let control = format!("[{}, {}]", id.state, id.cache);
    let control = if m.is_accepting(&id.state) {
        style.green(&control)
    } else {
        style.cyan(&control)
    };
    let arrow = if id.step == 0 { " " } else { "⊢" };
    let via = match id.applied {
        Some(i) => style.dim(&format!("   (δ #{})", i + 1)),
        None => style.dim("   (ID inicial)"),
    };
    format!(
        "  {} {arrow} {}{}{}{via}",
        style.dim(&format!("{:>5}", id.step)),
        join_symbols(&id.tape.left),
        control,
        join_symbols(&beta),
    )
}

/// Bloque final: aceptada / rechazada, motivo y contenido de la cinta.
pub fn verdict(m: &TuringMachine, result: &SimulationResult, style: &Style) -> String {
    let tape = if result.final_tape.is_empty() {
        "(vacía, solo blancos)".to_string()
    } else {
        result.final_tape_text()
    };
    let (label, reason) = match &result.halt {
        Halt::Accepted => {
            let last = result.ids.last().map(|id| id.state.as_str()).unwrap_or("");
            (
                style.green("ACEPTADA ✔"),
                format!(
                    "la última ID contiene el estado de aceptación {last} ({} pasos)",
                    result.steps()
                ),
            )
        }
        Halt::NoTransition {
            state,
            cache,
            symbol,
        } => (
            style.red("RECHAZADA ✘"),
            format!(
                "no existe δ([{state}, {cache}], {symbol}) y {state} no es de aceptación ({} pasos)",
                result.steps()
            ),
        ),
        Halt::StepLimit(n) => (
            style.red("RECHAZADA ✘"),
            format!("la MT no se detuvo en {n} pasos (posible ciclo infinito)"),
        ),
        Halt::InvalidInput(bad) => (
            style.red("RECHAZADA ✘"),
            format!(
                "`{bad}` no pertenece al alfabeto de entrada {}",
                set(&m.input_alphabet)
            ),
        ),
    };
    format!(
        "\n  Resultado:   {label}\n  Motivo:      {reason}\n  Cinta final: {}\n",
        style.yellow(&tape)
    )
}

/// Tabla con una fila por cadena simulada.
pub fn summary(results: &[SimulationResult], style: &Style) -> String {
    let shown: Vec<String> = results
        .iter()
        .map(|r| {
            if r.input.is_empty() {
                "ε".to_string()
            } else {
                r.input.clone()
            }
        })
        .collect();
    let finals: Vec<String> = results
        .iter()
        .map(|r| {
            if r.final_tape.is_empty() {
                "(vacía)".to_string()
            } else {
                r.final_tape_text()
            }
        })
        .collect();
    let w_in = shown
        .iter()
        .map(|s| s.chars().count())
        .max()
        .unwrap_or(0)
        .max(7);
    let w_out = finals
        .iter()
        .map(|s| s.chars().count())
        .max()
        .unwrap_or(0)
        .max(11);

    let mut out = style.bold(&rule("Resumen"));
    out += "\n";
    out += &format!(
        "  {:<w_in$}  {:<11}  {:>6}  {:<w_out$}\n",
        "Entrada", "Resultado", "Pasos", "Cinta final"
    );
    out += &format!(
        "  {}  {}  {}  {}\n",
        "─".repeat(w_in),
        "─".repeat(11),
        "─".repeat(6),
        "─".repeat(w_out)
    );
    for ((r, input), tape) in results.iter().zip(&shown).zip(&finals) {
        // Se rellena antes de colorear para que los códigos ANSI no rompan la alineación.
        let label = if r.accepted() {
            style.green(&format!("{:<11}", "ACEPTADA"))
        } else {
            style.red(&format!("{:<11}", "RECHAZADA"))
        };
        out += &format!(
            "  {:<w_in$}  {label}  {:>6}  {:<w_out$}\n",
            input,
            r.steps(),
            tape
        );
    }
    out
}
