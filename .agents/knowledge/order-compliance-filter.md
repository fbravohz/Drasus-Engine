# Filtro de Cumplimiento ADR Pre-Orden

Los checks que el Tech-Lead corre sobre una Feature/TTR antes de redactar su Orden de Trabajo, para que ningún ADR vigente quede sin aplicar. Distinto del [[.agents/knowledge/coherence-sweep.md]] (ese lo ejecuta el Architect en Etapa 0.4, produciendo preguntas-respuesta); este lo ejecuta el propio Tech-Lead como checklist estructural.

Slug ID: `ORDER-COMPLIANCE-FILTER`.

Tamaño: `16129 caracteres (~4032 tokens aprox., ratio 4:1)`.

---

## ORDER-COMPLIANCE-FILTER-0 — Lecturas Obligatorias

| Documento | De qué va | Cuándo leerlo (disparador) | Alcance |
|---|---|---|---|
| [[.agents/knowledge/governance.md]] | Gobernanza meta, supremacía absoluta. | `SIEMPRE` al iniciar cualquier skill. | Todos los agentes |
| [[.agents/knowledge/ui-verification-model.md]] | Contrato de Integración UI, SVF/Galería, Definición de Terminado por clasificación. | Al correr `ORDER-COMPLIANCE-FILTER-6` o `-15`. | TL |
| [[docs/adr/ADR-0141.md]] | Las tres tablas de checks M1-M12/R1-R7/F1-F3 del Modelado Relacional Soberano. | Al correr `ORDER-COMPLIANCE-FILTER-13`. | TL |

---

## ORDER-COMPLIANCE-FILTER-FV — Filtro de Verificación

**Aplica a:** el Tech-Lead, antes de crear cualquier Orden de Trabajo.

* **ORDER-COMPLIANCE-FILTER-FV-1:** ¿evalué activamente si la feature, el ADR o el SAD son los equivocados, en vez de asumir por defecto que el documento de mayor jerarquía gana? (`ORDER-COMPLIANCE-FILTER-1`)
* **ORDER-COMPLIANCE-FILTER-FV-2:** ¿barrí el índice completo de ADRs por título, no solo los que la spec ya cita? (`ORDER-COMPLIANCE-FILTER-3.2`)
* **ORDER-COMPLIANCE-FILTER-FV-3:** ¿pasé los 4 pasos de la auditoría ADR-0020 contra la tabla de persistencia real? (`ORDER-COMPLIANCE-FILTER-4`)
* **ORDER-COMPLIANCE-FILTER-FV-4:** ¿verifiqué la sección `## Puertos de Integración` con un tipo válido del catálogo ADR-0137? (`ORDER-COMPLIANCE-FILTER-5`)
* **ORDER-COMPLIANCE-FILTER-FV-5:** ¿clasifiqué la feature con el test de `UI-VERIFICATION-MODEL-2.3`, no por intuición? (`ORDER-COMPLIANCE-FILTER-6`)
* **ORDER-COMPLIANCE-FILTER-FV-6:** ¿corrí las tres tablas de checks de ADR-0141 contra toda migración/Shell/spec con persistencia? (`ORDER-COMPLIANCE-FILTER-13`)
* **ORDER-COMPLIANCE-FILTER-FV-7:** ¿toda migración tocada trae su ficha `docs/data-model/<tabla>.md` actualizada en el mismo cambio? (`ORDER-COMPLIANCE-FILTER-14`)
* **ORDER-COMPLIANCE-FILTER-FV-8:** ¿el resultado del filtro cae en uno de los tres cubos declarados, sin dejar una inconsistencia sin resolver ni escalar? (`ORDER-COMPLIANCE-FILTER-16`)

De ser satisfactoria cada validación de la lista, declara `ORDER-COMPLIANCE-FILTER - FILTRO DE VERIFICACION APROBADO`.

---

## ORDER-COMPLIANCE-FILTER-1 — Postura del Filtro: Contraste Bidireccional

