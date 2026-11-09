//! Motor de simulación.
//!
//! Una configuración de la MT es `[estado, caché]` + la cinta + la posición de la
//! cabeza. En cada paso se busca δ([estado, caché], símbolo bajo la cabeza) y se
//! aplica: cambia estado y caché, escribe en la cinta y mueve la cabeza. Después
//! de cada paso se guarda una descripción instantánea (ID).
//!
//! La máquina se detiene cuando:
//! - entra a un estado de aceptación → la cadena es **aceptada**;
//! - no existe transición para la configuración actual → **rechazada**;
//! - se alcanza el límite de pasos → se reporta como posible ciclo infinito.

use crate::machine::TuringMachine;
use crate::tape::{Symbol, Tape, TapeView, join_symbols, tokenize};
use std::fmt;

/// Límite de pasos por defecto para no colgarse con máquinas que no se detienen.
pub const DEFAULT_MAX_STEPS: usize = 10_000;

/// Descripción instantánea: `α [q, c] β`, donde α es la cinta a la izquierda
/// de la cabeza y β empieza con el símbolo que se está leyendo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstantaneousDescription {
    pub step: usize,
    pub state: String,
    pub cache: Symbol,
    pub tape: TapeView,
    /// Índice (en `machine.transitions`) de la transición que produjo esta ID.
    /// Es `None` para la ID inicial.
    pub applied: Option<usize>,
}

impl fmt::Display for InstantaneousDescription {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut beta = vec![self.tape.head.clone()];
        beta.extend(self.tape.right.iter().cloned());
        write!(
            f,
            "{}[{}, {}]{}",
            join_symbols(&self.tape.left),
            self.state,
            self.cache,
            join_symbols(&beta)
        )
    }
}

/// Motivo por el que terminó la simulación.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Halt {
    /// Se llegó a un estado de aceptación.
    Accepted,
    /// No hay δ definida para la configuración actual.
    NoTransition {
        state: String,
        cache: Symbol,
        symbol: Symbol,
    },
    /// Se alcanzó el límite de pasos sin detenerse.
    StepLimit(usize),
    /// La cadena contiene algo que no está en el alfabeto de entrada.
    InvalidInput(String),
}

/// Resultado completo de simular una cadena.
#[derive(Debug, Clone)]
pub struct SimulationResult {
    pub input: String,
    pub ids: Vec<InstantaneousDescription>,
    pub halt: Halt,
    /// Contenido de la cinta al terminar (sin blancos de los extremos).
    pub final_tape: Vec<Symbol>,
}

impl SimulationResult {
    pub fn accepted(&self) -> bool {
        self.halt == Halt::Accepted
    }

    /// Número de transiciones aplicadas.
    pub fn steps(&self) -> usize {
        self.ids.len().saturating_sub(1)
    }

    pub fn final_tape_text(&self) -> String {
        join_symbols(&self.final_tape)
    }
}

/// Estado de una ejecución en curso. Permite avanzar paso por paso.
pub struct Simulator<'m> {
    machine: &'m TuringMachine,
    state: String,
    cache: Symbol,
    tape: Tape,
    steps: usize,
}

/// Lo que ocurrió al intentar dar un paso.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    /// Se aplicó la transición con este índice.
    Moved(usize),
    /// La máquina ya se detuvo.
    Halted(Halt),
}

