# STORY-070 · Declaración por tramo y reloj de bróker — ajuste del lector ya construido

> **Orden de trabajo — Spec-Driven.** Fuente de verdad de la ejecución. Vive en git, no en el chat.

| Campo | Valor |
|---|---|
| **ID** | STORY-070 |
| **Título** | Declaración por tramo y reloj de bróker — ajuste del lector ya construido |
| **Tipo** | Story |
| **Épica (Fase)** | EPIC-1 — Soberanía de Datos (`ingest`) |
| **Sprint** | 2 |
| **Estado** | ✅ **CERRADA — QA APTO** (13/13 criterios; gate de mutación 0 supervivientes en ambos crates) |
| **Responsable** | Rust-Engineer (Sonnet, **Modo Docente**) · auditan Tech-Lead + QA-Engineer |
| **Creada** | 2026-07-26 |
| **Completada** | 2026-07-27 |

---

## 0. Resumen ejecutivo

El lector del archivo de MetaTrader se cerró hace un día (STORY-068) y funciona. Después, un escalamiento al Architect cambió **la forma de la declaración** que el usuario hace sobre su archivo: el reloj deja de ser una zona horaria plana, y la declaración deja de ser una por archivo para volverse una por **tramo** dentro del archivo.

Esta Story pone el código al día con esa decisión. **No es una corrección de un defecto:** lo construido era correcto contra la doctrina de su momento. Es que la doctrina cambió al descubrirse, sobre datos reales, que un archivo de 25 años no es homogéneo.

Va **antes** del hito de la interfaz porque la pantalla tiene que pedirle al usuario exactamente los campos que esta Story fija. Construir la pantalla primero sería construirla sobre un formulario que iba a cambiar.

---

## 1. Especificación de origen (qué specs implementa)

- **Decisión rectora:** `docs/adr/ADR-0162.md` — **enmienda 2026-07-26**. Es la fuente canónica de esta Story; su sección "Enmienda 2026-07-26" contiene los cuatro cambios de esquema con su justificación.
- **Escalamiento que la originó:** `docs/execution/TASK-069-architect-escalation-reloj-de-broker.md` (los dos casos y su evidencia medida).
- **Informe del Architect:** `.agents/plans/architect-2026-07-26-declaracion-por-tramo.md` (qué se decidió, qué se retó, y el impacto en lo ya construido).
- **Features afectadas:** [`data-import-wizard`](../features/data-import-wizard.md) (dueña del ledger) · [`instrument-catalog`](../features/instrument-catalog.md) (dueña de la tabla nueva del reloj).
- **ADRs que siguen gobernando sin cambio:** ADR-0141 (modelado relacional) · ADR-0020 (contrato de persistencia) · ADR-0163 (idempotencia de reimportación) · ADR-0137 (hexágono) · ADR-0133 (pirámide de pruebas + gate de mutación) · LDR-0008 (las cinco condiciones del registro de declaración siguen vigentes íntegras).

---

## 2. Objetivo (una frase llana)

Que el usuario pueda declarar el reloj real de su archivo y declarar por tramos cuando su archivo no es uniforme, sin que el sistema pueda sellar en silencio una uniformidad que el propio dato contradice.

---

## 3. Agentes y Modo de Acompañamiento (ADR-0120)

| Agente | Etapa | Depende de | Modo |
|---|---|---|---|
| **Rust-Engineer** | Etapa 2 — esquema e implementación | — | **Docente** (implementa y escribe la lección) |
| **Tech-Lead** | gate de mutación + auditoría | Rust-Engineer | — |
| **QA-Engineer** | **Etapa 5 — gate obligatorio** | Rust-Engineer | Autónomo |

**Etapas no activadas y por qué:**

- **Etapa 0.3 y 0.4 (Quant y Architect):** ya corridas. La decisión de dominio la acaba de sellar el Architect en ADR-0162, con la evidencia del Quant como insumo y las siete comprobaciones de razonamiento crítico aplicadas y visibles. No hay pregunta de diseño abierta que esta Story deba resolver.
- **Etapas 0.5, 3 y 4 (UI-Designer, Bridge, Flutter):** el diseño de la pantalla corre **en paralelo** a esta Story, no dentro de ella — la forma de la declaración ya está fija en el ADR, así que el diseñador no está bloqueado. La construcción de la pantalla es el hito siguiente.
- **Etapa 6 (Quant post-código):** no hay fórmula ni cálculo numérico nuevo. Es esquema y política de rechazo.