El ADR y el SAD no son "máxima fuente de verdad" que la feature acata sin más — son parte del contraste, no su árbitro automático.

**Aplica a:** el Tech-Lead, en todo el filtro.

* **ORDER-COMPLIANCE-FILTER-1.1 — Los Tres Pueden Estar Mal:**

  reta activamente a los tres — la feature, el ADR en cuestión, el SAD —: ¿la feature contradice al ADR/SAD, o es el ADR/SAD el que quedó desactualizado, inconsistente con un ADR posterior, o admite una mejora que la realidad del trabajo actual revela? No asumas que el documento de mayor jerarquía gana por defecto — investiga cuál de los tres está mal.

* **ORDER-COMPLIANCE-FILTER-1.2 — Este Principio Gobierna Todo el Filtro:**

  aplica con especial fuerza al barrido de ADRs (`ORDER-COMPLIANCE-FILTER-3`) y al de coherencia con el SAD (`ORDER-COMPLIANCE-FILTER-10`): ambos se ejecutan con mentalidad de desafío, no de acatamiento.

**Filtro de verificación:** `ORDER-COMPLIANCE-FILTER-FV-1`.

---

## ORDER-COMPLIANCE-FILTER-2 — Stack Limpio

* **ORDER-COMPLIANCE-FILTER-2 — Tecnologías Rechazadas:** si la spec menciona una tecnología fuera del stack fijo (Python, Ray, ZeroMQ, FastAPI, Tauri, React, TypeScript, Numba, Node.js, Redis, PostgreSQL, ClickHouse, o un puerto de una era anterior del proyecto), el Tech-Lead la corrige directo en `docs/features/<feature>.md` — autoridad propia, no requiere escalar — y documenta la corrección en §8 de la Orden.

---

## ORDER-COMPLIANCE-FILTER-3 — ADRs Vigentes y Barrido Completo

No basta con los ADRs que la spec ya cita — hay decisiones arquitectónicas que aplican a una feature y nunca se tomaron en cuenta al escribirla.

**Aplica a:** el Tech-Lead, en toda Orden nueva.

* **ORDER-COMPLIANCE-FILTER-3.1 — Vigencia de lo Citado:**

  ¿los ADRs citados existen y su función declarada coincide con su contenido real? ¿alguno fue enmendado o extendido por un ADR posterior que contradice lo que la spec afirma? Corrige la referencia si es solo nomenclatura; escala al Architect si hay ambigüedad de diseño real.

* **ORDER-COMPLIANCE-FILTER-3.2 — Barrido del Índice Completo:**

  lee el índice [[docs/ADR.md]] (una línea por ADR) y marca, por el título, los candidatos a aplicar a este trabajo concreto por dominio, capa o naturaleza transversal. Abre bajo demanda solo los candidatos. Para cada ADR que aplique y no esté reflejado en la spec/Orden: si es una restricción o patrón claro, incorpóralo a la Orden (citas + criterio + restricción) y corrige la spec si la contradice, documentando en §8; si revela una ambigüedad o decisión de diseño nueva, escala al Architect. Deja constancia en §8 de qué ADRs barriste y cuáles incorporaste. Aplica el mismo criterio a nivel de épica: verifica los ADRs rectores de la fase activa aunque la feature puntual no los nombre.

**Filtro de verificación:** `ORDER-COMPLIANCE-FILTER-FV-2`.

---

## ORDER-COMPLIANCE-FILTER-4 — Auditoría ADR-0020, Contrato de Persistencia

Checklist de 4 pasos sobre la tabla de persistencia de la feature.

**Aplica a:** el Tech-Lead, en toda feature con persistencia.

* **ORDER-COMPLIANCE-FILTER-4.1 — Grupo I Completo:**

  la tabla declara los 6 campos universales: `id`, `created_at`, `updated_at`, `audit_hash`, `audit_chain_hash`, `event_sequence_id`. Si falta alguno, añádelo tú mismo.

