# Plan — Estandarización de la Biblioteca de Componentes (UI)

## Contexto

La galería en `ui/lib/gallery/` se está convirtiendo en una **biblioteca de componentes reutilizables de producción** (la lógica de negocio se añadirá después; hoy las piezas son cáscara visual). Varios agentes y modelos la construyeron sin un estándar común, y el resultado es desigual.

El diagnóstico (3 exploraciones en paralelo) arroja un hallazgo central: **la arquitectura ya es correcta, pero no se respeta**. Existe un provider global sólido (`DrasusThemeState` + `DrasusTheme` en `ui/lib/drasus_theme.dart`) y `docs/DESIGN.md` ya prescribe el modelo "todo componente se alimenta de tokens dinámicos, con override interno". El problema es de **cumplimiento y de piezas faltantes del provider**, no de diseño:

- En la muestra auditada (15 componentes), **~47% está parcial o totalmente fuera del estándar**: colores `Colors.white`/`Colors.black`/`Color(0xFF…)` hardcodeados, radios `BorderRadius.circular(6/8/10)` en vez de tokens `Gx.r*`, bordes con glow índigo fijo en vez del color de énfasis, font sizes sueltos.
- **El color de fuente base no es configurable** y desaparece sobre fondo claro (paletas `slate`/`paper`, que son blancas — ver `ui/lib/drasus_theme.dart:115-128`). Esto es el bug visible más grave.
- **El "glass mejorado" no es un modo global.** Existe como `glassEnhanced()` (`ui/lib/gallery/gallery_fx.dart:220`) pero el enum `DrasusSurfaceMode` solo tiene 3 valores (`glass`/`tint`/`solid`), así que no se puede seleccionar globalmente y se aplica a mano de forma inconsistente.
- **Faltan tokens**: grosor de borde, escala de padding/margen, borde global tintado con el énfasis, y un color de texto base dinámico.
- **Cero comentarios de bloque** en español describiendo cada widget (incumple la política de comentarios de `base/SKILL.md`).

**Decisiones del usuario ya tomadas:**
1. Formalizar **4 modos de fondo** en un enum extensible (el 4º = el "glass mejorado" = `glassEnhanced`).
2. Color de fuente: **auto por paleta + override manual** (cada paleta trae un color de texto legible por defecto; además un selector manual).
3. Refactor de componentes: **4 lotes Sonnet en paralelo** (ingenieros Flutter), tras congelar el contrato de tokens.
4. **Formalizar todo dentro de la Épica 0 (Fundación) con el flujo completo de Tech Lead.** La galería se construyó a lo largo de ~10-15 commits **sin Story ni Orden de Trabajo** (deuda de gobernanza, confirmada contra `.claude/state/tech-lead/PROGRESS.md` y `docs/execution/`). Hay que registrarla retroactivamente como entregable planificado de EPIC-0 y ejecutar la estandarización con Stories numeradas, Órdenes de Trabajo, gate de QA y bitácora — como debió hacerse desde el inicio.

**Resultado esperado:** una biblioteca uniforme donde cada componente toma color de fondo, énfasis, color de fuente, bordes, radios, padding, glow y gradientes del provider con una o pocas líneas, con override interno por componente (estilo CSS + child elements), y donde cambiar un control del panel se propaga a TODA la galería al instante.

---

## Arquitectura de la solución

Dos capas, en orden estricto. La **Fase A (contrato de tokens) debe congelarse antes** de despachar la Fase B, para no refactorizar componentes contra un blanco móvil.

### Patrón a respetar (ya existe, es el estándar)
- **Modo de superficie global**: `DrasusThemeState.globalSurfaceMode` (estático, sin `BuildContext`) leído por `frosted()`, `GlassSurface`, `glassEnhanced()`.
- **Énfasis y paleta**: `DrasusTheme.of(context)?.accentColor` / `?.backgroundPalette` (reactivo vía `InheritedNotifier`).
- **Tokens dinámicos**: getters `Gx.surfaceFill/surfacePanel/surfaceCard` en `ui/lib/gallery/gallery_tokens.dart` que leen el modo global y devuelven el color correcto.
- **Wrappers de superficie**: `frosted()`, `panelSurface()`, `cardSurface()`, `PanelFromDecoration`, `glassEnhanced()` en `gallery_fx.dart`.
- **Override interno**: parámetros `glow`, `semanticColor`, `color` que permiten a cada componente teñir su interacción/estado sin romper el global.

---

