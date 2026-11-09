//! Exporta el diagrama de estados de la MT.
//!
//! - Graphviz (`.dot`): se puede convertir a imagen con `dot -Tsvg mt.dot -o mt.svg`
//!   o pegar en https://dreampuf.github.io/GraphvizOnline
//! - Mermaid (`.md` / `.mmd`): GitHub lo dibuja directamente dentro de un README.
//!
//! Las transiciones entre el mismo par de estados se agrupan en una sola flecha.
//! Cada etiqueta usa la notación `leído, caché → caché', escrito, movimiento`;
//! si la MT no usa la caché se omite: `leído → escrito, movimiento`.

use crate::machine::{Transition, TuringMachine};

/// Etiqueta de una transición sobre una flecha.
fn label(m: &TuringMachine, t: &Transition) -> String {
    if m.uses_cache() {
        format!(
            "{}, {} → {}, {}, {}",
            t.key.symbol, t.key.cache, t.action.new_cache, t.action.write, t.action.movement
        )
    } else {
        format!(
            "{} → {}, {}",
            t.key.symbol, t.action.write, t.action.movement
        )
    }
}

/// Agrupa las transiciones por (origen, destino) respetando el orden del archivo.
fn edges(m: &TuringMachine) -> Vec<(&str, &str, Vec<String>)> {
    let mut grouped: Vec<(&str, &str, Vec<String>)> = Vec::new();
    for t in &m.transitions {
        let (from, to) = (t.key.state.as_str(), t.action.next.as_str());
        match grouped.iter_mut().find(|(f, d, _)| *f == from && *d == to) {
            Some((_, _, labels)) => labels.push(label(m, t)),
            None => grouped.push((from, to, vec![label(m, t)])),
        }
    }
    grouped
}

fn escape_dot(text: &str) -> String {
    text.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Diagrama en formato Graphviz DOT.
pub fn to_dot(m: &TuringMachine) -> String {
    let mut out = String::new();
    out += "digraph MT {\n";
    out += "  rankdir=LR;\n";
    out += &format!("  label=\"{}\";\n  labelloc=t;\n", escape_dot(&m.name));
    out += "  node [shape=circle, fontname=\"Helvetica\"];\n";
    out += "  edge [fontname=\"Helvetica\", fontsize=10];\n";
    out += "  inicio [shape=point];\n";
    for state in &m.states {
        let shape = if m.is_accepting(state) {
            "doublecircle"
        } else {
            "circle"
        };
        out += &format!("  \"{}\" [shape={shape}];\n", escape_dot(state));
    }
    out += &format!("  inicio -> \"{}\";\n", escape_dot(&m.initial_state));
    for (from, to, labels) in edges(m) {
        out += &format!(
            "  \"{}\" -> \"{}\" [label=\"{}\"];\n",
            escape_dot(from),
            escape_dot(to),
            escape_dot(&labels.join("\n"))
        );
    }
    out += "}\n";
    out
}

/// Identificador seguro para Mermaid (solo letras, números y `_`).
fn mermaid_id(state: &str) -> String {
    let clean: String = state
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    format!("s_{clean}")
}

fn escape_mermaid(text: &str) -> String {
    text.replace('"', "#quot;")
}

/// Diagrama en formato Mermaid (flowchart), listo para pegar en Markdown.
pub fn to_mermaid(m: &TuringMachine) -> String {
    let mut out = String::new();
    out += "```mermaid\nflowchart LR\n";
    out += "  inicio([inicio])\n";
    for state in &m.states {
        let id = mermaid_id(state);
        let name = escape_mermaid(state);
        if m.is_accepting(state) {
            out += &format!("  {id}(((\"{name}\")))\n");
        } else {
            out += &format!("  {id}((\"{name}\"))\n");
        }
    }
    out += &format!("  inicio --> {}\n", mermaid_id(&m.initial_state));
    for (from, to, labels) in edges(m) {
        out += &format!(
            "  {} -->|\"{}\"| {}\n",
            mermaid_id(from),
            escape_mermaid(&labels.join("<br/>")),
            mermaid_id(to)
        );
    }
    out += "```\n";
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::parse_str;

    fn machine() -> TuringMachine {
        TuringMachine::from_raw(
            parse_str(
                "mt:
  states: [q0, qf]
  input_alphabet: [a]
  tape_alphabet: [a, B]
  initial_state: q0
  accept_states: [qf]
  transitions:
    - {state: q0, read: a, write: a, move: R, next: q0}
    - {state: q0, read: B, write: B, move: S, next: qf}
",
            )
            .unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn dot_tiene_estados_y_flechas() {
        let dot = to_dot(&machine());
        assert!(dot.contains("\"qf\" [shape=doublecircle]"));
        assert!(dot.contains("inicio -> \"q0\""));
        assert!(dot.contains("\"q0\" -> \"q0\" [label=\"a → a, R\"]"));
    }

    #[test]
    fn mermaid_agrupa_y_marca_aceptacion() {
        let mmd = to_mermaid(&machine());
        assert!(mmd.starts_with("```mermaid"));
        assert!(mmd.contains("s_qf(((\"qf\")))"));
        assert!(mmd.contains("s_q0 -->|\"B → B, S\"| s_qf"));
    }
}
