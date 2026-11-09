//! Cinta de la Máquina de Turing.
//!
//! La cinta es infinita en ambas direcciones. Solo guardamos la parte que ya fue
//! visitada en un `VecDeque`; cuando la cabeza se sale por la izquierda o por la
//! derecha agregamos un blanco en ese extremo. Así la cinta crece bajo demanda.

use std::collections::VecDeque;
use std::fmt;

/// Un símbolo de la cinta. Se usa `String` para permitir símbolos de más de un
/// carácter (por ejemplo `a1` o `#`).
pub type Symbol = String;

/// Desplazamiento de la cabeza después de escribir.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    Left,
    Right,
    Stay,
}

impl Move {
    /// Convierte `L`, `R` o `S` (sin importar mayúsculas) en un movimiento.
    pub fn parse(text: &str) -> Option<Move> {
        match text.trim().to_ascii_uppercase().as_str() {
            "L" => Some(Move::Left),
            "R" => Some(Move::Right),
            "S" => Some(Move::Stay),
            _ => None,
        }
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let letra = match self {
            Move::Left => "L",
            Move::Right => "R",
            Move::Stay => "S",
        };
        f.write_str(letra)
    }
}

/// Cinta con su cabeza lectora/escritora.
#[derive(Debug, Clone)]
pub struct Tape {
    cells: VecDeque<Symbol>,
    head: usize,
    blank: Symbol,
}

/// Fotografía de la parte relevante de la cinta: lo que está a la izquierda de la
/// cabeza, el símbolo bajo la cabeza y lo que está a la derecha. Se usa para
/// construir las descripciones instantáneas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TapeView {
    pub left: Vec<Symbol>,
    pub head: Symbol,
    pub right: Vec<Symbol>,
}

impl Tape {
    /// Crea la cinta con la entrada escrita desde la posición 0 y la cabeza sobre
    /// el primer símbolo. Una entrada vacía deja la cabeza sobre un blanco.
    pub fn new(input: &[Symbol], blank: &str) -> Tape {
        let mut cells: VecDeque<Symbol> = input.iter().cloned().collect();
        if cells.is_empty() {
            cells.push_back(blank.to_string());
        }
        Tape { cells, head: 0, blank: blank.to_string() }
    }

    /// Símbolo bajo la cabeza.
    pub fn read(&self) -> &Symbol {
        &self.cells[self.head]
    }

    /// Escribe un símbolo en la celda actual.
    pub fn write(&mut self, symbol: &str) {
        self.cells[self.head] = symbol.to_string();
    }

    /// Mueve la cabeza, agregando blancos si se llega a un extremo.
    pub fn shift(&mut self, movement: Move) {
        match movement {
            Move::Left => {
                if self.head == 0 {
                    self.cells.push_front(self.blank.clone());
                } else {
                    self.head -= 1;
                }
            }
            Move::Right => {
                self.head += 1;
                if self.head == self.cells.len() {
                    self.cells.push_back(self.blank.clone());
                }
            }
            Move::Stay => {}
        }
    }

    /// Parte de la cinta entre el primer y el último símbolo no blanco, ampliada
    /// para que siempre incluya la celda donde está la cabeza.
    pub fn view(&self) -> TapeView {
        let not_blank = |s: &Symbol| *s != self.blank;
        let first = self.cells.iter().position(not_blank).unwrap_or(self.head);
        let last = self.cells.iter().rposition(not_blank).unwrap_or(self.head);
        let start = first.min(self.head);
        let end = last.max(self.head);

        TapeView {
            left: self.cells.range(start..self.head).cloned().collect(),
            head: self.cells[self.head].clone(),
            right: self.cells.range(self.head + 1..=end).cloned().collect(),
        }
    }

    /// Contenido final de la cinta sin los blancos de los extremos.
    pub fn contents(&self) -> Vec<Symbol> {
        let not_blank = |s: &Symbol| *s != self.blank;
        match (self.cells.iter().position(not_blank), self.cells.iter().rposition(not_blank)) {
            (Some(first), Some(last)) => self.cells.range(first..=last).cloned().collect(),
            _ => Vec::new(),
        }
    }
}

