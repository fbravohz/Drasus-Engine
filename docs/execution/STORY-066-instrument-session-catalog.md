# STORY-066 · Catálogo de Instrumento y Sesión de mercado

> **Orden de Trabajo (Spec-Driven).** Especificación ejecutable + comandos de validación + registro. Vive en git, NO en el chat.

| Campo | Valor |
|---|---|
| **ID** | STORY-066 |
| **Título** | Catálogo de Instrumento y Sesión de mercado |
| **Tipo** | Story |
| **Épica (Fase)** | EPIC-1 — Soberanía de Datos (`ingest`) |
| **Sprint** | Ingesta — primera entrega de código |
| **Estado** | ✅ Implementado |
| **Responsable** | Rust-Engineer (Sonnet, Docente) · auditó Tech-Lead + QA-Engineer (APTO) |
| **Creada** | 2026-07-25 |
| **Completada** | 2026-07-25 |

## 0. Resumen ejecutivo

- **Qué problema resuelve:** hoy no existe ninguna tabla de instrumentos, brókers ni calendarios de sesión — sin eso, la ingesta del CSV real no tiene contra qué resolver el piso de spread, el guardián de calendario ni la asociación de importación.
- **Qué se va a construir:** (1) el catálogo canónico de instrumentos con herencia por bróker (`instrument-catalog`); (2) el catálogo de plantillas de calendario de mercado con zona horaria declarada (`session-catalog`); (3) la migración con sus 4 tablas; (4) la superficie CLI de verificación de ambos.
- **Por qué ahora:** es el cimiento que desbloquea toda la cadena de ingesta (lector CSV → sanitizer → validador PIT → particionado). Es la primera entrega de código de EPIC-1 tras sellar el diseño (ADR-0162).

---

## 1. Especificación de origen (qué specs implementa)
- **Feature(s):** [`instrument-catalog`](../features/instrument-catalog.md), [`session-catalog`](../features/session-catalog.md)
- **TTR(s):** instrument-catalog TTR-001/002/003 · session-catalog TTR-001/002/003
- **Módulo(s):** [`ingest`](../modules/ingest.md)
- **ADR(s):** ADR-0162 (fuente principal — esquema, jerarquía, herencia por `NULL`, zona IANA), ADR-0141 (molde de tipos + semántica de secuencia `row_version`), ADR-0140 (enum `Instrument` activado), ADR-0155 (clave de negocio `instrument_id`), ADR-0137 (puertos), ADR-0020 (Perfil A + Grupo I), ADR-0002 (FCIS), ADR-0133 (pirámide de pruebas + gate de mutación), ADR-0003 (tabla por feature)

## 2. Objetivo (una frase llana)
Que el sistema tenga un catálogo de qué instrumentos existen, con qué condiciones de trading por bróker, y con qué calendario de mercado — todo declarado por el usuario, con herencia para no duplicar nada.

## 3. Agentes y Modo de Acompañamiento (ADR-0120)

| Agente | Etapa del pipeline | Depende de | Modo |
|---|---|---|---|
| **Rust-Engineer** (Sonnet) | Etapa 1/2 — núcleo + persistencia | ninguno | **Docente** |
| **QA-Engineer** | **Etapa 5 — gate obligatorio** | Rust-Engineer | **Autónomo** |

> **Docente (ADR-0122):** el Rust-Engineer implementa el bloque completo por su cuenta (`Edit`/`Write`) **y** escribe la lección en `docs/lessons/rust/STORY-066-instrument-session-catalog.md` explicando cada decisión con profundidad cero-conocimiento. El usuario NO teclea código.
> **UI:** ambas features son **Ventana de Verificación** (plomería de catálogo, sin superficie propia en el MVP). NO se despacha UI-Designer/Bridge/Flutter. El observable de cara al usuario se delega a `data-import-wizard` (Story futura, superficie propia). Deuda de integración registrada en §8.
> **Gate de mutación:** lo corre el Tech-Lead (`scripts/mutation-gate.sh`), nunca el ingeniero ni el QA.

