# Instrument Catalog

**Carpeta:** `./features/instrument-catalog/`
**Estado:** En Diseño
**Última actualización:** 2026-07-23 (ADR-0162)
**Decisión Arquitectónica Asociada:** ADR-0140 (activa el registro `Instrument`) · ADR-0154 (distingue de la tabla de capacidades) · ADR-0155 (hogar de los atributos estáticos de instrumento) · ADR-0162 (esta feature)

## ¿Qué es?

Es el catálogo canónico de instrumentos y sus términos de trading. Activa el registro `Instrument` que ADR-0140 dejó como enum sin tabla: guarda la identidad general de cada símbolo (pip size, tick size, moneda base/cotización) una sola vez, y los términos que varían por bróker (spread, comisión, swap) como un **override sparse** que hereda del canónico cuando no se declara.

**Problema:** StrategyQuant X repite el símbolo como fila completa de instrumento por cada bróker, aunque la mayoría de los campos son idénticos entre brókers — sin mecanismo de herencia. Nuestro sistema hoy no tiene ninguna tabla de instrumento en absoluto: `slippage-models.md` declara `SWAP_TRIPLE_DAY` como parámetro global, cuando en la realidad varía por bróker y por activo.

**Solución:** Jerarquía de dos niveles con herencia explícita por `NULL`. `instrument_master` es la identidad canónica (una fila por símbolo, sin bróker). `broker_instrument_terms` es el override — una fila solo cuando un bróker concreto ofrece ese instrumento, y cada campo es opcional: `NULL` significa "usa el valor canónico", nunca "dato faltante por error".

## Comportamientos Observables

- [ ] El usuario da de alta un instrumento nuevo (ej. `XAUUSD`); declara su clase de activo, moneda base/cotización, pip size y tick size canónicos → el sistema crea la fila en `instrument_master`.
- [ ] El usuario asocia ese instrumento a un bróker concreto y declara solo lo que difiere (ej. `default_spread` para ese bróker) → el resto de los campos de `broker_instrument_terms` quedan `NULL` y el sistema resuelve automáticamente el valor canónico al leer.
- [ ] Cualquier feature que pida los términos resueltos de `(instrument_id, venue)` recibe un único objeto ya combinado (`COALESCE` del override sobre el canónico) — nunca tiene que consultar dos tablas ni implementar la lógica de herencia por su cuenta.
- [ ] Si dos brókers ofrecen el mismo instrumento con condiciones distintas (spread, comisión), cada uno tiene su propia fila de override — nunca se funden ni se promedian.
- [ ] Si el usuario borra el override de un bróker, el instrumento vuelve a resolver 100% desde el canónico sin que nadie edite `instrument_master`.

## Restricciones

- **NUNCA se duplica un campo del canónico en el override "por si acaso".** `broker_instrument_terms` solo declara lo que ese bróker concreto necesita sobreescribir.
- **NUNCA `NULL` en el override se trata como error de captura.** Es la herencia declarada, y debe distinguirse en la interfaz de un campo que el usuario sí completó con un valor real que resulta ser cero.
- **NUNCA se fusiona este catálogo con la tabla de capacidades por bróker (ADR-0154).** Esa tabla declara qué puede traer un bróker; este catálogo declara en qué condiciones se simula una operación con él. Dominios de dato distintos.
- **NUNCA se inventa un piso de spread global igual para todo instrumento.** El piso vive por instrumento (`default_spread`), consumido por la Política de Integridad de Spread de `data-sanitizer-pipeline` — nunca una constante cableada en código.
- **NUNCA se borra `instrument_master` si tiene overrides o datasets de mercado que lo referencian** (`ON DELETE RESTRICT`, ADR-0141).

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| INSTRUMENT_ID_CASE | UPPERCASE | — | Normalización de `instrument_id` al persistir (coincide con la forma canónica de la tripleta, ADR-0155) | [FIJO] |
| COMMISSION_MODEL_DEFAULT | NONE | `NONE / PER_TRADE / PERCENTAGE / SIZE_BASED / STOCKPICKER` | Valor de `commission_model` cuando el usuario no declara ninguno al crear el override | CONFIG |

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** la resolución `COALESCE` campo-a-campo (override no-`NULL` gana, `NULL` hereda del canónico) es una función pura sin IO: entra el canónico + el override, sale el objeto resuelto.
- **Shell (Infraestructura):** persistencia SQLite de `instrument_master`/`broker_instrument_terms`, generación de UUIDv7, transacción `BEGIN IMMEDIATE` para altas/ediciones con su fila de auditoría.
- **Frontera Pública:** `ResolveInstrumentTerms(instrument_id, venue) → ResolvedInstrumentTerms`.

## Ciclo de Vida de la Feature — Instrument Catalog

### Entrada
- Declaración del usuario: identidad canónica del instrumento (alta) o términos específicos de un bróker (override).
- Consulta de un consumidor: `(instrument_id, venue)`.

### Proceso
- Alta/edición: valida el `asset_class` contra el enum de ADR-0140, valida `commission_model` contra su `CHECK`, persiste en transacción con su fila de auditoría.
- Resolución de lectura: toma la fila canónica + la fila de override (si existe) y combina campo a campo, override no-`NULL` gana.

### Salida
- `ResolvedInstrumentTerms`: objeto único con todos los términos de trading ya resueltos para ese `(instrument_id, venue)`.

### Contextos de Uso

