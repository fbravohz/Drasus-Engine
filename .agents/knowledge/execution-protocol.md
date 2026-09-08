# Protocolo de Ejecución del Tech-Lead

El ciclo de vida completo de una Orden de Trabajo: cómo se redacta y despacha, qué verifica el Tech-Lead antes de cerrarla, las 8 Etapas del pipeline con sus triggers, cuándo se escala al Architect, la Definición de Terminado y la gobernanza de secuenciación por fase.

Slug ID: `EXECUTION-PROTOCOL`.

Tamaño: `42503 caracteres (~10626 tokens aprox., ratio 4:1)`.

---

## EXECUTION-PROTOCOL-0 — Lecturas Obligatorias

| Documento | De qué va | Cuándo leerlo (disparador) | Alcance |
|---|---|---|---|
| [[.agents/knowledge/governance.md]] | Gobernanza meta, supremacía absoluta. | `SIEMPRE` al iniciar cualquier skill. | Todos los agentes |
| [[.agents/knowledge/order-compliance-filter.md]] | Checklist ADR/persistencia que corre el Tech-Lead antes de redactar la Orden. | Al ejecutar `EXECUTION-PROTOCOL-14` (Etapa 0). | TL |
| [[.agents/knowledge/coherence-sweep.md]] | El Barrido de Coherencia Documental que ejecuta el Architect. | Al ejecutar `EXECUTION-PROTOCOL-16` (Etapa 0.4). | TL, Architect |
| [[.agents/knowledge/ui-verification-model.md]] | SVF/Galería, Contrato de Integración UI, Definición de Terminado por clasificación. | Al ejecutar `EXECUTION-PROTOCOL-17`, `-21`, `-27`. | TL, UI-Designer, Flutter |
| [[.agents/knowledge/mutation-filter.md]] | Cómo correr y juzgar el filtro de mutación. | Al ejecutar `EXECUTION-PROTOCOL-9`. | TL |
| [[.agents/knowledge/test-discrimination.md]] | Cómo se demuestra que una prueba discrimina de verdad. | Al ejecutar `EXECUTION-PROTOCOL-2.1`. | TL |
| [[.agents/knowledge/execution-vocabulary.md]] | Identificadores de trabajo y protocolo de numeración. | Al crear cualquier Orden de Trabajo. | TL, Architect |
| [[.agents/knowledge/brief.md]] | Formato de §9 Cierre Ejecutivo de la Orden. | Al ejecutar `EXECUTION-PROTOCOL-23`, punto 2 del checklist. | TL |

---

## EXECUTION-PROTOCOL-FV — Filtro de Verificación

**Aplica a:** el Tech-Lead, en cada paso del ciclo de vida de una Orden de Trabajo.

* **EXECUTION-PROTOCOL-FV-1:** ¿pasé el criterio de aceptación por el ciclo rojo→verde antes de aceptarlo como cumplido? (`EXECUTION-PROTOCOL-2.1`)
* **EXECUTION-PROTOCOL-FV-2:** ¿reproduje yo mismo la evidencia de cierre, sin confiar en el reporte del ingeniero? (`EXECUTION-PROTOCOL-7`)
* **EXECUTION-PROTOCOL-FV-3:** ¿el filtro de mutación lo corrí yo, nunca un subagente, y en cuanto el núcleo estuvo listo — no solo al cierre final? (`EXECUTION-PROTOCOL-9`)
* **EXECUTION-PROTOCOL-FV-4:** ¿barrí unidades declaradas y secciones hermanas de la misma familia, no solo lo que la Orden pedía explícitamente? (`EXECUTION-PROTOCOL-10`)
* **EXECUTION-PROTOCOL-FV-5:** ¿verifiqué contra el código real cualquier lista de alcance construida a partir de un barrido documental? (`EXECUTION-PROTOCOL-11`)
* **EXECUTION-PROTOCOL-FV-6:** ¿actualicé el registro de ejecución y el estado de cabecera en el mismo turno en que cerré la etapa, sin dejarlo para el final? (`EXECUTION-PROTOCOL-12`)
* **EXECUTION-PROTOCOL-FV-7:** antes de avanzar cualquier etapa, ¿verifiqué que la etapa previa requerida esté `Completado`? (`EXECUTION-PROTOCOL-14` a `-25`)
* **EXECUTION-PROTOCOL-FV-8:** ¿corrí el Barrido de Cierre Documental completo — incluido §9 con `brief.md` — antes de proponer commits? (`EXECUTION-PROTOCOL-23`)
* **EXECUTION-PROTOCOL-FV-9:** ¿escalé al Architect por uno de los triggers exactos, con evidencia concreta, sin resolver yo la ambigüedad arquitectónica? (`EXECUTION-PROTOCOL-26`)
* **EXECUTION-PROTOCOL-FV-10:** ¿la feature cerrada tiene su Superficie de Verificación Funcional, o su Ventana de Verificación si es plomería, antes de marcarla `Completado`? (`EXECUTION-PROTOCOL-27`)

De ser satisfactoria cada validación de la lista, declara `EXECUTION-PROTOCOL - FILTRO DE VERIFICACION APROBADO`.

---

## EXECUTION-PROTOCOL-1 — Órdenes de Trabajo, Fuente de Verdad de Ejecución

Cada trabajo se ejecuta desde una Orden de Trabajo: un archivo en `docs/execution/<ID>-<slug>.md` (plantilla en [[docs/execution/_TEMPLATE.md]]). Es la especificación ejecutable y su registro; vive en git, no en el chat.

**Aplica a:** el Tech-Lead, para todo TTR/Feature seleccionado.

* **EXECUTION-PROTOCOL-1.1 — Filtro Previo:**

  antes de redactar la Orden, corre [[.agents/knowledge/order-compliance-filter.md]] completo. Solo tras su resultado (`ORDER-COMPLIANCE-FILTER-16`) se redacta la Orden.

---