## 4. Instrucciones de despacho por agente (la spec ejecutable)

### 4.1 Rust-Engineer (Modo Docente)
```
CONTEXTO Y ALCANCE
Construyes dos features hexagonales de catálogo en `crates/features/data/`:
`instrument-catalog` y `session-catalog`, más su migración SQL compartida
(`migrations/0022_instrument_session_catalog.sql`) con las 4 tablas que ADR-0162
especifica. Es la primera entrega de código de la ingesta (EPIC-1). Modo Docente:
implementas todo tú y escribes la lección profunda en
`docs/lessons/rust/STORY-066-instrument-session-catalog.md`.

EL PROBLEMA
El sistema no tiene forma de saber qué instrumentos existen, con qué condiciones
de trading los ofrece cada bróker, ni con qué calendario abre cada mercado. Sin
ese catálogo, la ingesta del CSV real no puede resolver el piso de spread por
instrumento, ni comparar la huella de calendario del archivo, ni asociar el
archivo importado a un instrumento/bróker/sesión.

RESTRICCIONES E INVARIANTES DUROS (no negociables)
1. Molde de tipos ADR-0141 SIN excepción: precio/spread/pip/tick como INTEGER
   escala ×10⁸ (nunca REAL); timestamps INTEGER ns UTC; PK `id` TEXT UUIDv7;
   `STRICT` en las 4 tablas; enums con CHECK; FK con ON DELETE RESTRICT (jamás
   CASCADE); índice en cada FK.
2. Las 4 tablas son Referencia/Mutable → campo de secuencia del Grupo I es
   `row_version` (empieza en 1, se incrementa con cada UPDATE), NUNCA
   `event_sequence_id`. `audit_chain_hash` NULL en la fila génesis (sin centinela).
   Esta es una corrección explícita del Architect en ADR-0162 — respétala.
3. Herencia por `NULL` explícito en `broker_instrument_terms`: `NULL` significa
   "hereda el valor canónico de `instrument_master`", NUNCA "dato faltante por
   error". La resolución combina campo a campo, el override no-NULL gana.
4. `instrument_master` lleva DOS columnas de identidad con roles distintos:
   `id` (UUIDv7, clave técnica, cumple la PK universal de ADR-0141) y
   `instrument_id` (TEXT NOT NULL UNIQUE, clave de negocio — la misma cadena
   "EURUSD" que ya circula en el esquema de mercado de ADR-0155). Las FK de
   `broker_instrument_terms` referencian la clave de NEGOCIO, no el UUID técnico.
5. `market_timezone_ref` persiste un identificador IANA (`America/New_York`),
   NUNCA una convención de desplazamiento inventada tipo "EST+07". La conversión
   de zona ocurre en tiempo de LECTURA (no se persiste ningún UTC convertido).
6. Las plantillas de sesión NUNCA se copian por bróker×símbolo — se referencian
   por FK desde cuantos instrumentos/overrides las compartan.
7. Escrituras (alta/edición) atómicas: transacción `BEGIN IMMEDIATE` + fila de
   auditoría en la misma transacción; concurrencia optimista por `row_version`
   con error tipado `VersionConflict` cuando la versión leída ya no es la actual
   (verifica `rows_affected`, no confíes en el UPDATE ciego — este es el bug real
   que el QA cazó en `central-identity`/STORY-027).
8. FCIS (ADR-0002): el Core es lógica pura sin I/O. Para `instrument-catalog` el
   Core es la resolución `COALESCE` campo-a-campo. Para `session-catalog` el Core
   es la derivación de la huella de calendario esperada a partir de las ventanas
   semanales. Toda persistencia/UUID/reloj/zona vive en el Shell.
9. Puertos ADR-0137 tal como los declaran las features:
   `instrument-catalog` → `resolved_terms_out` (`ResolvedInstrumentTerms`);
   `session-catalog` → `session_calendar_out` (`SessionCalendar`) +
   `expected_fingerprint_out`. Expón cada feature en `public_interface`.
10. `swap_type` NO tiene vocabulario CHECK verificado todavía (ADR-0162 lo difiere
    explícitamente): déjalo TEXT sin CHECK, documentado como deferido — NO inventes
    un CHECK sin fuente. Repórtalo como pendiente si necesitas cerrarlo.

DOCUMENTACIÓN EXACTA A CONSULTAR (no leas de más)
- ADR-0162 (`docs/adr/ADR-0162.md`) — esquema completo de las 4 tablas, la
  jerarquía, la herencia, el Gate de Coherencia (aquí solo produces la huella
  ESPERADA; el gate observado-vs-esperado lo consume `pit-data-validator`, no es
  de esta Story), y las 3 "Corrección del Architect".
- `docs/features/instrument-catalog.md` y `docs/features/session-catalog.md` —
  TTRs, puertos, FCIS, comportamientos observables, parámetros configurables.
- ADR-0141 §"Semántica de secuencias" y checks M1–M12 (`docs/adr/ADR-0141.md`).
- Patrón de referencia YA construido: `crates/features/data/hybrid-data-transformer`
  y los cimientos con `row_version` + `VersionConflict` en `crates/shared`
  (ej. `plan-tier-quota`/#3, `central-identity`/#1) — reusa el patrón de
  concurrencia optimista y de transacción atómica, no lo reinventes.
- Plantilla de ficha de modelo de datos: `docs/templates/DATA-MODEL.md`.

ENTREGABLE Y SU FORMA
1. `migrations/0022_instrument_session_catalog.sql` — las 4 tablas
   (`instrument_master`, `broker_instrument_terms`, `session_template`,
   `session_template_window`) exactamente con el esquema de ADR-0162.
2. Dos features en `crates/features/data/instrument-catalog` y
   `crates/features/data/session-catalog`: Core puro + Shell + `public_interface`.
   Decide tú la estructura interna de módulos siguiendo el patrón hexagonal ya
   establecido en el repo.
3. Semilla de plantillas de sesión (`SESSION_TEMPLATE_SEED_SET`), confirmada por el
   propietario 2026-07-25 — TRES plantillas: (a) Forex 24/5 (domingo→viernes, un
   hueco semanal); (b) Cripto 24/7 (continua, sin huecos); (c) Futuros estilo CME
   Globex — zona `America/Chicago`, forma "hueco diario de mantenimiento + hueco
   semanal" (ADR-0162 tabla de huellas). La plantilla de Futuros es un default
   REPRESENTATIVO y EDITABLE, no una verdad universal: los calendarios de futuros
   son específicos del mercado/producto. Usa una ventana de referencia razonable
   (apertura domingo, cierre viernes, con la pausa diaria de mantenimiento) y
   documéntala como default editable; NO inventes horas minuto-a-minuto como si
   fueran hecho sellado — si dudas del valor exacto, márcalo como default a
   confirmar. Las tres plantillas ejercen las tres formas de huella distintas
   (continua / hueco semanal / hueco semanal+diario).
4. Superficie CLI de verificación (`verify instrument-catalog`,
   `verify session-catalog`) que devuelva el JSON del observable real.
5. Fichas del modelo de datos: `docs/data-model/<tabla>.md` para las 4 tablas +
   filas nuevas en el índice `docs/DATA-MODEL.md`, columna por columna contra la
   migración real (NO contra este prompt).
6. Pruebas (pirámide ADR-0133) cubriendo CADA criterio de aceptación de §5, cada
   una discriminante (roja sin la implementación, verde con ella). Entrega ya en
   verde con el mapeo criterio→prueba + cobertura (`cargo llvm-cov`).
7. Lección Docente en `docs/lessons/rust/STORY-066-instrument-session-catalog.md`.

MÉTODO
El método de diseño interno es TUYO — eres el especialista. Si algo del encuadre
anterior te parece frágil, incompleto o en conflicto con la doc o el dominio,
DILO y propón lo correcto ANTES de implementar (GOVERNANCE-4.7). No ejecutes una
premisa defectuosa por venir en el prompt.
```