---

## 4. Instrucciones de despacho por agente (la spec ejecutable)

### 4.1 Rust-Engineer (Modo Docente)

Eres el **Rust-Engineer** de Drasus Engine. Antes de nada: lee `CLAUDE.md`, `.agents/knowledge/governance.md` completo, `.agents/skills/rust-engineer/SKILL.md` completo, y `.agents/knowledge/ledger-pattern.md` (vas a modificar un ledger append-only; ese documento es el molde canónico, nunca una feature ya implementada). Declara cada lectura al terminarla.

**Modo Docente:** además de implementar, amplías la lección existente `docs/lessons/rust/STORY-068-mt5-bar-reader.md` con lo que esta Story enseña — no crees un archivo nuevo: es la misma feature y la lección es por tema, no por ticket. El propietario aprende leyendo, no tecleando.

#### El problema

El crate `crates/features/data/data-import-wizard/` funciona y está en verde. Pero asume dos cosas que la doctrina ya no admite:

1. **Que el reloj de un archivo es una zona horaria del catálogo mundial.** No lo es cuando el bróker ancla su cambio de horario al calendario de otro país. El caso real, verificado: IC Markets opera a GMT+2 en invierno y GMT+3 en verano, con las **fechas de cambio de Estados Unidos**, no las europeas. Ninguna zona del catálogo describe eso.
2. **Que un archivo produce una sola declaración.** No, cuando el archivo no es uniforme. El archivo real del propietario mezcla al menos tres orígenes de dato a lo largo de 25 años: arranca en el año 2000 pese a que el bróker declarado se fundó en 2007, y el campo de volumen real es intermitente con un salto de seis órdenes de magnitud a mitad de 2017.

#### La especificación exacta — NO la re-derives

**`docs/adr/ADR-0162.md`, sección "Enmienda 2026-07-26", es tu especificación.** Contiene los cuatro cambios de esquema del ledger, la tabla nueva del reloj con sus cinco campos y sus tipos, los dos casos de heterogeneidad y por qué no se resuelven igual, y los dos chequeos del Gate de Homogeneidad con sus parámetros configurables y sus valores por defecto. **Léela y síguela.** No está abierta a interpretación y no la reabras: sus decisiones ya pasaron por el razonamiento crítico de dominio.

Si algo de esa sección te parece incorrecto o imposible de implementar como está escrito, **párate y repórtalo antes de implementarlo** — no lo ejecutes a ciegas ni lo contradigas en silencio.

#### Restricciones duras (invariantes, no se debaten)

- **Fase greenfield: corriges el baseline en su sitio.** Las migraciones `0022` y `0023` se editan directamente. **No apiles una migración correctiva.** Las únicas filas que existen hoy en el ledger son de prueba, así que no hay dato real que reinterpretar ni migrar.
- **El ledger sigue siendo append-only de verdad**, con sus candados a nivel de base de datos. Cambiar su grano no relaja ni una de las cuatro pruebas obligatorias del molde.
- **Nada se infiere.** El desplazamiento del reloj lo declara el usuario, igual que hoy declara la zona. La fecha desde la que un bróker opera es un hecho externo declarado, nunca deducido del archivo.
- **Detectar una fractura no es inferir su causa.** El Gate de Homogeneidad mide y muestra; no decide qué significa la fractura ni la corrige por su cuenta.
- **Se persiste el hecho, se calcula la interpretación.** Esta Story no añade ningún cálculo derivado. El reloj efectivo de un instante se resuelve al leer, a partir del ancla y el desplazamiento declarados.
- Precios y magnitudes en entero escalado ×10⁸; tiempo en enteros de nanosegundos UTC; nunca coma flotante en el dominio.

#### Lo que NO entra en esta Story

- **La pantalla del asistente.** Se diseña en paralelo y se construye después.
- **La conexión del canal de salida con el almacenamiento real** (`DEBT-036`). Entra en el hito siguiente.
- **El Gate de Homogeneidad como código de `pit-data-validator`.** Esa feature no existe en código todavía; su especificación ya quedó escrita. **Lo que sí entra aquí** es que el ledger no pueda sellar una fila cuyo tramo declarado sea anterior a la fecha de operación del bróker, cuando esa fecha está declarada — ese chequeo vive en el camino de importación y es verificable hoy.