* **ORDER-COMPLIANCE-FILTER-4.2 — Perfil Correcto:**

  el perfil declarado (A = Datos/Ingest, B = IA/R&D, C = Ops/Hot-Path <1ms, D = Ops/Auditoría/Forense) coincide con el tipo de feature. Si está mal, corrígelo tú mismo y ajusta los grupos.

* **ORDER-COMPLIANCE-FILTER-4.3 — Grupos Coherentes con el Perfil:**

  los campos usados (fuera del Grupo I) pertenecen a los grupos que el perfil autoriza (A→III+IV; B→II+III+IV; C→II+IV+V latencia/gobernanza; D→II+IV+V gobernanza/cumplimiento). Quita los campos ajenos al perfil; añade los obligatorios que falten.

* **ORDER-COMPLIANCE-FILTER-4.4 — Campos Dentro del Catálogo de 25:**

  todos los campos de la tabla están en el catálogo (Grupo I: `id`, `created_at`, `updated_at`, `audit_hash`, `audit_chain_hash`, `event_sequence_id`; Grupo II: `owner_id`, `institutional_tag`, `manifest_id`, `access_token_id`; Grupo III: `version_node_id`, `parent_id`, `logic_hash`, `data_snapshot_id`, `transformation_id`; Grupo IV: `process_id`, `session_id`, `node_id`; Grupo V: `portfolio_container_id`, `compliance_status_id`, `risk_audit_id`, `indicator_state_hash`, `execution_latency_ms`, `source_signal_id`, `signature_hash`). Excepción válida: campos propios de la feature, fuera del catálogo de gobernanza, marcados explícitamente como "campo propio fuera del catálogo". Variantes de latencia con precisión o contexto distinto (`latency_ns`, `recovery_latency_ms`, `heartbeat_latency_ms`) son válidas si están documentadas como derivadas de `execution_latency_ms`. Un campo no catalogado sin documentar se escala al Architect (requiere 3+ features usándolo para entrar al catálogo, o se queda como campo local).

**Filtro de verificación:** `ORDER-COMPLIANCE-FILTER-FV-3`.

---

## ORDER-COMPLIANCE-FILTER-5 — Puertos de Integración ADR-0137

* **ORDER-COMPLIANCE-FILTER-5 — Sección Obligatoria:** la feature tiene `## Puertos de Integración` con al menos un puerto declarado con ID de tipo válido del catálogo de ADR-0137. Si la sección no existe o está vacía, complétala tú mismo derivando los puertos de "Ciclo de Vida", "Comportamientos Observables" y "Dependencias". Si la feature es plomería (sin superficie propia y sin interacción tipada con otras features), declara al menos sus puertos técnicos (`Job`, `AuditEvent`, `TelemetrySample`, etc.). Si los puertos son ambiguos o exigen una decisión de tipado nueva fuera del catálogo, escala al Architect.

**Filtro de verificación:** `ORDER-COMPLIANCE-FILTER-FV-4`.

---

## ORDER-COMPLIANCE-FILTER-6 — Contrato de Integración UI

* **ORDER-COMPLIANCE-FILTER-6 — Clasificación Obligatoria:** toda feature declara "Superficie propia" o "Ventana de Verificación" en su spec — el Tech-Lead lo completa derivándolo si falta, con el test de clasificación exacto de `UI-VERIFICATION-MODEL-2.3`. Una feature de plomería sin su Ventana de Verificación registrada no se sella.

  **Lectura bajo demanda:** [[.agents/knowledge/ui-verification-model.md]] — detalle completo de la clasificación y de qué exige cada una para la Definición de Terminado.

**Filtro de verificación:** `ORDER-COMPLIANCE-FILTER-FV-5`.

---

## ORDER-COMPLIANCE-FILTER-7 — FCIS Íntegro

* **ORDER-COMPLIANCE-FILTER-7 — Core vs. Shell:** la sección FCIS distingue Core (lógica pura, cero imports de I/O) de Shell (efectos del sistema). Si hay lógica de infraestructura en el Core, corrígela tú mismo.

---

## ORDER-COMPLIANCE-FILTER-8 — TTRs Completos

