//! Estructura de datos de la Máquina de Turing.
//!
//! A partir de la configuración cruda (`config::RawMachine`) se construye una
//! `TuringMachine` validada. La función de transición se guarda en un `HashMap`
//! cuya llave es `(estado, caché, símbolo leído)`, así encontrar la transición a
//! aplicar en cada paso cuesta O(1).
//!
//! Igual que en clase, el control finito es una tupla `[estado, caché]`: la caché
//! permite "recordar" un símbolo de la cinta sin multiplicar los estados. Una
//! máquina que no la necesite simplemente deja la caché siempre en blanco.

use crate::config::{RawMachine, RawTransition, Scalar, SymbolPair};
use crate::tape::{Move, Symbol};
use std::collections::{HashMap, HashSet};
use std::fmt;

/// Lado izquierdo de δ: `δ([state, cache], symbol)`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TransitionKey {
    pub state: String,
    pub cache: Symbol,
    pub symbol: Symbol,
}

/// Lado derecho de δ: `([next, new_cache], write, movement)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Action {
    pub next: String,
    pub new_cache: Symbol,
    pub write: Symbol,
    pub movement: Move,
}

/// Una transición completa, en el orden en que aparece en el archivo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transition {
    pub key: TransitionKey,
    pub action: Action,
}

impl fmt::Display for Transition {
    /// Notación del curso: δ([q0, B], a) = ([q1, a], B, R)
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "δ([{}, {}], {}) = ([{}, {}], {}, {})",
            self.key.state,
            self.key.cache,
            self.key.symbol,
            self.action.next,
            self.action.new_cache,
            self.action.write,
            self.action.movement
        )
    }
}

/// Máquina de Turing de una cinta: M = (Q, Σ, Γ, δ, q0, B, F).
#[derive(Debug, Clone)]
pub struct TuringMachine {
    pub name: String,
    pub description: Option<String>,
    pub states: Vec<String>,
    pub input_alphabet: Vec<Symbol>,
    pub tape_alphabet: Vec<Symbol>,
    pub blank: Symbol,
    pub initial_state: String,
    pub accept_states: HashSet<String>,
    /// Transiciones en el orden del archivo (para mostrarlas y dibujarlas).
    pub transitions: Vec<Transition>,
    /// Índice de búsqueda: llave de δ → posición en `transitions`.
    index: HashMap<TransitionKey, usize>,
    /// Cadenas a simular listadas en el YAML.
    pub inputs: Vec<String>,
}

/// Todos los problemas encontrados al validar la máquina. Se reportan juntos
/// para que el usuario pueda corregir el YAML de una sola vez.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub problems: Vec<String>,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "la definición de la MT tiene {} problema(s):",
            self.problems.len()
        )?;
        for problem in &self.problems {
            writeln!(f, "  - {problem}")?;
        }
        Ok(())
    }
}

impl std::error::Error for ValidationError {}

const DEFAULT_BLANK: &str = "B";

fn texts(scalars: &[Scalar]) -> Vec<String> {
    scalars.iter().map(|s| s.0.trim().to_string()).collect()
}

fn duplicates(items: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut repeated = Vec::new();
    for item in items {
        if !seen.insert(item) && !repeated.contains(item) {
            repeated.push(item.clone());
        }
    }
    repeated
}

