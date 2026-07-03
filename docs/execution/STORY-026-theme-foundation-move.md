# STORY-026 — Mover cimientos de UI (tokens `Gx` + helpers de superficie) de `gallery/` a `theme/`

| Campo | Valor |
|---|---|
| **ID** | STORY-026 |
| **Tipo** | Story (refactor de deuda técnica de frontend) |
| **Épica (Fase)** | EPIC-0 — Fundación (design system) |
| **Sprint** | design-system |
| **Estado** | ✅ Terminado (2026-07-03, build linux verde, cimiento sin dependencia a `gallery/`) |
| **Creada** | 2026-07-03 |
| **Depende de** | STORY-025 (librería de componentes) ✅ |
| **Responsable** | Flutter-Engineer (Sonnet) · audita Tech-Lead |

## 1. Problema (una frase)

La librería de componentes (`ui/lib/components/`) importa sus tokens y superficies desde la carpeta `gallery/` — **dependencia de capas invertida**: la galería debería consumir a los componentes, no ser su cimiento.

Medido: **50** componentes importan `gallery/gallery_tokens.dart` (clase `Gx`) y **29** importan `gallery/gallery_fx.dart` (helpers de superficie).

## 2. Objetivo llano

Mover el cimiento compartido (los tokens `Gx` y los 4 helpers de superficie) a la capa de tema (`ui/lib/theme/`). Resultado: `componentes → theme` y `galería → theme + componentes`. **Cero cambio visual** — es mudanza + repunteo de imports, no se toca ningún valor de token ni lógica de widget.

## 3. Agentes y Modo de Acompañamiento (ADR-0120)

| Agente | Modelo | Modo |
|---|---|---|
| Flutter-Engineer | Sonnet | Autónomo |

## 4. Instrucciones de despacho (prompt exacto para el subagente)

> Eres el **Flutter-Engineer**. Antes de nada: lee `CLAUDE.md`, `.claude/skills/base/SKILL.md` y `.claude/skills/flutter-engineer/SKILL.md`. Declara que los leíste y procede.
>
> **Tarea (refactor puro, cero cambio de comportamiento):** eliminar la dependencia invertida `ui/lib/components/ → ui/lib/gallery/` moviendo el cimiento compartido a `ui/lib/theme/`.
>
> **Movimiento 1 — tokens `Gx`:**
> - Mueve el archivo completo `ui/lib/gallery/gallery_tokens.dart` → `ui/lib/theme/gx_tokens.dart`. La clase sigue llamándose `Gx` (NO la renombres). Ajusta su propio import interno: `import '../theme/theme_scope.dart';` → `import 'theme_scope.dart';` (ahora está en el mismo directorio).
>
> **Movimiento 2 — helpers de superficie:**
> - Crea `ui/lib/theme/surfaces.dart`. Mueve a él EXACTAMENTE estos 4 símbolos top-level desde `ui/lib/gallery/gallery_fx.dart`: `frosted(...)`, `panelSurface(...)`, `cardSurface(...)`, y la clase `PanelFromDecoration`. `surfaces.dart` importa lo que esos 4 necesiten: `package:flutter/material.dart`, `import 'gx_tokens.dart';` y `dart:ui` si `frosted` usa `BackdropFilter`.
> - **NO muevas** los primitivos de animación ni `glassEnhanced`: `frosted/panelSurface/cardSurface/PanelFromDecoration` salen; TODO lo demás (`glassEnhanced`, `HoverGlow`, `LightBurstText`, `HoverableChart`, `SonarPulseWidget`, `ScanRingWidget`, `InteractiveDag`, y los `*Painter`) se queda en `gallery_fx.dart`.
> - En `gallery_fx.dart`: cambia `import 'gallery_tokens.dart';` → `import '../theme/gx_tokens.dart';`. Si algún símbolo que se queda usa alguno de los 4 helpers movidos, añade `import '../theme/surfaces.dart';`.
>
> **Repunteo de imports (todo `ui/lib/`):**
> - Todo import de `gallery/gallery_tokens.dart` (o relativo equivalente) → apunta a `theme/gx_tokens.dart`.
> - Todo import de `gallery/gallery_fx.dart` que se usara para los 4 helpers de superficie → apunta a `theme/surfaces.dart`.
> - **Cuidado (crítico):** varios archivos importan `gallery_fx.dart` para superficies **Y** para primitivos (ej. `ScanRingWidget`). Esos archivos deben CONSERVAR el import de `gallery_fx.dart` (por el primitivo) **y además** añadir `theme/surfaces.dart` (por la superficie). Solo reemplaza el import por completo cuando el archivo únicamente usaba superficies.
> - **La profundidad relativa cambia por carpeta.** Ejemplos: `lib/components/x.dart` y `lib/tabs/x.dart` → `../theme/...`; `lib/gallery/sections/x.dart` y `lib/tabs/verification_bank/x.dart` → `../../theme/...`; `lib/x.dart` (nivel raíz) → `theme/...`. Calcula el prefijo correcto por archivo; no apliques un solo patrón a ciegas.
>
> **Método seguro:** haz los movimientos, luego corre `flutter analyze` e itera arreglando cada import roto hasta CERO errores nuevos (los 2 errores de `lib/src/rust/frb_generated.web.dart` son pre-existentes del target web y se ignoran). Finalmente corre `flutter build linux --release` y confirma verde.
>
> **Prohibido:** renombrar `Gx`, tocar valores de tokens, cambiar lógica de cualquier widget, o mover símbolos distintos a los 4 helpers indicados.
>
> **Entregable:** reporte con (1) los 2 archivos nuevos en `theme/`, (2) el nº de imports repunteados, (3) salida de los greps de aceptación, (4) confirmación de build verde.

