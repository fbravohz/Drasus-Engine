# DEBT-042 · El puente FFI nunca pasó el gate de mutación, y su primera corrida destapó nueve huecos previos

| Campo | Valor |
|---|---|
| **ID** | DEBT-042 |
| **Severidad** | 🟠 Media |
| **Estado** | Abierta |
| **Abierta** | 2026-07-31 |
| **Origen** | Primera corrida del gate de mutación sobre el crate `bridge`, durante STORY-073 |
| **Disparador de pago** | La próxima Story que toque cada módulo implicado; o barrido único antes de cerrar EPIC-1 |

---

## El hecho

El crate `bridge` se construyó antes de que el gate de mutación fuera obligatorio (ADR-0133 capa 8) y **nunca se le corrió**. La primera corrida devolvió **118 supervivientes de 215 mutantes**.

De esos 118, **107 están en `src/frb_generated.rs`** — código que escribe `flutter_rust_bridge_codegen`, no una persona. Se excluyeron del gate mediante `.cargo/mutants.toml` en esta misma Story: mutarlo mide la corrección de una herramienta de terceros, y el archivo se reescribe entero en cada regeneración, así que ninguna prueba escrita contra él sobrevive al siguiente cambio de superficie.

**Quedan 11 supervivientes en código escrito a mano.** Dos de ellos (`describe_confirm_error`) nacieron en STORY-073 y se cerraron en ella. **Los nueve restantes son anteriores** y son el contenido de esta deuda:

| Ubicación | Mutación que sobrevive |
|---|---|
| `api/clock.rs:34` | `get_clock_timestamp_ns` devuelve `0`, `1` o `-1` (3 mutantes) |
| `api/audit.rs:79` | `get_recent_audit_events` devuelve la lista vacía |
| `api/jobs.rs:66` | `get_jobs_summary` devuelve la lista vacía |
| `api/data_fetcher.rs:274` | `get_job_status` devuelve "sin estado" |
| `api/data_fetcher.rs:320` | `list_download_records` devuelve la lista vacía |
| `api/hybrid_data_transformer.rs:288` | `bar_type_to_string` devuelve `""` o `"xyzzy"` (2 mutantes) |

## Corrida de confirmación tras excluir el código generado (2026-07-31)

Con `.cargo/mutants.toml` en su sitio, el gate pasó de **215 mutantes en 5 minutos** a **45 en 2 minutos**, y el resultado quedó legible: **9 supervivientes, exactamente los nueve de la tabla de arriba**. Ninguno en `api/data_import_wizard.rs` — la superficie que STORY-073 construyó está limpia.

**Decisión del Tech-Lead, registrada para que no se lea como laxitud:** el crate `bridge` queda formalmente **NO APTO** en el gate, y se acepta así porque los nueve supervivientes son **anteriores a esta Story, están en cinco módulos que no tocó, y quedan rastreados aquí**. Exigirle a una Story que cierre huecos de cinco módulos ajenos para poder cerrarse es la forma de que nadie vuelva a correr el gate sobre un crate viejo — que es precisamente cómo se llegó hasta aquí.

Lo que **no** se acepta y por eso se cerró dentro de la Story: los dos supervivientes que nacieron en ella (`describe_confirm_error`).

## Por qué importa

El patrón es el mismo en casi todos: **una función que devuelve datos puede devolver nada, y ninguna prueba se entera**. En una capa cuya única razón de existir es transportar datos desde el motor hasta la pantalla, "devuelve la lista vacía sin que nadie lo note" es exactamente el fallo que hay que poder descartar — una pantalla vacía se lee como "no hay datos", no como "el transporte se rompió".

El del reloj es el más agudo: `get_clock_timestamp_ns` devolviendo `0` significa que toda la interfaz creería estar en enero de 1970, y nada lo contradiría.

`bar_type_to_string` es de la misma familia que el superviviente de `label()` cerrado en STORY-073: una etiqueta que viaja hacia la pantalla y puede ser basura sin que falle nada.

## Por qué no se pagó ahora

STORY-073 cerró lo que nació en ella. Estos nueve pertenecen a cinco módulos distintos —reloj, auditoría, trabajos, descargas, transformador— que esta Story no tocó, y arreglarlos exige entender qué debería afirmar cada uno, no añadir pruebas mecánicas.

Precedente idéntico: [`DEBT-034`](./DEBT-034.md), que registró los 49 supervivientes de `sovereign-data-fetcher` por la misma causa — cerrado antes de que el gate fuera formal.

## Relación con otras deudas

- [`DEBT-034`](./DEBT-034.md) — misma causa raíz: crates cerrados antes de que el gate de mutación fuera obligatorio. **Conviene pagarlas en el mismo barrido.**
- [`DEBT-019`](./debt/DEBT-019.md) — cobertura de mutación de la plomería de EPIC-0, misma familia.
