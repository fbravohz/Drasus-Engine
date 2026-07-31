# broker_server_clock

**Feature dueña:** [`instrument-catalog`](../features/instrument-catalog.md)
**Migración:** `migrations/0022_instrument_session_catalog.sql`
**Naturaleza:** Referencia/Mutable, una fila por bróker/venue (NO por instrumento — el reloj de servidor no varía entre símbolos del mismo bróker)
**Perfil ADR-0020:** Perfil A (Datos/Ingest) — Grupo I (universal) + Grupo IV (`node_id`/`process_id`), igual que sus cuatro hermanas de la misma migración (`instrument_master`/`broker_instrument_terms`/`session_template`/`session_template_window`)

## Columnas

| Columna | Tipo SQL | Null | Descripción |
|---|---|---|---|
| `id` | TEXT | NO | PK, UUIDv7 |
| `created_at` | INTEGER | NO | Nanosegundos desde epoch |
| `updated_at` | INTEGER | NO | Cambia con cada edición |
| `audit_hash` | TEXT | NO | SHA-256 de esta versión de la fila |
| `audit_chain_hash` | TEXT | SÍ | NULL en la fila génesis |
| `row_version` | INTEGER | NO | Arranca en 1, +1 en cada UPDATE |
| `node_id` | TEXT | SÍ | Huella del hardware que registró el alta/edición |
| `process_id` | TEXT | SÍ | PID del proceso que ejecutó la escritura |
| `venue` | TEXT | NO | Clave de negocio — cadena plana, sin FK (mismo trato que `broker_instrument_terms.venue`; no existe hoy un catálogo de venues como entidad propia). `UNIQUE` |
| `clock_anchor_timezone_ref` | TEXT | NO | Identificador IANA elegido por su calendario de cambios de horario de verano, NO necesariamente por coincidir con el offset literal del bróker |
| `clock_fixed_offset_minutes` | INTEGER | NO | `DEFAULT 0`. Constante con signo sumada al offset propio del ancla en TODO instante. `0` reproduce sin caso especial una zona IANA plana |
| `venue_operational_since_ns` | INTEGER | SÍ | Nanosegundos UTC (renombrado 2026-07-27, TASK-072 — antes `venue_operational_since` sin sufijo). `NULL` = no verificado, NUNCA "opera desde el origen de los tiempos". Hecho externo que el usuario declara/verifica, nunca inferido del archivo — gobierna el chequeo de fecha de operación del Gate de Homogeneidad de Procedencia |

## Claves

- **PK:** `id`
- **FK salientes:** ninguna
- **Únicas:** `venue` — un bróker tiene UN solo reloj de servidor declarado
- **Índices:** `node_id`

## Quién la referencia (FK entrante)

- Ninguna físicamente. Por convención de aplicación (ADR-0137 prohíbe acceso cross-crate a tablas ajenas): `data-import-wizard` recibe `venue_operational_since_ns` como parámetro de entrada inyectado por quien orquesta el import (Bridge/CLI), nunca consultando esta tabla directamente.

## Notas de diseño

- **Caso real que motiva esta tabla:** IC Markets opera a GMT+2 en invierno y GMT+3 en verano, con las fechas de cambio de horario de **Estados Unidos**, no las europeas. Ningún identificador IANA plano describe eso — de ahí el par ancla+desplazamiento: `clock_anchor_timezone_ref = "America/New_York"` (hereda las fechas de cambio de DST correctas, incluidas las históricas) + `clock_fixed_offset_minutes = 420` (7 horas) sí lo describe.
- **`clock_fixed_offset_minutes = 0` no es un caso especial** — es el mecanismo de declaración de zona plana de hoy, expresado sin pérdida como el caso particular de este mecanismo más general (ADR-0162 "Enmienda 2026-07-26").
- **El desplazamiento se declara, nunca se infiere** de volatilidad, noticia o cualquier proxy de mercado — mismo principio que rige `session_template.market_timezone_ref` desde ADR-0162 Restricción 7.
- **`venue_operational_since_ns` es un hecho externo verificado** (ej. fecha de fundación pública del bróker), no un cálculo derivado del archivo importado. Alimenta el chequeo de fecha de operación del Gate de Homogeneidad de Procedencia en `pit-data-validator`/`data-import-wizard`: un tramo declarado que antecede esta fecha se rechaza con motivo explícito; sin este campo declarado (`NULL`), ese chequeo no corre.
- **Concurrencia optimista real:** esta fila es el borrador desde el que se pre-rellena la declaración que el usuario sella como evidencia (LDR-0008) — dos ediciones concurrentes que parten del mismo `row_version` no pueden ambas tener éxito; la segunda recibe un conflicto de versión explícito en vez de pisar la primera en silencio (`InstrumentRepository::update_broker_server_clock`, probado en `persistence.rs`).
- **NUNCA se fusiona con `session_template`/`session_template_window`:** ese par declara la zona de referencia de los **límites de ventana de una plantilla de sesión** (convención de mercado, ej. "Forex abre domingo 22:00 hora de Nueva York"); esta tabla declara el **reloj de servidor de un bróker concreto** (hecho de infraestructura del bróker, no de mercado). Ambos usan el mismo widget de selector (patrón SQX, bandera DST) pero son declaraciones distintas, con dueñas distintas.

## Historial de corrección

La primera versión de esta tabla (STORY-070, 2026-07-26) llevaba únicamente los cinco campos de dominio, siguiendo la letra literal de la enmienda de ADR-0162 al pie: esa sección del ADR listaba solo esos cinco campos, sin repetir el recordatorio de Grupo I que aparece en el banner general de la migración. El Rust-Engineer implementó la especificación tal cual estaba escrita y reportó la inconsistencia frente a sus cuatro hermanas de la misma migración, en vez de decidir por su cuenta. El Tech-Lead escaló el conflicto (entre la letra de la enmienda y el Grupo I universal de ADR-0020) al Architect, que resolvió (TASK-071, 2026-07-26): fue una omisión de redacción, nunca una excepción deliberada — ADR-0020 no admite excepción para el Grupo I. El texto de ADR-0162 y esta tabla quedaron corregidos en el mismo día (STORY-070bis).

**TASK-072 (2026-07-27):** `venue_operational_since` → `venue_operational_since_ns`. El barrido de unidades del Tech-Lead detectó que era el único timestamp de dominio del repositorio sin sufijo de unidad en el nombre — la prosa de la propia celda ya decía "Nanosegundos UTC", pero el nombre no lo cargaba. El Architect confirmó que el patrón `_ns` para campos de dominio (distinto de `created_at`/`updated_at`, Grupo I) ya era precedente vigente en ADR-0141 §"Timestamps" desde su origen, y que ninguna enmienda de ADR-0162 argumentó nunca la ausencia del sufijo — transcripción sin verificar, no decisión. Se corrigió el texto de ADR-0162 y esta ficha; se generalizó la regla en ADR-0141 (check `M13`) para que el Gate del Tech-Lead la detecte por checklist en vez de depender de un barrido manual. **Retrofit de código completado el mismo día:** columna SQL de `0022` (in-situ, GREENFIELD), los tres structs y `persistence.rs` de `instrument-catalog`, el parámetro de `compute_broker_server_clock_audit_hash` en su núcleo puro, y el puerto/orquestador de `data-import-wizard`. Recuento de mutantes sin variación (115 y 90), que es la comprobación de que fue renombrado puro y no cambio de comportamiento.