#### El entregable

1. **La tabla del reloj de servidor por bróker**, con los cinco campos que el ADR especifica y sus tipos exactos, en la migración que corresponda según quién es su feature dueña.
2. **Los cuatro cambios del ledger** que el ADR enumera: las dos columnas de rango con su comprobación de orden, la restricción de unicidad relajada, y la sustitución del campo de zona plana por el par ancla + desplazamiento.
3. **El camino de importación al día:** acepta y sella una declaración por tramo; rechaza con motivo explícito un tramo anterior a la fecha de operación declarada del bróker; y resuelve el instante en nanosegundos a partir del ancla y el desplazamiento.
4. **Las fichas de modelo de datos** de las tablas tocadas y de la nueva, más su fila en el índice — columna por columna contra la migración real.
5. **El subcomando de verificación al día**, con su entrada de ejemplo reflejando la forma nueva de la declaración.
6. **La lección Docente ampliada.**

Decide tú la estructura interna, los nombres y el reparto entre núcleo puro y cáscara. Lo no negociable son las restricciones de arriba, la especificación del ADR, y los criterios de §5.

#### Cómo se prueba

Cada criterio de §5 se demuestra con **una prueba nombrada y discriminante**: debe fallar mientras el comportamiento no exista y pasar solo cuando esté bien implementado. Demuestra el ciclo rojo→verde en tu reporte.

**No lances `cargo-mutants`.** El gate de mutación lo corre el Tech-Lead; tu presupuesto se gasta escribiendo pruebas. Y ten presente el hallazgo de la Story anterior: 21 huecos de detección aparecieron en código que funcionaba y pasaba todas sus pruebas. Los que más costaron fueron los de funciones que devolvían un valor que ninguna prueba afirmaba — la huella del archivo y el instante de cierre de la barra. **Las funciones nuevas de esta Story que resuelven un instante o comparan dos fechas son exactamente de esa familia:** afirma sus valores, no solo que no revientan.

**Advertencia de método (`GOVERNANCE-4.7`):** este encargo te da el problema, las restricciones y la especificación ya sellada — deliberadamente **no** el método de implementación. Si el encuadre está mal planteado, dilo antes de ejecutarlo.

---

## 5. Criterio de aceptación (cada criterio ↔ su prueba)

| # | Criterio verificable | Prueba que lo demuestra |
|---|---|---|
| 1 | El reloj de IC Markets, declarado como ancla más desplazamiento, resuelve el instante correcto **en las dos estaciones**: una fecha de invierno y una de verano del archivo real dan el desfase esperado, distinto entre sí | prueba de oráculo con dos fechas reales del archivo, una de cada estación, afirmando el instante exacto en nanosegundos |
| 2 | El cambio de estación ocurre en la **fecha del calendario del ancla**, no en la europea: una fecha entre el segundo domingo de marzo y el último domingo de marzo resuelve con el desfase de verano | prueba con una fecha dentro de esa ventana de discrepancia, afirmando el instante exacto — falla si el ancla elegida usa el calendario europeo |
| 3 | Una declaración con desplazamiento cero produce **exactamente** el mismo instante que producía la declaración de zona plana anterior | prueba de equivalencia sobre una fecha real, contra el valor que la implementación anterior devolvía |
| 4 | Dos tramos disjuntos del mismo archivo producen **dos filas** del ledger, cada una con su rango, sin que la restricción de unicidad rechace la segunda | prueba que sella dos tramos del mismo archivo y verifica dos filas con rangos distintos |
| 5 | Reimportar **el mismo tramo exacto** del mismo archivo sigue siendo un no-op idempotente | prueba de reimportación de un tramo idéntico, verificando conteo estable |
| 6 | Un rango invertido o vacío se rechaza a nivel de base de datos | prueba que intenta insertar un rango con fin anterior o igual al inicio y verifica el aborto |
| 7 | Un tramo declarado que antecede la fecha de operación declarada del bróker **se rechaza con motivo explícito**; si esa fecha no está declarada, el chequeo no corre y no se asume ninguna | dos pruebas: una con fecha declarada que rechaza el tramo anterior, otra sin fecha declarada que lo acepta sin asumir nada |
| 8 | El ledger sigue siendo append-only real tras el cambio de grano: modificación y borrado abortan a nivel de base de datos, y la cadena de sellos sigue encadenando | las cuatro pruebas obligatorias del molde de `.agents/knowledge/ledger-pattern.md`, revalidadas, sin omitir ninguna |
| 9 | Dos escritores concurrentes sobre el ledger con grano nuevo no corrompen la secuencia ni pierden un evento | prueba de contención sostenida hasta agotar reintentos, más el clasificador de error transitorio ejercido directamente |
| 10 | Las migraciones editadas cumplen los checks de modelado relacional que apliquen, y las fichas de modelo de datos reflejan el esquema real columna por columna | inspección del Tech-Lead contra la migración, no contra el reporte del ingeniero |
| 11 | El subcomando de verificación devuelve el JSON correcto con la forma nueva de la declaración, contra el binario real | ejecución reproducida por el Tech-Lead |
| 12 | Las 46 pruebas de la Story anterior siguen en verde o quedan actualizadas con justificación explícita de por qué cambió cada una | suite completa del crate, con el diario de cambios de las que se tocaron |
| 13 | La tabla del reloj de bróker lleva el contrato de auditoría universal y el control de concurrencia optimista que sus tres hermanas del mismo catálogo llevan, y su ficha lo refleja | inspección del Tech-Lead contra la migración; más una prueba que verifica que dos ediciones concurrentes de la misma fila no se pisan en silencio |