## Fase A — Contrato de tokens · **STORY-020** (1 ingeniero Flutter, Sonnet, crítico/anti-rework)

Unidad única e indivisible: nadie refactoriza componentes hasta que esto compile y quede congelado.

### A.1 — Provider: `ui/lib/drasus_theme.dart`
- **Modos de superficie extensibles (N, no 4)**: el sistema debe permitir añadir modos sin límite. Sembrar con `enhancedGlass` como 4º modo, pero la arquitectura es un **registro de recetas de superficie**: cada modo define su receta de pintado en UN solo lugar (tabla/registro keyed por modo), y los componentes NUNCA ramifican por modo — solo consultan el wrapper. Añadir el modo 5/6/… = una entrada en el registro + su receta, cero cambios en componentes. El panel de configuración itera sobre los modos disponibles del registro (no una lista hardcodeada), así un modo nuevo aparece solo en el selector. (Dart compila el enum, así que "extensible" = extensión localizada de un sitio, no runtime.)
- **Espejo estático del énfasis**: añadir `_globalAccent` (variable estática, igual patrón que `_globalSurfaceMode` en línea 148), sincronizada en `load()` y `setAccent()`. Permite que los getters de token lean el énfasis sin `BuildContext`.
- **Color de fuente base configurable (auto + override)**:
  - Mapa `_kTextDefaults: Map<DrasusBackgroundPalette, Color>` con un color de texto legible por paleta (claro para fondos oscuros; **oscuro para `slate` y `paper`**).
  - Estado: `_textOverride` (`Color?`, null = auto) + espejo estático `_globalTextColor`.
  - Mutadores: `setTextColor(Color)` y `setTextColorAuto()`, persistidos en `SharedPreferences` (nuevas claves). `load()` resuelve el color efectivo. Al cambiar de paleta en modo auto, recalcular el texto base.
- Persistencia: nuevas claves `_kKeySurfaceMode` ya existe; añadir `_kKeyTextOverride`. El índice del enum de superficie ya se serializa por `indexOf`, compatible con el 4º valor.

### A.2 — Tokens: `ui/lib/gallery/gallery_tokens.dart`
- **Texto dinámico**: getters `Gx.textBase` (lee `_globalTextColor`) y derivados `Gx.textBaseSecondary/Label/Muted` (mismo color a opacidades escaladas). Estos **reemplazan el uso directo** de los `const Gx.textPrimary/textSecondary/textLabel/textMuted` en los componentes. Mantener los `const` como referencia interna (igual que `glassFill` se mantiene como raw).
- **Borde dinámico**: `Gx.borderBase` (borde estructural tintado con el énfasis estático) y `Gx.accentDynamic` (lee `_globalAccent`) para componentes sin `context`. Regla: **borde global = énfasis**; los colores de estado (verde/ámbar/rojo) quedan SOLO para señalización interna del componente vía parámetro.
- **Grosor de borde**: `Gx.borderHairline = 1.0`, `Gx.borderFocus = 1.5`.
- **Escala de espaciado** (la canoniza ya `DESIGN.md` §Spacing, base 4px): `Gx.space4/8/12/16/24/32/48/64` como `const double`, para padding y margen.
- **Surface getters**: extender `surfaceFill/Panel/Card` para contemplar `enhancedGlass`.

### A.3 — Wrappers: `ui/lib/gallery/gallery_fx.dart` + `ui/lib/widgets/glass_surface.dart`
- Integrar `enhancedGlass` en el `switch` de `frosted()`, `GlassSurface` y `PanelFromDecoration`: cuando el modo global sea `enhancedGlass`, usar la receta de `glassEnhanced()` (gradiente profundo + borde semántico/énfasis + glow amplio). Cuando un componente no provea `semanticColor`, **usar el énfasis dinámico** como color por defecto del borde.
- Limpiar hardcodes de los propios wrappers/widgets de este archivo (ej. `LightBurstText` usa `Colors.white` en `gallery_fx.dart:747`; `GlowButton.textColor` default; font sizes sueltos en `GlowButton`/`GlowInput`).

