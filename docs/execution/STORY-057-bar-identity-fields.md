# STORY-057 · Campos de identidad en la barra (tripleta símbolo × temporalidad × bróker)

> **Orden de trabajo — Spec-Driven.** Especificación cerrada por el Architect, lista para que el Tech-Lead la despache al Rust-Engineer. El Architect NO implementó código en esta Orden (fuera de su mandato) — solo decide forma y tipos.

| Campo | Valor |
|---|---|
| **ID** | STORY-057 |
| **Título** | Campos de identidad en la barra (`Bar`/`BarOutput`/`TransformRequest`) |
| **Tipo** | Story |
| **Épica (Fase)** | EPIC-1 — Ingest |
| **Sprint** | continuación de STORY-051 |
| **Estado** | ✅ **CERRADA** — implementada dentro del rehecho de STORY-051; sellada retroactivamente por el Tech-Lead el 2026-07-25 |
| **Responsable** | Rust-Engineer (Modo Autónomo — retrofit acotado, sin decisión de diseño abierta) · audita Tech-Lead |
| **Creada** | 2026-07-18 |
| **Completada** | — |

## 0. Resumen ejecutivo

- **Qué problema resuelve:** `Bar`/`BarOutput` no llevan quién es la serie (símbolo, bróker, proveedor, tipo y temporalidad de barra) — ADR-0155 los declara obligatorios por fila desde el día en que se aprobó, y STORY-051 los dejó pendientes a propósito (§8 punto 3, "no lo agregué por mi cuenta").
- **Qué se va a construir:** seis campos de identidad nuevos en `Bar`/`BarOutput`, los mismos seis como entrada obligatoria en `TransformRequest`, su propagación uniforme a cada barra de una corrida, y su inclusión en `compute_bars_audit_hash`.
- **Por qué ahora:** es uno de los tres puntos abiertos que STORY-051 §8 dejó bloqueando el despacho del puente FFI (Bridge → Flutter → QA). Sin esto, el esquema de `Bar` no es fiel a ADR-0155 y el puente expondría un contrato que hay que romper otra vez apenas se corrija.

---

## 1. Especificación de origen (qué specs implementa)

- **ADR(s):** [ADR-0155](../adr/ADR-0155.md) (esquema canónico + Restricción 8 + enmienda de tripleta 2026-07-18), [ADR-0141](../adr/ADR-0141.md) (mapeo de tipos SQLite/Rust/Parquet).
- **Feature(s):** [`hybrid-data-transformer`](../features/hybrid-data-transformer.md).
- **Origen del punto abierto:** [STORY-051](./STORY-051-hybrid-data-transformer.md) §8, punto 3 ("Campos de identidad de instrumento/venue ausentes en `Bar`").

## 2. Objetivo (una frase llana)

Que cada barra que produce el transformador diga, sin ambigüedad, de qué símbolo, qué bróker, qué proveedor y qué temporalidad es — para que nunca se pueda confundir con otra serie que por coincidencia tenga el mismo precio.

---

## 3. Decisión del Architect — forma exacta (no se re-abre en implementación)

### 3.1 Los seis campos, con tipo exacto

Todos **obligatorios** (no `Option`) en `Bar` y en `BarOutput` — ADR-0155 Restricción 8 los declara "obligatorios en toda fila de todos los datasets de este ADR", sin excepción para `Bars`:

| Campo | Tipo Rust | Tipo SQLite/Parquet (ADR-0141) | Origen del valor |
|---|---|---|---|
| `instrument_id` | `String` | `TEXT NOT NULL` | Copiado de `TransformRequest.instrument_id` |
| `venue` | `String` | `TEXT NOT NULL` | Copiado de `TransformRequest.venue` |
| `source_provider` | `String` | `TEXT NOT NULL` | Copiado de `TransformRequest.source_provider` |
| `bar_type` | `BarType` (enum nuevo) | `TEXT CHECK (col IN ('TIME','TICK_COUNT','VOLUME','DOLLAR','RENKO'))` | Copiado de `TransformRequest.bar_type` |
| `timeframe_value` | `String` | `TEXT NOT NULL` | Copiado de `TransformRequest.timeframe_value` |
| `data_snapshot_id` | `String` | `TEXT NOT NULL` | Copiado de `TransformRequest.data_snapshot_id` (cambia de `Option<String>` a `String`, ver §3.3) |