* **ORDER-COMPLIANCE-FILTER-8 — Postcondición Verificable:** cada TTR tiene problema, postcondición verificable y criterio concreto. Un TTR vacío o ambiguo es vacío de diseño — escala al Architect, no lo completes por tu cuenta.

---

## ORDER-COMPLIANCE-FILTER-9 — Referencias Huérfanas

* **ORDER-COMPLIANCE-FILTER-9 — Puerto, Proceso o Servicio Inexistente:** si la spec cita un puerto, proceso, daemon o servicio externo que ya no existe en el diseño, corrígelo tú mismo.

---

## ORDER-COMPLIANCE-FILTER-10 — Impacto en el SAD

Todo cambio que toque arquitectura de datos, contratos, dependencias entre módulos, gobernanza, flujos o SLAs se verifica contra el SAD.

**Aplica a:** el Tech-Lead, en cualquier corrección que haga durante el filtro o cualquier sellado post-auditoría.

* **ORDER-COMPLIANCE-FILTER-10.1 — Procedimiento:**

  identifica qué sección(es) de [[docs/SAD.md]] (índice de 20 secciones) cubren el área tocada — ej. SAD-08 Arquitectura de Datos, SAD-16 Grafo de Dependencias, SAD-20 Soberanía de Datos, SAD-11 Invariantes. Abre solo esa sección y compara.

* **ORDER-COMPLIANCE-FILTER-10.2 — Resultado:**

  si el SAD quedó desalineado por tu cambio, corrígelo con `Edit` quirúrgico y documenta en §8 qué sección se impactó. Si el desalineamiento revela una decisión de diseño nueva, escala al Architect. Si nada se ve afectado, déjalo constar igual ("SAD: sin impacto").

---

## ORDER-COMPLIANCE-FILTER-11 — Calidad de la Spec

Auditoría de consistencia interna: la documentación se escribió en distintos momentos y con distintos agentes.

**Aplica a:** el Tech-Lead, en toda feature antes de despachar.

* **ORDER-COMPLIANCE-FILTER-11.1 — Qué Verificar:**

  ¿la sección "¿Qué es esta feature?" describe comportamiento verificable o es marketing? ¿los comportamientos observables tienen la forma "cuando X, el sistema hace Y"? ¿hay secciones marcadas "pendiente" o vacías? ¿los TTRs de la misma feature se contradicen entre sí?

* **ORDER-COMPLIANCE-FILTER-11.2 — Resultado:**

  inconsistencias graves de spec se escalan al Architect. Problemas de redacción o secciones vacías sin impacto en el comportamiento se corrigen directo, documentando qué se corrigió en §8.

---

## ORDER-COMPLIANCE-FILTER-12 — Modelo de Tabla Única por Feature, ADR-0003

* **ORDER-COMPLIANCE-FILTER-12 — Sin Copias en Módulos Consumidores:** una Feature tiene una sola tabla, creada en la migración del primer módulo del pipeline que la construye. Los módulos consumidores posteriores acceden vía el puerto `public_interface` del módulo dueño, nunca repiten la migración. Si un consumidor necesita registrar datos propios relacionados, usa sus propias tablas con una referencia — no duplica la tabla de la feature. Al auditar una feature multi-consumidor, verifica que su contrato de persistencia sea único y esté en el módulo dueño correcto; si hay duplicación, escala al Architect.

---

## ORDER-COMPLIANCE-FILTER-13 — Modelado Relacional Soberano, ADR-0141

