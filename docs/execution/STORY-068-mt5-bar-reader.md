# STORY-068 · Lector del archivo de barras de MetaTrader — de archivo real a esquema canónico

> **Orden de trabajo — Spec-Driven.** Fuente de verdad de la ejecución. Vive en git, no en el chat.

| Campo | Valor |
|---|---|
| **ID** | STORY-068 |
| **Título** | Lector del archivo de barras de MetaTrader — de archivo real a esquema canónico |
| **Tipo** | Story |
| **Épica (Fase)** | EPIC-1 — Soberanía de Datos (`ingest`) |
| **Sprint** | 2 |
| **Estado** | ✅ **CERRADA** — QA APTO · gate de mutación 0 supervivientes (2ª ronda) |
| **Responsable** | Rust-Engineer (Sonnet, **Modo Docente**) · auditan Tech-Lead + QA-Engineer |
| **Creada** | 2026-07-25 |
| **Completada** | 2026-07-25 |

---

## 0. Resumen ejecutivo

Primera vez que dato de mercado real entra a Drasus. El propietario exportó de su terminal de MetaTrader un archivo de 9.2 millones de barras de un minuto de EURUSD, de 2000 a 2025. Esta Story construye la pieza que lo lee y lo convierte al esquema canónico del sistema **sin inventar un solo dato**, y que sella la declaración del usuario como evidencia inmutable antes de persistir nada.

**Alcance deliberadamente acotado al archivo de barras.** El archivo de ticks del mismo terminal queda para la Story siguiente: su dificultad real no es leer el archivo sino reconstruir la cotización de dos lados a partir de actualizaciones de un solo lado (ADR-0155, enmienda 2026-07-25), y esa pieza vive en otra feature (`data-normalization-layer`). Mezclarlas produciría una Story de dos hexágonos, contra ADR-0137.

---

## 1. Especificación de origen (qué specs implementa)

- **Feature:** [`data-import-wizard`](../features/data-import-wizard.md) — TTR-001 (motor de inferencia de esquema) y TTR-002 (ingestor masivo de archivo plano), acotados a la vía de plantilla de plataforma.
- **Feature tocada por retrofit:** [`instrument-catalog`](../features/instrument-catalog.md) — campo `real_volume_unit_declared` canonizado pero aún ausente de la migración.
- **ADRs rectores:** ADR-0154 (Drasus da conectores, no datos) · ADR-0155 (esquema canónico) · ADR-0156 (formato estándar de ingesta: rechazo por defecto, plantillas, mapeo declarado como evidencia) · ADR-0162 (catálogo instrumento×bróker×sesión, zona horaria declarada) · ADR-0163 (reimportación idempotente) · ADR-0141 (modelado relacional soberano) · ADR-0020 (contrato de persistencia) · ADR-0137 (hexágono con puertos tipados) · ADR-0117 (superficie de verificación en la misma Story) · ADR-0133 (pirámide de pruebas + gate de mutación).
- **Veredicto legal vinculante:** [LDR-0008](../legal/LDR-0008.md) — las cinco condiciones que sostienen la protección de responsabilidad son criterios de aceptación de esta Orden, no recomendaciones.
- **Gates previos:** Quant Etapa 0.3 (`.agents/plans/quant-2026-07-25-viabilidad-lector-mt5.md`, APTO-CON-CONDICIONES) · Architect Etapa 0.4 (`.agents/plans/architect-2026-07-25-barrido-lector-mt5.md`).

---

## 2. Objetivo (una frase llana)

Que el sistema lea el archivo real de barras de MetaTrader del propietario, lo convierta al formato interno sin adivinar nada, y deje constancia inmutable de qué declaró el usuario sobre ese archivo antes de guardar la primera fila.

---

## 3. Agentes y Modo de Acompañamiento (ADR-0120)

| Agente | Etapa | Depende de | Modo |
|---|---|---|---|
| **Rust-Engineer** | Etapa 2 — núcleo e implementación | — | **Docente** (implementa y escribe la lección) |
| **Tech-Lead** | gate de mutación + auditoría | Rust-Engineer | — |
| **QA-Engineer** | **Etapa 5 — gate obligatorio** | Rust-Engineer | Autónomo |

**Etapas no activadas y por qué:**