impl<'m> Simulator<'m> {
    /// Configuración inicial: estado q0, caché en blanco, cabeza en el primer símbolo.
    pub fn new(machine: &'m TuringMachine, input: &[Symbol]) -> Simulator<'m> {
        Simulator {
            machine,
            state: machine.initial_state.clone(),
            cache: machine.blank.clone(),
            tape: Tape::new(input, &machine.blank),
            steps: 0,
        }
    }

    /// ID de la configuración actual.
    pub fn snapshot(&self, applied: Option<usize>) -> InstantaneousDescription {
        InstantaneousDescription {
            step: self.steps,
            state: self.state.clone(),
            cache: self.cache.clone(),
            tape: self.tape.view(),
            applied,
        }
    }

    /// Aplica una transición, o indica por qué la máquina está detenida.
    pub fn step(&mut self) -> Step {
        if self.machine.is_accepting(&self.state) {
            return Step::Halted(Halt::Accepted);
        }

        let symbol = self.tape.read().clone();
        let Some(transition) = self.machine.lookup(&self.state, &self.cache, &symbol) else {
            return Step::Halted(Halt::NoTransition {
                state: self.state.clone(),
                cache: self.cache.clone(),
                symbol,
            });
        };

        let action = &transition.action;
        self.tape.write(&action.write);
        self.tape.shift(action.movement);
        self.state = action.next.clone();
        self.cache = action.new_cache.clone();
        self.steps += 1;

        let index = self
            .machine
            .position_of(transition)
            .expect("toda transición encontrada está indexada");
        Step::Moved(index)
    }

    pub fn steps(&self) -> usize {
        self.steps
    }

    pub fn tape(&self) -> &Tape {
        &self.tape
    }
}

/// Simula una cadena (tal como viene en el YAML) hasta que la máquina se detenga
/// o se alcance `max_steps`.
pub fn run(machine: &TuringMachine, input: &str, max_steps: usize) -> SimulationResult {
    let symbols = match tokenize(input, &machine.input_alphabet) {
        Ok(symbols) => symbols,
        Err(bad) => {
            return SimulationResult {
                input: input.to_string(),
                ids: Vec::new(),
                halt: Halt::InvalidInput(bad),
                final_tape: Vec::new(),
            };
        }
    };

    let mut sim = Simulator::new(machine, &symbols);
    let mut ids = vec![sim.snapshot(None)];

    let halt = loop {
        if sim.steps() >= max_steps && !machine.is_accepting(&sim.state) {
            break Halt::StepLimit(max_steps);
        }
        match sim.step() {
            Step::Moved(index) => ids.push(sim.snapshot(Some(index))),
            Step::Halted(halt) => break halt,
        }
    };

    SimulationResult {
        input: input.to_string(),
        ids,
        halt,
        final_tape: sim.tape().contents(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::parse_str;

    /// Ejercicio 2 del laboratorio 9: suma 1 a un número binario
    /// (aceptando en qhalt).
    const INCREMENTO: &str = "mt:
  states: [q0, q1, qhalt]
  input_alphabet: [0, 1]
  tape_alphabet: [0, 1, B]
  initial_state: q0
  accept_states: [qhalt]
  transitions:
    - {state: q0, read: 0, write: 0, move: R, next: q0}
    - {state: q0, read: 1, write: 1, move: R, next: q0}
    - {state: q0, read: B, write: B, move: L, next: q1}
    - {state: q1, read: 0, write: 1, move: R, next: qhalt}
    - {state: q1, read: 1, write: 0, move: L, next: q1}
    - {state: q1, read: B, write: B, move: L, next: qhalt}
";

    fn machine(yaml: &str) -> TuringMachine {
        TuringMachine::from_raw(parse_str(yaml).unwrap()).unwrap()
    }

    #[test]
    fn genera_las_ids_del_laboratorio_9() {
        let m = machine(INCREMENTO);
        let r = run(&m, "100", 100);
        let ids: Vec<String> = r.ids.iter().map(|id| id.to_string()).collect();
        assert_eq!(
            ids,
            [
                "[q0, B]100",
                "1[q0, B]00",
                "10[q0, B]0",
                "100[q0, B]B",
                "10[q1, B]0",
                "101[qhalt, B]B",
            ]
        );
        assert!(r.accepted());
        assert_eq!(r.final_tape_text(), "101");
        assert_eq!(r.steps(), 5);
    }

    #[test]
    fn rechaza_si_no_hay_transicion() {
        let m = machine(&INCREMENTO.replace(
            "    - {state: q1, read: 0, write: 1, move: R, next: qhalt}\n",
            "",
        ));
        let r = run(&m, "10", 100);
        assert_eq!(
            r.halt,
            Halt::NoTransition {
                state: "q1".into(),
                cache: "B".into(),
                symbol: "0".into()
            }
        );
        assert!(!r.accepted());
    }

    #[test]
    fn detecta_limite_de_pasos() {
        // q0 se mueve a la derecha para siempre sobre blancos.
        let m = machine(&INCREMENTO.replace(
            "{state: q0, read: B, write: B, move: L, next: q1}",
            "{state: q0, read: B, write: B, move: R, next: q0}",
        ));
        let r = run(&m, "1", 50);
        assert_eq!(r.halt, Halt::StepLimit(50));
        assert_eq!(r.steps(), 50);
    }

    #[test]
    fn reporta_simbolos_fuera_del_alfabeto() {
        let m = machine(INCREMENTO);
        assert_eq!(run(&m, "102", 100).halt, Halt::InvalidInput("2".into()));
    }

    #[test]
    fn la_cache_viaja_con_el_estado() {
        // Copia el primer símbolo al final de la cadena usando la caché.
        let m = machine(
            "mt:
  states: [q0, q1, qf]
  input_alphabet: [a, b]
  tape_alphabet: [a, b, B]
  initial_state: q0
  accept_states: [qf]
  transitions:
    - {state: q0, read: [a, B], write: [a, a], move: R, next: q1}
    - {state: q0, read: [b, B], write: [b, b], move: R, next: q1}
    - {state: q1, read: [a, a], write: [a, a], move: R, next: q1}
    - {state: q1, read: [b, a], write: [a, b], move: R, next: q1}
    - {state: q1, read: [a, b], write: [b, a], move: R, next: q1}
    - {state: q1, read: [b, b], write: [b, b], move: R, next: q1}
    - {state: q1, read: [B, a], write: [B, a], move: S, next: qf}
    - {state: q1, read: [B, b], write: [B, b], move: S, next: qf}
",
        );
        let r = run(&m, "bab", 100);
        assert!(r.accepted());
        assert_eq!(r.final_tape_text(), "babb");
        assert_eq!(r.ids[1].to_string(), "b[q1, b]ab");
    }
}