## EXECUTION-PROTOCOL-2 — Antes de Despachar, Redacción de la Orden

**Aplica a:** el Tech-Lead, al llenar la Orden desde la plantilla.

* **EXECUTION-PROTOCOL-2.1 — Criterio de Aceptación con Prueba que Primero Falla:**

  cada criterio de aceptación se expresa como una prueba nombrada y discriminante — una que falla (roja) mientras el comportamiento no exista y solo pasa (verde) cuando se implementa correctamente. En las instrucciones de despacho exige al ingeniero el ciclo rojo→verde: la prueba se escribe antes o junto con el código, y debe demostrarse que falla sin la implementación. Una prueba que pasa aunque el comportamiento esté ausente (verde-trivial) no cuenta como criterio cumplido. Aplica con especial fuerza a garantías de comportamiento (concurrencia, recuperación, atomicidad, límites de recursos): la prueba debe medir el comportamiento, no asumirlo.

  **Lectura bajo demanda:** [[.agents/knowledge/test-discrimination.md]] — catálogo completo de los cuatro patrones que dan verde sin probar nada.

* **EXECUTION-PROTOCOL-2.2 — Criterio Ejecutado por el Propietario, Recorrido Literal:**

  hay criterios que ningún agente puede ejecutar: los que exigen manejar la interfaz gráfica, un archivo que solo está en la máquina del propietario, o una credencial real. Para esos, la Orden entrega en §6 un recorrido paso a paso — dónde pulsar, qué debe aparecer en cada paso, cómo saber que salió bien — más las trampas conocidas (features vecinas que se confunden, gestos que todavía no funcionan). Un criterio que el Tech-Lead no puede ejecutar es un criterio que debe poder dictar: escribir "solo lo puede ejecutar el propietario" y nada más no es delegar, es dejar el criterio sin definir y trasladarle al propietario el trabajo de adivinar.
  * ❌ *Incorrecto:* cerrar un criterio con "el usuario debe verificarlo en la app" sin decir dónde ni qué esperar.
  * ✅ *Correcto:* entregar el recorrido literal completo — pasos, resultado esperado por paso, trampas conocidas.

  **Lectura bajo demanda:** [[.agents/memory/criterio-verificable-por-grep-no-es-criterio.md]] — caso completo de origen, con la causa raíz detallada.

* **EXECUTION-PROTOCOL-2.3 — No Precocinar la Solución en el Prompt de Despacho:**

  el prompt de un especialista (Quant, Architect, cualquier ingeniero) entrega cuatro cosas y nada más como método: el problema, las restricciones/invariantes duros, la doc exacta a consultar, el entregable y su forma. Deja que el especialista derive el método — para eso es especialista. Si crees saber la solución, va como "hipótesis a validar o refutar", jamás como el camino a seguir. Un subagente tiende a ejecutar la premisa que recibe en vez de retarla: el criterio senior no se pierde en el agente, se pierde en el encuadre del despacho.
  * ❌ *Incorrecto:* darle al especialista la solución ya masticada, enmarcada como el único camino.
  * ✅ *Correcto:* entregar problema + restricciones + doc + forma del entregable, y marcar cualquier solución propia como hipótesis a validar.

  **Lectura bajo demanda:** [[.agents/memory/despacho-no-precocinar-solucion.md]] — caso completo de origen, con la causa raíz detallada.

**Filtro de verificación:** `EXECUTION-PROTOCOL-FV-1`.

---

## EXECUTION-PROTOCOL-3 — Despacho Real

Paso separado del anterior, nunca automático.

**Aplica a:** el Tech-Lead, tras redactar la Orden.

* **EXECUTION-PROTOCOL-3.1 — Modo Autónomo:**

  el Tech-Lead despacha vía `Agent` (o el mecanismo de su plataforma) usando las instrucciones ya escritas en la Orden — nunca improvisa el prompt en el chat.

* **EXECUTION-PROTOCOL-3.2 — Modo Mentor/Revisión:**

  el paso de despacho queda en manos del usuario, que invoca el skill correspondiente directamente. Esto no exime al Tech-Lead de auditar y cerrar — solo cambia quién hizo la invocación.

---

## EXECUTION-PROTOCOL-4 — Tras Auditar, Cierre del Paso

* **EXECUTION-PROTOCOL-4 — Registro Obligatorio:** sin importar el Modo, tras auditar el Tech-Lead registra la ejecución en la Orden (fecha, agente/modelo, resultado, evidencia), sella los documentos fuente como implementados, actualiza estado + enlace en el ROADMAP, y entrega al usuario los comandos de validación.

---

## EXECUTION-PROTOCOL-5 — Si la Spec Cambia

* **EXECUTION-PROTOCOL-5 — Editar y Re-Despachar:** la Orden de Trabajo se edita y se re-despacha; el cambio queda reflejado y versionado — nunca una Orden nueva paralela para el mismo trabajo.

---

## EXECUTION-PROTOCOL-6 — Relación Entre los Tres Registros

Sin duplicar contenido entre ellos.

**Aplica a:** el Tech-Lead, al escribir cualquiera de los tres.

* **EXECUTION-PROTOCOL-6.1 — Orden de Trabajo:**

  `docs/execution/` — el detalle de cada trabajo: qué se pidió, cómo validar, qué pasó.

* **EXECUTION-PROTOCOL-6.2 — ROADMAP:**

  [[docs/ROADMAP.md]] — el backlog/mapa: estado de cada trabajo + enlace a su Orden.

* **EXECUTION-PROTOCOL-6.3 — PROGRESS:**

  [[.agents/state/tech-lead/PROGRESS.md]] — el tablero/índice: dónde estamos y el siguiente paso. Apunta a las Órdenes; no copia su detalle.

---

## EXECUTION-PROTOCOL-7 — Verificación Independiente, No Confíes, Verifica

El reporte del ingeniero no es prueba de cierre. Antes de marcar cualquier entregable `Completado`, el Tech-Lead reproduce él mismo la evidencia con sus propias herramientas.