- **Etapa 0.5 (UI-Designer), 3 (Bridge) y 4 (Flutter):** la superficie visual del asistente de importación (arrastrar archivo, vista previa, confirmación del mapeo) es una pantalla compleja que merece su propio diseño. Esta Story entrega la verificación por el harness genérico ya existente (Banco de Pruebas) y el comando de verificación; **queda registrado en §8 como deuda de superficie**, a saldar en la Story del asistente visual.
- **Etapa 1 y 6 (Quant):** el Quant ya emitió veredicto en Etapa 0.3 sobre este archivo concreto. No hay fórmula nueva que auditar: esta Story es mapeo de campo y política de rechazo, cero cálculo derivado.

---

## 4. Instrucciones de despacho por agente (la spec ejecutable)

### 4.1 Rust-Engineer (Modo Docente)

Eres el **Rust-Engineer** de Drasus Engine. Antes de nada: lee `CLAUDE.md`, `.agents/knowledge/governance.md` completo, `.agents/skills/rust-engineer/SKILL.md` completo, y `.agents/knowledge/ledger-pattern.md` (vas a construir un ledger append-only — **ese documento es el molde canónico, nunca una feature ya implementada**). Declara cada lectura al terminarla.

**Modo Docente:** además de implementar, escribes la lección en `docs/lessons/rust/STORY-068-mt5-bar-reader.md`, explicando cada decisión con profundidad cero-conocimiento. El propietario aprende leyendo esa lección, no tecleando código.

#### El problema

El propietario exportó de su terminal de MetaTrader un archivo real: `datasets/EURUSD_M1_2000-01-01_2025-05-01.csv`. 9,231,695 filas de barras de un minuto, separado por tabulaciones, con finales de línea de Windows, cabecera `<DATE> <TIME> <OPEN> <HIGH> <LOW> <CLOSE> <TICKVOL> <VOL> <SPREAD>`. **Es tu insumo real y tu banco de pruebas — no un ejemplo.** Es grande: muestrea con herramientas de línea de comandos, nunca lo cargues entero para inspeccionarlo, y recorta los fragmentos que necesites como material de prueba dentro del repositorio.

Hoy el sistema no puede leerlo. Al terminar tu trabajo debe poder leerlo, convertirlo al esquema canónico, y haber dejado constancia inmutable de lo que el usuario declaró sobre ese archivo antes de guardar nada.

#### Restricciones duras (invariantes, no se debaten)

- **Nada se adivina.** Un archivo sin plantilla de plataforma reconocida y sin mapeo confirmado por el usuario se rechaza con motivo explícito. Nunca una importación parcial ni una interpretación inventada.
- **Se persiste el hecho, se calcula la interpretación.** Esta Story no calcula ni un promedio, ni un spread derivado, ni ningún indicador. Solo mapea campos. Es exactamente la clase de error que hundió una Story anterior.
- **Precios y magnitudes en entero escalado ×10⁸; tiempo en enteros de nanosegundos UTC. Nunca coma flotante en el dominio.**
- **Ningún campo que el archivo entregue se descarta.** El esquema es abierto por arriba.
- **La declaración del usuario es evidencia, no configuración:** vive en un ledger append-only, sellado ANTES de que se persista la primera fila de dato.
- **Fase greenfield:** el baseline de migraciones se edita in-situ. No apiles una migración correctiva sobre `0022` para un campo que falta — corrígela.

#### Lo que ya está decidido y NO se reabre

**Lee íntegra la sección "Preguntas resueltas antes de implementar" de `.agents/plans/architect-2026-07-25-barrido-lector-mt5.md`.** Son diez preguntas que un ingeniero se haría al construir esto, ya respondidas con cita del documento que las resuelve. Existe para que no improvises. Si crees que alguna respuesta está mal, **dilo antes de implementarla** — no la ejecutes a ciegas ni la contradigas en silencio.

Presta atención especial a: la unidad del volumen real nunca se asume (pregunta 3); el instrumento y su plantilla de sesión deben existir antes de importar (pregunta 9); la zona horaria de origen jamás se infiere de volatilidad ni de noticias (sección "Lo ya especificado").

#### Documentación a consultar

