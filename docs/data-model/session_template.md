# session_template

**Feature dueña:** [`session-catalog`](../features/session-catalog.md)
**Migración:** `migrations/0022_instrument_session_catalog.sql`
**Naturaleza:** Mutable (`row_version`) — plantilla de calendario compartida, nunca copiada por bróker×símbolo.
**Perfil ADR-0020:** Perfil A (Datos/Ingest) — Grupo I (universal) + Grupo IV (`node_id`/`process_id`)

## Columnas

| Columna | Tipo SQL | Null | Descripción |
|---|---|---|---|
| `id` | TEXT | NO | PK, UUIDv7 — clave técnica que referencian `instrument_master.default_session_template_id` y `broker_instrument_terms.session_template_id_override` |
| `created_at` | INTEGER | NO | Nanosegundos desde epoch |
| `updated_at` | INTEGER | NO | Cambia con cada edición |
| `audit_hash` | TEXT | NO | SHA-256 de esta versión de la fila |
| `audit_chain_hash` | TEXT | SÍ | NULL en la fila génesis |
| `row_version` | INTEGER | NO | Arranca en 1, +1 en cada UPDATE |
| `node_id` | TEXT | SÍ | Huella del hardware que registró el alta/edición |
| `process_id` | TEXT | SÍ | PID del proceso que ejecutó la escritura |
| `template_code` | TEXT | NO | UNIQUE — clave de negocio, ej. `"FOREX_245"`, `"CRYPTO_247"`, `"FUTURES_CME_GLOBEX"` |
| `display_name` | TEXT | NO | Etiqueta legible para el usuario |
| `market_timezone_ref` | TEXT | NO | Identificador **IANA** (ej. `"America/New_York"`), NUNCA una convención de desplazamiento inventada (Corrección del Architect #3, ADR-0162). La conversión de zona ocurre en tiempo de LECTURA — esta columna nunca guarda un UTC ya convertido |

## Claves

- **PK:** `id`
- **FK salientes:** ninguna
- **Únicas:** `UNIQUE(template_code)`
- **Índices:** `node_id`

## Quién la referencia (FK entrante)

- `session_template_window.session_template_id` — las ventanas semanales de esta plantilla
- `instrument_master.default_session_template_id` — calendario por defecto de un instrumento
- `broker_instrument_terms.session_template_id_override` — calendario específico de un bróker

## Notas de diseño

- Un template por FORMA DE MERCADO (`FOREX_245`, `CRYPTO_247`, `FUTURES_CME_GLOBEX`…), referenciado por FK desde cuantos instrumentos/overrides lo compartan — NUNCA se copia por bróker×símbolo (Restricción FIJA #2, ADR-0162). Es la respuesta directa al defecto de StrategyQuant X que el propietario detectó.
- `SESSION_TEMPLATE_SEED_SET` (confirmado por el propietario 2026-07-25): **tres** plantillas semilla — Forex 24/5, Cripto 24/7, y Futuros estilo CME Globex. La plantilla de Futuros es un default REPRESENTATIVO y EDITABLE (zona `America/Chicago`, hueco diario de mantenimiento de 15 min + hueco semanal) — los calendarios de futuros reales son específicos de cada mercado/producto; ver `docs/lessons/rust/STORY-066-instrument-session-catalog.md`.
