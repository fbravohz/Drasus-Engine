# STORY-023 · Tipografía unificada Rajdhani y escala tipográfica vía provider

> **Plantilla de Orden de Trabajo (Spec-Driven).** Es la especificación ejecutable: contiene la instrucción EXACTA que recibió el agente, los comandos de validación, y el registro de lo que pasó. Vive en git, NO en el chat.

| Campo | Valor |
|---|---|
| **ID** | STORY-023 |
| **Título** | Tipografía unificada Rajdhani y escala tipográfica vía provider |
| **Tipo** | Story |
| **Épica (Fase)** | EPIC-0 — Fundación (herramienta interna de diseño) |
| **Sprint** | — |
| **Estado** | ✅ Implementado — build verde, escala tipográfica dinámica activa |
| **Responsable** | Flutter-Engineer (registro retroactivo — 2026-06-26) |
| **Creada** | 2026-06-27 (registro retroactivo: trabajo ejecutado 2026-06-26) |
| **Completada** | 2026-06-26 |

## 0. Resumen ejecutivo

- **Qué problema resuelve:** la tipografía usaba tres familias distintas (SpaceGrotesk, Inter, JetBrainsMono) más la dependencia `google_fonts`. Los helpers de la escala tipográfica de `Gx` (`microLabel`, `body`, `label`, etc.) llevaban `fontSize`/`color` literales hardcodeados, no reaccionaban al cambio de tema. Las pestañas operativas (Reloj, Trabajos, Auditoría, Dashboard) usaban `TextStyle` con literales directos en vez de los tokens del sistema.
- **Qué se construyó:**
  1. Sustitución de las tres familias por **Rajdhani** (Regular 400, Medium 500, SemiBold 600, Bold 700) — una sola familia, sin dependencia de google_fonts.
  2. Método `_syncStyleScale()` en `DrasusThemeState`: sincroniza los espejos estáticos `_globalMicroLabel`/`_globalLabel`/`_globalBody`/etc. desde el `TextTheme` del provider, de modo que todos los helpers `Gx.*` reflejan la fuente y el color activos sin `BuildContext`.
  3. Migración de los helpers nombrados de `Gx` (10 getters de escala) a delegar a `DrasusThemeState` en vez de construir `TextStyle` con literales.
  4. Tokenización de las pestañas: eliminación de `Colors.grey/red/amber/Color(0xFF…)` y `fontFamily: 'monospace'` directos, reemplazados por `Theme.of(context).textTheme.*` y `Gx.*`.
  5. Split de `drasus_theme.dart`: extracción de los datos const (enums, paletas, defaults) a `theme/drasus_palettes.dart`, eliminando la dependencia circular con `theme/drasus_tokens.dart`.
- **Por qué ahora:** los helpers Gx con literales bypassaban el provider — cambiar de paleta/fuente no actualizaba el texto de las pestañas operativas. Requisito para que el sistema de tema sea coherente al 100%.

---

## 1. Especificación de origen

- **Feature(s):** ninguna — es herramienta interna (design system).
- **ADR(s):** ADR-0138 (contrato de tokens extensible), ADR-0121 (identificadores en inglés, comentarios en español).
- **Regla nueva grabada:** `docs/DESIGN.md` §Tipografía — "Todo texto DEBE consumir el theme provider" (2026-06-26).
- **Skill actualizado:** `base/SKILL.md` §"Detección de Bypass del Provider / Deuda Técnica Arquitectónica".

## 2. Objetivo

Que cualquier widget de la app que muestre texto use el token del provider activo, de modo que cambiar paleta/fuente/modo se propague a toda la UI sin excepción.

## 3. Agentes y Modo de Acompañamiento (ADR-0120)

| Agente | Etapa | Modo |
|---|---|---|
| Flutter-Engineer | Etapa 4 — Interfaz | Autónomo (registro retroactivo) |

## 4. Instrucciones de despacho

> Registro retroactivo: el trabajo se ejecutó fuera del flujo formal de Orden de Trabajo. Esta sección documenta la intención para trazabilidad.

Cambios realizados:

1. **Fuentes:** eliminar `Inter-{Regular,Medium}.ttf`, `JetBrainsMono-{Regular,Medium}.ttf`, `SpaceGrotesk-Medium.ttf`. Añadir `Rajdhani-{Regular,Medium,SemiBold,Bold}.ttf`. Actualizar `pubspec.yaml` (eliminar `google_fonts`, declarar única familia `Rajdhani` con pesos 400/500/600/700).

