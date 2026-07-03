# STORY-025 · Librería de Componentes Funcional, Neutral al Estilo y al Proyecto

> **Orden de Trabajo (Spec-Driven).** Especificación ejecutable. Vive en git, no en el chat. Si la spec cambia, se edita aquí y se re-despacha.

| Campo | Valor |
|---|---|
| **ID** | STORY-025 |
| **Título** | Librería de Componentes Funcional, Neutral al Estilo y al Proyecto |
| **Tipo** | Story (fundacional de frontend) |
| **Épica (Fase)** | EPIC-0 — Fundación (design system) · prerrequisito de UI de feature EPIC-1+ |
| **Sprint** | design-system |
| **Estado** | ✅ Implementado (librería completa y verde; reescritura SVF STORY-024 pendiente) |
| **Responsable** | Flutter-Engineer (Sonnet) por batch · auditó Tech-Lead + QA-Engineer |
| **Creada** | 2026-06-29 |
| **Completada** | — |

## 0. Resumen ejecutivo

- **Qué problema resuelve:** hoy la galería ES la "librería" pero sus ~39 componentes son **showcase sin lógica** y con el **estilo horneado en el nombre** (`Glow*`/`Glass*`); los básicos (tabla, banner, tooltip, empty, chip) ni existen como clase (están inline). Cada feature reinventa componentes → deriva. (Causa raíz: STORY-024 SVF.)
- **Qué se va a construir:** (1) una **librería real** de componentes funcionales aislados en `ui/lib/components/`, **neutrales al estilo y al proyecto** (clases desnudas, consumo `import ... as ui`), estilo 100% por tema global; (2) la **galería pasa a consumidora** con mocks; (3) la capa de tema `Drasus*` se **renombra a neutral**; (4) índice mantenido `ui/COMPONENTS.md`.
- **Por qué ahora:** prerrequisito de toda UI de feature (ADR-0138 enmienda 2026-06-29). Sin esto, la SVF de STORY-024 y cada feature futura repiten el antipatrón.

---

## 1. Especificación de origen
- **ADR(s):** **ADR-0138** (enmienda 2026-06-29 — RECTORA), **ADR-0139** (carve-out cimientos), ADR-0106, ADR-0136, ADR-0121 (identificadores en inglés).
- **Docs:** `docs/DESIGN.md` (tokens, componentes, motion), memoria `[[galeria-componentes-estado]]`.
- **Lecciones:** `flutter-engineer/SKILL.md` §2e/§2f.

## 2. Objetivo (una frase llana)
Convertir el catálogo estético de la galería en una librería de componentes **funcionales, aislados, reutilizables al 100% y neutrales** (el estilo lo decide el tema, no el componente ni su nombre), con la galería como consumidora de demostración y un índice consultable.

## 3. Agentes y Modo de Acompañamiento (ADR-0120)

| Agente | Etapa | Depende de | Modo |
|---|---|---|---|
| Flutter-Engineer | Etapa 4 — implementación (por batch) | Batch anterior | **Autónomo** |
| QA-Engineer | Etapa 5 — gate obligatorio (por batch) | Flutter-Engineer | Autónomo |

> Se despacha **un batch a la vez**: Flutter implementa → TL audita (build + grep) → QA APTO → se cierra el batch → siguiente. El Batch 0 bloquea a todos.

---

## 4. Instrucciones de despacho (la spec ejecutable)

### 4.0 Convenciones canónicas (FIJO — aplican a TODOS los batches)

**Ubicación y namespace.** Cada componente vive en su archivo en `ui/lib/components/<nombre_snake>.dart`. Un barrel `ui/lib/components/components.dart` reexporta todos. Los consumidores (galería, tabs, features) importan con namespace: `import 'package:<app>/components/components.dart' as ui;` → `ui.Button`, `ui.Table`. **Cero referencia al proyecto** en clases/archivos de componente (rename-proof).

**Nombres neutrales.** Clases desnudas, sin prefijo de estilo (`Glow`/`Glass`) ni de proyecto (`Drasus`). Las colisiones con Material se resuelven por el namespace `ui.`.

**Contrato funcional obligatorio.** Cada componente expone callbacks/controllers/value + estados (default/hover/focus/disabled/error/loading según aplique). **PROHIBIDO** un componente solo-estético sin binding.