* **ORDER-COMPLIANCE-FILTER-13 — Tres Tablas de Checks:** toda migración SQL, todo Shell de feature que persista, y toda spec con persistencia se auditan contra las tres tablas de checks de ADR-0141 — no las copies aquí, la fuente canónica vive en [[docs/adr/ADR-0141.md]] §"Gate del Tech Lead". En síntesis: **M1–M12** por migración (precios `INTEGER ×10⁸`, nunca `REAL`; timestamps `INTEGER` ns UTC; PK `TEXT` UUIDv7; enums con `CHECK`; JSON con `json_valid`; FK con `ON DELETE RESTRICT`, jamás CASCADE; índice en cada FK; `event_sequence_id UNIQUE` en append-only vs. `row_version` en mutables; `audit_chain_hash` NULL en génesis sin sentinel; referencia Parquet con formato + CHECK; `STRICT` en toda tabla nueva). **R1–R7** por Shell (pool con `foreign_keys=ON` + `busy_timeout`; transacción única estado+auditoría con `BEGIN IMMEDIATE`; conversión de escala solo en Shell; UTC; reconciler Parquet documentado; DuckDB propio vs. puerto ajeno). **F1–F3** por spec (perfil de retención; sección de puertos; `schema_version` si escribe Parquet). Corrige cada check incumplido si es implementación, y deja constancia en §8. Fase GREENFIELD vigente: el baseline de migraciones se edita in-situ (recrear con `STRICT` y UUIDv7) en vez de apilar migraciones correctivas.

**Filtro de verificación:** `ORDER-COMPLIANCE-FILTER-FV-6`.

---

## ORDER-COMPLIANCE-FILTER-14 — Sincronización del Modelo de Datos, ADR-0153

* **ORDER-COMPLIANCE-FILTER-14 — Ficha en el Mismo Cambio:** toda Story/Task que crea o modifica una migración en `migrations/*.sql` actualiza, en el MISMO cambio, la ficha `docs/data-model/<tabla>.md` (plantilla [[docs/templates/DATA-MODEL.md]]) — tabla nueva → ficha nueva + fila nueva en el índice [[docs/DATA-MODEL.md]]; columna/FK/CHECK modificado → ficha actualizada, columna por columna contra la migración real, no contra el resumen del ingeniero. Si tocaste la migración y no la ficha, la Story no está completa — no la cierres.

**Filtro de verificación:** `ORDER-COMPLIANCE-FILTER-FV-7`.

---

## ORDER-COMPLIANCE-FILTER-15 — Tres Manifestaciones de UI + Canal #2

* **ORDER-COMPLIANCE-FILTER-15 — Pre-Despacho a Flutter:** toda Story de feature con Superficie propia entrega, además del backend: su sección en el Banco de Verificación (SVF), su Dashboard widget (`dashboard_registry.dart` con `available: true`, read-only), y su nodo Canvas DAG solo si la infra Canvas existe (si no, registra la deuda en `PROGRESS.md`/ROADMAP). Antes de despachar a Flutter, la feature doc debe tener `## Cáscara Visual` + su sección del Banco (inputs/observables) + spec del widget.

  **Lectura bajo demanda:** [[.agents/knowledge/ui-verification-model.md]] — verificación de cimiento UI (componentes de galería como widgets funcionales), criterio de cierre completo (SVF + widget + nodo/deuda + `drasus verify`) y detalle canónico en ADR-0117/0136/0142.

---

## ORDER-COMPLIANCE-FILTER-16 — Resultado del Filtro

Todo filtro cae en exactamente uno de tres cubos.

**Aplica a:** el Tech-Lead, al cerrar el filtro.

* **ORDER-COMPLIANCE-FILTER-16.1 — Sin Inconsistencias:**

  crea la Orden y despacha.

* **ORDER-COMPLIANCE-FILTER-16.2 — Inconsistencia Corregible por el Tech-Lead:**

  stack/nomenclatura/campos ADR-0020/FCIS — corrígelas in-situ, documenta en §8 de la Orden, y procede.

* **ORDER-COMPLIANCE-FILTER-16.3 — Bloqueo que Exige al Architect:**

  ambigüedad de diseño, TTR incompleto, decisión arquitectónica nueva, o un ADR/SAD que el contraste de `ORDER-COMPLIANCE-FILTER-1` reveló equivocado, obsoleto o mejorable — para, escala al Architect, espera la actualización de spec/ADR/SAD antes de crear la Orden.

**Filtro de verificación:** `ORDER-COMPLIANCE-FILTER-FV-8`.