`docs/features/data-import-wizard.md` (tu feature), `docs/adr/ADR-0155.md` (esquema canónico — lee la tabla de `Bars` campo por campo), `docs/adr/ADR-0156.md` (formato de ingesta y las ocho restricciones), `docs/adr/ADR-0162.md`, `docs/adr/ADR-0141.md` §"Gate del Tech Lead" (los checks de migración M1–M12 y de shell R1–R7 son tu lista de verificación, no lectura opcional), `docs/legal/LDR-0008.md` (los cinco criterios que el veredicto legal añade al ledger), y los dos informes de gate citados arriba.

Como referencia de estructura hexagonal, los dos crates que se cerraron ayer en `crates/features/data/instrument-catalog` y `session-catalog` son el precedente vivo más cercano.

#### El entregable

Un crate hexagonal nuevo para la feature del asistente de importación, bajo `crates/features/data/`, con:

1. **La plantilla de plataforma de MetaTrader 5 para archivos de barras** — el mapeo de columnas resuelto y versionado. Recuerda que una plantilla la afirma Drasus, no el usuario: si está mal construida, responde Drasus (LDR-0008). Su corrección es tu responsabilidad, no la del usuario.
2. **El lector del archivo** que produce filas del esquema canónico de barra, con la identidad de serie completa (instrumento, plaza, proveedor, tipo de barra, temporalidad, referencia del conjunto de datos) tomada de los metadatos que el usuario declara, nunca del nombre del archivo.
3. **El ledger append-only de declaración de mapeo**, sellado antes de persistir dato, con todo lo que exigen `data-import-wizard.md` §Persistencia y los cinco criterios de LDR-0008. Sigue el molde canónico de `.agents/knowledge/ledger-pattern.md` íntegro, incluidas sus cuatro pruebas obligatorias.
4. **El retrofit del campo de unidad de volumen real** en la migración `0022` (edición in-situ, greenfield) más su ficha de modelo de datos actualizada.
5. **La ficha nueva de modelo de datos** de la tabla del ledger, y su fila en el índice.
6. **El subcomando de verificación** de la feature, con su entrada de ejemplo real.
7. **La lección Docente.**

Decide tú la estructura interna, los nombres y el reparto entre núcleo puro y cáscara — eso es tu oficio, no mi encargo. Lo que no es negociable son las restricciones de arriba y los criterios de aceptación de §5.

#### Cómo se prueba (regla del proyecto, no preferencia)

Cada criterio de aceptación de §5 se demuestra con **una prueba nombrada y discriminante**: debe fallar mientras el comportamiento no exista y pasar solo cuando esté bien implementado. Una prueba que pasa aunque el comportamiento esté ausente no cuenta como criterio cumplido. Demuestra el ciclo rojo→verde en tu reporte.

**No lances el gate de mutación** (`cargo-mutants`): lo corre el Tech-Lead. Tu presupuesto se gasta escribiendo pruebas, no esperando una corrida de CPU.

**Advertencia de método (`GOVERNANCE-4.7`):** este encargo te da el problema, las restricciones y las decisiones ya cerradas — deliberadamente **no** el método. Si el encuadre está mal planteado, dilo antes de ejecutarlo.

---

## 5. Criterio de aceptación (cada criterio ↔ su prueba)