**`BarType` — enum nuevo en `domain.rs`, mismo patrón que `AggressorSide`:**
- Variantes: `Time`, `TickCount`, `Volume`, `Dollar`, `Renko` — las cinco del `CHECK` de ADR-0155, sin excepción (Inundación de Fundaciones, ADR-0020 §7.3: canonizar las cinco cuesta cero aunque esta feature solo produzca `Time` hoy).
- Espejo serializable en `schemas.rs` (`BarTypeInput` o nombre equivalente que el ingeniero decida, con `#[derive(Serialize, Deserialize)]`), convertido con `From`, igual que `AggressorSide`/`AggressorSideInput`.
- **Esta feature (`aggregate_trades_to_bars`, agregación de `TradeTick` por intervalo fijo) siempre produce `BarType::Time`.** Las otras cuatro variantes existen para cuando `algorithmic-bars` (aún no construida) las necesite — no se fabrica lógica para producirlas aquí.

### 3.2 Por qué campos por fila y no solo a nivel de lote — coste de almacenamiento

ADR-0155 Restricción 8 ya fija esto como FIJO (no es una decisión nueva que el Architect esté tomando aquí, es su aplicación literal): estos campos van **por fila física**, no solo a nivel de `TransformRequest`/ledger. La objeción obvia —"¿no es desperdicio repetir el mismo `instrument_id` un millón de veces?"— no aplica en Parquet: es un formato columnar con *dictionary encoding* y *run-length encoding* por columna. Una columna con **un único valor distinto** repetido en toda una partición comprime a un puñado de bytes (el diccionario tiene una entrada; el RLE la repite con un contador), no a `8 bytes × N filas`. El coste real es despreciable; el beneficio (una fila de `Bars` es auto-descriptiva, no depende de que el lector conserve el contexto del `TransformRequest` que la produjo) es alto. Esto también resuelve la pregunta "¿se deriva al vuelo o se materializa": se **materializa**, porque derivarla al vuelo requeriría que todo lector de una partición Parquet supiera reconstruir el `TransformRequest` original — información que el archivo Parquet, por diseño (ADR-0035), no conserva por sí solo.

### 3.3 Cambio al contrato de entrada — `TransformRequest`

Seis campos nuevos, todos obligatorios (sin `Option`, consistente con "el ingest no arranca sin identidad conocida" — un `TransformRequest` sin bróker conocido no es un caso válido, no un caso con `None`):

```
instrument_id: String
venue: String
source_provider: String
bar_type: BarTypeInput   (ver §3.1 — casi siempre BarTypeInput::Time en esta feature)
timeframe_value: String  (forma legible, ej. "1H" — NO se deriva de target_interval_ns; ver nota abajo)
```

Más el cambio de tipo de un campo **ya existente**:

- **`data_snapshot_id: Option<String>` → `data_snapshot_id: String` (obligatorio).** Es un cambio incompatible del contrato público — todo llamador actual (harness, pruebas, cualquier consumidor de `public_interface`) que hoy construye un `TransformRequest` con `data_snapshot_id: None` deja de compilar y debe suministrar un valor real. Es la consecuencia directa de que cada `Bar` de salida ahora exige `data_snapshot_id` no nulo (§3.1) — no hay forma de cumplir la Restricción 8 con un origen opcional.

**Nota sobre `timeframe_value` vs. `target_interval_ns`:** son dos representaciones del mismo intervalo, ninguna deriva de la otra automáticamente en esta Orden. `target_interval_ns` (ya existente) es la forma numérica que el Núcleo usa para agregar; `timeframe_value` (nueva) es la forma canónica de texto que ADR-0155 exige en el esquema físico y que alimenta la tripleta (`EURUSD-1M-PEPPERSTONE`). El llamador los suministra ambos, coherentes entre sí — **no se construye aquí una tabla de mapeo `nanosegundos → "1H"`**: inventar esa tabla sin una fuente que la gobierne (qué temporalidades son "estándar") sería alucinar una convención fuera del alcance de esta Orden. Si esa tabla ya existe en otro punto del sistema (revisar antes de escribir código), reutilizarla; si no existe, el llamador la suministra explícita.

