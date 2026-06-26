# STORY-019 · Centralización del Design System (ADR-0138) — tokens `Gx` + widgets primitivos

> **Registro retroactivo (2026-06-25).** Construido ad-hoc sin Orden de Trabajo; el código existe y ADR-0138 ya lo citaba como "STORY-019" pese a no estar registrado. Esta Orden lo formaliza, no lo re-ejecuta. Spec de origen: ADR-0138.

| Campo | Valor |
|---|---|
| **ID** | STORY-019 |
| **Título** | Centralización del Design System — tokens + widgets primitivos |
| **Tipo** | Story |
| **Épica (Fase)** | EPIC-0 — Fundación |
| **Sprint** | UI / Galería |
| **Estado** | 🟡 Parcial — la migración incremental que ADR-0138 prescribe está INCOMPLETA; la completa STORY-021 |
| **Responsable** | Flutter-Engineer (construido ad-hoc) · formaliza Tech-Lead |
| **Creada** | 2026-06-25 (retroactiva) |
| **Completada** | construido antes de 2026-06-25; cierre con STORY-020 + STORY-021 |

## 0. Resumen ejecutivo
- **Problema:** los helpers visuales (vidrio, odómetro, arco, scan eléctrico) estaban pegados y divergentes en muchas secciones.
- **Qué se construyó:** la capa de tokens `Gx` (`gallery_tokens.dart`) y cinco widgets primitivos reutilizables (`ui/lib/widgets/`), con las superficies leyendo el modo global.
- **Por qué se formaliza ahora:** ADR-0138 ya lo daba por existente ("STORY-019") sin que estuviera registrado; además su migración quedó a medias, que es la causa raíz del desorden que STORY-020/021 corrigen.

## 1. Especificación de origen
- **Spec de origen:** ADR-0138 (Design System Centralization).
- **ADR(s):** ADR-0138 + su enmienda 2026-06-25 (Tema Extensible), ADR-0139 (principio rector).

## 2. Objetivo (una frase llana)
Tener una sola fuente de verdad para tokens y comportamientos visuales repetidos, para que un cambio de token se propague a toda la UI sin tocar N sitios.

## 3. Agentes y Modo de Acompañamiento
| Agente | Etapa | Depende de | Modo |
|---|---|---|---|
| Flutter-Engineer | Etapa 4 | ninguno | Autónomo (construido ad-hoc) |

## 4. Artefacto real (lo que existe en código)
- `ui/lib/gallery/gallery_tokens.dart` — fachada `Gx`: colores, helpers tipográficos, radios, glow, gradientes, getters dinámicos de superficie.
- `ui/lib/widgets/` — `glass_surface.dart`, `odometer_number.dart`, `animated_arc.dart`, `electric_line_chart.dart`, `electric_primitives.dart`.
- `ui/lib/theme/drasus_tokens.dart` — las 4 `ThemeExtension`.

## 5. Criterio de aceptación (verificado por inspección retroactiva)
| # | Criterio | Evidencia |
|---|---|---|
| 1 | Existen tokens centralizados (no constantes sueltas) | `gallery_tokens.dart` + `drasus_tokens.dart` |
| 2 | Existen ≥2 widgets primitivos reutilizables | 5 widgets en `ui/lib/widgets/` |
| 3 | Las superficies leen el modo global | getters `Gx.surface*` + wrappers en `gallery_fx.dart` |

## 6. Comandos de validación
```bash
cd ui && flutter analyze && flutter run -t lib/gallery/gallery_preview_main.dart
```

## 7. Registro de ejecución
- 2026-06-25 · Tech-Lead · REGISTRO RETROACTIVO · código verificado presente. Estado 🟡 Parcial: la migración incremental de ADR-0138 NO está completa — muchos componentes aún hardcodean color/radio/borde y no reaccionan a todos los modos. Esa deuda es exactamente el alcance de STORY-021.

## 8. Pendientes derivados
- Extender el contrato de tokens (modos extensibles, color de fuente, borde=énfasis, espaciado/grosor) → STORY-020.
- Completar la migración: estandarizar TODOS los componentes contra el contrato → STORY-021.
