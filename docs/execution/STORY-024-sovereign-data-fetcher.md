# STORY-024 · Descarga híbrida soberana de datos de mercado (Sovereign Fetcher)

> **Orden de Trabajo (Spec-Driven).** Es la especificación ejecutable: la instrucción EXACTA que recibe el agente, los comandos para validar por cuenta propia, y el registro de lo que pasó. Vive en git, NO en el chat. Si la spec cambia, se EDITA aquí y se re-despacha.

| Campo | Valor |
|---|---|
| **ID** | STORY-024 |
| **Título** | Descarga híbrida soberana de datos de mercado |
| **Tipo** | Story |
| **Épica (Fase)** | EPIC-1 — Soberanía de Datos (`ingest`) |
| **Sprint** | 1 (EPIC-1) |
| **Estado** | En curso (Modo Docente — Orden lista, pendiente invocación del usuario) |
| **Responsable** | Rust-Engineer (Sonnet, Modo Docente) · audita Tech-Lead + QA-Engineer |
| **Creada** | 2026-06-27 |
| **Completada** | — |

## 0. Resumen ejecutivo

- **Qué problema resuelve:** descargar años de históricos de mercado por API REST tarda días y choca con los límites de tasa del broker. Sin datos masivos y confiables, no hay backtest que valga.
- **Qué se va a construir:**
  1. El **descargador masivo concurrente** (Bulk): baja volcados comprimidos `.zip` de buckets públicos (ej. Binance Vision) saturando el ancho de banda con varios hilos.
  2. El **reconciliador Delta** (REST): rellena el hueco entre el último volcado y "ahora" usando la API REST, sin volver a pedir lo que el Bulk ya cubre.
  3. El **primer crate hexagonal de dominio** del proyecto (`crates/features/data/sovereign-data-fetcher/`), que estrena el patrón ADR-0137 (feature autónoma con puertos tipados, dependiendo solo de `shared`).
- **Por qué ahora:** es el punto de entrada del pipeline de datos (EPIC-1). Todo lo demás (normalización, sanitización, validación, almacenamiento) consume lo que produce esta feature.

---

## 1. Especificación de origen (qué specs implementa)
- **Feature:** [`sovereign-data-fetcher`](../features/sovereign-data-fetcher.md) — TTR-001 (descargador Bulk asíncrono) + TTR-002 (reconciliador Delta REST).
- **TTR de módulo:** [`ingest`](../modules/ingest.md) TTR-006 (Orquestación de Descarga Híbrida).
- **ADRs:** ADR-0034 (Ingesta Híbrida Soberana), ADR-0137 (feature hexagonal + puertos tipados), ADR-0002 (FCIS), ADR-0003/ADR-0006 (propiedad de datos + migraciones centralizadas), ADR-0020 V2 (contrato de persistencia, Perfil A), ADR-0016 (Local-First), ADR-0133 (pirámide de pruebas).
- **ADRs añadidos por el barrido completo del Gate (2026-06-27):** ADR-0011 (patrón de trabajo asíncrono — la descarga es un Job durable con recuperación), ADR-0105 (datos pesados = Polars/Arrow, nunca `Vec<struct>` por millones de filas), ADR-0093 (seguridad soberana — credenciales AES-256-GCM; aquí solo aplica como diferido: esta Story usa datos públicos sin credenciales), ADR-0008 (parámetros configurables), ADR-0012 (concurrencia consciente de recursos). Ver §8 para el detalle del barrido.

## 2. Objetivo (una frase llana)
Que el sistema pueda traer a disco años de datos de mercado de forma rápida, confiable y sin sesgo, combinando descargas masivas comprimidas con un relleno fino por API.

## 3. Agentes y Modo de Acompañamiento (ADR-0120)

| Agente | Etapa del pipeline | Depende de | Modo |
|---|---|---|---|
| Rust-Engineer | Etapa 2 — Implementación Core | ninguno | **Docente** |
| **QA-Engineer** | **Etapa 5 — gate obligatorio** | Rust-Engineer | **Autónomo** |