/// Une símbolos para mostrarlos. Si todos miden un carácter se pegan (`aabb`);
/// si alguno es más largo se separan con espacios (`a1 a2 b1`) para no confundirlos.
pub fn join_symbols(symbols: &[Symbol]) -> String {
    if symbols.iter().all(|s| s.chars().count() == 1) {
        symbols.concat()
    } else {
        symbols.join(" ")
    }
}

/// Divide una cadena de entrada en símbolos del alfabeto.
///
/// - Si la cadena tiene espacios, cada palabra es un símbolo (`"a1 a2 b1"`).
/// - Si no, se toma en cada posición el símbolo más largo del alfabeto que
///   coincida, de modo que `"aab"` con alfabeto `{a, b}` da `[a, a, b]`.
///
/// Devuelve el primer fragmento que no pertenece al alfabeto como error.
pub fn tokenize(input: &str, alphabet: &[Symbol]) -> Result<Vec<Symbol>, String> {
    let input = input.trim();
    if input.contains(char::is_whitespace) {
        return input
            .split_whitespace()
            .map(|word| {
                if alphabet.iter().any(|s| s == word) {
                    Ok(word.to_string())
                } else {
                    Err(word.to_string())
                }
            })
            .collect();
    }

    let mut by_length: Vec<&Symbol> = alphabet.iter().filter(|s| !s.is_empty()).collect();
    by_length.sort_by_key(|s| std::cmp::Reverse(s.len()));

    let mut symbols = Vec::new();
    let mut rest = input;
    while !rest.is_empty() {
        match by_length.iter().find(|s| rest.starts_with(s.as_str())) {
            Some(symbol) => {
                symbols.push(symbol.to_string());
                rest = &rest[symbol.len()..];
            }
            None => {
                let bad: String = rest.chars().next().into_iter().collect();
                return Err(bad);
            }
        }
    }
    Ok(symbols)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn syms(text: &str) -> Vec<Symbol> {
        text.chars().map(|c| c.to_string()).collect()
    }

    #[test]
    fn la_cinta_crece_hacia_ambos_lados() {
        let mut tape = Tape::new(&syms("ab"), "B");
        tape.shift(Move::Left);
        assert_eq!(tape.read(), "B");
        tape.write("X");
        tape.shift(Move::Right);
        tape.shift(Move::Right);
        tape.shift(Move::Right);
        assert_eq!(tape.read(), "B");
        assert_eq!(join_symbols(&tape.contents()), "Xab");
    }

    #[test]
    fn la_vista_recorta_blancos_pero_incluye_la_cabeza() {
        let mut tape = Tape::new(&syms("ab"), "B");
        tape.shift(Move::Right);
        tape.shift(Move::Right);
        tape.shift(Move::Right);
        let view = tape.view();
        assert_eq!(view.left, syms("abB"));
        assert_eq!(view.head, "B");
        assert!(view.right.is_empty());
    }

    #[test]
    fn entrada_vacia_es_un_blanco() {
        let tape = Tape::new(&[], "B");
        assert_eq!(tape.read(), "B");
        assert!(tape.contents().is_empty());
    }

    #[test]
    fn tokeniza_por_simbolo_mas_largo_y_por_espacios() {
        let alfabeto: Vec<Symbol> = ["a", "b", "ab#"].iter().map(|s| s.to_string()).collect();
        assert_eq!(tokenize("aab#b", &alfabeto).unwrap(), vec!["a", "ab#", "b"]);
        assert_eq!(tokenize("a ab# b", &alfabeto).unwrap(), vec!["a", "ab#", "b"]);
        assert_eq!(tokenize("abc", &alfabeto), Err("c".to_string()));
        assert!(tokenize("", &alfabeto).unwrap().is_empty());
    }
}