| # | Criterio verificable | Prueba que lo demuestra |
|---|---|---|
| 1 | Una línea real del archivo, con su final de línea de Windows intacto, se parsea completa: el último campo (`SPREAD`) sale como entero, nunca como error ni como nulo silencioso | prueba de oráculo alimentada con una línea literal del archivo real, incluido el `\r` |
| 2 | Los cinco decimales de precio se convierten a entero escalado ×10⁸ sin pérdida, ida y vuelta | prueba de ida y vuelta sobre valores reales del archivo, incluidos los extremos observados |
| 3 | Fecha y hora del archivo producen un instante en nanosegundos UTC usando la zona horaria **declarada por el usuario**, y una declaración ausente hace fallar la importación en vez de asumir UTC | dos pruebas: una con zona declarada verifica el instante exacto; otra sin zona verifica el rechazo tipado |
| 4 | Un archivo sin plantilla reconocida y sin mapeo confirmado se **rechaza** con motivo explícito, sin persistir ni una fila | prueba que intenta importar un archivo de cabecera desconocida y verifica cero filas persistidas + error tipado |
| 5 | El ledger de declaración de mapeo queda sellado **antes** de que exista la primera fila de dato: si la persistencia de datos falla, la declaración ya está registrada y es recuperable | prueba que fuerza el fallo de la persistencia de datos y verifica que la fila del ledger sobrevive |
| 6 | El ledger es genuinamente inmutable: modificarlo o borrarlo aborta a nivel de base de datos, y su cadena de sellos encadena con la fila anterior | las cuatro pruebas obligatorias del molde de `.agents/knowledge/ledger-pattern.md`, sin omitir ninguna |
| 7 | Dos escritores concurrentes contra el ledger no corrompen la secuencia ni pierden un evento en silencio | prueba de contención sostenida hasta agotar reintentos, con clasificador de error transitorio ejercido directamente |
| 8 | El valor del volumen real se persiste crudo aunque su unidad no esté declarada, y la corrida queda marcada como no verificada en vez de afirmar una unidad | prueba con unidad sin declarar que verifica ambas cosas: valor intacto + marca presente |
| 9 | Una barra con movimiento de precio real y contador de actividad en cero **no** se persiste como vela ordinaria: queda marcada | prueba alimentada con las dos filas reales del archivo (2019-11-01) que exhiben esa imposibilidad |
| 10 | Importar dos veces el mismo tramo idéntico no duplica ni reescribe nada (no-op) | prueba de reimportación del mismo fragmento verificando conteo estable y contenido idéntico |
| 11 | El subcomando de verificación devuelve el JSON correcto contra el binario real | ejecución reproducida por el Tech-Lead, no solo prueba unitaria |
| 12 | La migración `0022` corregida cumple los checks de modelado relacional que apliquen, y su ficha de modelo de datos refleja el esquema real columna por columna | inspección del Tech-Lead contra la migración, no contra el reporte del ingeniero |
| 13 | **La unidad del spread reportado queda sellada en la declaración de mapeo**, no implícita en el nombre de un campo: el valor crudo sigue crudo, pero el registro dice en qué unidad está y contra qué magnitud del catálogo se reconstruye | prueba que verifica que la declaración sellada contiene la unidad declarada del spread, y prueba discriminante de que una importación con unidad de spread ausente se rechaza en vez de asumirla |

**Añadido el 2026-07-25 tras observación del propietario (criterio 13).** El propietario señaló que el número de spread de MetaTrader no significa nada sin el instrumento: 10 en EURUSD, en USDJPY, en WTI y en el S&P 500 son cuatro magnitudes distintas. Es correcto, y la auditoría del Tech-Lead confirmó el hueco: la entrega guardaba el número crudo con el nombre `spread_reported_raw` (honesto, no disfrazado) pero **la declaración sellada no registraba en qué unidad estaba** — el mismo tratamiento que sí recibió el volumen (`real_volume_unit_declared`) no se le había dado al spread. Verificación empírica sobre el archivo real: los valores de 2024 son mayoritariamente `0` con picos de `10`–`14`, magnitudes que solo son coherentes leídas como **puntos** de MetaTrader (último dígito del precio), no como pips — en pips serían spreads de 10 a 14 pips en EURUSD, imposibles para ese periodo. El factor de diez entre ambas lecturas es exactamente el riesgo que este criterio cierra. **No cambia la doctrina:** el valor crudo se sigue persistiendo crudo (ADR-0155 Restricción 1) y la conversión sigue siendo cálculo de lectura contra el tamaño de punto del catálogo (`canonical_tick_size` y su override por bróker) — lo que se corrige es que el registro diga qué es el número, en vez de dejarlo a la interpretación del lector.

---

## 6. Comandos de validación (para el usuario — copy/paste)

```bash
# 1. Compila todo el proyecto
cargo build --workspace

# 2. Corre las pruebas de la feature nueva
cargo test -p data-import-wizard

# 3. Verifica que no hay advertencias del analizador
cargo clippy --workspace --all-targets -- -D warnings

# 4. Prueba el lector contra el binario real (Canal #2)
cargo run -p app -- verify data-import-wizard --input '<el ejemplo que la feature declara>' | jq .
```

---

## 7. Registro de ejecución (bitácora cronológica)