impl TuringMachine {
    /// Construye y valida la máquina.
    pub fn from_raw(raw: RawMachine) -> Result<TuringMachine, ValidationError> {
        let mut problems = Vec::new();

        let states = texts(&raw.states);
        let input_alphabet = texts(&raw.input_alphabet);
        let tape_alphabet = texts(&raw.tape_alphabet);
        let accept: Vec<String> = texts(&raw.accept_states);
        let initial_state = raw.initial_state.0.trim().to_string();
        let blank = raw
            .blank
            .map(|b| b.0.trim().to_string())
            .unwrap_or_else(|| DEFAULT_BLANK.to_string());

        let state_set: HashSet<&String> = states.iter().collect();
        let tape_set: HashSet<&String> = tape_alphabet.iter().collect();

        // --- Conjuntos ---------------------------------------------------------
        if states.is_empty() {
            problems.push("`states` no puede estar vacío".to_string());
        }
        for (name, list) in [
            ("states", &states),
            ("input_alphabet", &input_alphabet),
            ("tape_alphabet", &tape_alphabet),
        ] {
            for dup in duplicates(list) {
                problems.push(format!("`{dup}` está repetido en `{name}`"));
            }
            if list.iter().any(|s| s.is_empty()) {
                problems.push(format!("`{name}` contiene un elemento vacío"));
            }
        }
        if !state_set.contains(&initial_state) {
            problems.push(format!(
                "el estado inicial `{initial_state}` no está en `states`"
            ));
        }
        for state in &accept {
            if !state_set.contains(state) {
                problems.push(format!(
                    "el estado de aceptación `{state}` no está en `states`"
                ));
            }
        }
        for symbol in &input_alphabet {
            if !tape_set.contains(symbol) {
                problems.push(format!(
                    "el símbolo de entrada `{symbol}` no está en `tape_alphabet` (Σ ⊆ Γ)"
                ));
            }
        }
        if !tape_set.contains(&blank) {
            problems.push(format!("el blanco `{blank}` debe estar en `tape_alphabet`"));
        }
        if input_alphabet.contains(&blank) {
            problems.push(format!(
                "el blanco `{blank}` no puede estar en `input_alphabet`"
            ));
        }

        // --- Transiciones ------------------------------------------------------
        let mut transitions = Vec::new();
        let mut index = HashMap::new();
        for (i, raw_t) in raw.transitions.iter().enumerate() {
            let number = i + 1;
            match build_transition(raw_t, &blank) {
                Ok(t) => {
                    let mut check_state = |s: &String, field: &str| {
                        if !state_set.contains(s) {
                            problems.push(format!(
                                "transición #{number}: `{field}: {s}` no está en `states`"
                            ));
                        }
                    };
                    check_state(&t.key.state, "state");
                    check_state(&t.action.next, "next");

                    for (s, what) in [
                        (&t.key.symbol, "símbolo leído"),
                        (&t.key.cache, "caché leída"),
                        (&t.action.new_cache, "caché escrita"),
                        (&t.action.write, "símbolo escrito"),
                    ] {
                        if !tape_set.contains(s) {
                            problems.push(format!(
                                "transición #{number}: {what} `{s}` no está en `tape_alphabet`"
                            ));
                        }
                    }

                    // Determinismo: una sola transición por (estado, caché, símbolo).
                    if let Some(&previous) = index.get(&t.key) {
                        problems.push(format!(
                            "transiciones #{} y #{number} tienen el mismo lado izquierdo \
                             δ([{}, {}], {}): la MT debe ser determinista",
                            previous + 1,
                            t.key.state,
                            t.key.cache,
                            t.key.symbol
                        ));
                    } else {
                        index.insert(t.key.clone(), transitions.len());
                    }
                    transitions.push(t);
                }
                Err(msg) => problems.push(format!("transición #{number}: {msg}")),
            }
        }

        if !problems.is_empty() {
            return Err(ValidationError { problems });
        }

        Ok(TuringMachine {
            name: raw.name.unwrap_or_else(|| "Máquina de Turing".to_string()),
            description: raw.description,
            states,
            input_alphabet,
            tape_alphabet,
            blank,
            initial_state,
            accept_states: accept.into_iter().collect(),
            transitions,
            index,
            inputs: texts(&raw.inputs),
        })
    }

    /// Busca δ([state, cache], symbol). `None` significa que la máquina se detiene.
    pub fn lookup(&self, state: &str, cache: &str, symbol: &str) -> Option<&Transition> {
        let key = TransitionKey {
            state: state.to_string(),
            cache: cache.to_string(),
            symbol: symbol.to_string(),
        };
        self.index.get(&key).map(|&i| &self.transitions[i])
    }

    /// Posición de una transición dentro de `transitions` (para numerarla al mostrarla).
    pub fn position_of(&self, transition: &Transition) -> Option<usize> {
        self.index.get(&transition.key).copied()
    }

    pub fn is_accepting(&self, state: &str) -> bool {
        self.accept_states.contains(state)
    }

    /// ¿La máquina usa la caché en alguna transición?
    pub fn uses_cache(&self) -> bool {
        self.transitions
            .iter()
            .any(|t| t.key.cache != self.blank || t.action.new_cache != self.blank)
    }
}