## 5. Criterio de aceptación (verificable)

| # | Criterio | Prueba |
|---|---|---|
| 1 | Componentes ya NO dependen de `gallery/` | `grep -rnE "import.*gallery/(gallery_tokens\|gallery_fx)" ui/lib/components/` = **0** |
| 2 | Los cimientos viven en `theme/` | existen `ui/lib/theme/gx_tokens.dart` y `ui/lib/theme/surfaces.dart` |
| 3 | `Gx` intacto (no renombrado) | `grep -rn "class Gx" ui/lib/theme/gx_tokens.dart` = 1 |
| 4 | Sin errores nuevos | `flutter analyze` solo los 2 errores pre-existentes de `frb_generated.web.dart` |
| 5 | Build verde | `flutter build linux --release` → `✓ Built ...` |
| 6 | Cero cambio visual | movimiento puro; sin diffs en valores de tokens ni lógica de widgets |

## 6. Comandos de validación (usuario)

```bash
cd ui
grep -rnE "import.*gallery/(gallery_tokens|gallery_fx)" lib/components/   # debe dar 0
ls lib/theme/gx_tokens.dart lib/theme/surfaces.dart                       # deben existir
flutter analyze 2>&1 | grep -E "error •"                                  # solo los 2 de frb_generated.web.dart
flutter build linux --release
```

## 7. Registro de ejecución

- 2026-07-03 · Tech-Lead · Orden creada y despachada a Flutter-Engineer (Sonnet, Autónomo).
- 2026-07-03 · Flutter-Engineer (Sonnet) · Movió `Gx` (`git mv` gallery_tokens→`theme/gx_tokens.dart`) y los 4 helpers de superficie a `theme/surfaces.dart`; repunteó 80 archivos. Reporte: build verde.
- 2026-07-03 · Tech-Lead · **Auditoría 1 halló residuo:** `theme/surfaces.dart` seguía importando `gallery/gallery_fx.dart` por `glassEnhanced` (dependencia `theme→gallery` relocalizada). Devuelto al mismo agente (SendMessage, contexto intacto) para mover `glassEnhanced` también (único consumidor: `frosted`, ya en `theme/`).
- 2026-07-03 · Flutter-Engineer (Sonnet) · Movió `glassEnhanced` a `theme/surfaces.dart`, borró el import a `gallery/` y el `dart:ui` huérfano de `gallery_fx.dart`.
- 2026-07-03 · Tech-Lead · **Auditoría 2 (cierre) reproducida:** `grep gallery/ en lib/theme/` = 0; `grep gallery_tokens|gallery_fx en lib/components/` = 0; `glassEnhanced` solo en `theme/surfaces.dart`; `flutter analyze` solo los 2 errores web pre-existentes; `flutter build linux --release` = `✓ Built`. **APROBADO.**

### Etapa 7 — ¿Qué aprendimos?
Mi Orden fue **demasiado estrecha**: al mandar mover `frosted` sin sus sub-helpers privados (`glassEnhanced`), la dependencia invertida no se eliminó — se **relocalizó** a `theme/`. El agente lo detectó y reportó; la auditoría lo confirmó y se corrigió barato (SendMessage, no reimplementación). **Regla destilada al `tech-lead/SKILL.md`:** al mover un símbolo para romper una dependencia de capas, incluir sus sub-helpers privados; verificar con `grep` que la capa destino no importe la origen tras el movimiento.

## 8. Gate de Coherencia / desviaciones

- **Contraste bidireccional:** el smell fue detectado por el propio Tech-Lead durante el cierre de STORY-025 (`components/` importa `gallery/`). No lo pide un ADR nuevo; materializa la intención de ADR-0138 enmienda 2026-06-29 (la galería es CONSUMIDORA, no cimiento). Sin escalamiento al Architect (es implementación, no diseño).
- **SAD:** sin impacto (reorganización de capas de UI, no de arquitectura de datos/dependencias entre módulos).
- **Alias:** el alias de namespace de componentes es `custom_ui` (decisión del propietario 2026-07-03); esta Story no lo toca.
