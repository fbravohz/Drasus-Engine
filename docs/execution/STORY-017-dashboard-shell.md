# STORY-017 · Cáscara del Tablero (Dashboard) — rejilla tipo Bento + registro de widgets

> **Registro retroactivo (2026-06-25).** Construido ad-hoc sin Orden de Trabajo; el código existe. Esta Orden lo formaliza, no lo re-ejecuta. Spec de origen: plan de feedback del 2026-06-24 §2.

| Campo | Valor |
|---|---|
| **ID** | STORY-017 |
| **Título** | Cáscara del Tablero — rejilla Bento + registro de widgets |
| **Tipo** | Story |
| **Épica (Fase)** | EPIC-0 — Fundación |
| **Sprint** | UI / Galería |
| **Estado** | 🟡 Parcial — cáscara visual lista; los widgets reales se enchufan en sus épicas |
| **Responsable** | Flutter-Engineer (construido ad-hoc) · formaliza Tech-Lead |
| **Creada** | 2026-06-25 (retroactiva) |
| **Completada** | construido antes de 2026-06-25 |

## 0. Resumen ejecutivo
- **Problema:** no había un contenedor de tablero donde, a futuro, vivan los widgets operativos.
- **Qué se construyó:** una cáscara de tablero con rejilla tipo Bento y un registro de widgets (catálogo de piezas disponibles), sin lógica de negocio.
- **Por qué se formaliza ahora:** cerrar la deuda de gobernanza junto con el resto del trabajo UI.

## 1. Especificación de origen
- **Spec de origen:** `.agents/plans/tengo-feedback-1-en-peaceful-breeze.md` §2 (Dashboard Shell).
- **ADR(s):** ADR-0136 (arquitectura visual Dashboard + Canvas), ADR-0117 (Cáscara Delgada).

## 2. Objetivo (una frase llana)
Tener el tablero contenedor (rejilla + catálogo de widgets) listo para que, en cada épica, se le enchufen los widgets reales sin rediseñar la pantalla.

## 3. Agentes y Modo de Acompañamiento
| Agente | Etapa | Depende de | Modo |
|---|---|---|---|
| Flutter-Engineer | Etapa 4 | ninguno | Autónomo (construido ad-hoc) |

## 4. Artefacto real (lo que existe en código)
- `ui/lib/tabs/dashboard_tab.dart` — la cáscara del tablero (rejilla Bento, estado vacío).
- `ui/lib/tabs/dashboard_registry.dart` — registro/catálogo de widgets disponibles.

## 5. Criterio de aceptación (verificado por inspección retroactiva)
| # | Criterio | Evidencia |
|---|---|---|
| 1 | Existe una cáscara de tablero con rejilla | `dashboard_tab.dart` |
| 2 | Existe un registro de widgets desacoplado | `dashboard_registry.dart` |

## 6. Comandos de validación
```bash
cd ui && flutter analyze && flutter run -t lib/gallery/gallery_preview_main.dart
```

## 7. Registro de ejecución
- 2026-06-25 · Tech-Lead · REGISTRO RETROACTIVO · código verificado presente en `ui/tabs/`. Estado 🟡 Parcial (cáscara sin widgets reales, que llegan por épica).

## 8. Pendientes derivados
- Estandarización visual de la cáscara contra el contrato de tokens → STORY-021 (si aplica a piezas de la cáscara).
- Widgets reales del tablero → sus épicas respectivas.
