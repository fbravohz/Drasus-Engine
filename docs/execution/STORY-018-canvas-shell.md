# STORY-018 · Cáscara del Lienzo (Canvas) — DAG interactivo + lista de features

> **Registro retroactivo (2026-06-25).** Construido ad-hoc sin Orden de Trabajo; el código existe. Esta Orden lo formaliza, no lo re-ejecuta. Spec de origen: plan de feedback del 2026-06-24 §3.

| Campo | Valor |
|---|---|
| **ID** | STORY-018 |
| **Título** | Cáscara del Lienzo — DAG interactivo + lista de features |
| **Tipo** | Story |
| **Épica (Fase)** | EPIC-0 — Fundación |
| **Sprint** | UI / Galería |
| **Estado** | 🟡 Parcial — cáscara visual lista; los nodos reales con datos llegan por épica |
| **Responsable** | Flutter-Engineer (construido ad-hoc) · formaliza Tech-Lead |
| **Creada** | 2026-06-25 (retroactiva) |
| **Completada** | construido antes de 2026-06-25 |

## 0. Resumen ejecutivo
- **Problema:** no había lienzo relacional donde representar el grafo de features/pipeline.
- **Qué se construyó:** una cáscara de lienzo con un DAG interactivo y un panel lateral con la lista de features, sin lógica de negocio.
- **Por qué se formaliza ahora:** cerrar la deuda de gobernanza junto con el resto del trabajo UI.

## 1. Especificación de origen
- **Spec de origen:** `.claude/plans/tengo-feedback-1-en-peaceful-breeze.md` §3 (Canvas Shell) y §D (nodos DAG).
- **ADR(s):** ADR-0136 (Canvas), ADR-0137 (puertos tipados), ADR-0117 (Cáscara Delgada).

## 2. Objetivo (una frase llana)
Tener el lienzo contenedor (grafo interactivo + lista de features) listo para que las épicas le enchufen nodos reales con sus puertos.

## 3. Agentes y Modo de Acompañamiento
| Agente | Etapa | Depende de | Modo |
|---|---|---|---|
| Flutter-Engineer | Etapa 4 | ninguno | Autónomo (construido ad-hoc) |

## 4. Artefacto real (lo que existe en código)
- `ui/lib/tabs/canvas_tab.dart` — la cáscara del lienzo (DAG interactivo + panel de features).
- `ui/lib/gallery/sections/section_dag_nodes.dart` — catálogo de nodos y estados del DAG en la galería.

## 5. Criterio de aceptación (verificado por inspección retroactiva)
| # | Criterio | Evidencia |
|---|---|---|
| 1 | Existe una cáscara de lienzo con DAG interactivo | `canvas_tab.dart` |
| 2 | Existe el catálogo de nodos/estados en galería | `section_dag_nodes.dart` |

## 6. Comandos de validación
```bash
cd ui && flutter analyze && flutter run -t lib/gallery/gallery_preview_main.dart
```

## 7. Registro de ejecución
- 2026-06-25 · Tech-Lead · REGISTRO RETROACTIVO · código verificado presente en `ui/tabs/` y `ui/gallery/sections/`. Estado 🟡 Parcial.

## 8. Pendientes derivados
- Estandarización visual de la cáscara y de los nodos contra el contrato de tokens → STORY-021.
- Nodos reales con puertos tipados → sus épicas respectivas.
