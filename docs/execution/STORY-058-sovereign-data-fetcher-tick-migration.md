# STORY-058 · Migración de `sovereign-data-fetcher` de `Tick` a `QuoteTick`/`TradeTick`

> **Orden de trabajo — Spec-Driven.** Especificación cerrada por el Architect. Alcance deliberadamente pequeño y separado de STORY-057 (crate distinto, feature distinta, precedente STORY-024).

| Campo | Valor |
|---|---|
| **ID** | STORY-058 |
| **Título** | Migrar `OutputPorts.ticks_out` de `Tick` (obsoleto) a `QuoteTick`/`TradeTick` |
| **Tipo** | Task |
| **Épica (Fase)** | EPIC-1 — Ingest |
| **Sprint** | continuación de STORY-051/STORY-057 |
| **Estado** | Pendiente |
| **Responsable** | Rust-Engineer (Modo Autónomo — mecánico, sin decisión de diseño) · audita Tech-Lead |
| **Creada** | 2026-07-18 |
| **Completada** | — |

## 0. Resumen ejecutivo

- **Qué problema resuelve:** `crates/shared/src/types/mod.rs` ya declara `QuoteTick`/`TradeTick` (STORY de esta misma sesión, Pieza 3 de la Orden del Architect), pero `sovereign-data-fetcher::public_interface::OutputPorts` sigue anclado al marcador `Tick`, ahora obsoleto, contradiciendo el catálogo vigente de ADR-0137.
- **Qué se va a construir:** reemplazar el campo único `ticks_out: Tick` por dos campos `quote_ticks_out: QuoteTick` / `trade_ticks_out: TradeTick`, y retirar el import de `Tick` de este crate.
- **Por qué ahora:** es el punto abierto que el propio `Tick` (marcador obsoleto en `shared`) señala en su doc-comment; hasta que se resuelva, el código sigue contradiciendo a ADR-0137 en el único lugar donde el tipo obsoleto todavía se usa.

## 1. Especificación de origen

- **ADR(s):** [ADR-0137](../adr/ADR-0137.md) (Enmienda 2026-07-18 — división de `Tick`), [ADR-0155](../adr/ADR-0155.md).
- **Feature(s):** [`sovereign-data-fetcher`](../features/sovereign-data-fetcher.md).
- **Origen del punto abierto:** [STORY-051](./STORY-051-hybrid-data-transformer.md) §8, punto 2.

## 2. Objetivo (una frase llana)

Que el descargador soberano deje de anunciar en el Canvas un tipo de dato (`Tick`) que ya no existe en el catálogo, y anuncie los dos tipos reales que produce.

## 3. Decisión del Architect — alcance exacto

**Archivo afectado:** `crates/features/data/sovereign-data-fetcher/src/public_interface.rs` (único lugar del workspace, fuera de `shared`, que referencia `Tick` — verificado por búsqueda exhaustiva 2026-07-18).

**Cambio:**
1. `use shared::types::{Bars, Tick};` → `use shared::types::{Bars, QuoteTick, TradeTick};`.
2. `pub struct OutputPorts { pub ticks_out: Tick, pub bars_out: Bars }` → dos campos de salida en vez de uno:
   - `pub quote_ticks_out: QuoteTick` — cotizaciones Bid/Ask crudas (mismo dato que hoy describe el doc-comment "Transacciones crudas Bid/Ask/Last", pero solo la mitad Bid/Ask).
   - `pub trade_ticks_out: TradeTick` — ejecuciones/Last crudas (la otra mitad del doc-comment actual).
3. Doc-comment de `OutputPorts` (línea ~103-107 hoy) actualizado: ya no dice "los tipos son stubs que indican QUÉ clase de dato produce" en singular sobre `Tick` — debe explicar que la feature produce **dos** primitivos físicos distintos desde el mismo flujo de descarga (Bulk + Delta), consistente con ADR-0155 restricción 1 ("`sovereign-data-fetcher` y `broker-connector`... producen ahora ambos tipos").
4. **Sin cambio de lógica.** Son marcadores de canvas de tamaño cero (`struct QuoteTick;`, `struct TradeTick;`) — no hay dato real que migrar, transformar ni demultiplexar en este crate. El demultiplexado real de un archivo mixto (MT5) es responsabilidad de `data-normalization-layer`, no de este crate (ADR-0155 ya lo asigna así) — **no construyas lógica de demux aquí**, esto es solo la declaración del puerto.

**Qué NO toca esta Orden:**
- No retira `Tick` de `crates/shared/src/types/mod.rs` — otros consumidores futuros podrían aparecer; el marcador se retira en una limpieza aparte una vez que una búsqueda confirme cero referencias.
- No toca `broker-connector` (mismo tipo de deuda, pero esa feature aún no está implementada — nada que migrar hoy).
- No construye el contrato físico de `QuoteTick`/`TradeTick` (columnas Polars/Arrow) — el doc-comment actual ya aclara que eso lo define el transformador aguas abajo; sigue siendo cierto tras esta migración.