**Contexto 1: Importación de datos (Ingest)**
- `data-sanitizer-pipeline` consulta `default_spread` resuelto para calibrar el piso de spread de un instrumento durante la sanitización.

**Contexto 2: Simulación (Validate)**
- `backtest-engine` consulta términos resueltos como fallback del selector `SPREAD_SOURCE_MODE` en modo `FIXED`, y como fuente futura de comisión/swap (migración diferida desde los `CONFIG` globales actuales de `slippage-models`).

## Tareas (TTRs)

### TTR-001: Alta y Edición de Instrumento Canónico
- Implementa la validación y persistencia de `instrument_master`: identidad, clase de activo, pip/tick size canónicos, plantilla de sesión por defecto.

### TTR-002: Override de Términos por Bróker
- Implementa la persistencia sparse de `broker_instrument_terms`, con `NULL` explícito como semántica de herencia.

### TTR-003: Resolución de Términos (`COALESCE`)
- Implementa la función pura de resolución campo-a-campo y el puerto de lectura `ResolveInstrumentTerms`.

## Puertos de Integración (ADR-0137)

| Puerto | ID de tipo | Dirección | Cardinalidad | Descripción |
|---|---|---|---|---|
| `resolve_terms_in` | `(interno)` | Input | 1 | Solicitud `(instrument_id, venue)` — payload específico de esta feature, no tipo de dominio |
| `resolved_terms_out` | `ResolvedInstrumentTerms` | Output | 1 | Términos de trading ya resueltos (canónico + override combinados) |

## Cáscara Visual (Thin Shell)

> **Pendiente — Etapa 0.5 (skill `ui-designer`).** Esta sección no la rellena el Architect; se completa antes del despacho al Flutter Engineer, siguiendo `docs/DESIGN.md`. Contrato funcional mínimo para esa etapa: el Canvas debe presentar un único nodo "Instrumento" que resuelve y muestra los niveles canónico + override fusionados (ver TASK-065 §4) — nunca dos paneles separados que obliguen al usuario a razonar sobre la herencia.

### Ventana de Verificación Visual
El observable de esta feature (plomería de catálogo, sin superficie propia de primera clase en el MVP) es el conjunto de campos resueltos de `ResolvedInstrumentTerms`, visible en la pestaña/panel de `data-import-wizard` cuando el usuario asocia un bróker a un instrumento durante la importación.

---

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016):** 100% Local.
- **Fidelidad (ADR-0017):** No aplica (catálogo de referencia, no simulación).
- **Inundación de Fundaciones (ADR-0020):** Perfil **A. Datos/Ingest** — Identidad (I) + Linaje de Datos (III) + Hardware (IV). **Corrección de secuencia (ADR-0141 §"Semántica de secuencias"):** `instrument_master` y `broker_instrument_terms` son tablas Referencia/Mutable, no event-stores append-only — el campo de secuencia del Grupo I es `row_version` (empieza en 1, se incrementa con cada `UPDATE`), no `event_sequence_id`.

## Persistencia (Inundación de Fundamentos — ADR-0020)

Tablas propias, esquema completo en ADR-0162. Resumen del set de relevancia técnica:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Clave técnica UUIDv7 |
| | `instrument_id` | Clave de negocio — la misma cadena que ya circula en `QuoteTick`/`TradeTick`/`Bars` (ADR-0155 Restricción 8) |
| | `created_at` | Timestamp de alta |
| | `updated_at` | Timestamp de última edición |
| | `row_version` | Contador de versión de la fila (mutable, ver corrección de secuencia arriba) |
| | `audit_hash` | Hash de la fila |
| | `audit_chain_hash` | `NULL` en la fila génesis |
| **III. Linaje** | — | No aplica: catálogo declarado por el usuario, no dato de mercado con procedencia externa |
| **IV. Hardware** | `node_id` | ID del hardware que registró el alta/edición |
| | `process_id` | PID del proceso que ejecutó la escritura |

**Rastro de Evidencia:** el `row_version` y el `audit_hash` de cada edición permiten reconstruir el historial de cambios de términos de un instrumento/bróker para auditoría de `feedback` (¿cambió el spread declarado entre dos backtests?).

## Dependencias y Bloqueantes

- Bloqueado por: ninguno — puede construirse en cuanto se apruebe ADR-0162.
- Bloquea: la migración de `slippage-models`/`backtest-engine` de sus `CONFIG` globales actuales a términos resueltos (diferida, no bloqueante para esta feature).
- **"Consumido por" = accede al puerto, no al esquema:** [`data-sanitizer-pipeline`](./data-sanitizer-pipeline.md) (piso de spread), [`backtest-engine`](./backtest-engine.md) (`SPREAD_SOURCE_MODE=FIXED` fallback), [`data-import-wizard`](./data-import-wizard.md) (UI de asociación bróker↔instrumento).
- **Contrato de Integración UI:** **Ventana de Verificación** (Feature de plomería, sin superficie propia en el MVP) — consumidora principal: `data-import-wizard`, pestaña "Importar". El observable de esta feature son los campos de `ResolvedInstrumentTerms` visibles en el paso de asociación bróker↔instrumento del asistente.

## Dependencias
**Depende de:**
- Ninguna.

**Consumido por:**
- [`data-sanitizer-pipeline`](./data-sanitizer-pipeline.md), [`backtest-engine`](./backtest-engine.md), [`data-import-wizard`](./data-import-wizard.md).
