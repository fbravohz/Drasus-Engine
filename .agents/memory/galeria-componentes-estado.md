---
name: galeria-componentes-estado
description: "Estado de la Galería de Componentes (pestaña Components), fuentes embebidas, golden tests y README de ui/."
metadata: 
  node_type: memory
  type: project
  originSessionId: cd158787-0796-4910-892c-ca56cc35eebe
---

Galería de Componentes de Drasus (render-only, sin FFI/Bridge) viva en `ui/lib/gallery/`, registrada como pestaña "Components" en `ui/lib/panel_operativo.dart` (pareo posicional 1:1 en TabBar/TabBarView). Spec maestro: `docs/DESIGN-COMPONENTS-GALLERY.md`; tokens en `docs/DESIGN.md`.

**Estado al 2026-06-23:**
- Catálogo prácticamente completo (~140 piezas) en `gallery_tab.dart` + `gallery/sections/*`. Cerradas las piezas STD (cascader, transfer/dual-list, date-range, time-picker, color-picker, file-upload/dropzone, mention-input, split-button, back-to-top, anchor/scrollspy).
- `gallery_tokens.dart` (clase `Gx`) es el ÚNICO sitio con hex. Iconos vía `iconsax_plus` (tokens `Gx.icon*`); descartado phosphor_flutter (incompatible con Flutter 3.44).
- Lenguaje visual: glow (BoxShadow) + gradientes dentro de familia semántica + vidrio esmerilado Apple (BackdropFilter). PROHIBIDO aberración cromática RGB / "cristales Vivid" (al usuario le quedaron mal, ya removidos).

**Fuentes:** embebidas en `ui/assets/fonts/` y declaradas en `pubspec.yaml` (Space Grotesk w500, Inter w400/w500, JetBrains Mono w400/w500). Ya NO se usa `google_fonts` en runtime → 100% offline. **Pendiente:** `JetBrainsMono-Medium.ttf` pesa 2.4 MB porque es variante NerdFont; reemplazar por la limpia (~110 KB) de fonts.google.com/specimen/JetBrains+Mono cuando haya red (no rompe nada, solo infla la app).

**Golden tests:** `ui/test/gallery_golden_test.dart` con PNG en `ui/test/goldens/` (gallery_full_scroll.png, gallery_fundamentos.png). Las fuentes reales se cargan con `FontLoader` en `setUpAll` para que el texto salga legible y no como cajas Ahem.

**Preview aislado:** `ui/lib/gallery/gallery_preview_main.dart` monta solo la galería sin RustLib.init → `flutter run -d linux -t lib/gallery/gallery_preview_main.dart`.

**Documentación:** `ui/README.md` reúne todos los comandos (toolchain, compilar bridge Rust, deps por SO, tests, galería, golden, release) con §11 explicando fuentes/ttf/golden en lenguaje llano. Solo Linux está configurado como target (existe `ui/linux/`); macOS/Windows/Android/iOS por habilitar con `flutter create --platforms=...`.

Trabajo de implementación se hizo vía subagentes Sonnet (ver [[roles-explicitos-y-subagentes]]); el usuario no quiere que el Architect actúe como ingeniero frontend directamente.