> **Etapas que NO aplican a esta Story:** Etapa 0.5 (UI-Designer) y Etapas 3-4 (Bridge/Flutter) — la feature no declara superficie propia; la UI de progreso de descarga pertenece a `background-download-manager`, que se construye aparte. Etapas 1 y 6 (Quant) — no aplica: esta feature es ingesta de datos, no fórmula, estrategia ni métrica estadística.

> **Modo Docente (ADR-0122) — lo despacha el Tech-Lead:** el Rust-Engineer implementa cada bloque por su cuenta (`Edit`/`Write`, como Autónomo) y escribe la lección en `docs/lessons/rust/STORY-024-sovereign-data-fetcher.md` (un archivo por Story, ADR-0124) explicando cada decisión de diseño con profundidad cero-conocimiento. El usuario NO teclea código en Docente — aprende leyendo la lección y el código ya escrito. Por eso **el Tech-Lead SÍ despacha esta Story** (vía subagente Sonnet); la enseñanza se materializa como el archivo de lección, no como diálogo en vivo. El Tech-Lead retoma para auditar y cerrar (Etapa 5 QA + verificación independiente).

## 4. Instrucciones de despacho por agente (la spec ejecutable)

### 4.1 Rust-Engineer

```
Construye la feature `sovereign-data-fetcher` (descarga híbrida soberana de datos de mercado) como el PRIMER crate hexagonal de dominio del proyecto. Esta es la entrada del pipeline de datos de EPIC-1.

ALCANCE DE ESTA STORY: solo TTR-001 (descargador Bulk asíncrono concurrente) y TTR-002 (reconciliador Delta vía REST). NO implementes TTR-003 (webhook listener) ni TTR-004 (conversor de datos alternativos) — son alcance mayor diferido.

LECTURA OBLIGATORIA ANTES DE ESCRIBIR CÓDIGO:
- docs/features/sovereign-data-fetcher.md (la spec completa, ya auditada y corregida por el Tech-Lead).
- docs/adr/ADR-0034.md (ingesta híbrida Bulk+Delta — la decisión que gobierna).
- docs/adr/ADR-0137.md (feature hexagonal: InputPorts/OutputPorts tipados; cada feature-crate depende SOLO de `shared`).
- docs/adr/ADR-0020.md sección Perfil A (qué se persiste).

ESTRUCTURA (ADR-0137 + ADR-0002 FCIS):
1. Crea el crate `crates/features/data/sovereign-data-fetcher/` copiando la estructura de `crates/features/_TEMPLATE/`. Añádelo al workspace en `Cargo.toml` raíz. Su ÚNICA dependencia interna permitida es `shared` — prohibido depender de otro crate de feature.
2. SALIDA = DATOS CRUDOS, NO parsees a structs tipados. Según la spec ("raw_segmented_data... listos para la capa de normalización") + ADR-0105 (todo DataFrame pesado se maneja con Polars/Arrow, NUNCA `Vec<struct>` para millones de filas) + SAD §8 ("Polars/Arrow para mover grandes volúmenes OHLCV"): el fetcher SOLO descarga y descomprime — produce los datos crudos en disco (archivos descomprimidos) + metadatos del segmento. NO parsea a `Tick`/`Bars` ni construye colecciones tipadas; eso es trabajo del transformador (TTR-007, que carga a Polars en su propia Story). Los structs `Tick`/`Bars` siguen como marcadores de tipo de puerto en `crates/shared/src/types/mod.rs` — NO los pueblas en esta Story; su representación física Polars/Arrow se decide al construir el transformador. El puerto de salida solo indica QUÉ clase de dato es (ticks vs barras).
3. NÚCLEO PURO (domain, sin NINGÚN import de I/O — nada de reqwest, tokio, std::fs, sqlx):
   - Detección de hueco (gap): dado el último timestamp cubierto por el Bulk y el "ahora", calcula el rango Delta exacto a pedir por REST.
   - Reconciliación de timestamps: al unir el borde Bulk↔Delta, elimina solapamientos/duplicados de forma determinista.
   - Priorización de descarga (Bulk-first): dado el rango solicitado y el inventario Bulk disponible, decide qué tramos cubre el Bulk y qué tramo residual va a REST. NUNCA debe enviar a REST un tramo que el Bulk ya cubre.
   - Verificación de espacio en disco (cálculo puro): dado el tamaño estimado del Bulk y los bytes disponibles, devuelve suficiente/insuficiente.
4. PUERTOS HEXAGONALES como traits (frontera para poder probar sin red):
   - `trait BulkSource`: listar inventario de archivos para un rango + descargar un archivo.
   - `trait DeltaSource`: pedir el tramo REST de un rango.
   Implementa el adaptador REAL en la cáscara (orchestrator) y un adaptador FALSO en los tests (fixtures en memoria). El núcleo y la orquestación se prueban con el adaptador falso — JAMÁS golpees la red real en un test.
5. CÁSCARA (orchestrator): cliente HTTP asíncrono (usa `reqwest` con `rustls-tls`, NO openssl — TLS en Rust puro, portable), descompresor `.zip` (crate `zip`), escritura a sistema de archivos, concurrencia con `tokio` (ya en el workspace) respetando CONCURRENT_DOWNLOADS, reintentos de Bulk fallido y de Delta hasta DELTA_SYNC_RETRY. **Cada descarga es un TRABAJO ASÍNCRONO DURABLE** modelado con la infraestructura existente de `async-job-executor` (tipo `Job` en `shared`, ADR-0011 + SAD §8): `process_id` único y persistente (lo exige TTR-006), ciclo de estados QUEUED→RUNNING→DONE/FAILED en SQLite, y **recuperación automática al reiniciar** (una descarga Bulk interrumpida se reanuda, no se pierde). La concurrencia debe ser consciente de recursos para no saturar otros pipelines (ADR-0012). Todos los parámetros (CONCURRENT_DOWNLOADS, DELTA_SYNC_RETRY) son configurables, nunca hardcodeados (ADR-0008).

ALCANCE DE FUENTES (ADR-0093, diferido): solo datos de mercado PÚBLICOS — volcados Bulk de Binance Vision + endpoints REST públicos de klines/trades, que NO requieren credenciales. El manejo seguro de credenciales de API (cifrado AES-256-GCM, ADR-0093) para fuentes autenticadas/privadas queda DIFERIDO a cuando exista `sovereign-security` y se añadan fuentes que lo necesiten. NO implementes gestión de claves en esta Story.
6. PERSISTENCIA (Perfil A, ADR-0020 V2 + ADR-0006 migraciones centralizadas): crea la migración de la tabla del registro de descarga en la carpeta raíz `./migrations/` (cadena lineal centralizada, ADR-0006; el migrador es `sqlx::migrate!("../../migrations")` en `crates/shared/src/persistence/pool.rs`) con el siguiente número correlativo. Campos EXACTOS de la tabla de la feature: Grupo I (id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id) + Grupo III (data_snapshot_id, logic_hash) + Grupo IV (node_id, process_id) + el campo propio de dominio `source_endpoint`. NO incluyas `execution_latency_ms` (es Grupo V, ajeno a Perfil A — la duración la lleva el `Job` de async-job-executor). El módulo de persistencia del crate usa el pool de `shared`, no crea su propio pool.

RESTRICCIONES (de la spec, son invariantes):
- NUNCA usar REST para periodos que ya existen en volcados Bulk.
- NUNCA iniciar la ingesta si el espacio en disco es insuficiente para el Bulk estimado.
- La descarga es asíncrona y no bloquea el hilo principal.

PRUEBAS (pirámide ADR-0133): unitarios deterministas para TODO el núcleo puro; integración para la cáscara usando los adaptadores falsos. Cubre CADA criterio de aceptación del §5 de esta Orden con una prueba nombrada. Entrega ya en verde con el mapeo criterio→prueba + cobertura (`cargo llvm-cov`).

COMENTARIOS (base/SKILL.md): comentario de bloque en español antes de cada función describiendo qué hace y qué devuelve; comentario de línea en la lógica no obvia (guardas, bordes, reconciliación). Identificadores en inglés (ADR-0121). Todo `unwrap()`/`expect()` en producción requiere comentario que justifique por qué no puede fallar.

Al terminar, sella la feature (banner de implementación con fecha + enlace a esta Orden) y escribe la lección en docs/lessons/rust/STORY-024-sovereign-data-fetcher.md (un archivo por Story, conceptos anclados al código real que produjiste).
```