### 3.4 Sello de auditoría — `compute_bars_audit_hash` SÍ incluye identidad

**Decisión: sí, los seis campos entran en el hash, por fila, dentro del mismo bucle que ya serializa los demás campos de cada `Bar`** (mismo patrón que el resto: `instrument_id`, `venue`, `source_provider`, la representación de texto de `bar_type`, `timeframe_value`, `data_snapshot_id` se añaden a la lista `fields` existente).

**Por qué por fila y no una vez fuera del bucle:** aunque en el patrón de llamada actual todas las barras de una corrida comparten la misma identidad (un `TransformRequest` = un dataset), el propio esquema de ADR-0155 no impide estructuralmente que un `Vec<Bar>` mezcle identidades — nada en el tipo `Bar` lo prohíbe. Hashear por fila mantiene el hash honesto al contenido físico real de cada fila (coherente con el propio doc-comment de `BarOutput`, "espejo 1:1 de `Bar`") en vez de asumir una invariante que el tipo no impone. Es la corrección directa al defecto que el propio ADR describe: "dos series distintas con el mismo contenido OHLCV no deberían producir el mismo sello" — hoy sí lo producen; con este cambio, dejan de hacerlo.

**Consecuencia esperada, no una regresión:** cualquier valor de hash fijo usado hoy como oráculo en una prueba existente cambia. Es correcto — el hash está protegiendo más contenido que antes.

### 3.5 Versión de esquema

`SCHEMA_VERSION` (`orchestrator.rs`) sube de `"2.1"` a **`"3.0"`** — el propio precedente en el código (comentario junto a la constante) reserva el salto de versión mayor para una reestructuración de magnitud comparable a "1.0"→"2.0" (esa vez: representación entera del volumen). Añadir seis columnas obligatorias nuevas a la fila física y romper el contrato de `TransformRequest` (`data_snapshot_id` deja de ser opcional) es esa misma magnitud. El Rust-Engineer puede ajustar el número exacto si al implementar encuentra un criterio de versión ya escrito que diga lo contrario — no es un valor arbitrario, es la aplicación del criterio ya documentado en el código.

### 3.6 Fuera de alcance (deliberado, no lo agregues)

- **`RawTrade`/`TradeTickInput` (el tick de entrada) NO llevan estos campos en esta Orden.** El punto abierto de STORY-051 §8 habla explícitamente de la barra, no del tick de entrada — la identidad del tick físico (`QuoteTick`/`TradeTick`) es un problema de `sovereign-data-fetcher`/`data-normalization-layer` (ver STORY-058), no de este crate de agregación pura.
- **La tabla ledger `transformation_records` (SQLite) NO cambia de esquema.** ADR-0155 ya distinguió Grupo I/III del ledger de la fila de mercado (su "Corrección del Architect al insumo del Quant-Engineer") — el ledger sigue siendo metadatos de la corrida, no del dataset. No dupliques los seis campos ahí; ya están cubiertos por el `data_snapshot_id` que el ledger conserva.
- **No se construye la tabla `nanosegundos → texto de temporalidad`** — ver nota en §3.3.

---

## 4. Instrucciones de despacho (Rust-Engineer, Modo Autónomo)

```
Implementa STORY-057 tal como está especificada en docs/execution/STORY-057-bar-identity-fields.md
§3 completa. La forma exacta de los tipos NO es negociable (ya está decidida por el Architect);
las decisiones de nombre de función interna, organización de helpers y estructura del `From` sí
son tuyas. Actualiza todos los call-sites rotos por el cambio de `data_snapshot_id` a obligatorio
(tests, harness, cualquier constructor de `TransformRequest`). Actualiza el doc-comment de `Bar`
y `BarOutput` para reflejar los seis campos nuevos. Corre el criterio de aceptación de la tabla §5
antes de entregar.
```

**Resumen de lo implementado** (lo llena el Rust-Engineer al ejecutar):
<pendiente>

## 5. Criterio de aceptación (cada criterio ↔ su prueba)