### A.4 — Panel de configuración: `ui/lib/tabs/settings_drawer.dart`
- Añadir control para el **4º modo** de superficie.
- Añadir control de **color de fuente**: toggle "Automático por paleta" + selector manual cuando se desactiva.
- Verificar que el control de énfasis y de paleta siguen reactivos.
- **Selector de color híbrido reutilizable** (decisión del usuario): un solo widget que combina **swatches curados** (presets rápidos que respetan el sistema de diseño) **+ rueda de color HSV** desplegable para elegir cualquier color. Se usa de forma uniforme en TODOS los controles de color (énfasis, fuente y futuros) — nada de selectores ad-hoc por control. El color de fuente mantiene además el modo "auto por paleta" como red de seguridad de contraste.
  - Implementación: widget propio compacto (rueda HSV en `CustomPainter`, sin servicios externos, coherente con local-first) **o** un paquete mantenido y offline (ej. `flutter_colorpicker`); el ingeniero de STORY-020 decide, prefiriendo no añadir dependencia si el widget propio es razonable. Vive en `ui/lib/widgets/` para reutilizarse.

### A.5 — Documento de diseño: `docs/DESIGN.md`
- Actualizar §Tokens (modos de superficie: ahora 4; tokens nuevos de texto base, borde, grosor, espaciado) y §"Modo Global de Superficie". Edición quirúrgica, no reescritura. (Lo ejecuta el UI-Designer o se pliega al ingeniero de Fase A; al ser cambio de contrato visual, queda reflejado en el doc fuente.)

**Cierre Fase A:** `flutter build linux` (o `flutter analyze`) verde + verificación visual de que el panel cambia los 4 modos, el énfasis y el color de texto. Hasta aquí, **congelar** el contrato.

---

## Fase B — Refactor componente por componente · **STORY-021** (4 ingenieros Flutter Sonnet, en paralelo)

**Cobertura: 100%, sin muestreo.** Se auditan y normalizan ABSOLUTAMENTE TODOS los componentes de `ui/lib/gallery/` (~160 piezas en las 13 secciones + los widgets de `gallery_fx.dart`/`gallery_painters.dart`). El diagnóstico inicial fue una muestra de 15 solo para detectar patrones; la ejecución es exhaustiva. Mecanismo anti-omisión: cada lote arranca enumerando TODOS los componentes de sus archivos en una **checklist nominal** dentro de su Orden de Trabajo, y ningún componente se da por cerrado sin marca explícita. El gate de QA (Fase C) verifica la checklist contra el código — un componente sin tocar es un defecto que regresa el lote.

Lotes disjuntos. Cada ingeniero recibe en su Orden de Trabajo el **contrato congelado de Fase A** (tabla corta embebida en el prompt, no leer archivos enormes) + la checklist de estándar. Tarea por cada componente de su lote:

1. Reemplazar todo color hardcodeado por token (`Gx.textBase`, `Gx.borderBase`/`accentDynamic`, surface getters, colores semánticos solo para estado interno).
2. Reemplazar `BorderRadius.circular(n)` literales por `Gx.rPanel/rButton/rInput/rChip` (excepto `999` para pills, que es intencional).
3. Reemplazar padding/margen literales por la escala `Gx.space*`.
4. Toda superficie visible usa un wrapper (`frosted`/`panelSurface`/`cardSurface`/`glassEnhanced`/`PanelFromDecoration`) y reacciona a los **4 modos** — nunca `Color` sólido suelto, nunca `const` en la instanciación de superficie.
5. Títulos/subtítulos y bordes a nivel global toman el **énfasis**; estados (verde/ámbar/rojo) solo donde el componente señala estado.
6. **Comentario de bloque en español antes de cada widget/clase** (qué hace, qué parámetros recibe, qué tokens consume) — política de `base/SKILL.md`.
7. Dejar cada componente parametrizable (props con defaults), listo para reutilizar en el desarrollo real.
8. **Arreglar bugs de interacción.** Muchos componentes hacen "cosas raras" al hacer clic (estados que no se resetean, gestos que no responden o disparan de más, hover/foco pegados, animaciones que se cortan, taps fuera del área esperada). Al estandarizar cada componente, el ingeniero **prueba la interacción y corrige el comportamiento defectuoso** que esté a su alcance sin añadir lógica de negocio (sigue siendo cáscara visual). Los bugs que dependan de la capa de lógica futura se anotan como pendiente en la Orden, no se inventan. El refinamiento fino se hará al usar los componentes en el desarrollo real; aquí se deja el comportamiento base correcto.