**Aplica a:** el Tech-Lead, en toda auditoría de entregable.

* **EXECUTION-PROTOCOL-7.1 — Rust:**

  corre `cargo build`/`cargo test` él mismo; cuenta los tests; revisa warnings.

* **EXECUTION-PROTOCOL-7.2 — Flutter, Obligatorio para Toda Story con Código Dart:**

  corre `flutter build <platform>` él mismo antes de despachar el QA. Sin `flutter build` verde no se despacha QA. Tras cualquier cambio a `crates/bridge` o sus dependencias (`shared`, `features/*`), o una regeneración de bindings, corre `cargo build --release -p bridge` antes de probar en la app — un `.so` viejo no da "Failed to load", da un error de `Content hash` fuera de sincronía y la app arranca sin abrir ventana (detalle canónico en [[.agents/skills/bridge-engineer/SKILL.md]] §"Recompilar en el momento oportuno"). Si el SDK de Flutter no está instalado, eso es un bloqueo: no se despacha el QA de la Story Flutter hasta tenerlo disponible y haber corrido `flutter build <platform>` en verde — el compilador Dart es el único verificador de tipos entre bindings Rust→Dart, y errores ahí no son visibles en revisión de código fuente.

* **EXECUTION-PROTOCOL-7.3 — Cobertura del Criterio, No Solo "Verde":**

  para cada criterio de aceptación, confirma que existe una prueba nombrada que lo ejerce de verdad — "60 tests verdes" no cierra nada si el criterio crítico no tiene una prueba que lo ejecute. Corre `cargo llvm-cov --workspace --summary-only` para medir cobertura y detectar lógica del filtro sin ejercer.

* **EXECUTION-PROTOCOL-7.4 — Estructura y Arquitectura:**

  inspecciona los archivos clave (ej. una cáscara y un núcleo) para confirmar FCIS, cero lógica donde no debe haberla.

* **EXECUTION-PROTOCOL-7.5 — CLI de Verificación, Canal #2:**

  si la feature expone `verify()` en su `public_interface`, reproduce él mismo su salida real con `cargo run -p app -- verify <feature-id> --input '<json>' | jq .` antes de cerrar. No reemplaza la cobertura de criterios; la complementa.

* **EXECUTION-PROTOCOL-7.6 — Ediciones Documentales:**

  corre los `grep` de verificación — el rastro viejo en 0, el nuevo el número esperado de veces.

* **EXECUTION-PROTOCOL-7.7 — Migraciones y Contratos:**

  confirma el artefacto real (campos exactos, idempotencia) contra la fuente (ADR), no contra el resumen del ingeniero.

* **EXECUTION-PROTOCOL-7.8 — Modelo de Datos:**

  si la Story tocó `migrations/*.sql`, confirma que `docs/data-model/<tabla>.md` quedó creada/actualizada en el mismo cambio (y la fila del índice si es tabla nueva) — columna por columna contra la migración real. Sin esto, no se cierra la Story aunque el resto esté verde.

* **EXECUTION-PROTOCOL-7.9 — El Ingeniero Entrega Su Propio Verde:**

  cada ingeniero escribe y corre sus pruebas (pirámide: unitarios + integración + proptest si hay lógica cuantitativa + fuzzing si hay frontera externa) y entrega ya en verde con el mapeo criterio→prueba + cobertura; el Tech-Lead reproduce y cierra.

* **EXECUTION-PROTOCOL-7.10 — Si la Verificación Contradice el Reporte:**

  el entregable regresa al ingeniero (defecto de implementación) o se escala al Architect (defecto de diseño). Nunca se cierra sobre confianza.

**Filtro de verificación:** `EXECUTION-PROTOCOL-FV-2`.

---

## EXECUTION-PROTOCOL-8 — Activación del QA-Engineer, Filtro Obligatorio

* **EXECUTION-PROTOCOL-8 — Sin Excepción de Fase:** QA-Engineer (Etapa 5) es filtro obligatorio antes de cerrar cualquier Story de código, desde EPIC-0 en adelante. El Tech-Lead no marca un ticket `Completado` sin veredicto APTO del QA — si el ingeniero puede escribir código incorrecto que pasa sus propios tests, ese es exactamente el riesgo que el QA existe para detectar. Pre-dinero real (cualquier EPIC): las Pruebas de Guerra del QA skill §3 son bloqueantes de release.

---

## EXECUTION-PROTOCOL-9 — Filtro de Mutación

* **EXECUTION-PROTOCOL-9 — Lo Corre Solo el Tech-Lead, Durante el Trabajo:** toda Story que añada/cambie lógica de correctitud en `domain/` o `persistence/` no se cierra sin el filtro en 0 supervivientes. En cuanto un ingeniero entrega el núcleo de una Story — antes de despachar el puente y mucho antes del QA — el Tech-Lead corre el filtro sobre su crate: diagnosticar temprano evita rondas de ida y vuelta que un cierre tardío del filtro multiplica. Para ledgers append-only exige las tres pruebas companion del molde canónico [[.agents/knowledge/ledger-pattern.md]] — nunca una feature de referencia por antigüedad, que puede propagar el mismo defecto a implementaciones posteriores.

  **Lectura bajo demanda:** [[.agents/knowledge/mutation-filter.md]] — cómo correr el script, leer el veredicto y juzgar un superviviente equivalente.

**Filtro de verificación:** `EXECUTION-PROTOCOL-FV-3`.

---

## EXECUTION-PROTOCOL-10 — Barrido de Unidades y Secciones Hermanas

Antes de aceptar una entrega, dos familias de fallos que ningún filtro automático detecta porque el código es correcto.

**Aplica a:** el Tech-Lead, al auditar cualquier entrega.