**Plan de Implementación / Enseñanza** (Rust-Engineer · 2026-06-27 · Modo Docente):

| Bloque | Archivo | Concepto enseñado |
|---|---|---|
| 1 | `migrations/0006_sovereign_data_fetcher.sql` | Perfil A ADR-0020 V2: qué campos llevan los registros de datos |
| 2 | `Cargo.toml` workspace | Cómo se añade un crate hexagonal al workspace |
| 3 | `Cargo.toml` feature crate | Dependencias explícitas: `sqlx` en `[dependencies]`, no solo en `[dev-dependencies]` |
| 4 | `src/domain.rs` | FCIS: lógica pura sin imports de I/O; enum para resultados de dominio |
| 5 | `src/schemas.rs` | Structs de configuración configurable (ADR-0008) y persistencia (Perfil A) |
| 6 | `src/persistence.rs` | Repositorio SQLite; encadenamiento de hashes para integridad |
| 7 | `src/public_interface.rs` | Traits como puertos hexagonales; `async_trait` para `dyn Trait`; `OutputPorts` stubs |
| 8 | `src/orchestrator.rs` | `Semaphore` de Tokio para concurrencia; ciclo de vida Job ADR-0011; retry loops; adaptadores reales |
| 9 | `src/lib.rs` | Estructura del crate hexagonal; visibilidad privada de módulos internos |
| 10 | `tests/integration_tests.rs` | Adaptadores falsos; `Arc<AtomicUsize>` para medir concurrencia; `tempfile` para durabilidad |