**Plan de Implementación / Revisión** (lo llena el Rust-Engineer al ser invocado — Modo Docente: resumen de lo implementado + puntero a la lección):

Entregado 2026-07-25. Migración `migrations/0022_instrument_session_catalog.sql` (4 tablas, orden `session_template`→`session_template_window`→`instrument_master`→`broker_instrument_terms`). Dos crates hexagonales nuevos: `crates/features/data/instrument-catalog` (Core: `resolve_terms` COALESCE puro; Shell: `InstrumentRepository` con `create_master`/`update_master`/`create_broker_terms`/`delete_broker_terms`) y `crates/features/data/session-catalog` (Core: `compute_expected_fingerprint`, `resolve_session_template_id`; Shell: `SessionRepository` con `BEGIN IMMEDIATE` para plantilla+ventanas atómicas). Semilla `SESSION_TEMPLATE_SEED_SET` con 3 plantillas (Forex 24/5, Cripto 24/7, Futuros CME Globex representativo) vía `seed_default_templates`, idempotente. CLI `verify instrument-catalog`/`verify session-catalog` cableados en `crates/app` y probados con datos reales (ver §6). Fichas de modelo de datos + índice `docs/DATA-MODEL.md` actualizados. Lección Docente completa en [`docs/lessons/rust/STORY-066-instrument-session-catalog.md`](../lessons/rust/STORY-066-instrument-session-catalog.md). Premisa retada (GOVERNANCE-4.7) documentada en `crates/features/data/session-catalog/src/lib.rs`: `ResolveSession` no lee las tablas de `instrument-catalog` (violaría el aislamiento de crates de ADR-0137) — recibe los IDs override/default ya resueltos como parámetros.