* **EXECUTION-PROTOCOL-10.1 — Unidades:**

  enumera todos los campos que se persisten crudos y verifica que cada uno lleve su unidad sellada — no solo el que la Orden mencionaba explícitamente. Un campo del mismo tipo y riesgo que otro ya sellado, pero con tratamiento distinto, es la señal de que la auditoría revisó lo que la Orden pedía en vez de barrer la familia entera.

* **EXECUTION-PROTOCOL-10.2 — Secciones Hermanas:**

  cuando acotes el checklist de un filtro a un especialista (ej. pedir al QA solo los checks de migración y de shell), el resto no desaparece: pasa a ser tuyo. Complétalo con tu propia autoridad de Filtro en el cierre, en vez de darlo por revisado.

**Filtro de verificación:** `EXECUTION-PROTOCOL-FV-4`.

---

## EXECUTION-PROTOCOL-11 — Una Lista de Alcance Documental No Cubre el Código

* **EXECUTION-PROTOCOL-11 — Verificar Contra el Código Real:** cuando un especialista entregue "los archivos que hay que tocar" tras revisar fichas y ADRs, esa lista está construida con `grep` sobre documentación. Despacha siempre con la instrucción explícita de verificarla contra el código real en vez de confiar en ella a ciegas — trátala como punto de partida, no como perímetro. Un nombre repetido en varios sitios se corrige donde alguien miró y sobrevive donde nadie miró; el único filtro confiable es barrer el código, no la lista. Verificación barata de que un renombrado fue puro: el recuento de mutantes no debe variar (`cargo mutants --list -p <crate> | wc -l`), lo que evita relanzar el filtro completo.

**Filtro de verificación:** `EXECUTION-PROTOCOL-FV-5`.

---

## EXECUTION-PROTOCOL-12 — Sincronía Documental por Etapa, Nunca al Final

* **EXECUTION-PROTOCOL-12 — Fila y Estado en el Mismo Turno:** la fila del Registro de Ejecución de la Orden se escribe al auditar cada etapa, no acumulada al cierre. El estado y el banner de la cabecera son parte de la etapa, no un adorno — si el documento dice que un bloque sigue sin despachar y ya se entregó, está mintiendo a quien lo lea después. Cada vez que se cierra una etapa: fila del registro + estado de cabecera, en el mismo turno.

**Filtro de verificación:** `EXECUTION-PROTOCOL-FV-6`.

---

## EXECUTION-PROTOCOL-13 — Accionables Bloqueados por el Propietario

* **EXECUTION-PROTOCOL-13 — Marca Explícita y Rastreada:** cuando un trabajo requiera algo que solo el propietario puede aportar — archivos de datos reales, credenciales, una decisión de negocio, una compra, un acceso — crea una Task y marca explícitamente en su cabecera y en `PROGRESS.md`/`ROADMAP.md`: `🔴 BLOQUEADO POR EL USUARIO`, con qué se necesita exactamente y en qué formato. No basta con mencionarlo en prosa ni dejarlo en una conversación: si no está rastreado como Task con esa marca, se pierde. El propietario debe poder barrer sus bloqueos sin releer un chat.

---

## EXECUTION-PROTOCOL-14 — Etapa 0, Selección Autónoma de TTR

**Trigger:** ciclo continuo. Al cerrar un TTR (Etapa 5/6), o al iniciar trabajo, se vuelve aquí.

**Aplica a:** el Tech-Lead.

* **EXECUTION-PROTOCOL-14.1 — Proceso:**

  1. Lee ROADMAP §3-4 → identifica la fase activa y su "Entregable Alpha".
  2. Recorre `modules/*.md` del/los módulo(s) de esa fase → filtra TTRs P0 cuya Precondición ya está `Completado`.
  3. Aplica `EXECUTION-PROTOCOL-28` (Gobernanza ROADMAP): si el TTR no corresponde a la fase activa, o los Spikes de Viabilidad bloqueantes no están resueltos (filtro EPIC-0), el TTR queda `Secuenciado / En Espera` — elige el siguiente candidato.
  4. Para el TTR seleccionado, lee su(s) Feature(s) referenciada(s) y los ADRs citados.

* **EXECUTION-PROTOCOL-14.2 — Clasificación:**

  clasifica el TTR/Feature como (a) "matemática/estrategia/métrica" → activa Etapas 1 y 6, y/o (b) su Contrato de Integración UI declara "Superficie propia" → activa Etapas 3-4 bajo el Techo Fijo (ADR-0117). Toda feature pasa además por el Filtro de Viabilidad Experta (`EXECUTION-PROTOCOL-15`) antes de la Etapa 0.5.

---

## EXECUTION-PROTOCOL-15 — Etapa 0.3, Viabilidad Experta de Dominio

**Trigger:** toda feature/TTR al crear su Orden de Trabajo o al reevaluar un desarrollo — el filtro más temprano, antes de gastar diseño, arquitectura o código. Va antes del barrido del Architect (`EXECUTION-PROTOCOL-16`); su salida es insumo de esa etapa.

**Aplica a:** el Tech-Lead, despachando Quant-Engineer y/o General-Counsel.

* **EXECUTION-PROTOCOL-15.1 — Sub-Filtro Quant:**

  el Quant-Engineer analiza la feature y verifica viabilidad y fallos de especificación/bridge para su área (fórmulas, sesgos, sim/real, supuestos estadísticos), citando siempre qué ADR/feature/SAD aplica.

* **EXECUTION-PROTOCOL-15.2 — Sub-Filtro Legal/Fiscal:**

  el General-Counsel se despacha cuando la feature toque superficie legal/fiscal (datos personales/PII, flujos transfronterizos, pagos/facturación, asesoría financiera, T&C/EULA, licenciamiento, telemetría/consentimiento, portabilidad/olvido de datos, marketplace, KYC/AML, nexo fiscal). Emite veredicto APTO / APTO-CON-CONDICIONES / NO-APTO con la exposición concreta citada.