/// Convierte `read`/`write` del YAML en (símbolo, caché).
///
/// - `read: [símbolo, caché]` (formato del enunciado) o `read: símbolo` (caché = blanco).
/// - `write: [caché nueva, símbolo]` o `write: símbolo` (caché nueva = blanco).
fn build_transition(raw: &RawTransition, blank: &str) -> Result<Transition, String> {
    let (symbol, cache) = match &raw.read {
        SymbolPair::Pair(v) if v.len() == 2 => (v[0].0.trim(), v[1].0.trim()),
        SymbolPair::Single(s) => (s.0.trim(), blank),
        SymbolPair::Pair(v) => {
            return Err(format!(
                "`read` debe tener 2 elementos [símbolo, caché], tiene {}",
                v.len()
            ));
        }
    };
    let (new_cache, write) = match &raw.write {
        SymbolPair::Pair(v) if v.len() == 2 => (v[0].0.trim(), v[1].0.trim()),
        SymbolPair::Single(s) => (blank, s.0.trim()),
        SymbolPair::Pair(v) => {
            return Err(format!(
                "`write` debe tener 2 elementos [caché, símbolo], tiene {}",
                v.len()
            ));
        }
    };
    let movement = Move::parse(&raw.movement.0)
        .ok_or_else(|| format!("movimiento `{}` inválido (use L, R o S)", raw.movement.0))?;

    Ok(Transition {
        key: TransitionKey {
            state: raw.state.0.trim().to_string(),
            cache: cache.to_string(),
            symbol: symbol.to_string(),
        },
        action: Action {
            next: raw.next.0.trim().to_string(),
            new_cache: new_cache.to_string(),
            write: write.to_string(),
            movement,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::parse_str;

    fn build(yaml: &str) -> Result<TuringMachine, ValidationError> {
        TuringMachine::from_raw(parse_str(yaml).unwrap())
    }

    const BASE: &str = "mt:
  states: [q0, q1, qf]
  input_alphabet: [a, b]
  tape_alphabet: [a, b, B, X]
  initial_state: q0
  accept_states: [qf]
  transitions:
    - {state: q0, read: [a, B], write: [a, X], move: R, next: q1}
    - {state: q1, read: b, write: X, move: L, next: qf}
";

    #[test]
    fn construye_la_funcion_de_transicion() {
        let m = build(BASE).unwrap();
        let t = m.lookup("q0", "B", "a").unwrap();
        assert_eq!(t.action.next, "q1");
        assert_eq!(t.action.new_cache, "a");
        assert_eq!(t.action.write, "X");
        assert_eq!(t.action.movement, Move::Right);
        assert_eq!(t.to_string(), "δ([q0, B], a) = ([q1, a], X, R)");

        // Formato corto: caché en blanco.
        let t = m.lookup("q1", "B", "b").unwrap();
        assert_eq!(t.action.new_cache, "B");
        assert!(m.lookup("q1", "B", "a").is_none());
        assert!(m.uses_cache());
    }

    #[test]
    fn detecta_no_determinismo() {
        let yaml =
            format!("{BASE}    - {{state: q0, read: [a, B], write: [b, B], move: L, next: q0}}\n");
        let err = build(&yaml).unwrap_err();
        assert!(err.problems[0].contains("determinista"), "{err}");
    }

    #[test]
    fn reporta_todos_los_problemas_juntos() {
        let yaml = BASE
            .replace("initial_state: q0", "initial_state: q9")
            .replace("next: qf", "next: qz")
            .replace("move: L", "move: Z")
            .replace("write: [a, X]", "write: [a, Y]");
        let err = build(&yaml).unwrap_err();
        let all = err.to_string();
        assert!(all.contains("estado inicial `q9`"), "{all}");
        assert!(all.contains("movimiento `Z`"), "{all}");
        assert!(all.contains("`Y` no está en `tape_alphabet`"), "{all}");
    }

    #[test]
    fn el_blanco_debe_estar_en_la_cinta_y_no_en_la_entrada() {
        let yaml = BASE.replace("tape_alphabet: [a, b, B, X]", "tape_alphabet: [a, b, X]");
        assert!(build(&yaml).unwrap_err().to_string().contains("blanco `B`"));
        let yaml = BASE.replace("input_alphabet: [a, b]", "input_alphabet: [a, b, B]");
        assert!(
            build(&yaml)
                .unwrap_err()
                .to_string()
                .contains("no puede estar en `input_alphabet`")
        );
    }
}