**Estilo por tema, nunca por el componente.** El componente consume tokens dinámicos (la capa de tema renombrada) y wrappers de superficie. **PROHIBIDO**: `Color(0x...)` crudo (solo `Colors.transparent`), ramificar por modo/estilo dentro del componente, `const` en widgets de superficie. Antes de entregar: `grep -nE "Color\(0x|Glow|Glass|Drasus" ui/lib/components/` sale limpio.

**Galería = consumidora.** Cada sección de la galería deja de DEFINIR el componente y pasa a CONSUMIRLO de `ui.`, alimentándolo con **mocks** (datos y callbacks simulados que imprimen/actualizan estado local de demo). La galería mostrando el componente con mocks **es** su superficie de verificación.

**Índice.** `ui/COMPONENTS.md` — tabla por componente: `Componente · Archivo · Propósito · Props/Callbacks · Estados · Consumidores`. Se actualiza en el MISMO batch que toca el componente (Definición de Terminado).

### 4.1 Mapa de nombres — Componentes (current → neutral)

> `ui.<Neutral>` en `ui/lib/components/<snake>.dart`.

| Actual | Neutral | Origen actual |
|---|---|---|
| GlowButton | Button | gallery_fx.dart |
| GlowSwitch | Switch | gallery_fx.dart |
| GlowSlider | Slider | gallery_fx.dart |
| GlowInput | Input | gallery_fx.dart |
| GlowDropdown | Dropdown | gallery_fx.dart |
| GlowCalendar | Calendar | gallery_fx.dart |
| GlowToggleButton | ToggleButton | section_buttons_extended.dart |
| GlowLoadingButton | LoadingButton | section_buttons_extended.dart |
| GlowButtonGroup | ButtonGroup | section_buttons_extended.dart |
| GlowFab | Fab | section_buttons_extended.dart |
| GlowSegmented | Segmented | section_buttons_extended.dart |
| GlowProgressCircular | ProgressCircular | section_data_display_extended.dart |
| GlowTreeTable | TreeTable | section_data_display_extended.dart |
| GlowCarousel | Carousel | section_data_display_extended.dart |
| GlowNotificationCard | NotificationCard | section_feedback_extended.dart |
| GlowPopconfirm | Popconfirm | section_feedback_extended.dart |
| GlowStepper | Stepper | section_feedback_extended.dart |
| GlowAccordion | Accordion | section_feedback_extended.dart |
| GlowCombobox | Combobox | section_inputs_extended.dart |
| GlowMultiSelect | MultiSelect | section_inputs_extended.dart |
| GlowNumberInput | NumberInput | section_inputs_extended.dart |
| GlowTextarea | Textarea | section_inputs_extended.dart |
| GlowOtpInput | OtpInput | section_inputs_extended.dart |
| GlowRating | Rating | section_inputs_extended.dart |
| GlowFormField | FormField | section_inputs_extended.dart |
| GlowPagination | Pagination | section_nav.dart |
| GlowTreeView | TreeView | section_nav.dart |
| GlowScrollspy | Scrollspy | section_nav.dart |
| GlowCascader | Cascader | section_std_missing.dart |
| GlowTransferList | TransferList | section_std_missing.dart |
| GlowDateRangePicker | DateRangePicker | section_std_missing.dart |
| GlowTimePicker | TimePicker | section_std_missing.dart |
| GlowColorPicker | ColorPicker | section_std_missing.dart (+ dedup con widgets/color_picker.dart) |
| GlowDropzone | Dropzone | section_std_missing.dart |
| GlowMentionInput | MentionInput | section_std_missing.dart |
| GlowSplitButton | SplitButton | section_std_missing.dart |
| GlowBackToTop | BackToTop | section_std_missing.dart |
| GlassBentoCard | BentoCard | tabs/dashboard_tab.dart |
| GlassSurface | Surface | widgets/glass_surface.dart |

### 4.2 Componentes BÁSICOS a CREAR (hoy inline en la galería, sin clase)