**Añadido el 2026-07-26 (criterio 13), tras la resolución de TASK-071.** El Rust-Engineer implementó `broker_server_clock` con los cinco campos que la enmienda de ADR-0162 listaba literalmente, y **reportó** que esa lista no incluía el contrato de auditoría universal que ADR-0020 exige y que las tres tablas hermanas de la misma migración sí llevan. Implementar la letra del ADR y señalar la discrepancia, en vez de desviarse en silencio, es el comportamiento correcto. El Tech-Lead escaló porque el conflicto era **entre dos ADRs vigentes** — su autoridad cubre corregir una feature que contradice un ADR, no un ADR que contradice a otro. **El Architect resolvió:** los cinco campos eran únicamente los de dominio, con el Grupo I implícito; ADR-0020 no admite excepción para ese grupo y nada en la enmienda pretendió abrirla. Fue omisión de redacción al añadir la tabla en una sección separada de la que trae el recordatorio. El texto de ADR-0162 ya quedó corregido; esta Story aterriza la corrección en el esquema.

---

## 6. Comandos de validación (para el usuario — copy/paste)

```bash
cargo build --workspace
cargo test -p data-import-wizard
cargo clippy --workspace --all-targets -- -D warnings
cargo run -p app -- verify data-import-wizard | jq .
```

---

## 7. Registro de ejecución (bitácora cronológica)

- 2026-07-27 · **CERRADA — QA APTO.** Los 13 criterios verificados y reproducidos por el QA contra el binario real (no por reporte): 61/61 en `data-import-wizard`, 30/30 en `instrument-catalog`, clippy limpio, `verify` ejecutado en tres variantes (por defecto, reloj de IC Markets con desplazamiento 420, y rechazo por fecha de operación del bróker). **Gate de mutación APTO en ambos crates** (115 y 90 mutantes, 0 supervivientes). Sin defectos. Una sola observación no bloqueante, promovida a [`DEBT-039`](../debt/DEBT-039.md).
- 2026-07-27 · Tech-Lead · **Retrofit de [TASK-072](./TASK-072-architect-escalation-unidad-venue-operational-since.md) encadenado a esta Story.** El barrido de unidades detectó que `venue_operational_since` era el único timestamp de dominio del repositorio sin sufijo de unidad; el Architect dictaminó que era notación de documento y no decisión sellada, renombró a `venue_operational_since_ns`, y **elevó la convención a regla verificable** (`M13`, Gate del Tech-Lead, ADR-0141). `accepted_at` → `accepted_at_ns` entró en la misma tanda por ser el mismo crate. Renombrado puro: recuento de mutantes idéntico antes y después (115/90), que es la comprobación de que no cambió comportamiento. El ingeniero encontró **dos puntos que la lista del Architect no cubría** — un parámetro de `compute_broker_server_clock_audit_hash` (el barrido documental se hizo sobre fichas y ADR, no sobre código) y un comentario que citaba el nombre viejo. Barrido de las 7 columnas restantes del repositorio → [`DEBT-038`](../debt/DEBT-038.md).
- 2026-07-26/27 · Tech-Lead · **Criterio 13 añadido a mitad de ejecución** (resolución de [TASK-071](./TASK-071-architect-escalation-grupo-i-broker-server-clock.md)). Encadenado aquí en vez de abrirse como Story propia: misma tabla, misma migración y mismos crates ya abiertos.
- 2026-07-26 · Tech-Lead · Orden creada tras el veredicto del Architect en TASK-069. **Etapas 0.3 y 0.4 declaradas cumplidas por trabajo previo** (evidencia del Quant + decisión canonizada del Architect con las siete comprobaciones de dominio aplicadas). Alcance acotado deliberadamente al esquema y al camino de importación; la pantalla y la conexión al almacenamiento quedan para el hito siguiente. Diseño de la pantalla despachado **en paralelo**, posible porque el ADR ya fija la forma final de la declaración.