Lección consolidada: `docs/lessons/rust/STORY-024-sovereign-data-fetcher.md`

## 5. Criterio de aceptación (cada criterio ↔ su prueba)

| # | Criterio verificable | Prueba que lo demuestra |
|---|---|---|
| 1 | El cálculo de hueco produce el rango Delta correcto a partir del corte del Bulk | `delta_range_computed_from_bulk_cutoff` |
| 2 | Un tramo cubierto por el Bulk NUNCA se pide por REST (Bulk-first) | `bulk_covered_range_never_uses_rest` |
| 3 | La ingesta aborta antes de descargar si el disco es insuficiente | `ingest_aborts_when_disk_insufficient` (dominio) + `fetch_aborts_when_orchestrator_detects_insufficient_disk` (integración) |
| 4 | El borde Bulk↔Delta queda sin barras/ticks duplicados (reconciliación) | `bulk_delta_boundary_has_no_duplicates` |
| 5 | La descarga concurrente respeta el límite CONCURRENT_DOWNLOADS | `concurrent_downloads_respect_max_limit` |
| 6 | Un archivo Bulk fallido se reintenta automáticamente | `failed_bulk_download_is_retried` |
| 7 | El Delta reintenta hasta DELTA_SYNC_RETRY antes de rendirse | `delta_sync_retries_up_to_limit` |
| 8 | El registro de descarga persiste con los campos del Perfil A (sin Grupo V) | `download_record_persisted_with_profile_a_fields` |
| 9 | El núcleo (domain) no tiene imports de I/O (FCIS) | grep: `0` ocurrencias de `reqwest\|tokio\|std::fs\|sqlx` en `domain/` |
| 10 | Una descarga interrumpida se reanuda al reiniciar (trabajo asíncrono durable, ADR-0011) | `interrupted_download_recovers_on_restart` |