2. **`drasus_theme.dart`:**
   - Cambiar `_kDefaultFont{Display,Sans,Mono}` a `'Rajdhani'`.
   - Añadir espejos estáticos `_globalMicroLabel`/`_globalLabel`/`_globalBody`/`_globalBodySecondary`/`_globalSubheading`/`_globalPanelTitle`/`_globalSectionHeading`/`_globalZuiTitle`/`_globalDataSmall`/`_globalDataHero` (10 espejos de `TextStyle`).
   - Añadir getters estáticos públicos (`globalMicroLabel`, etc.) en `DrasusThemeState`.
   - Añadir `_syncStyleScale()`: lee el `TextTheme` del `buildThemeData()` en curso y actualiza los 10 espejos.
   - Llamar `_syncStyleScale()` al final de `buildThemeData()` y en `load()`.

3. **`gallery_tokens.dart`:** los 10 getters nombrados de `Gx` delegan a los espejos de `DrasusThemeState` (eliminar literales `fontSize`/`color`).

4. **Tabs (4 archivos):** reemplazar `TextStyle(fontFamily: 'monospace', color: Colors.grey/red/amber/Color(0xFF…))` con `Theme.of(context).textTheme.*?.copyWith(color: Gx.*)`.

5. **Split de `drasus_theme.dart`:** mover datos const (enums, paletas, mapas de defaults) a `ui/lib/theme/drasus_palettes.dart`. Actualizar `drasus_tokens.dart` para importar `drasus_palettes.dart` directamente (rompe dependencia circular). Añadir `export 'theme/drasus_palettes.dart'` en `drasus_theme.dart` para backward compatibility.

## 5. Criterio de aceptación

| # | Criterio | Evidencia |
|---|---|---|
| 1 | `flutter build linux` verde. | Verificado en build. |
| 2 | Cambiar paleta de `bunker` a `paper` propaga el color de texto a las tabs operativas. | Verificación visual. |
| 3 | Cero hardcodes de `fontFamily: 'monospace'` o `Colors.*` en las tabs. | `grep -rn "fontFamily.*mono\|Colors\." ui/lib/tabs/` → 0 resultados en texto de chrome. |
| 4 | Los 10 helpers `Gx.*` de escala delegan a `DrasusThemeState`. | Revisión de `gallery_tokens.dart`. |
| 5 | Sin dependencia circular entre `drasus_theme.dart` y `drasus_tokens.dart`. | `flutter analyze` 0 errores. |

## 6. Comandos de validación

```bash
cd /var/home/fbravohz/Documentos/Entornos/Personal/Drasus-Engine/ui

flutter analyze
flutter build linux
grep -rn "fontFamily.*monospace\|Colors\.grey\|Colors\.red\|Colors\.amber" ui/lib/tabs/
# → debe devolver 0 resultados
```

## 7. Registro de ejecución

- 2026-06-26 · Flutter-Engineer · Trabajo ejecutado ad-hoc (fuera del flujo formal de Orden). Fuentes sustituidas, `_syncStyleScale()` añadido, helpers `Gx` migrados, tabs tokenizadas. `flutter build linux` verde.
- 2026-06-26 · Flutter-Engineer · Split de `drasus_theme.dart`: `theme/drasus_palettes.dart` creado, dependencia circular eliminada. QA-Engineer auditó el split → APTO.
- 2026-06-27 · Tech-Lead · Orden creada retroactivamente para formalizar el trabajo y habilitar el commit agrupado.

## 8. Pendientes derivados

- La lección transversal `docs/lessons/dart-flutter/provider-bypass-detection.md` documenta el patrón detectado en esta Story.
- Validar que los helpers `Gx.uiSans()`, `Gx.displayGrotesque()`, `Gx.dataMono()` (los raw, no los nombrados) siguen siendo válidos para casos excepcionales con justificación de comentario.
- Si en el futuro se añade una 4ª familia tipográfica, el único cambio es: nuevo `.ttf` en `assets/`, nueva entrada en `pubspec.yaml`, y actualizar `_kDefaultFont*` en `drasus_theme.dart`.