**Reparto sugerido (~13 secciones + archivos núcleo):**
- **Lote 1**: `section_inputs_extended.dart`, `section_buttons_extended.dart`, `section_std_missing.dart`.
- **Lote 2**: `section_nav.dart`, `section_feedback_extended.dart`, `section_data_display_extended.dart`.
- **Lote 3**: `section_dataviz_new.dart`, `section_dataviz_quant.dart`, `section_dataviz_extended.dart`.
- **Lote 4**: `section_dag_nodes.dart`, `section_animations.dart`, `section_trade_tape.dart`, `section_drasus_core_extended.dart`, + barrido de los widgets en `gallery_fx.dart` no cubiertos en A.3.

Archivos representativos del patrón objetivo (ya correctos, usar como molde): `section_buttons_extended.dart:31-55` (`GlowToggleButton`), `section_std_missing.dart` (`GlowCascader`).

---

## Fase C — Gate de calidad · cierre de STORY-021 (QA-Engineer, bloqueante)

- `flutter build linux` verde (yo lo reproduzco antes de despachar QA — sin build verde no hay QA, regla de `base/SKILL.md`).
- Verificación visual matricial: los **4 modos de superficie** × paletas clave (al menos `bunker` oscuro y **`paper` claro**) → todo componente legible, bordes en énfasis, sin texto invisible.
- Auditoría de hardcodes residuales: `grep` de `Colors.white|Colors.black|Color(0x` y `BorderRadius.circular(` con literales numéricos (salvo 999) en `ui/lib/gallery/` → debe quedar en cero (excepto raws internos en `gallery_tokens.dart`).
- Auditoría de comentarios de bloque en español por widget.
- **Verificación de cobertura total:** confrontar la checklist nominal de cada lote contra el código — TODO componente marcado y tocado. Un componente omitido regresa el lote (no se cierra STORY-021 con cobertura parcial).
- **Verificación de interacción:** ejercer el clic/hover/foco/gestos de cada componente en la app real; ningún "comportamiento raro" residual sin corregir o sin anotar como pendiente justificado.

---

## Formalización en la Épica 0 (gobernanza Tech Lead)

La galería y el sistema de tema se construyeron sin gobernanza. **Hallazgo:** el plan previo `tengo-feedback-1-en-peaceful-breeze.md` (2026-06-24) definió STORY-016 a 019 — y su código existe — pero ninguna se registró en el ROADMAP ni tuvo Orden de Trabajo (por eso ADR-0138 ya cita "STORY-019" inexistente en el ROADMAP). Decisión del usuario: **formalización completa**. Numeración: el máximo semánticamente usado es 019; los siguientes libres son **STORY-020** y **STORY-021**.

### G.0 — Registro retroactivo de STORY-016 a 019 (deuda histórica)
- Verificar en código qué construyó cada una (terminado/parcial) antes de registrarlas honestamente:
  - **STORY-016** — DrasusTheme: acento dinámico + paleta de fondo + panel de settings (`ui/lib/drasus_theme.dart`, `ui/lib/tabs/settings_drawer.dart`).
  - **STORY-017** — Dashboard Shell (Bento Grid + widget registry) — verificar existencia/estado.
  - **STORY-018** — Canvas Shell (DAG interactivo + feature list) — verificar (`ui/lib/tabs/canvas_tab.dart` existe).
  - **STORY-019** — Centralización del Design System (ADR-0138): tokens `Gx` + widgets primitivos en `ui/lib/widgets/`.
- Crear Órdenes de Trabajo retro (`docs/execution/STORY-016..019-*.md`) que documenten honestamente: spec de origen = el plan previo; artefacto = el código real; nota de que se construyó ad-hoc sin Orden; estado real auditado. No inventar mapeo criterio→prueba donde no lo hubo; declarar "registro retroactivo".

### G.1 — Registro en el ROADMAP (ficha EPIC-0)
- En `docs/ROADMAP.md`, ficha **EPIC-0**, añadir a la tabla de estado las filas STORY-016 a 021, con enlace a sus Órdenes y estado real. Añadir el entregable **"Biblioteca de Componentes UI (Galería) + Sistema de Tema"** con nota honesta de construcción ad-hoc ahora formalizada y estandarizada.
- Edición quirúrgica (`Edit` en bloques pequeños), nunca reescritura de la ficha.