## 6. Comandos de validación (para el usuario — copy/paste)
```bash
cd /var/home/fbravohz/Documentos/Entornos/Personal/Drasus-Engine
cargo test -p sovereign-data-fetcher            # pruebas de la feature
cargo test -p shared                            # tipos Tick/Bars + plomería
cargo clippy --workspace --all-targets -- -D warnings
cargo llvm-cov --workspace --summary-only       # % de cobertura de líneas
# FCIS: el núcleo no toca I/O (debe dar 0)
grep -rnE "reqwest|tokio|std::fs|sqlx" crates/features/data/sovereign-data-fetcher/src/domain* || echo "núcleo limpio"
```

## 7. Registro de ejecución (bitácora cronológica)
- 2026-06-27 · Tech-Lead · Orden creada. Gate de Coherencia corrido sobre la feature spec (correcciones en §8).
- 2026-06-27 · Tech-Lead · Gate AMPLIADO (barrido ADR completo + impacto SAD, regla del usuario): incorporados ADR-0011/0105/0093/0008/0012; corregida la salida del fetcher (datos crudos, no parseo a structs) para alinear con ADR-0105 y SAD §8. Detalle en §8. Modo Docente → **despachado por el Tech-Lead** (corrección de criterio: Docente lo despacha el Tech-Lead, no el usuario).
- 2026-06-27 · Rust-Engineer (Sonnet, Docente) · 1ª entrega. Crate creado, 20 pruebas verdes, cobertura dominio 98.92%, FCIS limpio, migración Perfil A correcta. Lección creada.
- 2026-06-27 · Tech-Lead · Verificación independiente reproducida: build/clippy/test/cobertura coinciden con el reporte; migración y stubs `Tick`/`Bars` verificados.
- 2026-06-27 · QA-Engineer (Sonnet, Autónomo) · **NO APTO.** Bug que las pruebas no atrapaban: la concurrencia es **falsa** — `orchestrator.rs` descarga secuencialmente (loop con `await` completo antes de soltar el permiso; sin `JoinSet`/`spawn`), el `Semaphore` es decorativo → Criterio 5 incumplido. La prueba `concurrent_downloads_respect_max_limit` pasa trivialmente (pico=1; aserciones `<=3` y `>0`). Tech-Lead confirmó el defecto leyendo `orchestrator.rs:142-187`. Ambos defectos = implementación → regresados al Rust-Engineer (NO escala al Architect). + observaciones menores (lección §3 a corregir, precondición de `reconcile_boundary`).
- 2026-06-27 · Tech-Lead · Regresión despachada al Rust-Engineer (mismo agente, contexto intacto): `JoinSet` para concurrencia real (`Arc<dyn BulkSource>`), prueba corregida a `peak >= 2 && peak <= límite`, lección §3 actualizada. Pendiente re-auditoría + re-gate QA.
- 2026-06-27 · Rust-Engineer · Regresión resuelta. Cambios: (1) `orchestrator.rs` — bucle secuencial reemplazado por `JoinSet::spawn`; firma cambiada a `Arc<dyn BulkSource>`; comment explica el patrón incorrecto vs correcto; retries Bulk anotados para escalamiento. (2) `public_interface.rs` — misma firma + `use std::sync::Arc`. (3) `domain.rs` — `debug_assert!` de sortedness en `reconcile_boundary`. (4) `integration_tests.rs` — todas las llamadas a `fetch` usan `Arc::new(source)`; test `concurrent_downloads_respect_max_limit` usa `flavor = "multi_thread"` y aserción `peak >= 2` (honesta). (5) `Cargo.toml` — `time` añadido a dependencias de producción de tokio. (6) Lección §3 reescrita con patrón correcto + explicación del defecto. 20/20 tests verde, clippy limpio.
- 2026-06-27 · Rust-Engineer · Implementación completa. 20 tests en verde (14 unitarios + 6 integración). Clippy limpio. FCIS verificado (0 imports I/O en domain/). Cobertura domain.rs 98.92%, public_interface.rs 100%, persistence.rs 88.48%, orchestrator.rs 76.12% (24% sin cubrir = adaptadores HTTP reales, no probados contra red real en CI). Feature sellada. Lección escrita en `docs/lessons/rust/STORY-024-sovereign-data-fetcher.md`.
- 2026-06-27 · Tech-Lead · Re-auditoría independiente del fix de regresión: 20/20 tests verde, clippy limpio. Leído `orchestrator.rs` — concurrencia real confirmada (`JoinSet::spawn` + permiso `Semaphore` adquirido DENTRO de la tarea + `Arc<dyn BulkSource>` clonado por tarea + recolección fail-fast). Test `concurrent_downloads_respect_max_limit` ahora honesto: `FakeCountingBulkSource` mide el pico con atómicos y exige `2 <= peak <= 3` (fallaría con el código secuencial viejo).
- 2026-06-27 · QA-Engineer (Sonnet) · **APTO.** Re-gate independiente: concurrencia genuina (no teatro) verificada con razonamiento del runtime multi-thread; 10/10 criterios verdes; sin fuga de permisos (Drop de `OwnedSemaphorePermit`), sin deadlock, fail-fast con `abort_all` al salir del JoinSet, sin panic no capturado; FCIS intacto; lección §3 ya enseña el patrón correcto. 2 observaciones NO bloqueantes: (a) el Criterio 3 lo cubren dos tests (dominio + integración), el segundo se llama `fetch_aborts_when_orchestrator_detects_insufficient_disk`; (b) retries Bulk fijos en 3, anotados en código para escalamiento.
- 2026-06-27 · Tech-Lead · **STORY-024 SELLADA (APTA).** TTR-001 + TTR-002 cerrados. Feature spec y lección ya selladas. Observación (a) de QA incorporada al Criterio 3. Observación (b) registrada en §8 como pendiente de decisión. ROADMAP/PROGRESS actualizados.