- 2026-07-25 · Quant-Engineer (Sonnet, Etapa 0.3) · Veredicto **APTO-CON-CONDICIONES** sobre los archivos reales. 7 condiciones, 7 preguntas abiertas. Hallazgo de mayor impacto: el 60.9% de los ticks reales de MetaTrader son unilaterales, contra lo que el esquema canónico asumía. Informe: `.agents/plans/quant-2026-07-25-viabilidad-lector-mt5.md`.
- 2026-07-25 · Architect (Sonnet, Etapa 0.4) · Barrido de coherencia. Canonizó la reconstrucción de cotización unilateral (ADR-0155), la declaración de unidad de volumen real (ADR-0162) y dos chequeos de imposibilidad nuevos (`pit-data-validator`). Cero contradicciones entre documentos vigentes. Entregable: `.agents/plans/architect-2026-07-25-barrido-lector-mt5.md`.
- 2026-07-25 · Tech-Lead · **Auditoría de ambos gates reproducida.** Verifiqué contra el archivo real el conteo de ticks unilaterales y las dos filas imposibles del 2019-11-01. Verifiqué que el precedente que el Architect invoca para justificar el arrastre acotado (ADR-0114 regla (c), sub-cláusula 2026-07-23 sobre `spread_reported`) **existe y dice lo que afirma** — no es una técnica nueva introducida de contrabando, es doctrina ya sellada por el propietario aplicada a un campo distinto. Ediciones documentales revisadas una a una: quirúrgicas, con banner de enmienda y trazabilidad. Aceptados ambos gates.
- 2026-07-25 · Rust-Engineer (Sonnet, Modo Docente) · **Implementación completa de §4.1.** Crate nuevo `crates/features/data/data-import-wizard/` (`domain.rs` puro: plantilla MT5, parseo de línea, escala de precio, resolución de zona horaria IANA vía `chrono-tz`, detección de imposibilidad física; `persistence.rs`: ledger `import_mapping_declarations` con retry `BEGIN IMMEDIATE` + clasificador de contención; `orchestrator.rs`: cablea sellado del ledger ANTES del hand-off a `bar_sink` (puerto, no dependencia de crate); `public_interface.rs`: `verify()` + CLI). Migración `0022` retrofit (`real_volume_unit_declared`) + migración nueva `0023` (ledger, con triggers `no_update`/`no_delete`). Fichas `docs/data-model/import_mapping_declarations.md` + `broker_instrument_terms.md` actualizada + índice `DATA-MODEL.md`. Subcomando `drasus verify data-import-wizard` cableado en `crates/app`. 29 pruebas (unitarias + integración con SQLite en archivo real), ciclo rojo→verde demostrado para los 10 criterios verificables por código; `cargo build`/`clippy -D warnings`/`test` en verde workspace-completo. Cobertura del crate: 93.63% regiones, 95.82% líneas (`cargo llvm-cov`). Lección Docente: `docs/lessons/rust/STORY-068-mt5-bar-reader.md`. Decisión propia no cubierta por el encargo (declarada, no ejecutada en silencio): `spread_reported` se persiste crudo en puntos enteros de MT5, SIN escalar por `canonical_tick_size` — escalarlo exigiría que este crate consultara `instrument_master` de `instrument-catalog`, prohibido por ADR-0137; la semántica exacta ya estaba marcada como "sin verificar por bróker, opcional, no bloqueante" (pregunta 6 del barrido del Architect). **Gate de mutación (`cargo-mutants`) queda pendiente — lo corre el Tech-Lead, según el encargo.**
- 2026-07-25 · Rust-Engineer (Sonnet, Modo Docente) · **Corrección de 21 supervivientes de mutación (8 grupos) + criterio de aceptación #13.** Añadidas 17 pruebas nuevas (46 en total en el crate): plantilla MT5 verificada campo a campo, borde de signo/decimales/plus-sign en el convertidor de precio, borde de cero en el formateador inverso, valor exacto del cierre de barra, discriminación de ambas ramas de la imposibilidad de actividad, hash de archivo no-constante, número de línea exacto en errores de parseo, mensajes/causa de error de repositorio verificados. Ciclo rojo→verde demostrado y revertido para los 21 mutantes reportados, aplicando la mutación EXACTA que `cargo-mutants` habría generado. **Criterio #13 (unidad del spread reportado):** verificada de forma independiente la observación empírica del propietario sobre `datasets/EURUSD_M1_....csv` (junio 2024: pico en `0`, luego `10`–`14`; filas 2025-04-30/05-01 confirmadas byte a byte) — coincide exactamente con lo reportado. Implementado `MT5_SPREAD_REPORTED_UNIT = "POINTS_MT5"` como hecho FIJO de la plantilla (no declaración variable como la unidad de volumen); `CanonicalBarRow.spread_reported_unit` viaja con cada fila; `import_mapping_declarations.spread_reported_unit_declared` (retrofit migración `0023`, `NOT NULL`, mismo trato de rechazo que la zona horaria); rechazo tipado (`CanonicalMappingError::MissingSpreadUnitDeclaration`) si la plantilla no la declara. Doctrina intacta: el valor crudo se sigue persistiendo sin escalar (ninguna conversión por `canonical_tick_size`). `cargo build`/`clippy -D warnings`/`test` en verde workspace-completo (build limpio en todo el árbol, sin fallos). Cobertura del crate: 95.60% regiones, 97.61% líneas (subió desde 93.63%/95.82%). Lección Docente ampliada con Conceptos 11 (gate de mutación) y 12 (unidad sellada junto al dato) en `docs/lessons/rust/STORY-068-mt5-bar-reader.md`.