| Neutral | Propósito | Fuente a extraer |
|---|---|---|
| Table | Tabla de datos (columnas, filas, hover, header) | inline en `_dataDisplay()` / usado por SVF |
| Banner | Mensaje contextual (info/success/warn/error) | inline en `_feedback()` |
| Tooltip | Tooltip sobre hover/focus | inline (hoy se usa `Tooltip` Material) |
| Empty | Estado vacío (ícono + mensaje) | inline en `_feedback()`/`_dataDisplay()` |
| Chip | Chip/tag/pill con estado semántico | inline en `_foundations()`/`_dataDisplay()` |
| Badge | Badge numérico/punto | inline en `_foundations()` |
| Card | Tarjeta de contenido genérica | inline en `_layout()` |
| KeyValue | Fila etiqueta→valor (usada por SVF/dashboard) | inline (`_keyValue` en feature docs) |
| DatePicker | Selector de fecha simple | derivar de Calendar/DateRangePicker |
| Tabs | Barra de pestañas (reemplaza `TabBar` Material en operational_panel) | Material hoy |
| Dialog | Diálogo modal | inline/Material hoy |
| Sheet | Bottom/side sheet | inline (`_WidgetCatalogSheet` en dashboard_tab) |

### 4.3 Mapa de nombres — Capa de tema (Drasus* → neutral)

| Actual | Neutral | Archivo (actual → target) |
|---|---|---|
| DrasusApp | AppRoot | lib/main.dart |
| DrasusTheme | ThemeScope | lib/drasus_theme.dart → lib/theme/theme_scope.dart |
| DrasusThemeState | ThemeState | (idem) |
| DrasusGlass | GlassTokens | lib/theme/drasus_tokens.dart → lib/theme/tokens.dart |
| DrasusMotion | MotionTokens | (idem) |
| DrasusSurfaces | SurfaceTokens | (idem) |
| DrasusPalette | PaletteTokens | (idem) |
| DrasusSurfacePalette | SurfacePalette | lib/theme/drasus_palettes.dart → lib/theme/palettes.dart |

> `Gx` (gallery_tokens.dart) NO es referencia al proyecto ni componente; se mantiene como fachada de acceso a tokens dinámicos por ahora. Su retiro (ADR-0138) es pendiente derivado, no parte de STORY-025.

### 4.4 Plan por BATCHES (despacho secuencial, Autónomo)

> Cada batch: Flutter implementa (consumiendo Batch 0) → TL audita (`flutter analyze` + `flutter build linux` + grep limpio) → QA APTO → marcar ✅ en el tracker §4.5 → siguiente batch.

- **Batch 0 — Cimiento (BLOQUEA a todos):** crear `ui/lib/components/` + barrel; renombrar la capa de tema (§4.3) y actualizar TODOS sus consumidores; crear `ui/COMPONENTS.md` (esqueleto + convención). Sin componentes aún. Build verde.
- **Batch 1 — Básicos:** Button, Input, Dropdown, Segmented, Switch, Slider, Surface, Table, Banner, Tooltip, Empty, Chip, Badge, Card, KeyValue, DatePicker.
- **Batch 2 — Inputs extendidos:** Combobox, MultiSelect, NumberInput, Textarea, OtpInput, Rating, FormField, MentionInput, Cascader.
- **Batch 3 — Botones/acciones + feedback:** ToggleButton, LoadingButton, ButtonGroup, Fab, SplitButton, NotificationCard, Popconfirm, Stepper, Accordion.
- **Batch 4 — Data display + navegación + pickers + overlays:** ProgressCircular, TreeTable, Carousel, Pagination, TreeView, Scrollspy, BackToTop, TransferList, DateRangePicker, TimePicker, ColorPicker, Dropzone, Calendar, BentoCard, Tabs, Dialog, Sheet.

### 4.5 TRACKER MAESTRO (registro de todo lo pendiente — se marca al cerrar cada ítem)

**Batch 0 — Cimiento** ✅ (2026-06-29, build verde, 0 referencias de clase Drasus*)
- [x] `ui/lib/components/` + barrel `components.dart`
- [x] Renombrar tema Drasus*→neutral (§4.3) + actualizar consumidores
- [x] `ui/COMPONENTS.md` (esqueleto + convención)

**Batch 1 — Básicos (16)** ✅ (2026-06-29, build verde, grep limpio)
- [x] Button · [x] Input · [x] Dropdown · [x] Segmented · [x] Switch · [x] Slider · [x] Surface · [x] Table · [x] Banner · [x] Tooltip · [x] Empty · [x] Chip · [x] Badge · [x] Card · [x] KeyValue · [x] DatePicker

**Batch 2 — Inputs extendidos (9)** ✅ (2026-06-29, build verde, grep limpio)
- [x] Combobox · [x] MultiSelect · [x] NumberInput · [x] Textarea · [x] OtpInput · [x] Rating · [x] FormField · [x] MentionInput · [x] Cascader

