# STORY-020 · Contrato de tokens extensible (cimiento de la biblioteca)

| Campo | Valor |
|---|---|
| **ID** | STORY-020 |
| **Título** | Contrato de tokens extensible: modos N, color de fuente, borde=énfasis, espaciado |
| **Tipo** | Story |
| **Épica (Fase)** | EPIC-0 — Fundación |
| **Sprint** | UI / Galería — Estandarización |
| **Estado** | ✅ Implementado |
| **Responsable** | Flutter-Engineer (Sonnet) · auditó Tech-Lead + QA |
| **Creada** | 2026-06-25 |
| **Completada** | 2026-06-25 |

## 0. Resumen ejecutivo
- **Problema:** el provider de tema no permite configurar el color de fuente (texto invisible sobre fondo claro), el "glass mejorado" no es un modo seleccionable, y faltan tokens de borde/grosor/espaciado. Sin esto, estandarizar componentes sería contra un blanco móvil.
- **Qué se construye:** (1) modos de superficie N-extensibles vía registro de recetas; (2) color de fuente base configurable (auto por paleta + override manual); (3) borde estructural global = énfasis + tokens de grosor; (4) escala de espaciado; (5) selector de color híbrido reutilizable (swatches + rueda HSV); (6) DESIGN.md actualizado.
- **Por qué ahora:** es el contrato anti-retrabajo que congela STORY-021 (estandarización masiva). Indivisible.

## 1. Especificación de origen
- **Spec de origen:** plan `.agents/plans/estamos-teniendo-problemas-importantes-hazy-cloud.md` §Fase A.
- **ADR(s):** ADR-0138 + enmienda 2026-06-25 (Tema Extensible), ADR-0139, ADR-0121 (idioma de código/comentarios).

## 2. Objetivo (una frase llana)
Dejar el provider y los tokens listos para que cualquier propiedad de estilo (modos, color de fuente, bordes, espaciado y futuras) se controle desde el panel con una o pocas líneas, y para que añadir un modo nuevo no obligue a tocar componentes.

## 3. Agentes y Modo de Acompañamiento (ADR-0120)
| Agente | Etapa del pipeline | Depende de | Modo |
|---|---|---|---|
| Flutter-Engineer | Etapa 4 (implementación) | ninguno | **Autónomo** |
| **QA-Engineer** | **Etapa 5 — gate obligatorio** | Flutter-Engineer | **Autónomo** |

## 4. Instrucciones de despacho por agente

