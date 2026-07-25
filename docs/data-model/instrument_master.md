# instrument_master

**Feature dueña:** [`instrument-catalog`](../features/instrument-catalog.md)
**Migración:** `migrations/0022_instrument_session_catalog.sql`
**Naturaleza:** Mutable (`row_version`) — catálogo canónico, una fila por símbolo, sin bróker.
**Perfil ADR-0020:** Perfil A (Datos/Ingest) — Grupo I (universal, `row_version` en vez de `event_sequence_id`) + Grupo IV (`node_id`/`process_id`)

## Columnas

| Columna | Tipo SQL | Null | Descripción |
|---|---|---|---|
| `id` | TEXT | NO | PK, UUIDv7 — clave TÉCNICA (Corrección del Architect #1, ADR-0162) |
| `created_at` | INTEGER | NO | Nanosegundos desde epoch |
| `updated_at` | INTEGER | NO | Cambia con cada edición |
| `audit_hash` | TEXT | NO | SHA-256 de esta versión de la fila |
| `audit_chain_hash` | TEXT | SÍ | NULL en la fila génesis |
| `row_version` | INTEGER | NO | Arranca en 1, +1 en cada UPDATE |
| `node_id` | TEXT | SÍ | Huella del hardware que registró el alta/edición |
| `process_id` | TEXT | SÍ | PID del proceso que ejecutó la escritura |
| `instrument_id` | TEXT | NO | UNIQUE — clave de NEGOCIO, la misma cadena ("EURUSD") que ya circula en `QuoteTick`/`TradeTick`/`Bars` (ADR-0155 Restricción 8) |
| `asset_class` | TEXT | NO | `CHECK IN ('FOREX','FUTURE','CFD','CRYPTO','EQUITY')` — vocabulario de ADR-0140 |
| `base_currency` | TEXT | SÍ | Forex/CFD; NULL si no aplica |
| `quote_currency` | TEXT | SÍ | Forex/CFD; NULL si no aplica |
| `canonical_pip_size` | INTEGER | NO | ×10⁸, convención de mercado (no de un bróker concreto) |
| `canonical_tick_size` | INTEGER | NO | ×10⁸ |
| `default_session_template_id` | TEXT | SÍ | FK → `session_template(id)`, calendario por defecto del instrumento |
| `strike_price` | INTEGER | SÍ | ×10⁸ — Opciones/Futuros (ADR-0140/ADR-0155) |
| `expiration_date` | INTEGER | SÍ | Nanosegundos UTC — Opciones |
| `option_type` | TEXT | SÍ | `CHECK (NULL OR IN ('CALL','PUT'))` |
| `option_style` | TEXT | SÍ | `CHECK (NULL OR IN ('AMERICAN','EUROPEAN','OTHER'))` |
| `contract_multiplier` | INTEGER | SÍ | Contador plano (típicamente 100), SIN escala ×10⁸ |
| `occ_symbol` | TEXT | SÍ | Formato OSI/OCC de 21 caracteres |
| `contract_code` | TEXT | SÍ | Futuros |
| `contract_expiration_date` | INTEGER | SÍ | Futuros, nanosegundos UTC |

## Claves

- **PK:** `id`
- **FK salientes:** `default_session_template_id` → `session_template(id)` `ON DELETE RESTRICT`
- **Únicas:** `UNIQUE(instrument_id)` — un símbolo, una sola fila canónica
- **Índices:** `node_id`, `default_session_template_id` (FK hijo, ADR-0141 M7)

## Quién la referencia (FK entrante)

- `broker_instrument_terms.instrument_id` — referencia esta clave de NEGOCIO (`instrument_id`), no el UUID técnico `id`

## Notas de diseño

- Dos columnas de identidad con roles distintos (Corrección del Architect #1, ADR-0162): `id` (UUIDv7, clave técnica, cumple la PK universal de ADR-0141) e `instrument_id` (clave de negocio, la misma cadena que ya circula en todo dataset de mercado). Colapsarlas habría violado uno de los dos invariantes.
- `underlying_instrument_id` (que ADR-0155 sí documenta para Opciones) queda **fuera** del esquema de esta migración a propósito: ADR-0162 enumera explícitamente 8 atributos de Opciones/Futuros (no 9) al activar este registro — ver "Pendientes" en la lección Docente de STORY-066.
- `swap_type` (en la tabla hija `broker_instrument_terms`) queda deliberadamente sin `CHECK` — vocabulario diferido, ADR-0162 Restricción #10.