**Batch 3 — Botones/acciones + feedback (9)** ✅ (2026-06-30, build verde, grep limpio)
- [x] ToggleButton · [x] LoadingButton *(consolidado en `ui.Button(loading:)`)* · [x] ButtonGroup · [x] Fab · [x] SplitButton · [x] NotificationCard · [x] Popconfirm · [x] Stepper · [x] Accordion

**Batch 4 — Data display + navegación + pickers + overlays (17)** ✅ (2026-06-30, build verde, cierre logrado)
- [x] ProgressCircular · [x] TreeTable · [x] Carousel · [x] Pagination · [x] TreeView · [x] Scrollspy · [x] BackToTop · [x] TransferList · [x] DateRangePicker · [x] TimePicker · [x] ColorPicker · [x] Dropzone · [x] Calendar · [x] BentoCard · [x] Tabs · [x] Dialog · [x] Sheet

**Cierre** ✅ (2026-06-30)
- [x] Galería 100% consumidora (cero clases `Glow*`/`Glass*` de componente en `gallery/`)
- [x] `grep -rnE "class Glow" ui/lib/` = 0 ; `class Glass` = solo `GlassTokens` (tema); `GlassSurface`→`FrostedSurface`
- [x] `grep -rnE "class Drasus" ui/lib/` = 0 (quedan strings de nombre de producto, no clases)
- [x] `ui/COMPONENTS.md` cubre los 51 componentes
- [ ] SVF de STORY-024 reescrita consumiendo `ui.*` (cierra STORY-024) — **siguiente paso**

## 5. Criterio de aceptación (por batch)

| # | Criterio verificable | Prueba |
|---|---|---|
| 1 | Componentes del batch en `ui/lib/components/`, nombres desnudos, exportados en el barrel | `grep` + `flutter analyze` |
| 2 | Cada componente expone contrato funcional (callbacks/value/estados) | revisión QA del API + uso real en galería |
| 3 | Cero `Color(0x`, `Glow`, `Glass`, `Drasus` en `ui/lib/components/` | `grep -rnE "Color\(0x\|Glow\|Glass\|Drasus" ui/lib/components/` = 0 |
| 4 | La galería consume `ui.*` con mocks (no define el componente) | `grep` + `flutter build linux` verde |
| 5 | `ui/COMPONENTS.md` actualizado con los componentes del batch | inspección |
| 6 | Estilo reacciona al cambio de modo/paleta global (no hardcode) | prueba de widget / inspección visual en galería |

## 6. Comandos de validación
```bash
cd ui
flutter analyze
flutter build linux
grep -rnE "class (Glow|Glass)" lib/                 # objetivo final: 0
grep -rnE "Color\(0x|Glow|Glass|Drasus" lib/components/   # 0
```

## 7. Registro de ejecución
- 2026-06-29 · Tech-Lead · Orden creada · modo Autónomo, big-bang en 5 batches, tracker §4.5.
- 2026-06-29 · Flutter-Engineer (Sonnet) · **Batch 0 APROBADO** · TL reprodujo: `flutter build linux` verde, `grep "\bDrasus[A-Z]" lib/` = 0 referencias de clase, archivos viejos eliminados, clases neutrales (`ThemeScope`/`GlassTokens`/`MotionTokens`/`SurfaceTokens`/`PaletteTokens`/`SurfacePalette`/`AppRoot`) presentes. 14 consumidores actualizados.
- 2026-06-29 · Flutter-Engineer (Sonnet) · **Batch 1 APROBADO** · 16 básicos en `ui/lib/components/` con contrato funcional real (callbacks/controller/estados), grep `Color(0x|Glow|Glass|Drasus` = 0, galería consumidora con mocks, índice actualizado. **Corrección del TL:** `KeyValue` era `const` Stateless leyendo `Gx` (congelaría color al cambiar paleta) → `const` removido para igualar el patrón de los demás Stateless. Build verde reproducido tras la corrección. Desviaciones aceptadas: `Button` +variante `danger`; `value` nullable = modo no-controlado (idioma Flutter, features pasan value explícito); `Tooltip` con OverlayEntry propio (evita colisión Material).
- 2026-06-29 · Flutter-Engineer (Sonnet) · **Batch 2 APROBADO** (TL reprodujo build verde + grep limpio; verificado que `FormField` Stateless NO es const → sin el defecto del Batch 1; `Glow*` migrados eliminados) · 9 inputs extendidos en `ui/lib/components/`: Combobox, MultiSelect, NumberInput, Textarea, OtpInput, Rating, FormField, MentionInput, Cascader. `grep Color(0x|Glow|Glass|Drasus` en `lib/components/` = 0. `flutter build linux` verde. GlowCombobox/MultiSelect/NumberInput/Textarea/OtpInput/Rating/FormField migrados de `section_inputs_extended.dart`; GlowCascader/MentionInput migrados de `section_std_missing.dart`. Galería consumidora con mocks en `gallery_registry.dart`. `COMPONENTS.md` actualizado con las 9 filas. Desviaciones: `MultiSelect` usa `selected?` nullable (modo no controlado, idioma Flutter, consistente con Batch 1); `OtpInput.length` no-const (StatefulWidget con const constructor ✅); `FormField` oculta el `FormField` de Material con `hide` (igual que `Chip`/`Tooltip`).