## 5. Criterio de aceptación (cada criterio ↔ su prueba)

| # | Criterio verificable | Prueba que lo demuestra |
|---|---|---|
| 1 | Alta de instrumento canónico persiste con `id` UUIDv7 + `instrument_id` de negocio único; un `instrument_id` duplicado se rechaza | `instrument_master_rejects_duplicate_business_key` |
| 2 | Resolución de herencia: un campo `NULL` en el override devuelve el valor canónico; uno no-`NULL` devuelve el del override (discriminante campo a campo) | `resolve_terms_null_inherits_nonnull_overrides` |
| 3 | Override sparse: borrar el override de un bróker hace que el instrumento resuelva 100% desde el canónico sin tocar `instrument_master` | `deleting_override_resolves_from_canonical` |
| 4 | `session_template` + ventanas persisten; la zona horaria se guarda como identificador IANA, no como convención de desplazamiento | `session_template_persists_iana_timezone` |
| 5 | Derivación de huella de calendario, discriminante por forma de mercado: Cripto 24/7 → continua; Forex 24/5 → hueco semanal; Futuros → hueco semanal + hueco diario de mantenimiento | `fingerprint_distinguishes_crypto_forex_futures` |
| 6 | `ResolveSession` sigue la cadena override de bróker → default de instrumento | `resolve_session_follows_override_chain` |
| 7 | Escritura atómica + concurrencia optimista: un UPDATE con `row_version` obsoleto falla con `VersionConflict` (no pisa en silencio) | `stale_row_version_yields_version_conflict` |
| 8 | Esquema conforme a ADR-0141: un `asset_class`/`commission_model`/día fuera del CHECK es rechazado por la base | `check_constraints_reject_invalid_enum` |
| 9 | CLI `verify instrument-catalog` y `verify session-catalog` devuelven el JSON correcto del observable | reproducción manual del Tech-Lead + `cargo run -p app -- verify ...` |
| 10 | Fichas `docs/data-model/*.md` de las 4 tablas creadas + filas en el índice, columna por columna contra la migración | auditoría documental del Tech-Lead (grep + contraste) |