* **EXECUTION-PROTOCOL-15.3 — Salida:**

  veredicto(s) de viabilidad en §8 de la Orden, más la lista de preguntas y hallazgos del Quant que pasa íntegra al Architect en `EXECUTION-PROTOCOL-16`. NO-APTO de cualquiera bloquea diseño/implementación. Los requisitos que el Legal rebote se vuelven criterios de aceptación. Si la feature no toca un área, esa sub-parte queda "No Aplica" en §8.

---

## EXECUTION-PROTOCOL-16 — Etapa 0.4, Barrido de Coherencia Documental

**Trigger:** toda feature/TTR, sin excepción, antes de escribir una sola línea de la Orden — no es opcional ni se omite porque la feature parezca simple.

**Aplica a:** el Architect, despachado por el Tech-Lead con los hallazgos íntegros de `EXECUTION-PROTOCOL-15` como insumo.

* **EXECUTION-PROTOCOL-16 — Ejecución:** el procedimiento completo (los 5 pasos del encargo, los tres cubos de clasificación, el entregable, la regla de bloqueo) vive en [[.agents/knowledge/coherence-sweep.md]] — este es el punto del pipeline donde se dispara.

  **Nota de numeración:** las Órdenes de Trabajo comparten un único espacio de numeración entre `STORY-`/`TASK-`, sin importar el tipo — protocolo completo en [[.agents/knowledge/execution-vocabulary.md]] (`EXECUTION-VOCABULARY-2.1`).

---

## EXECUTION-PROTOCOL-17 — Etapa 0.5, Diseño Visual

**Trigger:** feature seleccionada en Etapa 0 declara "Superficie propia" (ADR-0117) y no tiene aún una sección `## Cáscara Visual` actualizada.

**Aplica a:** UI-Designer, despachado por el Tech-Lead.

* **EXECUTION-PROTOCOL-17.1 — Rol:**

  lee la feature, clasifica su contexto de superficie (Dashboard widget / Canvas Vista Relacional / Canvas Vista Interior / Inspector Panel, ADR-0136), detecta y corrige violaciones arquitectónicas UI, y escribe `## Cáscara Visual (Thin Shell)` en `docs/features/<feature>.md` con el vocabulario canónico de `DESIGN.md`.

* **EXECUTION-PROTOCOL-17.2 — Salida y Omisión:**

  reporte al Tech-Lead: contexto de superficie asignado + componentes principales + violaciones corregidas. Se omite la etapa si la sección ya existe y está actualizada. Features de plomería reciben solo la nota de observable, no pantalla completa — la etapa igual aplica para dejar esa nota.

* **EXECUTION-PROTOCOL-17.3 — Filtro Bloqueante:**

  si la feature tiene Superficie propia y esta etapa no se completó, no se despacha al Flutter Engineer (Etapa 4, ADR-0135).

---

## EXECUTION-PROTOCOL-18 — Etapa 1, Validación Cuantitativa Pre-Código

**Trigger:** Feature spec marcada como matemática/estrategia (Etapa 0).

**Aplica a:** Quant-Engineer, despachado por el Tech-Lead.

* **EXECUTION-PROTOCOL-18 — Rol y Salida:** audita fórmula/diseño experimental antes de escribir código (look-ahead, survivorship, overfitting, fórmula de referencia citada). Salida: veredicto APTO/NO APTO sobre el diseño. NO APTO escala al Architect (`EXECUTION-PROTOCOL-26`) para corregir la Feature spec, bloqueando la Etapa 2. Si la Feature no está marcada como matemática, la etapa es "No Aplica" y se salta directo a Etapa 2.

---

## EXECUTION-PROTOCOL-19 — Etapa 2, Implementación Core

**Trigger:** TTR + Feature spec con veredicto APTO de Etapa 1 (si aplicaba).

**Aplica a:** Rust-Engineer, despachado por el Tech-Lead.

* **EXECUTION-PROTOCOL-19 — Verificación y Salida:** el Tech-Lead verifica que el Rust-Engineer cumplió su Filtro de Lectura Pre-Código (TTR, Feature spec, ADRs citados) antes de aceptar el entregable: `public_interface.rs`, domain, persistence con el Grupo I (universal) más los campos del Perfil Técnico que la Feature spec declara (`ORDER-COMPLIANCE-FILTER-4`) — nunca los 25 campos completos salvo que el perfil lo exija explícitamente; una tabla con los 25 campos calcados sin justificación de perfil es defecto de implementación y regresa sin cerrarse. Si la Feature no requiere exposición a UI/headless, fin de cadena de implementación — se despacha directo a Etapa 5.

---

## EXECUTION-PROTOCOL-20 — Etapa 3, Contrato de Integración

**Trigger:** contrato de tipos Rust congelado (`public_interface.rs` estable) y Feature spec marcada con superficie UI/headless.

**Aplica a:** Bridge-Engineer, despachado por el Tech-Lead.

* **EXECUTION-PROTOCOL-20 — Bloqueo y Salida:** si el Rust-Engineer no congeló el contrato, no se despacha al Bridge-Engineer (evita rework). Salida esperada: bindings `flutter_rust_bridge` generados, contratos Arrow/Protobuf documentados.

---

## EXECUTION-PROTOCOL-21 — Etapa 4, Interfaz

**Trigger:** bindings del Bridge compilando y disponibles.

**Aplica a:** Flutter-Engineer, despachado únicamente por el Tech-Lead.

* **EXECUTION-PROTOCOL-21.1 — Restricción Dura:**

  el Flutter-Engineer nunca recibe trabajo directo del Rust-Engineer; siempre despachado por el Tech-Lead vía el entregable del Bridge-Engineer.

* **EXECUTION-PROTOCOL-21.2 — Salida:**

  Cáscara Delgada (UI Thin Shell) bajo el Techo Fijo de ADR-0117 — sin lógica de negocio, como pestaña/sección nueva (máximo una por Feature) del Panel Operativo Fundacional. Cumple la SVF declarada en el Contrato de Integración UI: control que dispara la operación real vía `public_interface`, visualización del resultado real vía FFI/gRPC, observable persistido visible tras recargar.

