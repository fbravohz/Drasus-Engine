---
name: banco-estrategias-portafolios-clusters
description: "ADR-0153 (2026-07-12) — fusión de Cluster, eje Workspace/Pipeline, primitivo Selección efímera, semántica de borrado en Canvas. Auditoría del banco de estrategias frente a StrategyQuant X."
metadata:
  node_type: memory
  type: project
  originSessionId: 38b24ffc-f13d-4a05-8f8c-b66f8a11a85b
---

## Contexto — por qué se abrió esta sesión

El usuario (dueño del producto, ex-power-user de StrategyQuant X) pidió auditar si Drasus tiene resuelto el "banco de estrategias/portafolios" al estilo SQX (tabla de miles de filas, stats dinámicas, mover/copiar entre databanks por módulo) y cómo aplica a nodos/features/módulos del Canvas. Se hizo una investigación exhaustiva de docs/ antes de proponer nada (ver [[arquitectura-visual-canvas]] para el Canvas base).

## Hallazgo de partida — mucho ya estaba resuelto (y mejor que SQX)

- **Git-like versioning** (ADR-0005 + `strategy-versioning.md`): ya cubre exactamente la idea de "ramas para no perderse clonando" que el usuario había diseñado — DAG inmutable, `parent_hash`, branches, diff visual + revert. No requirió cambios.
- **Ownership de datos entre módulos** (ADR-0003 Regla de Tabla Única + ADR-0137): una tabla, un dueño, todo el resto referencia por puerto. Nunca hay "desaparece del databank anterior" como en SQX — mejora estructural deliberada.

## Decisiones nuevas de esta sesión (ADR-0153)

1. **Dos ejes ortogonales, no un solo árbol:** Eje de Entidad (`Cluster → Portfolio → Strategy → Logic Block`, el dato) vs. Eje de Proceso (`Workspace → Pipeline → Módulo/Feature`, lo que opera sobre el dato). Se conectan por puerto tipado (ADR-0137), nunca por containment — una Feature no "posee" las estrategias que procesa.
2. **Cluster fusionado:** el nivel 0 del Canvas (ADR-0136) y el "Federated Portfolio Cluster" (ADR-0090) son la misma entidad — dueña: `federated-portfolio` (tabla `clusters` nueva). `pca-toxicity-analyzer`'s `cluster_label` se renombró a `family_label` (ADR-0072 enmendado) para no chocar con el nombre de la entidad.
3. **Workspace (nuevo, nivel 0 del Eje de Proceso):** contenedor visual de N Pipelines. Se navega con el MISMO zoom in-place del canvas único (ADR-0136 preserva su regla FIJO "el canvas es único, no dos canvas" — Workspace NO es una pestaña/canvas separado). Dueña: `pipeline-registry`. **No aísla datos** — una estrategia puede estar referenciada por Pipelines de Workspaces distintos simultáneamente; fue una decisión defendida explícitamente contra la analogía inicial del usuario (fábrica de bebidas con líneas físicamente separadas) porque las estrategias son dato, no materia física, y aislar físicamente resucitaría el problema de copy-paste de SQX que este proyecto evita a propósito.
4. **Selección (nuevo primitivo, en `databank-manager`):** conjunto efímero y MUTABLE de referencias a estrategias (sin peso, sin versionado git-like) — el escalón intermedio entre "una estrategia suelta" y un Portafolio formal. Responde "si tengo 5.000 estrategias para retestear, ¿es automáticamente un Portafolio?" → No. Se arma en el Grid View (nueva superficie tabular de `databank-manager`, complementaria al Canvas nodal que no escala a miles de filas), se conecta directo a un puerto de nodo, y opcionalmente se "promueve a Portafolio" si el usuario quiere pesos/reutilización versionada.
5. **Semántica de borrado en Canvas:** `DISCARDED` (enum `touch_nature` de `expedition_lineage`, ya existía en ADR-0150) es la respuesta correcta para descartar una Strategy/Portfolio/Cluster — nunca DELETE físico, siempre recuperable. Aplica SOLO a `artifact_kind` `STRATEGY`/`PORTFOLIO`/`CLUSTER`. Quitar un nodo Feature/Módulo del lienzo de un Pipeline es distinto: solo edita la topología versionada del Pipeline, no escribe linaje.
6. **Corrección propia:** el Architect asumió mal en un primer momento que "borrar un nodo del Canvas = enviarlo a Withdraw". Falso: `withdraw` es el ciclo de vida de una estrategia que YA llegó a operar con capital real/papel (FSM OPERANDO→PAUSADA→RETIRADA, `order-fsm`, ventana de veto) — no tiene nada que ver con descartar un borrador en fase de I+D.
7. **Deuda documental encontrada y corregida:** `docs/modules/withdraw.md` seguía describiendo la estructura pre-ADR-0137 (módulo dueño de tablas `retirement_records`/`terminal_snapshots` directamente). Se corrigió: esas tablas son propiedad de `order-fsm` (Regla de Tabla Única).