| # | Criterio verificable | Prueba que lo demuestra |
|---|---|---|
| 1 | `Bar` y `BarOutput` tienen los seis campos de §3.1, ninguno `Option` | inspección de tipos + `cargo build` |
| 2 | `TransformRequest` exige los cinco campos nuevos + `data_snapshot_id` ya no es `Option` | `cargo build` falla en cualquier call-site que no los suministre, y todos quedan corregidos |
| 3 | Dos `TransformRequest` idénticos salvo `venue` producen `Bar`s con `venue` distinto, nunca fundidos | prueba nueva: `bars_carry_distinct_venue_when_requests_differ_only_in_venue` (nombre orientativo) |
| 4 | `compute_bars_audit_hash` produce hashes distintos para dos lotes de `Bar` con OHLCV idéntico pero identidad distinta | prueba nueva: `audit_hash_differs_when_only_identity_differs` |
| 5 | Todas las pruebas existentes de `domain.rs`/`schemas.rs`/`orchestrator.rs` siguen en verde tras actualizar sus fixtures a los campos nuevos | `cargo test -p hybrid-data-transformer` |
| 6 | `bar_type` es siempre `Time` en la salida de `aggregate_trades_to_bars` (esta feature no produce barras algorítmicas) | prueba existente de agregación, extendida para verificar `bar_type` |

## 6. Comandos de validación (para el usuario — copy/paste)

```bash
cargo build --workspace
cargo test -p hybrid-data-transformer
cargo clippy --workspace --all-targets -- -D warnings
```

## 7. Registro de ejecución (bitácora cronológica)
- 2026-07-18 · Architect (Sonnet) · Especificación cerrada, sin implementación de código (fuera de mandato del Architect).
- 2026-07-18 · Rust-Engineer (Sonnet) · Implementada **dentro del rehecho de STORY-051**, no como despacho separado — por eso esta Orden quedó sin sellar en su momento. Commit `d2a78bc`.
- 2026-07-25 · Tech-Lead · **Auditoría de sellado retroactivo reproducida** al detectar el desfase documental (la Orden decía `Pendiente` con el código ya entregado). Verificado: los seis campos de identidad viven en `schemas.rs:158-163` sin `Option`; `BarIdentity` se propaga desde `orchestrator.rs:96` a cada barra; las pruebas discriminantes de los criterios 3 y 4 existen con nombre propio (`bars_carry_distinct_venue_when_requests_differ_only_in_venue` en `domain.rs:931`, `audit_hash_differs_when_only_venue_differs_despite_identical_ohlcv` en `domain.rs:1181`, y la más fuerte `audit_hash_differs_when_any_single_identity_field_differs` en `domain.rs:1203`, que cubre campo por campo); criterio 5 verde (76 pruebas de la crate, 0 fallos). **Lección de proceso:** una Orden absorbida por otra se sella igual, en el mismo turno en que se absorbe — si no, el registro miente sobre lo ya construido.

## 8. Pendientes derivados / decisiones

- Si al implementar se descubre que ya existe en el sistema una tabla `target_interval_ns → timeframe_value` estándar, usarla en vez de exigir que el llamador la suministre a mano; si no existe, queda como trabajo futuro de conveniencia (no bloquea esta Story).
- El renombre de `bar_timestamp_ns` → `bar_open_timestamp_ns` (para calzar textualmente con ADR-0155) es opcional en esta Story — mecánico y de bajo riesgo, pero no forma parte del criterio de aceptación; el Rust-Engineer puede incluirlo si el costo de tocarlo junto a este cambio es marginal.

---

## 9. Cierre ejecutivo (para el usuario — CEO)

```
ESTADO: 🟢 COMPLETADO

PROGRESO MACRO:
- Cada vela que produce el sistema lleva grabado a quién pertenece: qué instrumento,
  de qué bróker, de qué proveedor y de qué temporalidad. Antes dos velas con los mismos
  números eran indistinguibles aunque vinieran de brókers distintos; ahora no pueden
  confundirse ni fundirse, y su sello de auditoría cambia si cualquiera de esos datos
  cambia — aunque los precios sean idénticos.

FRICCIONES Y DEUDA:
- Ninguna nueva. El trabajo se entregó dentro del rehecho del agregador de barras y su
  orden quedó sin sellar durante una semana: el registro decía "pendiente" mientras el
  código ya estaba en el repositorio. Corregido, sin impacto sobre el producto.

INPUT REQUERIDO DEL CEO:
- Ninguna.
```