* **EXECUTION-PROTOCOL-21.3 — Filtro de Integración:**

  antes de cerrar, se revisa si alguna Feature de plomería completada previamente declaró esta Feature como su Ventana de Verificación (deuda de integración visual registrada en `PROGRESS.md`/ROADMAP). Si la hay, el observable correspondiente debe quedar visible en esta Cáscara Delgada — si no, la etapa no está completa.

  **Lectura bajo demanda:** [[.agents/knowledge/ui-verification-model.md]] — detalle completo de SVF/Galería y el test de clasificación.

---

## EXECUTION-PROTOCOL-22 — Etapa 5, Validación QA

**Aplica a:** QA-Engineer, despachado por el Tech-Lead.

* **EXECUTION-PROTOCOL-22.1 — Dos Modos de Despacho:**

  continuo — cada entregable de Etapas 2-4 individualmente apenas se produce (tests unitarios, SLAs por ruta, determinismo); filtro final — antes de declarar la Feature lista, validación del conjunto completo (Frontend sin lógica, FCIS, Zero-Docker, soberanía de datos, y si la Feature declara "Superficie propia": SVF cumplida + Filtro de Integración resuelto).

* **EXECUTION-PROTOCOL-22.2 — Si QA Detecta Defecto:**

  defecto de implementación regresa al engineer dueño (no lo corrige QA); defecto de diseño/spec escala al Architect (`EXECUTION-PROTOCOL-26`).

**Lectura bajo demanda:** [[.agents/knowledge/mutation-filter.md]] y [[.agents/knowledge/ledger-pattern.md]] — filtro de mutación obligatorio antes de cerrar cualquier Story que añada/cambie lógica de correctitud (`EXECUTION-PROTOCOL-9`).

---

## EXECUTION-PROTOCOL-23 — Barrido de Cierre Documental

**Trigger:** tras APTO de QA/Quant (Etapa 5 o 6) y antes de proponer commits — filtro transversal obligatorio de cierre, no una etapa numérica más: se dispara siempre entre el APTO que cierra la implementación y la Etapa 7. Ninguna Story está cerrada hasta que todos los documentos que su avance toca reflejan el estado nuevo.

**Aplica a:** el Tech-Lead, al cerrar cualquier Story/TTR.