**⚠️ Distinción crítica (descubierta 2026-06-28, STORY-024 SVF): showcase ≠ librería usable.** La galería es **render-only**: los componentes están dibujados pero la mayoría NO exponen callbacks ni binding de datos (`GlowButton` sin `onPressed`, `GlowDropdown`/`GlowSegmented` sin `onChanged`, `GlowInput` sin `controller`), y varios que `DESIGN.md` nombra NO existen como clase (`GlowTable`, `GlowEmpty`, `GlowBanner`, `GlowTooltip`, `GlowDatePicker`). Catálogo de `DESIGN.md` ≠ componentes implementados ≠ componentes funcionales. Consecuencia real: el Designer especificó y el Flutter intentó consumir componentes no usables → el Flutter los reimplementó inline (deriva + un color hardcodeado en bypass del provider). **Decisión del usuario "Librería real primero" — refinada 2026-06-28 (la arquitectura correcta, NO "galería con callbacks"):** el problema es estructural, no solo "faltan callbacks". Auditado: 37 componentes `Glow*` + 2 `Glass*` (`GlassBentoCard`, `GlassSurface`) — **el estilo está horneado en el NOMBRE**, lo que viola la neutralidad de estilo. Requisitos del usuario para la refactorización fundacional (ANTES de construir más UI de features):
  1. **Componentes neutrales al estilo, uno por concepto** (`Button`, `Table`, `Dropdown`, `Input`, `Banner`, `Tooltip`, `Empty`…), SIN prefijo de estilo. PROHIBIDO duplicar el mismo concepto por estilo (no `GlowButton` + `GlassButton`). El aspecto (glow/glass/etc.) lo decide la **configuración global** (`DrasusTheme`/`Gx`), que YA existe.
  2. **Aislados y 100% reutilizables** — cada componente en su archivo propio, en su capa propia (p.ej. `ui/lib/components/`), consumido tanto por la galería como por las features.
  3. **La galería es CONSUMIDORA, no dueña:** solo muestra cada componente y simula su lógica/datos con **mocks**. NO define los componentes ni los acopla.
  4. **Índice de componentes mantenido** (en el README del front o doc dedicado) para que los ingenieros lo consulten.
  5. Dejar **separado y documentado** componentes / sus usos / su lógica / sus implementaciones ANTES de construir más.
La SVF de STORY-024 se reescribe para consumir esa librería al final.

**✅ EJECUTADO — STORY-025 (2026-06-30, cerrada 2026-07-03):** ADR-0138 enmendado (enmienda 2026-06-29) + ADR-0139 carve-out. Librería completa en `ui/lib/components/`: **51 componentes** funcionales, clases desnudas neutrales, estilo por tema. **Alias de namespace DEFINITIVO: `custom_ui`** (decisión del propietario 2026-07-03) — `import '<pkg>/components/components.dart' as custom_ui;` → `custom_ui.Button`, `custom_ui.Table`… Elige `custom_ui` (no `ui`) porque no colisiona con `dart:ui as ui`; el parche `uic` quedó obsoleto. Galería recableada como consumidora con mocks. Tema renombrado `Drasus*`→neutral (`ThemeScope`/`GlassTokens`/`MotionTokens`/`SurfaceTokens`/`PaletteTokens`/`SurfacePalette`/`AppRoot`); `GlassSurface`→`FrostedSurface`. `LoadingButton` vive como `custom_ui.Button(loading:)`. Nombre de producto centralizado en `ui/lib/app_meta.dart` (`kAppName`/`kAppVersion`) — chrome rename-proof. Índice mantenido en `ui/COMPONENTS.md`. Cierre verificado: `class Glow`=0, `class Glass`=solo `GlassTokens`, build linux verde. **✅ SVF de STORY-024 REESCRITA y CERRADA (2026-07-03):** `sovereign_data_fetcher_section.dart` consume `custom_ui.*` (Input/Dropdown/Segmented/Button/Banner/Chip/KeyValue/Empty/Table/Tooltip/Surface); lógica FFI/estado intacta; `ScanRingWidget` (primitivo de animación en `gallery_fx`) se mantiene. **Reglas para nuevas features:** consume `custom_ui.*`; NUNCA reimplementes un componente inline (ver [[verification-surface-svf]]); StatelessWidget que lee `Gx.*` no es `const`; cero hex fuera del gamut de un picker. **Deuda restante (una sola):** mover `panelSurface`/`PanelFromDecoration`/`cardSurface`/`frosted` de `gallery_fx.dart` a `theme/` (requiere mover `Gx` también) — refactor propio, no "menor". Lecciones grabadas en `ui-designer`, `flutter-engineer` (§2e/§2f) y `tech-lead`.