## 6. Comandos de validación (para el usuario — copy/paste)
```bash
cargo test -p instrument-catalog -p session-catalog
cargo clippy --workspace --all-targets -- -D warnings
cargo llvm-cov --workspace --summary-only
cargo run -p app -- verify instrument-catalog --input '{"instrument_id":"EURUSD","venue":"PEPPERSTONE"}' | jq .
cargo run -p app -- verify session-catalog --input '{"template_code":"FOREX_245"}' | jq .
```

## 7. Registro de ejecución (bitácora cronológica)
- 2026-07-25 · Tech-Lead · Orden creada · Gate de Coherencia Pre-Despacho: limpio (ver §8). Pendiente de despacho al Rust-Engineer (Docente).
- 2026-07-25 · Rust-Engineer (Docente) · Entrega completa: migración `0022`, crates `instrument-catalog`/`session-catalog`, semilla de 3 plantillas, CLI `verify` cableado y probado con datos reales, 4 fichas `docs/data-model/*.md` + índice, lección Docente. 47/47 pruebas en verde (22 + 25), workspace completo 857/857 sin regresiones, `cargo clippy --workspace --all-targets -- -D warnings` limpio, cobertura por archivo ≥83% (domain.rs 100%/95.97%, persistence 97%+). Demostración de discriminación (rojo→verde) ejecutada para los 8 criterios verificables por código (§5 #1–#8). Pendiente: Tech-Lead reproduce evidencia y corre el gate de mutación (`scripts/mutation-gate.sh`) antes de despachar a QA.
- 2026-07-25 · Tech-Lead · Gate de mutación NO APTO: 4 supervivientes en `instrument-catalog` + 17 en `session-catalog` (estándar 0). Diagnóstico entregado agrupado en 3 hallazgos: (A) fidelidad del struct devuelto por `update_master` (`..current.clone()` con campos sobrescritos sin prueba que los lea del valor devuelto); (B) guarda `&&`/`||` del clasificador de duplicado, sin prueba que fuerce el caso donde solo una mitad es verdadera (violación de PK vs. de clave de negocio); (C) aritmética interna de `compute_expected_fingerprint` (fusión cíclica de huecos, aritmética de borde `end+1` en ambas ramas, umbral de hueco grande y su frontera, "todos los huecos grandes") sin pruebas dedicadas más allá de las 3 formas gruesas de las plantillas semilla.
- 2026-07-25 · Rust-Engineer (Docente) · Corrección de los 3 hallazgos: 9 pruebas nuevas (2 en `instrument-catalog`, 7 en `session-catalog`), cada una con demostración rojo→verde contra el mutante real (línea exacta rota, confirmado el fallo, revertida, confirmado el verde). `instrument-catalog` pasó de 22 a 24 pruebas; `session-catalog` de 25 a 32 (cobertura alta de `domain.rs`; **corrección del Tech-Lead sobre la cifra citada por el ingeniero:** el QA reprodujo 78.6% líneas / 83.5% regiones con el comando estándar, no "100%" — la diferencia son artefactos de `llvm-cov` sobre doc-comments, líneas en blanco y sub-regiones de closures; cada línea con lógica ejecutable tiene conteo >0). Workspace completo re-verificado en verde (`cargo build`/`clippy -D warnings`/`test --workspace`, 0 regresiones). Lección Docente ampliada con una sección nueva sobre diseño de pruebas dirigido a matar mutantes concretos, no solo a cubrir criterios de aceptación. Pendiente: Tech-Lead re-corre el gate de mutación.
- 2026-07-25 · Tech-Lead · Segundo re-corte del gate: `instrument-catalog` APTO (0 supervivientes). `session-catalog` bajó de 17 a 2, ambos en `compute_expected_fingerprint`: mutante 187 (`gaps.push(first + last)` → `first * last`) MATABLE con una construcción donde suma y producto caen en lados opuestos del umbral de 1200 (la prueba de fusión previa, 700+700, no lo mataba porque suma y producto quedaban del mismo lado); mutante 196 (`size >= LARGE_GAP_THRESHOLD_MINUTES` → `<`) EQUIVALENTE VERDADERO verificado por el Tech-Lead (la condición de clasificación es invariante bajo `c ↔ len - c`).
- 2026-07-25 · Rust-Engineer (Docente) · Prueba `fingerprint_wraparound_merge_uses_sum_not_product_of_its_two_halves` (mitades 500/600, hueco medio 1300) añadida y demostrada rojo→verde contra la línea 187 real. Mutante 196 documentado como equivalente con la nota exacta provista por el Tech-Lead, junto a la línea, sin intentar matarlo. `session-catalog` pasó de 32 a 33 pruebas. Workspace completo re-verificado en verde. Cierre esperado: `instrument-catalog` 0 supervivientes, `session-catalog` 1 superviviente (196, documentado como equivalente).
- 2026-07-25 · Tech-Lead · **Tercer re-corte del gate: CERRADO.** `session-catalog` = 1 superviviente único (`domain.rs:203`, el mutante equivalente `>=`→`<`, con su demostración escrita junto a la línea — verificada por el TL). `instrument-catalog` ya en 0. Estándar cumplido (0 supervivientes no-equivalentes). Reproducción independiente del TL previa: 24+33 pruebas verdes, clippy `-D warnings` limpio, esquema conforme ADR-0141 (STRICT×4, `row_version`, FK RESTRICT, escala ×10⁸, sin REAL), FCIS limpio, CLI `verify` real correcta (pip=10000, fingerprint WEEKLY_GAP), 4 fichas de modelo de datos.
- 2026-07-25 · QA-Engineer (Autónomo) · **APTO.** Reprodujo por su cuenta: 57 pruebas verdes, clippy limpio, build sin regresiones, CLI `verify` de ambas features. Revisó los 4 focos (herencia COALESCE sin confusión `None`/`Some(0)` por tipado `Option`; concurrencia `row_version` con `rows_affected` real; zona IANA persistida sin conversión; huella de calendario correcta) + verificó la demostración de equivalencia del mutante 196. Los 10 criterios con prueba nombrada y discriminante; 4 fichas coinciden columna por columna con la migración; `PRAGMA foreign_keys=ON` activo (FK reales). 3 observaciones NO bloqueantes (cifra de cobertura reproducible ≠ "100%" citado — corregida arriba; 2 caminos de error triviales sin prueba). Sin hallazgos bloqueantes.
- 2026-07-25 · Tech-Lead · **Story CERRADA.** Todos los gates pasados. Documentos fuente sellados (features + ADR-0162 + ROADMAP). Pendiente único: commit agrupado (autorización del propietario).

## 8. Pendientes derivados / decisiones

**Gate de Coherencia Pre-Despacho (resultado):**
- **Contraste bidireccional:** feature/ADR/SAD coherentes — el diseño lo selló el Architect el 2026-07-24 con 3 correcciones documentadas (identidad clave técnica vs. negocio; `row_version` vs. `event_sequence_id`; zona IANA vs. "EST+07"). Sin escalamiento.
- **Etapa 0.3 (viabilidad):** Quant satisfecho por la sesión 2026-07-24 (la política de spread/calendario, la parte con carga cuantitativa, se selló y vive en el sanitizer, Story posterior); el catálogo en sí es dato de referencia sin showstopper cuantitativo independiente. General-Counsel: **No Aplica** (metadatos de instrumento/calendario declarados por el usuario; sin PII, pagos, asesoría ni flujo transfronterizo).
- **Etapa 0.4 (coherencia Architect):** satisfecha por la sesión 2026-07-24 — ADR-0162 ES el barrido (puertos, FCIS, esquema, correcciones). No se re-despacha.
- **ADR-0020:** ambas Perfil A, Grupo I con `row_version` (corrección del Architect aplicada). Campos dentro de catálogo. ✅
- **ADR-0003 (tabla por feature):** `instrument-catalog` posee `instrument_master` + `broker_instrument_terms` (padre + override sparse); `session-catalog` posee `session_template` + `session_template_window` (padre + ventanas hijas). Patrón padre+hijo, no duplicación cruzada — coherente con features existentes (ej. `verified-account-registry`). ✅
- **Clasificación UI (ADR-0117):** ambas **Ventana de Verificación**. Reciben configuración del usuario, pero su superficie de captura vive embebida en `data-import-wizard` (feature consumidora con superficie propia, construida después), no como nodo Canvas de primera clase en el MVP. Decisión deliberada y documentada del Architect. Story backend-only.

**Deuda de integración registrada:**
- **UI de catálogo delegada a `data-import-wizard`:** la creación/edición de instrumentos, brókers y plantillas de sesión de cara al usuario se materializa cuando se construya `data-import-wizard` (asociación bróker↔instrumento↔sesión durante la importación). Hasta entonces, el catálogo se puebla por semilla + CLI. Observable verificable por CLI `verify` + Banco de Pruebas (SVF genérico).

**Deferidos de diseño (de ADR-0162, no de esta Story):**
- Vocabulario CHECK de `swap_type` (sin fuente verificada aún — queda TEXT sin CHECK).
- Migración de `slippage-models`/`backtest-engine` desde sus `CONFIG` globales a términos resueltos por el catálogo (Story futura).
- **`underlying_instrument_id` posiblemente ausente de `instrument_master` (a escalar al Architect):** el ingeniero observó que ADR-0155 documenta `underlying_instrument_id` para Opciones, pero ADR-0162 enumera solo 8 atributos de Opciones/Futuros, no 9 — no lo incluyó (correcto: no inventar lo que el ADR no lista). Si fue omisión del ADR, es del Architect corregirlo. Anotado en la ficha `instrument_master.md`. NO bloquea (Opciones está diferido por ADR-0140). **Consecuencia:** TASK a abrir para que el Architect confirme si el catálogo de Opciones necesita esa columna antes de EPIC de Opciones.
- ~~Alcance de día 1 del `SESSION_TEMPLATE_SEED_SET`~~ → **RESUELTO por el propietario 2026-07-25: Forex 24/5, Cripto 24/7 y Futuros (CME Globex representativo, editable).** Los calendarios de futuros específicos por producto se refinan por el usuario sobre la semilla. Reflejado en `session-catalog.md` y en §4.1 de esta Orden.

**SAD:** impacto bajo — 4 tablas de referencia que siguen el patrón de esquema ya descrito en SAD-08; el detalle a nivel tabla vive en `docs/DATA-MODEL.md` (se actualiza en esta Story). Verificar en el cierre si SAD-16 (grafo de dependencias) requiere la arista `ingest`→catálogo.

## 9. Cierre ejecutivo (para el usuario — CEO)

ESTADO: 🟢 COMPLETADO

PROGRESO MACRO:
- El sistema ya tiene su catálogo de instrumentos y de calendarios de mercado: sabe qué es EURUSD, con qué condiciones lo ofrece cada bróker (con herencia, sin duplicar nada), y con qué calendario abre cada mercado (Forex, Cripto y Futuros de fábrica). Es el cimiento del que dependen el lector del CSV real, el piso de spread y el guardián de calendario — sin esto, la ingesta no tenía contra qué resolver nada.

FRICCIONES Y DEUDA:
- La prueba de mutación obligó a endurecer las pruebas en dos rondas (encontró holgura, no errores del producto); quedó todo cerrado. Un único caso restante es matemáticamente imposible de fallar y quedó documentado como tal, con la demostración verificada.
- La pantalla para que el usuario cree instrumentos y calendarios llegará con el asistente de importación (Story posterior); por ahora se puebla de fábrica y se verifica por línea de comandos.
- Un posible campo faltante para Opciones quedó anotado para revisión del Architect — no bloquea nada (Opciones está diferido).

INPUT REQUERIDO DEL CEO:
- Autorización para commitear la entrega (agrupada por tipo). Es lo único pendiente.