- 2026-07-25 · Tech-Lead · **Gate de mutación, ronda 1 — NO APTO.** `scripts/mutation-gate.sh data-import-wizard`: 101 mutantes, **21 supervivientes**. Devueltos al ingeniero con el diagnóstico ya hecho y agrupado en 8 focos, para que su presupuesto fuera a escribir pruebas y no a diagnosticar. Los tres de mayor consecuencia: (1) el contenido de la plantilla de plataforma podía vaciarse o llenarse de basura sin que nada fallara — hueco directo contra LDR-0008, donde Drasus responde por su propia plantilla; (2) `compute_source_file_hash` podía devolver una constante sin detección, y es a la vez clave de idempotencia (ADR-0163) y evidencia de identidad del archivo — con hash constante la prueba de reimportación **seguía pasando** mientras dos archivos distintos colisionaban; (3) `derive_bar_close_timestamp_ns` podía devolver `0`/`1`/`-1` sin que nada avisara, y el etiquetado de barra es la defensa anti-look-ahead de ADR-0155.
- 2026-07-25 · Tech-Lead · **Gate de mutación, ronda 2 — APTO.** 103 mutantes, **0 supervivientes** (88 cazados, 14 inviables, 1 timeout en el contador de reintento de `persistence.rs:141` — patrón ya aceptado en el proyecto desde STORY-045). **El ingeniero no reclamó ni un solo superviviente como equivalente genuino:** los 21 tenían prueba faltante legítima. Reproducidas además 46 pruebas verdes, `clippy --workspace --all-targets -D warnings` limpio, y el sellado de la unidad de spread confirmado en la base de datos (`spread_reported_unit_declared NOT NULL` en `0023`), no solo en el código.
- 2026-07-25 · QA-Engineer (Sonnet, Etapa 5) · **Veredicto APTO.** Auditoría independiente: leyó los seis archivos del crate completos, ambas migraciones, y corrió su propia evidencia sin fiarse del reporte del ingeniero ni del Tech-Lead. Contrastó las líneas "reales" citadas en el código contra el archivo de 9.2 millones: **el fixture de la anomalía de 2019-11-01 es idéntico byte a byte a las líneas 7187216–7187223 del archivo real.** Verificó las cinco condiciones técnicas de LDR-0008 campo por campo contra el esquema. Los trece criterios ejercidos por pruebas nombradas que los ejercen de verdad, no que pasan por casualidad. **Tres defectos menores, ninguno bloqueante** (un comentario que llamaba "mínimo del archivo" a un valor bajo real que no es el extremo global; la ficha de `instrument_master` sin la referencia entrante nueva; el resumen de cabecera de `DATA-MODEL.md` sin actualizar el conteo) — **los tres corregidos por el Tech-Lead en este mismo cierre.** Un hueco de diseño señalado y **no bloqueante por alcance** → `DEBT-036`.
- 2026-07-25 · Tech-Lead · **Cierre.** Corregidos los tres defectos menores del QA. Completada la sección `## Puertos de Integración (ADR-0137)` que faltaba en la feature doc (el QA lo señaló; el encargo había excluido los checks de documento de su lista, así que lo asumo yo por autoridad de Gate). Feature sellada 🟡 Parcial. Deudas nuevas abiertas: `DEBT-035` (pantalla de la vía manual) y `DEBT-036` (referencia de trazabilidad en el puerto de salida). `TEST.md` y `DATA-MODEL.md` actualizados.