## Gap identificado y registrado, NO resuelto en esta sesión

**Pipeline como proceso recurrente/en bucle** ("corre hasta que lo pares/modifiques/reinicies", modo daemon) — distinto del disparo reactivo por condición de mercado que sí existe (`event-driven-pipeline-triggers`). Registrado como `DEBT-024` (🔴 Alta, urgente por pedido explícito del usuario) + nota en ROADMAP EPIC-7, con disparador: antes de cerrar el diseño de EPIC-7, sesión dedicada.

## Archivos tocados (para referencia, no releer si no cambiaron)

Nuevo: `docs/adr/ADR-0153.md`, `docs/debt/DEBT-024.md`. Editados: `docs/ADR.md`, `docs/DEBT.md`, `docs/adr/ADR-0090.md`, `docs/adr/ADR-0072.md`, `docs/adr/ADR-0136.md`, `docs/adr/ADR-0137.md`, `docs/features/federated-portfolio.md`, `docs/features/pca-toxicity-analyzer.md`, `docs/features/canvas-navigation.md`, `docs/features/databank-manager.md`, `docs/features/pipeline-registry.md`, `docs/features/order-fsm.md`, `docs/modules/withdraw.md`, `docs/modules/manage.md`, `docs/modules/validate.md`, `docs/sad/SAD-06.md`, `docs/ROADMAP.md`.

## Segunda mitad de la sesión (2026-07-12, continuación) — TODO ejecutado

El usuario pidió proceder con los tres pendientes en un mismo turno:

1. **Documentación ER de base de datos — EJECUTADA.** Se leyeron las 20 migraciones completas (verificado: 29 tablas reales, no 50 como sugería un grep impreciso). Creado: `docs/templates/DATA-MODEL.md` (plantilla — única categoría de doc donde el detalle técnico literal SÍ es obligatorio, excepción documentada a "Lo Prohibido" de `TEMPLATES.md`), `docs/DATA-MODEL.md` (índice con las 29 tablas, patrón dominante 1:N vía `owner_id→accounts`, sin pivotes M:N todavía, lista de referencias suaves sin FK física), y 29 fichas en `docs/data-model/<tabla>.md`. Protocolo de sincronización obligatorio anotado: toda migración nueva/modificada actualiza su ficha en el mismo cambio.
2. **Contradicción FSM en `docs/modules/withdraw.md` — CORREGIDA.** La regla "nunca OPERANDO→RETIRADA sin pasar por PAUSADA" chocaba con el Lifecycle documentado ("usuario fuerza retiro manual, bypasea PAUSED"). Resuelto distinguiendo: degradación automática SIEMPRE pasa por PAUSADA (protección contra falsos positivos); retiro manual forzado por el usuario (`ReasonCode=User`) puede saltarla (decisión ya consciente, no necesita ventana de reconsideración).
3. **`DEBT-024` (Pipeline recurrente) — DISEÑO CERRADO, severidad bajada de 🔴 Alta a 🟠 Media.** `event-driven-pipeline-triggers` se generalizó a dos modos de disparo: reactivo (ya existía) + recurrente (TTR-003 nuevo). Diseño: run-state por Pipeline (`RUNNING`/`PAUSED`/`STOPPED`) que lanza una **Expedition nueva en cada iteración** (nunca una Expedition de larga duración — preserva reproducibilidad y la integridad del DSR/ADR-0151). Pausar el bucle permite modificar la topología (nueva versión git-like en `pipeline-registry`) sin perder histórico. Queda pendiente solo la implementación real (disparador: cuando EPIC-7 se construya).

Archivos adicionales tocados en esta mitad: `docs/templates/DATA-MODEL.md`, `docs/templates/TEMPLATES.md`, `docs/DATA-MODEL.md`, `docs/data-model/*.md` (29 archivos), `docs/modules/withdraw.md` (fix FSM), `docs/features/event-driven-pipeline-triggers.md`, `docs/debt/DEBT-024.md`, `docs/DEBT.md`, `docs/ROADMAP.md`.

**No queda ningún pendiente abierto de esta sesión.**