## 8. Pendientes derivados / decisiones

**Correcciones del Gate de Coherencia aplicadas por el Tech-Lead a `docs/features/sovereign-data-fetcher.md` (2026-06-27):**
- Eliminada la fila duplicada de `data_snapshot_id` en la tabla de persistencia. El concepto "URL/endpoint de la fuente" pasó a campo propio de dominio documentado: `source_endpoint` (provenance, fuera del catálogo de 25, justificado por soberanía de datos).
- Eliminado `execution_latency_ms` de la tabla (era Grupo V, ajeno al Perfil A). La duración/progreso de la descarga la llevan el registro `Job` de `async-job-executor` y la telemetría de `background-download-manager`. Nota de perfil añadida al spec.
- Añadida la sección `## Puertos de Integración` (faltaba): salidas `ticks_out` (`Tick`) y `bars_out` (`Bars`), nodo fuente sin input de canvas. Tipos ya presentes en el catálogo de ADR-0137.

**Corrección de clasificación UI (2026-06-28, por reto del usuario):** STORY-024 se selló describiendo al fetcher como "plomería sin UI". **ERROR.** El fetcher produce tipos de dominio (`Tick`/`Bars`) y toma configuración del usuario (broker/símbolo/fechas/timeframe), por lo que es un nodo del canvas con **Superficie propia = Inspector Panel** (ADR-0136). STORY-024 entregó correctamente el **MOTOR de descarga** (TTR-001/002, backend, QA APTO); la **UI del inspector panel queda como entrega pendiente** (Story de UI futura: UI-Designer → Bridge → Flutter). El doc de la feature se corrigió añadiendo su Contrato de Integración UI. El progreso de descarga lo da `background-download-manager`; la exploración, `canvas-navigation`. Lección grabada en `tech-lead/SKILL.md` (test de clasificación plomería vs Superficie propia).