- 2026-06-30 · Flutter-Engineer (Sonnet) · **Batch 3 APROBADO** (TL reprodujo build verde + grep limpio; los 8 nuevos son StatefulWidget → sin defecto const; `Glow*` eliminados incl. `GlowSegmented` rezagado) · ToggleButton, ButtonGroup, Fab, SplitButton, NotificationCard, Popconfirm, Stepper, Accordion. **Decisión:** `LoadingButton` NO es archivo propio → consolidado en `ui.Button(loading:)` (sin duplicación; la galería lo demuestra con `_LoadingButtonDemo`). Nota: el primer despacho de Batch 3 murió por límite de sesión sin escribir nada; re-despachado y completado.
- 2026-06-30 · Flutter-Engineer (Sonnet) + **Tech-Lead (cierre directo)** · **Batch 4 APROBADO + CIERRE**. El agente creó los 17 (Parte A) pero murió por límite/API antes de la Parte B. El TL recuperó el árbol y remató directamente: (1) reparó 6 archivos rotos (faltaba import de `gallery_fx.dart` para `panelSurface`); (2) completó el barrel; (3) recableó los 13 sitios de `gallery_registry.dart` a `ui.*` con mocks; (4) eliminó las 18 clases `Glow*` huérfanas (script de borrado por bloque top-level); (5) renombró `GlassSurface`→`FrostedSurface` (elimina el token "Glass" del último primitivo). **Cierre verificado:** `class Glow`=0, `class Glass`=solo `GlassTokens`, `flutter build linux` verde, `flutter analyze` solo 3 errores pre-existentes. 51 componentes funcionales, neutrales, theme-driven; galería 100% consumidora. `LoadingButton` consolidado en `ui.Button(loading:)`.

## 8. Pendientes derivados / decisiones
- Retiro de la fachada `Gx` (ADR-0138) — limpieza posterior, no parte de STORY-025.
- Nombres neutrales de tema (§4.3) propuestos por el TL; el Flutter los aplica salvo objeción del propietario.
- **`test/widget_test.dart` referencia `MyApp` inexistente** (test plantilla stale, pre-existente — error en `flutter analyze`). Limpieza: actualizar a `AppRoot` o eliminar el test plantilla. No bloquea la migración.
- **35 menciones "Drasus" restantes** = nombre de producto en strings/comentarios (no clases). Opción propuesta al propietario: centralizar en una constante `kAppName` para rename-proof total del nombre visible (una sola línea a cambiar). Pendiente de decisión.
- **Dedup `ColorPickerWidget` (diferido, deuda menor):** `lib/widgets/color_picker.dart` (`ColorPickerWidget`, el selector híbrido canónico ADR-0138) sigue vivo y lo consume el `settings_drawer` real (3 sitios). NO viola el naming (sin prefijo Glow/Glass/Drasus). Es duplicado conceptual de `ui.ColorPicker`; consolidarlo toca UI de producción y debe preservar el selector híbrido → se difiere a una limpieza propia, no se forzó en STORY-025.
- **Smell arquitectónico (deuda):** los componentes de `ui/lib/components/` importan `panelSurface`/`PanelFromDecoration` desde `lib/gallery/gallery_fx.dart` (la capa de componentes depende de la galería, al revés de lo ideal). Esos helpers de superficie deberían vivir en la capa de tema (`lib/theme/`). Mover `panelSurface`/`cardSurface`/`frosted`/`PanelFromDecoration` a `theme/` es limpieza posterior.
- Tras cerrar STORY-025: reescribir la SVF de STORY-024 consumiendo `ui.*` y cerrar STORY-024.