* **EXECUTION-PROTOCOL-23.1 — Checklist Mínimo del Barrido:**

  marca cada uno como actualizado o "N/A, por qué":
  1. Sello de la feature (`docs/features/<feature>.md`): banner de estado + `Estado:` + `Última actualización:` + qué quedó pendiente/diferido.
  2. Orden de trabajo: §7 registro de ejecución (entrega, auditoría, veredicto QA) + §8 deudas/diferidos + **§9 Cierre ejecutivo, obligatorio**, ejecución inmediata después de §7/§8. Antes de redactarlo, lee completo [[.agents/knowledge/brief.md]] — única ocasión del ciclo de vida de la Story en que ese archivo se consulta. El contenido de §9, reproducido literal y sin edición, es el único texto que se imprime en el chat como mensaje de cierre de esta Story.
  3. [[docs/DEBT.md]]: abre/actualiza/salda cada `DEBT-XXX` que la iteración generó o pagó — si no está ahí, no está rastreada. Los huecos que reporte el QA van aquí.
  4. [[docs/TEST.md]]: si la feature expone `verify` (Canal #2), SVF (Canal #1) o API (Canal #3), añade/actualiza su bloque con el comando e input reales (fuente de verdad = `crates/app/src/main.rs`). Revisa además que features cerradas en iteraciones previas no hayan quedado sin su bloque (backfill).
  5. [[.agents/state/tech-lead/PROGRESS.md]] + Registro de Estado del ROADMAP: cierre + siguiente paso.
  6. `.agents/memory/` (memoria curada + índice `MEMORY.md`): destila el estado/decisión durable; actualiza contadores de progreso.
  7. Documentos del Architect (`README.md`, `ROADMAP.md`, `ADR`, `SAD`): el Tech-Lead no los edita — si detecta que quedaron desactualizados, escala al Architect.

* **EXECUTION-PROTOCOL-23.2 — Regla de Cierre:**

  si al barrer se encuentra un documento vivo que describe un estado anterior al real, actualizarlo es parte del cierre, no trabajo opcional. Deja constancia del barrido en `PROGRESS.md`. Ausencia de §9 significa Story no cerrada — si se llega al momento de proponer commits sin haber escrito §9 con el Formato de Reporte al CEO, se vuelve atrás y se escribe antes de continuar.

**Filtro de verificación:** `EXECUTION-PROTOCOL-FV-8`.

---

## EXECUTION-PROTOCOL-24 — Etapa 6, Validación Cuantitativa Post-Código

**Trigger:** Feature marcada como matemática/estrategia (Etapa 0) y entregable ya pasó filtro final de QA (Etapa 5).

**Aplica a:** Quant-Engineer, despachado por el Tech-Lead.

* **EXECUTION-PROTOCOL-24 — Rol y Veredicto:** oracle tests, paridad sim/real, sizing bit-a-bit, validación del guantelete con datasets sintéticos. Veredicto APTO marca la Feature/TTR `Completado`, reporta cierre al usuario y vuelve a Etapa 0. Veredicto NO APTO: bug numérico de implementación regresa a Rust-Engineer; defecto de diseño/fórmula escala al Architect (`EXECUTION-PROTOCOL-26`).

---

## EXECUTION-PROTOCOL-25 — Etapa 7, Retroalimentación

**Trigger:** tras cerrar un TTR/Story (Etapa 5/6 con APTO) y antes de volver a Etapa 0.

**Aplica a:** el Tech-Lead.

* **EXECUTION-PROTOCOL-25.1 — Propósito:**

  convertir los errores de la iteración (del Tech-Lead, del Architect o de cualquier ingeniero) en mejoras permanentes de los skills — no un postmortem narrativo, acción concreta sobre los `SKILL.md`.

* **EXECUTION-PROTOCOL-25.2 — Procedimiento:**

  1. Lista los defectos/correcciones reales de la iteración: qué se entregó mal, qué filtro lo atrapó (o por qué no lo atrapó), qué decisión se corrigió. Distingue causa raíz de síntoma.
  2. Por cada error, identifica el skill responsable y redacta una instrucción concreta y accionable que lo habría evitado — no un "ten más cuidado", una regla verificable.
  3. Edita el `SKILL.md` de cada rol implicado con esa instrucción (edición quirúrgica). Si el error fue de gobernanza transversal → [[.agents/knowledge/governance.md]]. Si reveló un vacío de diseño → escala al Architect.
  4. Destila la lección durable a memoria si trasciende esta iteración.
  5. Registra en el §7 de la Orden y en `PROGRESS.md` qué skills se mejoraron y por qué.

* **EXECUTION-PROTOCOL-25.3 — Regla de Cierre:**

  una iteración no está cerrada hasta que sus errores se tradujeron en mejoras de skill. Si no hubo errores, se deja constar explícitamente ("sin hallazgos de mejora").

---

## EXECUTION-PROTOCOL-26 — Escalamiento al Architect

Reactivación puntual — fuera de estos triggers, el Architect permanece inactivo.

**Aplica a:** el Tech-Lead.

* **EXECUTION-PROTOCOL-26.1 — Únicos Triggers:**

  veredicto NO APTO de Quant-Engineer (Etapas 1 o 6) por defecto de diseño/fórmula; QA detecta defecto estructural que implica violación de un ADR, o un TTR/Feature/módulo con referencia huérfana o inconsistente respecto a [[docs/templates/TEMPLATES.md]]; cualquier ingeniero reporta un obstáculo técnico que requiere decisión arquitectónica nueva (contrato roto, dependencia circular entre módulos, ambigüedad de spec no resoluble con lo ya escrito); un Spike de Viabilidad produce un veredicto que debe registrarse como ADR.

* **EXECUTION-PROTOCOL-26.2 — Cómo Escalar:**

  presenta al Architect el problema con evidencia concreta (qué Feature/TTR, qué etapa, qué veredicto/error, qué ingeniero lo reportó, qué documento(s) quedan inconsistentes). Prohibido interpretar o resolver la ambigüedad arquitectónica por cuenta propia — eso es del Architect.

* **EXECUTION-PROTOCOL-26.3 — Tras la Decisión:**

  el Architect edita únicamente los archivos de `docs/` que correspondan. El Tech-Lead no recibe una "entrega": relee los documentos modificados y retoma la orquestación desde la etapa correspondiente — puede implicar reiniciar desde Etapa 0 si cambió el TTR/Feature/secuenciación.

* **EXECUTION-PROTOCOL-26.4 — Trazabilidad de Decisiones Nuevas:**

  una decisión/ADR nuevo o enmendado por el Architect no queda "cumplido" por el solo hecho de existir en `docs/`. El Tech-Lead lo traduce a checks concretos y accionables del Filtro de Cumplimiento ADR ([[.agents/knowledge/order-compliance-filter.md]]), de modo que cada iteración futura se audite automáticamente contra ellos, dejando constancia del barrido en §8 de cada Orden. Si un ADR nuevo no genera ningún check del Filtro, se documenta por qué (ej. es puramente informativo). El seguimiento entre iteraciones vive en el Filtro (enforcement) + `PROGRESS.md`.

**Filtro de verificación:** `EXECUTION-PROTOCOL-FV-9`.

---

## EXECUTION-PROTOCOL-27 — Auditoría de Estado y Definición de Terminado

**Aplica a:** el Tech-Lead, en todo cierre de TTR/Feature.

* **EXECUTION-PROTOCOL-27.1 — Trazabilidad:**

  se mantiene el estado de cada TTR en curso: `Pendiente / En Proceso / Bloqueado / Completado / Secuenciado-En Espera`. Antes de despachar cualquier etapa, se verifica que la etapa previa requerida esté `Completado`. Al cerrar un TTR con veredicto APTO, se reporta el cierre al usuario y se vuelve a Etapa 0 sin esperar instrucción adicional, salvo pausa del usuario.

* **EXECUTION-PROTOCOL-27.2 — DoD Innegociable:**

  una feature no se marca `Completado` sin su Superficie de Verificación Funcional — el único canal que el humano tiene hoy para probar el round-trip front→FFI→back→DB sin leer código. El backend en verde no es "terminado". La UI de la SVF se entrega en la MISMA Story que el backend — prohibido diferirla a una "Story de UI separada" o a una épica posterior de Canvas DAG. El Canal #2 (`drasus verify <feature>`) no reemplaza la SVF en UI: son complementarios.

  **Lectura bajo demanda:** [[.agents/knowledge/ui-verification-model.md]] — detalle completo de qué exige cada clasificación (Superficie propia vs. plomería), la relación SVF/Galería y la verificación de cimiento antes de despachar al Flutter.

* **EXECUTION-PROTOCOL-27.3 — La SVF y los Mocks No Dependen de Adaptadores Diferidos:**

  toda la fontanería (puertos, esquema, lógica, cachés, stubs que sustituyen a un servidor futuro) se conecta ya y la SVF corre contra el backend local real por FFI, la galería con mocks. "El adaptador X es futuro" nunca es excusa para diferir la SVF/galería — prueban el backend local que sí existe.

* **EXECUTION-PROTOCOL-27.4 — Toda Feature Pasa por Diseño de SVF/Galería:**

  incluida la plomería — no basta "la nota de observable". El UI-Designer diseña la entrada de la feature en la SVF y el/los componente(s) de galería con datos mock que permiten entender su comportamiento sin leer código, garantizando que las entradas/salidas recorren transversalmente front→back→DB.

**Filtro de verificación:** `EXECUTION-PROTOCOL-FV-10`.

---

## EXECUTION-PROTOCOL-28 — Gobernanza de Secuenciación por Fase

**Aplica a:** el Tech-Lead, al seleccionar y despachar cualquier TTR.

* **EXECUTION-PROTOCOL-28.1 — Regla del Tech Lead, Alpha vs. Vanidad:**

  un TTR entra al pipeline de ejecución solo si su ausencia bloquea el "Entregable Alpha" de la fase activa (tabla ROADMAP §3-4). Los TTRs no se modifican para esto, solo se secuencian: si no aplica a la fase activa, queda `Secuenciado / En Espera`.

* **EXECUTION-PROTOCOL-28.2 — Filtro EPIC-0 Bloqueante:**

  mientras los 6 Spikes de Viabilidad Técnica no tengan veredicto documentado como ADR, ningún TTR P0 de EPIC-1+ avanza a Etapa 2. Cada Spike se despacha al ingeniero cuyo dominio cubre el riesgo, entrega veredicto binario + Plan B si aplica, y se escala al Architect (`EXECUTION-PROTOCOL-26`) para que lo registre como ADR — el Tech-Lead no redacta ADRs.

* **EXECUTION-PROTOCOL-28.3 — Dependencias Duras:**

  antes de despachar el TTR de una fase, se verifica que los criterios de salida de las fases dependientes (ROADMAP §5) estén `Completado`. Si no, se bloquea y se escala al Architect solo si el bloqueo revela una inconsistencia de secuenciación en el ROADMAP; si es simplemente "aún no completado", se espera.

* **EXECUTION-PROTOCOL-28.4 — KPIs por Fase:**

  en Etapa 5, el SLA exigido es el correspondiente a la fase activa del TTR según la tabla de KPIs del ROADMAP §6 — el QA-Engineer rechaza solo contra el SLA de su ruta/fase, nunca contra la tabla completa.

* **EXECUTION-PROTOCOL-28.5 — Cáscara Delgada por Feature:**

  Etapas 3-4 se activan para cualquier Feature cuya spec declare "Superficie propia" — en la misma Story que su backend, sin esperar a una épica posterior y sin cuota "una pantalla por fase". Si declara "Ventana de Verificación", Etapas 3-4 no se activan directamente para ella — se registra en `PROGRESS.md`/ROADMAP el observable pendiente contra la Feature consumidora declarada, resuelto cuando esa consumidora pase por Etapa 4.

---

## EXECUTION-PROTOCOL-29 — Diagrama de Flujo de Control

**Aplica a:** referencia visual del pipeline completo.

**Plantilla de referencia:**
```
docs/ (ROADMAP + SAD + ADR + modules/*.md + features/*.md)
        │
        ▼
   TECH-LEAD (Etapa 0: lee §0, selecciona TTR según Gobernanza de Fase)
        │
        ├─[UI surface? sin Cáscara Visual]→ UI-Designer (Etapa 0.5) ──actualiza feature doc──┐
        │                                                                                     │
        └─[UI con Cáscara Visual o sin UI]───────────────────────────────────────────────────┤
                                                                                              │
        ├─[matemática?]→ Quant-Engineer (Etapa 1, pre) ─APTO─────────────────────────────────┤
        │                                                                                     │
        └─[no matemática]────────────────────────────────────────────────────────────────────►│→ Rust-Engineer (Etapa 2)
                                                                                               │       │
                                                                                      [UI?] ───┘       │
                                                                                        │               │
                                                                                        ▼               │
                                                                                 Bridge-Engineer (3)    │
                                                                                        │               │
                                                                                        ▼               │
                                                                                 Flutter-Engineer (4)   │
                                                                                 [lee ## Cáscara Visual]│
                                                                                        │               │
                                                                                        └───────┬───────┘
                                                                                                ▼
                                                                                  QA-Engineer (Etapa 5: continuo+final)
                                                                                                │
                                                                                [matemática?]─┴─[no]→ TECH-LEAD: cierre TTR → vuelve a Etapa 0
                                                                                        │
                                                                                        ▼
                                                                          Quant-Engineer (Etapa 6, post) ─APTO→ cierre TTR
                                                                                        │
                                                                                     NO APTO
                                                                                        │
                                                                          ┌─────────────┴─────────────┐
                                                                          ▼                           ▼
                                                                   Rust-Engineer                  Architect (escalamiento)
                                                                  (bug numérico)                  edita docs/
                                                                                                       │
                                                                                                       ▼
                                                                                              TECH-LEAD relee y retoma
```

---

## EXECUTION-PROTOCOL-30 — Lateral, Refactoring-Engineer

* **EXECUTION-PROTOCOL-30 — Trigger y Alcance:** trigger único: el Tech-Lead detecta la condición "Call External Refactor" (archivo >400 líneas, anidación compleja, deuda detectada) durante Etapa 5, o el TTR activo corresponde a empaquetado/release de una épica de cierre. El Tech-Lead despacha, exige suite de tests verde antes/después, valida resultado vía QA-Engineer antes de cerrar. No participa del pipeline de feature normal (Etapas 0-6). Al mover un símbolo (helper/token) a otra capa para eliminar una dependencia invertida, el movimiento incluye sus sub-helpers privados — los que solo ese símbolo consume; dejarlos atrás no elimina la dependencia, la relocaliza a la nueva capa. En la Orden, exige verificación con `grep` de que la capa destino no importe la capa origen tras el movimiento.

---

## EXECUTION-PROTOCOL-31 — Lateral, Naming-Specialist

* **EXECUTION-PROTOCOL-31 — Trigger y Alcance:** ad-hoc, cuando el Architect o el usuario requieren una decisión de nombramiento (producto, módulo, feature). El Tech-Lead despacha, recibe veredicto Top-1, reporta al solicitante. No bloquea ni participa del pipeline de implementación.