**Decisiones pendientes (registradas en el cierre):**
- **Reintentos Bulk fijos en 3** (no configurables): el código (`orchestrator.rs`) lo documenta como deuda anotada — `delta_sync_retry` aplica solo al tramo REST (throttling variable por broker); los servidores de archivos estáticos del Bulk no tienen throttling. Si en el futuro se requiere configurarlos, introducir `bulk_download_retry: u32` en `FetcherConfig` — decisión del Tech-Lead/Architect, no se introdujo ahora para no añadir un parámetro sin demanda real (ADR-0008: solo se parametriza lo que lo necesita).
- **Corrección de la Orden al sellar:** la instrucción §4.1 decía crear la migración en `crates/shared/migrations/`; lo correcto (ADR-0006) es la carpeta raíz `./migrations/`. El Ingeniero ya la creó en la ruta correcta; la instrucción quedó corregida.

**Diferidos (otra Story / épica futura):**
- TTR-003 (Alternative Data Webhook Listener) y TTR-004 (Alternative Time-Series Converter) — datos alternativos asíncronos (sentimiento, noticias). Alcance mayor; se programan cuando el pipeline de datos crudos esté estable (candidato EPIC-3+, depende del generador/motor que los consuma).
- Prueba de integración contra el bucket real de Binance Vision (red en vivo) — no apta para CI (no determinista, lenta). Las pruebas de esta Story usan adaptadores falsos. La validación contra la fuente real queda como verificación manual opcional.

**Barrido ADR completo (regla del usuario 2026-06-27) — qué se revisó e incorporó:**
Se leyó el índice `docs/ADR.md` y se abrieron bajo demanda los candidatos por dominio (datos/ingesta), capa (persistencia, seguridad, async) y transversales. ADRs que aplicaban y NO estaban en la Orden original, ahora incorporados:
- **ADR-0011 (Async Job Pattern):** la descarga es una operación costosa → se modela como trabajo asíncrono durable (tipo `Job`, recuperación tras crash). TTR-006 ya lo insinuaba ("process_id único y persistente"). Incorporado al prompt §4.5 + criterio #10.
- **ADR-0105 (100% Polars) + SAD §8:** los datos pesados se manejan con Polars/Arrow, no `Vec<struct>`. **Reveló que la Orden original se contradecía con esto** (pedía parsear a structs `Tick`/`Bars`). Corregido: el fetcher emite datos crudos para la capa de normalización; el parseo a Polars es del transformador (TTR-007). `Tick`/`Bars` quedan como marcadores de puerto (stubs), no se pueblan aquí.
- **ADR-0093 (Seguridad Soberana):** credenciales AES-256-GCM. Aplica como DIFERIDO — esta Story usa solo datos públicos sin credenciales; la gestión segura de claves llega con `sovereign-security` + fuentes autenticadas.
- **ADR-0008 (Configurabilidad Universal):** CONCURRENT_DOWNLOADS/DELTA_SYNC_RETRY configurables, no hardcodeados. Incorporado.
- **ADR-0012 (Multi-Pipeline Paralela):** concurrencia consciente de recursos. Incorporado.
- Adyacentes NO de esta Story (anotados): ADR-0035/0036 (persistencia Hive/Parquet + DuckDB) → del partition-manager/resampler aguas abajo; ADR-0066 (Fail-Fast) → protocolo dedicado en ingest TTR-999.

**Impacto en el SAD:** **sin cambio necesario.** SAD §8 (Arquitectura de Datos) ya declara descargas como `jobs` con auto-recuperación (alineado con ADR-0011) y "Polars/Arrow para mover OHLCV" (alineado con ADR-0105). La corrección de la Orden la ALINEÓ hacia el SAD existente; no fue el SAD el que quedó desalineado.

**Decisión de diseño registrada:**
- `Tick`/`Bars` viven en `crates/shared/src/types/mod.rs` (no en el crate de la feature) porque el invariante ADR-0137 prohíbe que un crate de feature dependa de otro: los tipos de puerto del catálogo viven en `shared` para que cualquier feature los produzca/consuma. En esta Story quedan como stubs/marcadores; su contrato físico (Polars/Arrow) lo define el transformador aguas abajo.
