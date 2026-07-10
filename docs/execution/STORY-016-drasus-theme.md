# STORY-016 · Tema dinámico: color de énfasis + paleta de fondo + panel de ajustes

> **Registro retroactivo (2026-06-25).** Este trabajo se construyó de forma ad-hoc (sin Orden de Trabajo) entre commits previos; el código existe y está en uso. Esta Orden lo formaliza para cerrar la deuda de gobernanza, no para re-ejecutarlo. La spec de origen es el plan de feedback del 2026-06-24; el artefacto es el código real en `ui/`.

| Campo | Valor |
|---|---|
| **ID** | STORY-016 |
| **Título** | Tema dinámico: énfasis + paleta de fondo + panel de ajustes |
| **Tipo** | Story |
| **Épica (Fase)** | EPIC-0 — Fundación |
| **Sprint** | UI / Galería |
| **Estado** | 🟡 Parcial — el color de fuente base aún NO es configurable (lo añade STORY-020) |
| **Responsable** | Flutter-Engineer (construido ad-hoc, sin registro) · formaliza Tech-Lead |
| **Creada** | 2026-06-25 (retroactiva) |
| **Completada** | construido antes de 2026-06-25; cierre formal con STORY-020 |

## 0. Resumen ejecutivo
- **Problema:** la UI no tenía un tema configurable en runtime; los colores estaban dispersos.
- **Qué se construyó:** un provider de tema (`DrasusThemeState` + `DrasusTheme`) que expone color de énfasis, paleta de fondo (8 ambientaciones) y modo de superficie, todo persistido entre reinicios; y un panel de ajustes lateral para cambiarlos.
- **Por qué se formaliza ahora:** el propietario pidió saldar la deuda de gobernanza (trabajo construido sin Story) antes de estandarizar la biblioteca.

## 1. Especificación de origen
- **Spec de origen:** `.agents/plans/tengo-feedback-1-en-peaceful-breeze.md` §1 (acento dinámico + settings) y §C (paleta de fondo).
- **ADR(s):** ADR-0138 (design system), ADR-0139 (principio rector).

## 2. Objetivo (una frase llana)
Que el usuario pueda cambiar en caliente el color de énfasis, la ambientación de fondo y el modo de superficie de toda la app desde un panel, y que la elección sobreviva al reinicio.

## 3. Agentes y Modo de Acompañamiento
| Agente | Etapa | Depende de | Modo |
|---|---|---|---|
| Flutter-Engineer | Etapa 4 | ninguno | Autónomo (construido ad-hoc) |

## 4. Artefacto real (lo que existe en código)
- `ui/lib/drasus_theme.dart` — `DrasusThemeState extends ChangeNotifier` + `DrasusTheme extends InheritedNotifier`; `accentColor`, `backgroundPalette` (enum `DrasusBackgroundPalette` de 8), `surfaceMode` (enum `DrasusSurfaceMode`); persistencia en `SharedPreferences`; `buildThemeData()` con 4 `ThemeExtension`.
- `ui/lib/tabs/settings_drawer.dart` — panel lateral que muta énfasis/paleta/modo vía los mutadores del provider.
- `ui/lib/theme/drasus_tokens.dart` — las 4 `ThemeExtension` (`DrasusGlass`, `DrasusMotion`, `DrasusSurfaces`, `DrasusPalette`).

## 5. Criterio de aceptación (verificado por inspección retroactiva)
| # | Criterio | Evidencia |
|---|---|---|
| 1 | El énfasis cambia en runtime y persiste | `setAccent()` + `_kKeyAccent` en `drasus_theme.dart` |
| 2 | 8 paletas de fondo seleccionables y persistidas | enum `DrasusBackgroundPalette` + `_kPalettes` + `setPalette()` |
| 3 | El énfasis NO tiñe los colores semánticos de vitalidad | restricción FIJO de ADR-0138, respetada en tokens |

## 6. Comandos de validación
```bash
cd ui && flutter analyze && flutter run -t lib/gallery/gallery_preview_main.dart
# Abrir el panel de ajustes y cambiar énfasis y paleta; reiniciar y verificar persistencia.
```

## 7. Registro de ejecución
- 2026-06-25 · Tech-Lead · REGISTRO RETROACTIVO · código verificado presente en `ui/`. Estado 🟡 Parcial: falta el color de fuente base configurable, que se añade en STORY-020 (contrato de tokens).

## 8. Pendientes derivados
- Color de fuente base configurable (auto por paleta + override) → STORY-020.
- El énfasis debe propagarse a bordes/títulos de TODA la biblioteca → se hace cumplir en STORY-021.
