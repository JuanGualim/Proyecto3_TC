//! Lectura del archivo YAML.
//!
//! Este módulo solo se encarga de convertir el texto YAML en estructuras de Rust
//! ("configuración cruda"). No revisa si la máquina tiene sentido; eso lo hace
//! `machine.rs` al construir la `TuringMachine`.
//!
//! Formato esperado (el del enunciado del proyecto):
//!
//! ```yaml
//! mt:
//!   states: [q0, q1, qf]
//!   input_alphabet: [a, b]
//!   tape_alphabet: [a, b, B, X]
//!   initial_state: q0
//!   accept_states: [qf]
//!   transitions:
//!     - state: q0
//!       read: [a, B]      # [símbolo leído en la cinta, caché actual]
//!       write: [a, X]     # [caché nueva, símbolo que se escribe en la cinta]
//!       move: R
//!       next: q1
//!   inputs:
//!     - "aabb"
//! ```

use serde::de::{self, Deserializer, Visitor};
use serde::Deserialize;
use std::fmt;
use std::fs;
use std::path::Path;

/// Un valor escalar del YAML guardado siempre como texto.
///
/// YAML interpreta `0`, `1` o `true` como números/booleanos; para la MT todos los
/// símbolos y estados son texto, así que aceptamos cualquier escalar y lo
/// convertimos a `String`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scalar(pub String);

impl<'de> Deserialize<'de> for Scalar {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ScalarVisitor;

        impl Visitor<'_> for ScalarVisitor {
            type Value = Scalar;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("un símbolo o nombre (texto, número o booleano)")
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<Scalar, E> {
                Ok(Scalar(v.to_string()))
            }
            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Scalar, E> {
                Ok(Scalar(v.to_string()))
            }
            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Scalar, E> {
                Ok(Scalar(v.to_string()))
            }
            fn visit_f64<E: de::Error>(self, v: f64) -> Result<Scalar, E> {
                Ok(Scalar(v.to_string()))
            }
            fn visit_bool<E: de::Error>(self, v: bool) -> Result<Scalar, E> {
                Ok(Scalar(v.to_string()))
            }
        }

        deserializer.deserialize_any(ScalarVisitor)
    }
}

/// Raíz del documento: todo cuelga de la llave `mt`.
#[derive(Debug, Deserialize)]
pub struct Document {
    pub mt: RawMachine,
}

/// Descripción formal de la MT tal como viene en el archivo.
#[derive(Debug, Deserialize)]
pub struct RawMachine {
    /// Nombre y descripción son opcionales, solo se usan para mostrar.
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    pub states: Vec<Scalar>,
    pub input_alphabet: Vec<Scalar>,
    pub tape_alphabet: Vec<Scalar>,
    pub initial_state: Scalar,
    pub accept_states: Vec<Scalar>,
    /// Símbolo blanco. Si no se indica se usa `B`, como en el enunciado.
    #[serde(default)]
    pub blank: Option<Scalar>,
    pub transitions: Vec<RawTransition>,
    #[serde(default)]
    pub inputs: Vec<Scalar>,
}

/// Una función de transición δ([state, caché], símbolo) = ([next, caché'], escribir, move).
#[derive(Debug, Deserialize)]
pub struct RawTransition {
    pub state: Scalar,
    pub read: SymbolPair,
    pub write: SymbolPair,
    #[serde(rename = "move")]
    pub movement: Scalar,
    pub next: Scalar,
}

/// `read` y `write` normalmente son una lista de dos elementos (`[a, B]`), pero
/// también se permite un solo símbolo para máquinas que no usan caché.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum SymbolPair {
    Pair(Vec<Scalar>),
    Single(Scalar),
}

/// Error al leer o interpretar el archivo.
#[derive(Debug)]
pub enum ConfigError {
    Io(String),
    Yaml(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::Io(msg) => write!(f, "no se pudo leer el archivo: {msg}"),
            ConfigError::Yaml(msg) => write!(f, "YAML inválido: {msg}"),
        }
    }
}

impl std::error::Error for ConfigError {}

/// Interpreta un texto YAML.
pub fn parse_str(text: &str) -> Result<RawMachine, ConfigError> {
    serde_yaml::from_str::<Document>(text)
        .map(|doc| doc.mt)
        .map_err(|e| ConfigError::Yaml(e.to_string()))
}

/// Lee e interpreta un archivo YAML.
pub fn load_file(path: &Path) -> Result<RawMachine, ConfigError> {
    let text = fs::read_to_string(path)
        .map_err(|e| ConfigError::Io(format!("{}: {e}", path.display())))?;
    parse_str(&text)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EJEMPLO: &str = r#"
mt:
  states: [q0, q1, q2, qf]
  input_alphabet: [a, b]
  tape_alphabet: [a, b, B, X, Y]
  initial_state: q0
  accept_states: [qf]
  transitions:
    - state: q0
      read: [a, B]
      write: [a, B]
      move: R
      next: q1
    - state: q0
      read: [b, B]
      write: [b, B]
      move: R
      next: q1
  inputs:
    - "aabb"
    - "ab"
    - "aaabbb"
"#;

    #[test]
    fn lee_el_ejemplo_del_enunciado() {
        let raw = parse_str(EJEMPLO).unwrap();
        assert_eq!(raw.states.len(), 4);
        assert_eq!(raw.initial_state, Scalar("q0".into()));
        assert_eq!(raw.transitions.len(), 2);
        assert_eq!(raw.inputs.len(), 3);
        assert!(raw.blank.is_none());
        match &raw.transitions[0].read {
            SymbolPair::Pair(v) => assert_eq!(v, &[Scalar("a".into()), Scalar("B".into())]),
            SymbolPair::Single(_) => panic!("se esperaba un par"),
        }
    }

    #[test]
    fn acepta_simbolos_numericos() {
        let raw = parse_str(
            "mt:\n  states: [q0]\n  input_alphabet: [0, 1]\n  tape_alphabet: [0, 1, B]\n  \
             initial_state: q0\n  accept_states: [q0]\n  transitions: []\n  inputs: [101, \"\"]\n",
        )
        .unwrap();
        assert_eq!(raw.input_alphabet, vec![Scalar("0".into()), Scalar("1".into())]);
        assert_eq!(raw.inputs[0], Scalar("101".into()));
        assert_eq!(raw.inputs[1], Scalar("".into()));
    }

    #[test]
    fn reporta_yaml_invalido() {
        assert!(matches!(parse_str("mt: [esto no es"), Err(ConfigError::Yaml(_))));
    }
}