### 4.1 Flutter-Engineer
```
Eres el Flutter-Engineer de Drasus Engine. Antes de tocar nada:
1. Lee .claude/skills/base/SKILL.md COMPLETO y declara que lo aplicas.
2. Lee .claude/skills/flutter-engineer/SKILL.md COMPLETO.
3. Lee docs/adr/ADR-0138.md (incluida la "Enmienda 2026-06-25 — Tema Extensible") como contrato vinculante.

CONTEXTO: estandarizamos la biblioteca de componentes UI. Esta Story construye SOLO el contrato de tokens (cimiento); la estandarización de componentes es otra Story. Es trabajo crítico anti-retrabajo: déjalo compilando y coherente.

TAREA — implementa, en este orden, con comentarios de bloque en español antes de cada función/clase (política de base/SKILL.md):

A.1 — ui/lib/drasus_theme.dart
- Modos de superficie N-extensibles: añade `enhancedGlass` al enum DrasusSurfaceMode. Diseña el sistema como un REGISTRO de recetas (una receta de pintado por modo, en un solo lugar) de modo que añadir un 5º/6º modo sea una entrada + su receta, SIN tocar componentes. El panel debe poder iterar sobre los modos disponibles, no una lista hardcodeada.
- Espejo estático del énfasis: añade `_globalAccent` (variable estática, mismo patrón que `_globalSurfaceMode`), sincronizada en load() y setAccent().
- Color de fuente base configurable: mapa _kTextDefaults: Map<DrasusBackgroundPalette, Color> con color de texto legible por paleta (claro sobre fondos oscuros; OSCURO sobre slate y paper). Estado `_textOverride` (Color?, null=auto) + espejo estático `_globalTextColor`. Mutadores setTextColor(Color) y setTextColorAuto(), persistidos en SharedPreferences (clave nueva). load() resuelve el color efectivo; al cambiar de paleta en modo auto, recalcula el texto base.

A.2 — ui/lib/gallery/gallery_tokens.dart
- Getters dinámicos de texto: Gx.textBase (lee _globalTextColor) + Gx.textBaseSecondary/Label/Muted (mismo color a opacidades escaladas). Mantén los const textPrimary/etc. como referencia interna (raw), igual que glassFill.
- Borde dinámico: Gx.borderBase (borde estructural tintado con el énfasis estático) y Gx.accentDynamic (lee _globalAccent). Regla: borde global = énfasis; los colores semánticos (óptimo/alerta/crítico) NO se usan como borde global, solo señalización interna por parámetro.
- Grosor de borde: Gx.borderHairline = 1.0, Gx.borderFocus = 1.5.
- Escala de espaciado (base 4px, según DESIGN.md §Spacing): Gx.space4/8/12/16/24/32/48/64 como const double.
- Extiende los getters surfaceFill/surfacePanel/surfaceCard para contemplar enhancedGlass.

A.3 — ui/lib/gallery/gallery_fx.dart + ui/lib/widgets/glass_surface.dart
- Integra enhancedGlass en el switch de frosted(), GlassSurface y PanelFromDecoration: en modo enhancedGlass usa la receta de glassEnhanced() (gradiente profundo + borde semántico/énfasis + glow amplio). Cuando no haya semanticColor, usa el énfasis dinámico como color de borde por defecto.
- Limpia hardcodes de este archivo: LightBurstText usa Colors.white (línea ~747); GlowButton.textColor default; font sizes sueltos en GlowButton/GlowInput → usa helpers/tokens.

A.4 — ui/lib/tabs/settings_drawer.dart
- Control para los modos de superficie (iterando el registro, para que enhancedGlass y futuros aparezcan solos).
- Control de color de fuente: toggle "Automático por paleta" + selector manual cuando se desactiva.
- Selector de color híbrido REUTILIZABLE en ui/lib/widgets/: swatches curados (presets que respetan el sistema) + rueda de color HSV. Úsalo de forma uniforme en TODOS los controles de color (énfasis, fuente y futuros). Implementación: widget propio compacto (rueda HSV en CustomPainter, sin servicios externos) preferido para no añadir dependencia; si usas un paquete, que sea mantenido y offline. Verifica que énfasis y paleta siguen reactivos.

A.5 — docs/DESIGN.md
- Refleja en §Tokens y §"Modo Global de Superficie": modos extensibles, color de texto base configurable, tokens de borde/grosor/espaciado, selector híbrido, regla "borde global = énfasis". Edición quirúrgica, NO reescritura.

RESTRICCIONES:
- Identificadores en inglés, comentarios y doc-comments en español (ADR-0121).
- Ningún widget de superficie en const (impide reconstrucción al cambiar modo).
- Respeta DESIGN.md §Performance (BackdropFilter solo en chrome; nada de saveLayer/MaskFilter.blur en animación).
- Entrega con `flutter analyze` limpio. Si el SDK lo permite, `flutter build linux` verde.
- NO estandarices los componentes de las secciones (eso es STORY-021). Solo el contrato + los wrappers + el panel.

ENTREGA: resumen de lo implementado + confirmación de analyze/build verde + lista de archivos tocados.
```

**Plan de Implementación** (lo llena el Agente en Modo Autónomo):

> ✅ **Implementado** 2026-06-25 · Flutter-Engineer (Sonnet) · `flutter analyze` limpio (solo warnings pre-existentes en archivos no tocados) · `flutter build linux --debug` verde.

**A.1 — `ui/lib/drasus_theme.dart`**
- `enhancedGlass` añadido al enum `DrasusSurfaceMode`.
- `SurfaceModeRecipe` (class) + `kSurfaceModeRegistry` (const Map, público) como registro N-extensible.
- `_globalAccent` (Color) + `DrasusThemeState.globalAccent` (static getter).
- `kTextDefaults` (const Map público) con color de texto por paleta.
- `_textOverride` (Color?) + `effectiveTextColor` getter + `isTextColorAuto` getter.
- `_globalTextColor` (Color) + `DrasusThemeState.globalTextColor` (static getter).
- `setTextColor(Color)` y `setTextColorAuto()` con persistencia en SharedPreferences.
- `load()` actualizado: inicializa `_globalAccent`, `_globalTextColor`, lee override de texto.
- `setAccent()` sincroniza `_globalAccent`; `setPalette()` recalcula `_globalTextColor` en modo auto.

**A.2 — `ui/lib/gallery/gallery_tokens.dart`**
- `Gx.textBase/textBaseSecondary/textBaseLabel/textBaseMuted` (getters dinámicos).
- `Gx.accentDynamic` y `Gx.borderBase` (getters dinámicos, regla borde=énfasis).
- `Gx.borderHairline = 1.0` y `Gx.borderFocus = 1.5` (const double).
- `Gx.space4…space64` (8 const double de escala de espaciado).
- `surfaceFill/surfacePanel/surfaceCard` actualizados para contemplar `enhancedGlass`.

