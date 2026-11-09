//! Pruebas de integración: cargan las máquinas de `maquinas/` y comparan lo que
//! hace la MT contra la respuesta calculada directamente en Rust, para TODAS las
//! cadenas hasta cierta longitud.

use std::path::Path;
use turing_sim::config;
use turing_sim::machine::TuringMachine;
use turing_sim::simulator::{Halt, run};

fn load(file: &str) -> TuringMachine {
    // `file` puede incluir subcarpetas, p. ej. "extra/...".
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("maquinas")
        .join(file);
    let raw = config::load_file(&path).expect("YAML válido");
    TuringMachine::from_raw(raw).expect("MT válida")
}

/// Todas las cadenas sobre `alphabet` de longitud 0..=max_len.
fn all_strings(alphabet: &[char], max_len: usize) -> Vec<String> {
    let mut result = vec![String::new()];
    let mut layer = vec![String::new()];
    for _ in 0..max_len {
        layer = layer
            .iter()
            .flat_map(|w| alphabet.iter().map(move |c| format!("{w}{c}")))
            .collect();
        result.extend(layer.iter().cloned());
    }
    result
}

/// ¿w = a^i b^j c^k con i, j >= 1 y k = i·j?
fn is_multiplication(w: &str) -> bool {
    let i = w.chars().take_while(|&c| c == 'a').count();
    let j = w[i..].chars().take_while(|&c| c == 'b').count();
    let k = w[i + j..].chars().take_while(|&c| c == 'c').count();
    i + j + k == w.len() && i >= 1 && j >= 1 && k == i * j
}

#[test]
fn reconocedora_coincide_con_la_definicion_del_lenguaje() {
    let m = load("reconocedora_multiplicacion.yaml");
    for w in all_strings(&['a', 'b', 'c'], 8) {
        let r = run(&m, &w, 100_000);
        assert!(
            !matches!(r.halt, Halt::StepLimit(_)),
            "la MT no se detuvo con {w}"
        );
        assert_eq!(r.accepted(), is_multiplication(&w), "cadena {w}");
    }
}

#[test]
fn reconocedora_cadenas_del_yaml() {
    let m = load("reconocedora_multiplicacion.yaml");
    let resultados: Vec<bool> = m
        .inputs
        .iter()
        .map(|w| run(&m, w, 10_000).accepted())
        .collect();
    assert_eq!(resultados, [true, true, true, false, false, false]);
    assert!(m.inputs.iter().all(|w| w.len() >= 5));
}

#[test]
fn reconocedora_cadena_grande() {
    let m = load("reconocedora_multiplicacion.yaml");
    let w = format!("{}{}{}", "a".repeat(6), "b".repeat(7), "c".repeat(42));
    assert!(run(&m, &w, 100_000).accepted());
    let w = format!("{}{}{}", "a".repeat(6), "b".repeat(7), "c".repeat(41));
    assert!(!run(&m, &w, 100_000).accepted());
}

#[test]
fn ordenamiento_coincide_con_sort() {
    let m = load("alteradora_ordenamiento.yaml");
    for w in all_strings(&['a', 'b', 'c', 'd'], 6) {
        let r = run(&m, &w, 100_000);
        assert!(r.accepted(), "no aceptó {w}: {:?}", r.halt);
        let mut esperado: Vec<char> = w.chars().collect();
        esperado.sort();
        let esperado: String = esperado.into_iter().collect();
        assert_eq!(r.final_tape_text(), esperado, "cadena {w}");
        // La cabeza termina al inicio de la cadena ordenada.
        assert!(r.ids.last().unwrap().tape.left.is_empty(), "cadena {w}");
    }
}

#[test]
fn ordenamiento_rechaza_simbolos_fuera_del_alfabeto() {
    let m = load("alteradora_ordenamiento.yaml");
    assert_eq!(run(&m, "abxcd", 1000).halt, Halt::InvalidInput("x".into()));
}

#[test]
fn unario_a_binario_coincide_con_format() {
    let m = load("extra/alteradora_unario_a_binario.yaml");
    for n in 0..=64 {
        let r = run(&m, &"a".repeat(n), 100_000);
        assert!(r.accepted(), "n = {n}");
        assert_eq!(r.final_tape_text(), format!("{n:b}"), "n = {n}");
    }
}

#[test]
fn palindromos_coincide_con_reverse() {
    let m = load("extra/reconocedora_palindromos.yaml");
    for w in all_strings(&['a', 'b', 'c'], 7) {
        let es_palindromo = w.chars().eq(w.chars().rev());
        assert_eq!(run(&m, &w, 100_000).accepted(), es_palindromo, "cadena {w}");
    }
}