---

## 8. Pendientes derivados / decisiones

- **Deuda de superficie visual (ADR-0117).** Esta Story entrega verificación por el Banco de Pruebas y por comando de terminal, no la pantalla del asistente de importación (arrastrar archivo, vista previa de dato transformado, botón de confirmación). Esa pantalla es requisito de LDR-0008 para la vía de mapeo manual — **no para la vía de plantilla de plataforma**, que es la que esta Story construye. Se salda en la Story del asistente visual, que es la que habilita la vía manual.
- **El archivo de ticks queda para la Story siguiente**, junto con la reconstrucción de cotización unilateral en `data-normalization-layer` y el chequeo de cotización cruzada en `pit-data-validator`.
- **Tres bloqueos abiertos en el propietario, ninguno detiene esta Story:** identidad de los tres archivos `.dat` (TASK-067); confirmación de la zona horaria del archivo de 25 años (TASK-064 §3.5); convención de unidad del volumen real de ese archivo concreto. Los tres se responden en paralelo.
- **Validación de la semántica exacta del spread reportado** (TASK-064 §4): el mes solapado que la desbloquea ya está en el repositorio. El Architect la remitió al Tech-Lead como decisión de secuenciación. **Decisión del Tech-Lead: va aparte**, después del lector de ticks — sin el lector de ticks no hay con qué contrastar.

---

## 9. Cierre ejecutivo (para el usuario — CEO)

```
ESTADO: 🟢 COMPLETADO

PROGRESO MACRO:
- Drasus ya lee dato de mercado real. Le entregas el archivo que exportaste de tu
  terminal de MetaTrader y el sistema lo convierte a su formato interno sin inventar
  un solo valor: si falta la zona horaria, lo rechaza en vez de suponer; si no sabe en
  qué unidad viene el volumen, guarda el número intacto y lo marca como no verificado
  en vez de asumir; si el archivo no encaja en una plantilla conocida, lo rechaza
  entero antes que importarlo a medias.
- Antes de guardar la primera fila, sella un registro imborrable de qué se declaró
  sobre ese archivo: su huella digital, el mapeo aplicado, la versión del motor, la
  zona horaria y las unidades. La propia base de datos rechaza cualquier intento de
  modificarlo o borrarlo. Es lo que convierte "el usuario aceptó esto" en un hecho
  verificable en vez de una afirmación nuestra.

FRICCIONES Y DEUDA:
- Falta la pantalla del asistente de importación, y con ella la vía en que el usuario
  mapea columnas a mano cuando su archivo no encaja en ninguna plantilla. No es un
  olvido: esa vía solo protege legalmente si el usuario confirma sobre una vista previa
  de sus propios datos ya convertidos, así que el motor y la pantalla nacen juntos o
  no nacen. Se paga en la Story del asistente visual.
- El canal por el que salen las velas no puede llevar todavía la referencia a la
  declaración que las originó. Hoy no importa —nadie las persiste aún—, pero el día que
  se conecten con el almacenamiento real habrá que ensanchar ese canal, o cada archivo
  guardado nacería huérfano de la declaración que lo respalda.
- El filtro de mutación encontró 21 formas de romper el código sin que ninguna prueba
  avisara, incluida la huella del archivo, que podía vaciarse mientras la prueba de
  "no importar dos veces lo mismo" seguía en verde. Se cerraron las 21; ninguna resultó
  imposible de cubrir. El código entregado funcionaba desde el principio: lo que
  faltaba era la red que avisa cuando deja de funcionar.

INPUT REQUERIDO DEL CEO:
- Tres datos que solo tú puedes aportar, ninguno bloquea lo construido:
  (1) de dónde salieron los tres archivos `.dat` de Dukascopy e IC Markets — su formato
      no coincide con ningún estándar conocido y adivinarlo daría precios creíbles pero
      falsos; reexportarlos como texto es la salida barata;
  (2) qué bróker o terminal exportó el archivo de 25 años, para declarar su zona horaria
      — declararla mal corre todas las velas unas horas;
  (3) en qué unidad reporta tu bróker el volumen real de ese archivo. Si no lo sabes,
      déjalo sin declarar: el sistema ya guarda el número y lo marca, que es el
      comportamiento correcto.
```