**A.3 — `ui/lib/gallery/gallery_fx.dart` + `ui/lib/widgets/glass_surface.dart`**
- `frosted()`: rama `enhancedGlass` que delega a `glassEnhanced(Gx.accentDynamic)`.
- `glassEnhanced()`: aplica BackdropFilter también en modo `enhancedGlass`.
- `GlassSurface`: rama `enhancedGlass` con receta inline (gradiente profundo + borde énfasis + glow).
- `PanelFromDecoration`: soporta `enhancedGlass` vía `frosted()` (sin cambio adicional).
- Limpieza de hardcodes: `Colors.white` → `Gx.pureWhite` en `LightBurstText`; `TextStyle(fontSize:13)` → `Gx.uiSans(...)` en `GlowButton`; `TextStyle(fontSize:14)` → `Gx.uiSans(...)` en `GlowInput`.

**A.4 — `ui/lib/widgets/color_picker.dart` (nuevo) + `ui/lib/tabs/settings_drawer.dart`**
- `ColorPickerWidget`: selector híbrido canónico con swatches + rueda HSV (`_HsvDiscPainter`) + deslizador de brillo (`_ValueSlider`). Reutilizable en todos los controles de color.
- `_SurfaceModeOption`: ahora recibe `SurfaceModeRecipe` del registro — label y descripción sin switch hardcodeado.
- `_SectionSuperficie`: itera `kSurfaceModeRegistry.entries` (N-extensible; `enhancedGlass` aparece solo).
- `_SectionApariencia`: usa `ColorPickerWidget` para énfasis (reemplaza swatches estáticos) + añade subsección "COLOR DE FUENTE".
- `_TextColorControl` + `_SimpleToggle`: control "Automático por paleta" + selector manual `ColorPickerWidget`.
- `_AccentSwatch` (obsoleta): eliminada.

**A.5 — `docs/DESIGN.md`**
- §Tokens — Colors: añadidas filas `borderBase`, `borderHairline`, `borderFocus`, `textBase*`, `accentDynamic` con regla "borde global = énfasis".
- §Spacing Scale: nota sobre tokens Dart `Gx.space4…space64`; tabla actualizada.
- §Modo Global de Superficie: tabla ampliada con `enhancedGlass`; nota sobre registro N-extensible, color de texto configurable, selector híbrido canónico, regla `Gx.borderBase`.

## 5. Criterio de aceptación (cada criterio ↔ su prueba)
| # | Criterio verificable | Prueba que lo demuestra |
|---|---|---|
| 1 | `enhancedGlass` es seleccionable y añadir un modo no toca componentes | inspección del registro de recetas + `flutter run`, cambiar a enhancedGlass |
| 2 | El color de fuente es configurable (auto+manual) y el texto se ve sobre `paper` | `flutter run`, paleta paper → texto legible; toggle auto/manual |
| 3 | Bordes globales toman el énfasis; tokens de grosor/espaciado existen | grep de `Gx.borderBase/borderHairline/space*` + cambio de énfasis en vivo |
| 4 | Selector de color híbrido (swatches + rueda) reutilizable | inspección del widget en `ui/lib/widgets/` + uso en énfasis y fuente |
| 5 | `flutter analyze` limpio (y `flutter build linux` verde si hay SDK) | salida del comando |

## 6. Comandos de validación (para el usuario — copy/paste)
```bash
cd ui
flutter analyze
flutter build linux            # gate obligatorio para Stories Flutter
flutter run -t lib/gallery/gallery_preview_main.dart
# En el panel: probar modos (incl. glass mejorado), énfasis, color de fuente (auto/manual), paleta paper.
```

## 7. Registro de ejecución
- 2026-06-25 · Flutter-Engineer (Sonnet) · 1ª entrega · `flutter build linux` verde. Implementó A.1-A.5.
- 2026-06-25 · QA-Engineer (Sonnet) · **NO APTO** · 2 footguns `const` en widgets de superficie (preexistentes), 1 borde hardcodeado introducido en el drawer, 1 comentario inexacto, duplicación de mapa de paletas (ADR-0139).
- 2026-06-25 · Flutter-Engineer (Sonnet) · 5 correcciones aplicadas.
- 2026-06-25 · Tech-Lead · **APROBADO** · verificación independiente reproducida: `flutter build linux --debug` verde; `grep` confirma 0 constructores `const` de superficie, borde del drawer = `Gx.borderBase`, `_kPalettesPublic` eliminado y `kPalettes` única fuente. Contrato CONGELADO — desbloquea STORY-021.

## 8. Pendientes derivados / decisiones
- La estandarización de TODOS los componentes contra este contrato → STORY-021 (bloqueada por el cierre de esta Story).