### G.2 — Enmienda del ADR + Etapa 0.5 (DESIGN.md)
- **Enmienda del ADR (autorizada al Tech Lead por el usuario, sin escalar):** editar el ADR del tema dinámico (ADR-0138, y ADR-0139 si aplica) para enshrinar el **principio de tema extensible**: la capa de estilos es un **registro abierto** — superficies, colores, tipografía, espaciado y futuras propiedades de estilo se añaden sin cambio arquitectónico, controladas desde el provider y el panel. El ADR debe dar explícitamente el poder de extender cualquier propiedad de UI/estilo/tema a futuro. Edición quirúrgica al archivo `docs/adr/ADR-0138.md` (+ índice `docs/ADR.md` si cambia el título/alcance). Es el **primer paso de STORY-020**.
- **Etapa 0.5 (DESIGN.md):** reflejar el contrato visual nuevo (modos extensibles, color de texto base configurable, tokens de borde/grosor/espaciado, selector híbrido, regla "borde global = énfasis") en `docs/DESIGN.md` (§Tokens y §Modo Global de Superficie). Lo ejecuta el UI-Designer (dueño del estándar) o se pliega al ingeniero de STORY-020 con la spec que defino; queda reflejado en el doc fuente antes de cerrar el contrato.

### G.3 — Órdenes de Trabajo (`docs/execution/`)
- **STORY-020** → `docs/execution/STORY-020-token-contract.md` (Fase A). Un solo ingeniero Flutter; tarea crítica/anti-rework. Cierre congela el contrato.
- **STORY-021** → `docs/execution/STORY-021-component-standardization.md` (Fases B+C). Tabla de Agentes con los **4 lotes** (cada lote = un bloque de despacho §4 con su prompt exacto y sus secciones); gate de QA como criterio de cierre.
- Ambas desde la plantilla `docs/execution/_TEMPLATE.md`, con criterio de aceptación y comandos de validación.

### G.4 — Modo de acompañamiento (ADR-0120)
- Antes de despachar, confirmar con el usuario el Modo (Autónomo / Mentor / Revisión) de los ingenieros Flutter, por Story. Se registra en la tabla de la Orden, no en el chat. Bajo Mentor/Revisión yo no despacho: dejo la Orden lista y el usuario invoca `/flutter-engineer`.

### G.5 — Actualización de skills (vigilancia permanente)
- Petición explícita del usuario: que la disciplina quede grabada en los skills para que ningún modelo futuro la vuelva a omitir. Añadir, en edición quirúrgica:
  - **`.claude/skills/flutter-engineer/SKILL.md`**: regla de que todo componente de la biblioteca se construye/edita contra el contrato de tokens dinámicos (los 4 modos, énfasis, texto base, bordes, espaciado, glow), con comentario de bloque en español, **y con la interacción probada y libre de bugs** (clic/hover/foco/gestos) antes de entregar — cobertura 100%, prohibido muestrear.
  - **`.claude/skills/qa-engineer/SKILL.md`**: gate de UI que verifica cobertura nominal total + ausencia de hardcodes + reactividad a los 4 modos sobre fondo oscuro y claro + ausencia de bugs de interacción.
- No se duplica la política de comentarios (ya vive en `base/SKILL.md`); solo se referencia.

### G.6 — Cierre y bitácora
- Tras auditar (yo reproduzco `flutter build` + verificación visual, sin fiarme del reporte): sellar `DESIGN.md` y las features afectadas, actualizar la tabla de estado del ROADMAP y añadir entrada con fecha en `.claude/state/tech-lead/PROGRESS.md` (qué se hizo, evidencia, siguiente paso).
- **Git:** nada se commitea sin que lo pidas explícitamente en el turno; al autorizar, agrupar por tipo (`feat` UI, `docs` ROADMAP/DESIGN, etc.).

## Verificación end-to-end (comandos para el usuario)

```bash
cd ui
flutter analyze
flutter run -t lib/gallery/gallery_preview_main.dart
# En la app: abrir el panel de configuración y probar
#   - los 4 modos de fondo de componente
#   - cambiar el color de énfasis (bordes/títulos deben seguirlo)
#   - cambiar a paleta 'paper' (fondo blanco) → el texto debe verse
#   - cambiar el color de fuente manual y volver a automático
```

## Riesgos / notas

- **Anti-rework**: si la Fase B arranca antes de congelar A, cada cambio de contrato re-toca N archivos. Por eso A es indivisible y bloqueante.
- **`const` y reactividad**: ningún widget de superficie puede ser `const`, o no se reconstruye al cambiar de modo (regla ya documentada en `DESIGN.md`).
- **Performance**: respetar `DESIGN.md §Performance` — `BackdropFilter` solo en chrome, nada de `saveLayer`/`MaskFilter.blur` en animación; el 4º modo (`enhancedGlass`) usa blur 36 solo en modo glass real.