---

## 8. Pendientes derivados / decisiones

- **El Gate de Homogeneidad de Procedencia queda como especificación**, no como código, salvo el chequeo de fecha de operación del bróker que sí entra aquí. `pit-data-validator` no existe todavía en código; su especificación quedó completa en ADR-0162 y en la feature.
- **`DEBT-036` sigue abierta** y su disparador se cumple en el hito siguiente: al conectar el canal de salida con el almacenamiento real, ese canal debe cargar la referencia a la declaración sellada.
- **`DEBT-035` sigue abierta** y se salda en el hito siguiente: la pantalla, y con ella la vía de mapeo manual.

---

## 9. Cierre ejecutivo (para el usuario — CEO)

**Qué cambió, en una frase:** lo que el sistema te pedía declarar sobre un archivo de datos ahora se declara **por tramo**, no por archivo entero — porque un archivo de veinticinco años no tiene un solo origen, un solo bróker ni un solo reloj, y hacer que lo pareciera era firmar como cierto algo que no lo es.

**Tres cosas que antes no podías hacer y ahora sí:**

- **Declarar el reloj de un bróker que ningún calendario del mundo describe.** IC Markets cambia de horario en las fechas de Estados Unidos aunque su hora sea europea. Antes había que elegir un país y aceptar equivocarse unas cuatro semanas al año — sobre veinticinco años, casi cien semanas mal fechadas. Ahora se declara un calendario de referencia más un desplazamiento fijo, y ese caso se describe exacto.
- **Partir un mismo archivo en varios tramos** y declarar cada uno con su propio origen. Necesario, no teórico: tu archivo de veinticinco años mezcla al menos tres procedencias distintas bajo un solo nombre.
- **Que el sistema rechace un tramo fechado antes de que el bróker existiera** — cuando esa fecha está verificada. Si no lo está, no se inventa: no corre el chequeo.

**Lo que hay que decidir antes de que la pantalla llegue a tus manos:** el sistema todavía **no comprueba que el rango que declaras se parezca a lo que el archivo contiene**. Puedes declarar un siglo sobre un archivo de un día y lo aceptaría. No es un descuido —está diferido al validador, que es trabajo posterior— pero el orden planificado pone la pantalla **antes** que el validador, y eso significa que podrías sellar declaraciones antes de que exista lo que las contrasta. Queda registrado como deuda de severidad alta con disparador en el hito, con dos salidas posibles: un cotejo mínimo dentro del propio asistente, o adelantar el validador. Elegir es tuyo; dejarlo abierto sin elegir, no.

**Lo que este ciclo dejó más allá del código:** dos reglas nuevas que ya no dependen de que alguien se acuerde. Una obliga a que todo dato temporal diga en qué unidad está — nació de que tú cazaste el spread en la unidad equivocada, y desde hoy la caza un checklist. La otra es el hábito que la aplicó: barrer todos los campos de una entrega, no solo el que pidió la orden. Encontró dos errores más en el sitio donde nadie había mirado.