## 4. Instrucciones de despacho (Rust-Engineer, Modo Autónomo)

```
Implementa STORY-058 tal como está en docs/execution/STORY-058-sovereign-data-fetcher-tick-migration.md
§3. Es un cambio mecánico de un archivo: reemplaza el marcador de puerto `Tick` por `QuoteTick` +
`TradeTick` en `OutputPorts`, actualiza el import y el doc-comment. Verifica con `grep -rn "shared::types::Tick\b"
crates/` que no queda ninguna otra referencia al marcador obsoleto fuera de `crates/shared` antes de
entregar. No construyas lógica de demultiplexado ni contrato físico — no es el alcance de esta Orden.
```

**Resumen de lo implementado** (lo llena el Rust-Engineer):
`crates/features/data/sovereign-data-fetcher/src/public_interface.rs`: import cambiado a `use shared::types::{Bars, QuoteTick, TradeTick};`; `OutputPorts.ticks_out: Tick` reemplazado por `quote_ticks_out: QuoteTick` + `trade_ticks_out: TradeTick`; doc-comments del módulo y del struct actualizados para explicar la separación física cotización/ejecución (ADR-0155). Sin lógica de demultiplexado añadida — solo declaración de puerto, según §3.4. Prueba discriminante nueva: `public_interface::port_contract_tests::output_ports_expone_quote_y_trade_ticks_por_separado`. `Tick` NO se retiró de `crates/shared` — la Story lo reserva explícitamente para una limpieza aparte (§3 "Qué NO toca esta Orden" + §8); la búsqueda de cero referencias ya se cumple hoy (ver reporte del Rust-Engineer al Tech-Lead), así que el disparador de esa limpieza está listo cuando se despache.

## 5. Criterio de aceptación

| # | Criterio verificable | Prueba que lo demuestra |
|---|---|---|
| 1 | `OutputPorts` ya no importa ni referencia `Tick` | `cargo build --workspace` verde + `grep` sin resultados fuera de `crates/shared` |
| 2 | `OutputPorts` expone `quote_ticks_out: QuoteTick` y `trade_ticks_out: TradeTick` | inspección de tipo + `cargo build` |
| 3 | Ninguna prueba existente de `sovereign-data-fetcher` se rompe por el cambio de forma de `OutputPorts` (verificar si algún test construye `OutputPorts` literal y necesita el campo nuevo) | `cargo test -p sovereign-data-fetcher` |

## 6. Comandos de validación (para el usuario — copy/paste)

```bash
cargo build --workspace
cargo test -p sovereign-data-fetcher
cargo clippy --workspace --all-targets -- -D warnings
grep -rn "shared::types::Tick\b" crates/ --include=*.rs
```

## 7. Registro de ejecución
- 2026-07-18 · Architect (Sonnet) · Especificación cerrada, sin implementación de código.
- 2026-07-18 · Rust-Engineer (Sonnet, Modo Autónomo) · Implementado §3 completo. `cargo build --workspace`, `cargo test --workspace` (todo verde) y `cargo clippy -p sovereign-data-fetcher -p shared --all-targets -- -D warnings` (0 warnings) verificados. `Tick` no se retiró de `shared` (fuera de alcance de esta Orden, ver §3/§8) — pendiente de que el Tech-Lead despache la limpieza aparte.

- 2026-07-18 · Tech-Lead · Auditoría reproducida (25 pruebas verdes, clippy limpio, puertos separados, cero usuarios del marcador viejo) y **ejecutado el retiro de `Tick`** de `crates/shared/src/types/mod.rs`, con una nota de historia en su lugar. Disparador de §8 cumplido y verificado en el mismo turno; `cargo build`/`clippy`/`test` de `shared` + `sovereign-data-fetcher` verdes tras el retiro. **Constancia del gate de coherencia:** el encargo verbal del Tech-Lead pedía retirar el marcador, contra lo que §3 de esta Orden excluye explícitamente; el Rust-Engineer se detuvo, siguió el documento sellado por encima de la instrucción verbal y lo reportó. Comportamiento correcto — la Orden manda sobre el despacho.

## 8. Pendientes derivados / decisiones
- ~~Retiro definitivo del marcador `Tick` de `crates/shared/src/types/mod.rs`~~ — ✅ ejecutado por el Tech-Lead el 2026-07-18 (ver §7).
- `broker-connector` tiene la misma deuda latente (ADR-0137 lo lista como productor dual de `QuoteTick`/`TradeTick`), pero esa feature no existe en código todavía — no hay nada que migrar hasta que se construya.

---

## 9. Cierre ejecutivo (para el usuario — CEO)

<Lo llena el Tech-Lead al cerrar la Story.>
