# Proyecto 3 — Simulador de Máquinas de Turing

Curso: Teoría de la Computación

| Integrante | Usuario de GitHub |
|---|---|
| Juan Gualim | [@JuanGualim](https://github.com/JuanGualim) |
| Denil Parada | [@dparada2020225](https://github.com/dparada2020225) |
| Harry Méndez | [@hmndz3](https://github.com/hmndz3) |

**Video:** https://youtu.be/d6ZB2o4p4pA 

Simulador de Máquinas de Turing (MT) de **una cinta**, escrito en **Rust**. El programa:

1. Lee la **descripción formal** de una MT desde un archivo YAML y la valida.
2. **Simula** cada cadena de entrada configurada.
3. Muestra la **secuencia completa de descripciones instantáneas (IDs)** y la
   **configuración final de la cinta**.
4. Indica si cada cadena es **aceptada o rechazada**, y por qué.

Además: control finito con **caché** (`[estado, caché]`, como en el laboratorio 10),
modo **visual** de la cinta, modo **paso a paso**, exportación del **diagrama de estados**
(Graphviz o Mermaid), validación completa del YAML y pruebas exhaustivas.

## Máquinas incluidas

| MT | Archivo | Descripción |
|---|---|---|
| **Reconocedora** | [`maquinas/reconocedora_multiplicacion.yaml`](maquinas/reconocedora_multiplicacion.yaml) | L = { aⁱ bʲ cᵏ \| i, j ≥ 1, **k = i·j** }: verifica una multiplicación. No es un lenguaje libre de contexto. |
| **Alteradora** | [`maquinas/alteradora_ordenamiento.yaml`](maquinas/alteradora_ordenamiento.yaml) | **Ordena** una cadena sobre {a, b, c, d} con **bubble sort** sobre la cinta, usando la caché. `dcbaa → aabcd` |
| Extra | [`maquinas/extra/alteradora_unario_a_binario.yaml`](maquinas/extra/alteradora_unario_a_binario.yaml) | Convierte aⁿ al número n en binario. `aaaaa → 101` |
| Extra | [`maquinas/extra/reconocedora_palindromos.yaml`](maquinas/extra/reconocedora_palindromos.yaml) | Palíndromos sobre {a, b, c} usando la caché. |
| Extra | [`maquinas/extra/lab9_incremento_binario.yaml`](maquinas/extra/lab9_incremento_binario.yaml) | MT del laboratorio 9 (para comparar las IDs). |

La explicación de cada máquina, con su definición formal, sus funciones de transición y
su diagrama, está en [`docs/maquinas.md`](docs/maquinas.md). La arquitectura del programa
está en [`docs/arquitectura.md`](docs/arquitectura.md).

## Requisitos

- [Rust](https://www.rust-lang.org/tools/install) 1.85 o superior (edición 2024).
  En Linux/macOS:

  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

  En Windows, descargue `rustup-init.exe` desde la página oficial.

Las dependencias (`serde` y `serde_yaml`) las descarga `cargo` automáticamente.

## Compilar y ejecutar

```bash
git clone https://github.com/JuanGualim/Proyecto3_TC.git
cd Proyecto3_TC
cargo build --release
```

Ejecutar una máquina con las cadenas de su archivo YAML:

```bash
cargo run --release -- maquinas/reconocedora_multiplicacion.yaml
```

También se puede usar el binario ya compilado: `./target/release/turing_sim <archivo.yaml>`.

### Opciones

```text
turing_sim <archivo.yaml> [opciones]

  -i, --input <cadena>   Simula esta cadena en lugar de las del YAML
                         (se puede repetir; use "" para la cadena vacía)
  -m, --max-steps <n>    Límite de pasos por cadena (por defecto 10000)
  -q, --quiet            No muestra las IDs, solo el resultado de cada cadena
  -v, --visual           Dibuja la cinta y la cabeza debajo de cada ID
  -p, --paso-a-paso      Pausa después de cada ID (ENTER = siguiente paso,
                         c + ENTER = continuar sin pausas)
  -d, --diagrama <ruta>  Exporta el diagrama de estados: .dot (Graphviz)
                         o .md/.mmd (Mermaid)
      --no-color         Desactiva los colores
  -h, --help             Muestra la ayuda
```

### Ejemplos

```bash
# Simular una cadena específica mostrando la cinta en cada paso
cargo run --release -- maquinas/alteradora_ordenamiento.yaml -i dcbaa -v

# Avanzar paso por paso con ENTER
cargo run --release -- maquinas/reconocedora_multiplicacion.yaml -i abbcc -p

# Solo el resultado de varias cadenas
cargo run --release -- maquinas/reconocedora_multiplicacion.yaml -q -i aabbcccc -i aabbccc

# Exportar el diagrama de estados (Mermaid se ve directo en GitHub)
cargo run --release -- maquinas/reconocedora_multiplicacion.yaml -q -d diagrama.md
```

## Formato del archivo YAML

El formato es el del enunciado. `name`, `description` y `blank` son opcionales.

```yaml
mt:
  name: "Nombre de la máquina"          # opcional
  description: "Qué hace"               # opcional
  states: [q0, q1, qf]
  input_alphabet: [a, b]
  tape_alphabet: [a, b, X, B]
  initial_state: q0
  accept_states: [qf]
  blank: B                               # opcional, por defecto B
  transitions:
    - state: q0
      read: [a, B]      # [símbolo leído en la cinta, caché actual]
      write: [a, X]     # [caché nueva, símbolo que se escribe]
      move: R           # L, R o S (no moverse)
      next: q1
  inputs:
    - "aabb"
    - "ab"
```

La transición del ejemplo es **δ([q0, B], a) = ([q1, a], X, R)**: en el estado `q0` con
la caché vacía (`B`) se lee `a`; la MT pasa a `q1` guardando `a` en la caché, escribe `X`
y mueve la cabeza a la derecha.

- Si la máquina no usa la caché se puede escribir `read: a` y `write: X` (la caché queda en `B`).
- Los símbolos numéricos (`0`, `1`) se aceptan con o sin comillas.
- Si la cadena tiene espacios, cada palabra es un símbolo (`"a1 a2 b1"`), lo que permite
  usar símbolos de varios caracteres.

El simulador **valida** la definición antes de ejecutar: estados y símbolos existentes,
Σ ⊆ Γ, blanco ∈ Γ \ Σ, movimientos válidos y **determinismo** (no puede haber dos
transiciones con el mismo `[estado, caché]` y el mismo símbolo leído).

## Salida

Para cada cadena se imprime:

```text
── Entrada: "abbcc" ──────────────────────────────────────────────────
      0   [q0, B]abbcc     ID inicial
      1 ⊢ a[q1, B]bbcc     δ #1: δ([q0, B], a) = ([q1, B], a, R)
      2 ⊢ ab[q2, B]bcc     δ #3: δ([q1, B], b) = ([q2, B], b, R)
      ...
     32 ⊢ XbbZZ[qf, B]B    δ #33: δ([q12, B], B) = ([qf, B], B, S)

  Resultado:   ACEPTADA ✔
  Motivo:      la última ID contiene el estado de aceptación qf (32 pasos)
  Cinta final: XbbZZ
```

Cada ID usa la notación **α[q, c]β**: α es la cinta a la izquierda de la cabeza, `[q, c]`
es el estado y la caché, y β empieza con el símbolo que se está leyendo.

La MT **acepta** cuando entra a un estado de aceptación. **Rechaza** cuando no existe una
transición para la configuración actual, cuando la cadena tiene símbolos fuera de Σ o
cuando llega al límite de pasos (posible ciclo infinito). Al final se muestra una tabla
resumen con todas las cadenas.

## Estructura del repositorio

```text
Proyecto3_TC/
├── Cargo.toml
├── README.md
├── docs/
│   ├── arquitectura.md      # diseño del programa y estructuras de datos
│   └── maquinas.md          # explicación, transiciones y diagramas de cada MT
├── maquinas/
│   ├── reconocedora_multiplicacion.yaml
│   ├── alteradora_ordenamiento.yaml
│   └── extra/               # máquinas adicionales
├── src/
│   ├── main.rs              # línea de comandos
│   ├── lib.rs               # declara los módulos
│   ├── config.rs            # parsing del YAML
│   ├── machine.rs           # MT validada y función de transición
│   ├── tape.rs              # cinta infinita y parsing de la entrada
│   ├── simulator.rs         # motor de simulación e IDs
│   ├── display.rs           # salida en consola
│   └── diagram.rs           # exportación Graphviz / Mermaid
└── tests/
    └── maquinas.rs          # pruebas exhaustivas de las MT
```

## Pruebas

```bash
cargo test --release
```

Además de las pruebas unitarias de cada módulo, `tests/maquinas.rs` compara cada MT con
la respuesta calculada en Rust para **todas** las cadenas hasta cierta longitud. Por
ejemplo, la reconocedora se prueba con las 9 841 cadenas sobre {a, b, c} de longitud ≤ 8.

## Comandos para el video

```bash
# 4.1 MT reconocedora: 3 cadenas aceptadas y 3 rechazadas (todas de longitud ≥ 5)
cargo run --release -- maquinas/reconocedora_multiplicacion.yaml

# 4.2 MT alteradora: 5 cadenas de longitud ≥ 5
cargo run --release -- maquinas/alteradora_ordenamiento.yaml

# Una cadena con la cinta dibujada, para explicar el algoritmo
cargo run --release -- maquinas/alteradora_ordenamiento.yaml -i dcbaa -v -p
```
