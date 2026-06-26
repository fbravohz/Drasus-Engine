// Efectos e interacción de la galería: vidrio Apple (frosted), glow, gradientes
// y micro-animaciones funcionales. Son widgets de UI puros (sin lógica de
// negocio ni FFI): el estado que manejan es local y visual (hover, foco, valor
// de un slider, día seleccionado), permitido en una Cáscara Delgada.

import 'dart:math';
import 'dart:ui' as ui;
import 'package:flutter/material.dart';
import 'gallery_tokens.dart';
import 'gallery_painters.dart';
import '../drasus_theme.dart';

// Vidrio Apple — o lo que indique el modo global de superficie.
// Lee DrasusThemeState.globalSurfaceMode para decidir la receta:
//   glass → BackdropFilter + blur + rim (vidrio completo)
//   tint  → Solo glassFill, sin blur ni rim (panel translúcido)
//   solid → El color sólido indicado (por defecto panelSolid)
Widget frosted({
  required Widget child,
  EdgeInsets padding = const EdgeInsets.all(12),
  double radius = Gx.rChrome,
  double blur = 36,
  Color? solidColor,
  List<BoxShadow>? glow,
}) {
  final mode = DrasusThemeState.globalSurfaceMode;

  if (mode == DrasusSurfaceMode.solid) {
    return Container(
      padding: padding,
      decoration: BoxDecoration(
        // Gx.surfacePanel deriva del color de fondo de componentes (solid: tal cual).
        color: solidColor ?? Gx.surfacePanel,
        borderRadius: BorderRadius.circular(radius),
        border: Border.all(color: Gx.borderBase),
        boxShadow: glow,
      ),
      child: child,
    );
  }

  if (mode == DrasusSurfaceMode.tint) {
    return Container(
      padding: padding,
      decoration: BoxDecoration(
        // Color de componentes al 65%: translúcido pero visible, sin blur.
        color: Gx.surfaceFill.withOpacity(0.65),
        borderRadius: BorderRadius.circular(radius),
        border: Border.all(
          color: Gx.accentDynamic.withOpacity(0.035),
        ),
        boxShadow: glow,
      ),
      child: child,
    );
  }

  // mode == enhancedGlass: gradiente profundo + borde del énfasis dinámico + glow amplio.
  // Usa el énfasis dinámico como color de borde (la regla "borde global = énfasis").
  if (mode == DrasusSurfaceMode.enhancedGlass) {
    return glassEnhanced(
      child: child,
      semanticColor: Gx.accentDynamic,
      padding: padding,
      radius: radius,
      glow: glow,
    );
  }

  // mode == glass: vidrio Apple completo.
  return ClipRRect(
    borderRadius: BorderRadius.circular(radius),
    child: BackdropFilter(
      filter: ui.ImageFilter.blur(sigmaX: blur, sigmaY: blur),
      child: Container(
        padding: padding,
        decoration: BoxDecoration(
          // Gradiente sutil tintado con el color de componentes (0.18 de opacidad).
          gradient: LinearGradient(
            begin: Alignment.topCenter,
            end: Alignment.bottomCenter,
            colors: [
              Gx.componentBgBase.withOpacity(0.18),
              Colors.transparent,
            ],
          ),
          // Base del glass: color de componentes al 25% (translúcido sobre el blur).
          color: Gx.surfaceFill.withOpacity(0.25),
          borderRadius: BorderRadius.circular(radius),
          border: Border.all(
            color: Gx.accentDynamic.withOpacity(0.035),
          ),
          boxShadow: glow,
        ),
        child: child,
      ),
    ),
  );
}

// ─── Surface Builders ───
// Wrappers que reemplazan BoxDecoration(color: Gx.surfacePanel / surfaceCard).
// En modo glass, el hijo recibe vidrio completo (BackdropFilter + rim-light).
// En modo tint/solid, solo color de fondo — sin blur.
//
// USO:  Gx.panelSurface(child: ..., radius: Gx.rPanel)
//       en vez de Container(decoration: BoxDecoration(color: Gx.surfacePanel, ...))
//
// Para migrar patrones existentes sin reescribir toda la decoration:
//   Container(decoration: BoxDecoration(color: Gx.surfacePanel, ...), child: x)
//   → panelFromDecoration(decoration: BoxDecoration(color: Gx.surfacePanel, ...), padding: ..., child: x)

// Panel con efecto glass/tint/solid según el modo global. Wrapper sobre frosted() con surfacePanel.
Widget panelSurface({
  required Widget child,
  double radius = Gx.rPanel,
  EdgeInsets? padding,
  List<BoxShadow>? glow,
}) {
  return frosted(
    child: child,
    padding: padding ?? const EdgeInsets.all(12),
    radius: radius,
    // surfacePanel deriva del color de componentes (+4% ligereza en solid).
    solidColor: Gx.surfacePanel,
    glow: glow,
  );
}

// Card con efecto glass/tint/solid según el modo global. Wrapper sobre frosted() con surfaceCard.
Widget cardSurface({
  required Widget child,
  double radius = Gx.rPanel,
  EdgeInsets? padding,
  List<BoxShadow>? glow,
}) {
  return frosted(
    child: child,
    padding: padding ?? const EdgeInsets.all(10),
    radius: radius,
    // surfaceCard deriva del color de componentes (+8% ligereza en solid).
    solidColor: Gx.surfaceCard,
    glow: glow,
  );
}

// Drop-in wrapper para reemplazar Container(decoration: BoxDecoration(color: Gx.surfacePanel/Card), ...)
// sin reescribir toda la decoration existente.
class PanelFromDecoration extends StatelessWidget {
  final Widget child;
  final EdgeInsetsGeometry? padding;
  final EdgeInsetsGeometry? margin;
  final double? width;
  final double? height;
  final BoxConstraints? constraints;
  final AlignmentGeometry? alignment;
  final BoxDecoration decoration;
  final Color? solidColor;

  // No es const: lee el modo global estático y debe poder reconstruirse al
  // cambiar el modo. Un constructor const congelaría el modo de superficie
  // (regla DESIGN.md §Superficie: ningún widget de superficie en const).
  PanelFromDecoration({
    super.key,
    required this.child,
    this.padding,
    this.margin,
    this.width,
    this.height,
    this.constraints,
    this.alignment,
    required this.decoration,
    this.solidColor,
  });

  @override
  // Envuelve el Container original en frosted() si el modo no es solid; en solid usa la
  // decoration original sin modificar. Toma el borde y sombras de la decoration original.
  Widget build(BuildContext context) {
    final mode = DrasusThemeState.globalSurfaceMode;

    if (mode == DrasusSurfaceMode.solid) {
      return Container(
        padding: padding,
        margin: margin,
        width: width,
        height: height,
        constraints: constraints,
        alignment: alignment,
        decoration: decoration,
        child: child,
      );
    }

    // glass / tint / enhancedGlass: frosted() aplica la receta correcta de cada modo.
    final radiusGeom = decoration.borderRadius;
    double r = Gx.rPanel;
    if (radiusGeom != null) {
      final resolved = radiusGeom.resolve(Directionality.of(context));
      r = resolved.topLeft.x;
    }

    List<BoxShadow>? shadows;
    if (decoration.boxShadow != null) {
      shadows = decoration.boxShadow!
          .map((s) => BoxShadow(
              color: s.color,
              blurRadius: s.blurRadius,
              spreadRadius: s.spreadRadius,
              offset: s.offset))
          .toList();
    }

    return frosted(
      child: Container(
        margin: margin,
        alignment: alignment,
        child: child,
      ),
      padding: padding != null && padding is EdgeInsets ? padding as EdgeInsets : const EdgeInsets.all(12),
      radius: r,
      solidColor: solidColor,
      glow: shadows,
    );
  }
}

// ─── Vidrio Premium (Receta Result) ───
// Basado en los componentes Result (success/error) de section_feedback_extended.dart,
// que son el gold standard. A diferencia de frosted() que usa un gradiente uniforme
// [0x14AAAAFF, transparent] + BackdropFilter, este wrapper usa:
//   1. Gradiente [surfacePanel → deepSpace] — profundidad tonal dramática
//   2. Borde semántico coloreado — emphasis, no borderPanel neutro
//   3. Glow amplio del color semántico — blur 20, opacidad baja
//   4. BackdropFilter solo en glass mode, no en tint/solid

// Panel/card con gradiente profundo, borde semántico y glow amplio.
// glass:  BackdropFilter blur 36 + gradiente glassFill→deepSpace + borde semántico
// tint:   gradiente glassFill→deepSpace + borde semántico (sin blur)
// solid:  gradiente panelSolid→deepSpace + borde semántico (sin blur)
Widget glassEnhanced({
  required Widget child,
  required Color semanticColor,
  EdgeInsets padding = const EdgeInsets.all(16),
  double radius = Gx.rChrome,
  double blur = 36,
  List<BoxShadow>? glow,
}) {
  final mode = DrasusThemeState.globalSurfaceMode;

  // En solid: color de componentes directo; en glass/tint/enhancedGlass: mismo color
  // (los wrappers aplican la opacidad adecuada al renderizar).
  final fill = mode == DrasusSurfaceMode.solid ? Gx.surfacePanel : Gx.componentBgBase;

  final shadows = glow ?? Gx.glow(semanticColor, blur: 20, opacity: 0.15);

  Widget content = Container(
    padding: padding,
    decoration: BoxDecoration(
      gradient: Gx.linear([fill, Gx.canvasBase],
          begin: Alignment.topCenter, end: Alignment.bottomCenter),
      borderRadius: BorderRadius.circular(radius),
      border: Border.all(color: semanticColor.withAlpha(80)),
      boxShadow: shadows,
    ),
    child: child,
  );

  // glass y enhancedGlass aplican BackdropFilter; tint y solid solo el Container.
  if (mode == DrasusSurfaceMode.glass || mode == DrasusSurfaceMode.enhancedGlass) {
    return ClipRRect(
      borderRadius: BorderRadius.circular(radius),
      child: BackdropFilter(
        filter: ui.ImageFilter.blur(sigmaX: blur, sigmaY: blur),
        child: content,
      ),
    );
  }

  return content;
}

// ─── Interacción ───
class HoverGlow extends StatefulWidget {
  final Widget child;
  final Color color;
  final double radius;
  final double scale;
  const HoverGlow(
      {super.key,
      required this.child,
      required this.color,
      this.radius = Gx.rPanel,
      this.scale = 1.03});

  @override
  State<HoverGlow> createState() => _HoverGlowState();
}

class _HoverGlowState extends State<HoverGlow> {
  bool _h = false;
  @override
  // Renderiza el hijo con escala animada al hover y sombra glow; estado local _h.
  Widget build(BuildContext context) {
    return MouseRegion(
      onEnter: (_) => setState(() => _h = true),
      onExit: (_) => setState(() => _h = false),
      child: AnimatedScale(
        scale: _h ? widget.scale : 1.0,
        duration: const Duration(milliseconds: 160),
        curve: Curves.easeOut,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 220),
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(widget.radius),
            boxShadow: _h
                ? Gx.glowStrong(widget.color)
                : Gx.glow(widget.color, blur: 10, opacity: 0.16),
          ),
          child: widget.child,
        ),
      ),
    );
  }
}

// Botón con gradiente, glow potente, hover y "propagación de luz" al pulsar
// (un pulso de glow que estalla del centro hacia afuera, inspiración Reflect).
class GlowButton extends StatefulWidget {
  final String label;
  final List<Color> gradient;
  final Color glowColor;
  final Color? textColor;
  const GlowButton(
      {super.key,
      required this.label,
      required this.gradient,
      required this.glowColor,
      this.textColor});

  @override
  State<GlowButton> createState() => _GlowButtonState();
}

class _GlowButtonState extends State<GlowButton>
    with SingleTickerProviderStateMixin {
  late final AnimationController _burst =
      AnimationController(vsync: this, duration: const Duration(milliseconds: 460));
  bool _hover = false;
  bool _down = false;

  @override
  void dispose() {
    _burst.dispose();
    super.dispose();
  }

  @override
  // Botón con gradiente y glow progresivo (_hover), escala al presionar (_down) y pulso de
  // luz (_burst) al soltar. Animaciones locales de UI, no dependen del Bridge.
  Widget build(BuildContext context) {
    return MouseRegion(
      onEnter: (_) => setState(() => _hover = true),
      onExit: (_) => setState(() => _hover = false),
      cursor: SystemMouseCursors.click,
      child: GestureDetector(
        onTapDown: (_) => setState(() => _down = true),
        onTapUp: (_) {
          setState(() => _down = false);
          _burst.forward(from: 0); // dispara la explosión de luz
        },
        onTapCancel: () => setState(() => _down = false),
        child: AnimatedScale(
          scale: _down ? 0.96 : 1.0,
          duration: const Duration(milliseconds: 110),
          child: AnimatedBuilder(
            animation: _burst,
            builder: (_, child) {
              // Pulso: 0 → pico → 0 mientras la animación corre.
              final burst = sin(_burst.value * pi);
              final k = (_hover ? 1.2 : 0.75) + burst * 1.3;
              return Container(
                padding:
                    const EdgeInsets.symmetric(horizontal: 18, vertical: 11),
                decoration: BoxDecoration(
                  gradient: Gx.linear(widget.gradient),
                  borderRadius: BorderRadius.circular(Gx.rButton),
                  boxShadow: Gx.glowStrong(widget.glowColor, k),
                ),
                child: child,
              );
            },
            // Estilo del texto del botón: usa el helper uiSans (familia Inter)
            // en lugar de TextStyle suelto para respetar la voz tipográfica del sistema.
            child: Text(widget.label,
                style: Gx.uiSans(
                  fontSize: 13,
                  weight: FontWeight.w600,
                  color: widget.textColor ?? Gx.canvasBase,
                ).copyWith(letterSpacing: 0.3)),
          ),
        ),
      ),
    );
  }
}

// Switch funcional de palanca: alterna estado al tocar, con knob deslizante animado, gradiente y
// glow en ON. Estado local (_on) sin dependencia del Bridge.
class GlowSwitch extends StatefulWidget {
  final bool initial;
  final Color color;
  const GlowSwitch({super.key, this.initial = true, this.color = Gx.reactorGreen});
  @override
  State<GlowSwitch> createState() => _GlowSwitchState();
}

class _GlowSwitchState extends State<GlowSwitch> {
  late bool _on = widget.initial;
  @override
  // Switch de palanca con knob deslizante animado, gradiente y glow en ON; estado local _on.
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: () => setState(() => _on = !_on),
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 220),
        curve: Curves.easeOut,
        width: 48,
        height: 26,
        padding: const EdgeInsets.all(3),
        decoration: BoxDecoration(
          gradient: _on
              ? Gx.linear([
                  widget.color.withOpacity(0.4),
                  widget.color.withOpacity(0.15)
                ])
              : null,
          color: _on ? null : Gx.gaugeTrack,
          borderRadius: BorderRadius.circular(999),
          // OFF: borde estructural global (dinámico con el énfasis activo).
          border: Border.all(color: _on ? widget.color : Gx.borderBase),
          boxShadow: _on ? Gx.glow(widget.color, blur: 16, opacity: 0.5) : null,
        ),
        child: AnimatedAlign(
          duration: const Duration(milliseconds: 220),
          curve: Curves.easeOut,
          alignment: _on ? Alignment.centerRight : Alignment.centerLeft,
          child: Container(
            width: 18,
            height: 18,
            decoration: BoxDecoration(
              shape: BoxShape.circle,
              // OFF: color de knob usando token dinámico de texto muted.
              color: _on ? widget.color : Gx.textBaseMuted,
              boxShadow:
                  _on ? Gx.glow(widget.color, blur: 12, opacity: 0.8) : null,
            ),
          ),
        ),
      ),
    );
  }
}

// Slider funcional: se arrastra, con relleno en gradiente y manija con glow.
class GlowSlider extends StatefulWidget {
  final double initial;
  final List<Color> gradient;
  final Color glowColor;
  const GlowSlider(
      {super.key,
      this.initial = 0.62,
      this.gradient = Gx.gradTransition,
      this.glowColor = Gx.transitionIndigo});
  @override
  State<GlowSlider> createState() => _GlowSliderState();
}

class _GlowSliderState extends State<GlowSlider> {
  late double _v = widget.initial;
  void _set(double dx, double w) =>
      setState(() => _v = (dx / w).clamp(0.0, 1.0));
  @override
  // Slider con relleno degradado, glow en el track y manija circular; estado local _v.
  Widget build(BuildContext context) {
    return LayoutBuilder(builder: (ctx, box) {
      final w = box.maxWidth;
      return GestureDetector(
        onPanDown: (d) => _set(d.localPosition.dx, w),
        onPanUpdate: (d) => _set(d.localPosition.dx, w),
        child: SizedBox(
          height: 26,
          child: Stack(alignment: Alignment.centerLeft, children: [
            Container(
                height: 5,
                decoration: BoxDecoration(
                    color: Gx.gaugeTrack,
                    borderRadius: BorderRadius.circular(3))),
            FractionallySizedBox(
              widthFactor: _v,
              child: Container(
                  height: 5,
                  decoration: BoxDecoration(
                      gradient: Gx.linear(widget.gradient),
                      borderRadius: BorderRadius.circular(3),
                      boxShadow:
                          Gx.glow(widget.glowColor, blur: 10, opacity: 0.6))),
            ),
            Align(
              alignment: Alignment(_v * 2 - 1, 0),
              child: Container(
                  width: 16,
                  height: 16,
                  decoration: BoxDecoration(
                      shape: BoxShape.circle,
                      // Token dinámico para el knob del slider — legible en paper y bunker.
                      color: Gx.textBase,
                      boxShadow: Gx.glowStrong(widget.glowColor))),
            ),
          ]),
        ),
      );
    });
  }
}

// Input funcional: foco real (FocusNode) con borde y glow limpios — sin la
// aberración cromática que quedaba mal. El glow es la señal de foco.
class GlowInput extends StatefulWidget {
  final String hint;
  final String? initial;
  final Color color;
  const GlowInput(
      {super.key,
      required this.hint,
      this.initial,
      this.color = Gx.transitionIndigo});
  @override
  State<GlowInput> createState() => _GlowInputState();
}

class _GlowInputState extends State<GlowInput> {
  final FocusNode _f = FocusNode();
  late final TextEditingController _ctrl =
      TextEditingController(text: widget.initial);

  @override
  void initState() {
    super.initState();
    // Redibuja al ganar/perder foco para animar el glow.
    _f.addListener(() => setState(() {}));
  }

  @override
  void dispose() {
    _f.dispose();
    _ctrl.dispose();
    super.dispose();
  }

  @override
  // Input con glow en el borde al recibir foco (_f.hasFocus); sin lógica de negocio,
  // solo estado local de FocusNode.
  Widget build(BuildContext context) {
    final focused = _f.hasFocus;
    return panelSurface(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 11),
      radius: Gx.rInput,
      glow: focused ? Gx.glow(widget.color, blur: 18, opacity: 0.45) : null,
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 200),
        decoration: BoxDecoration(
          borderRadius: BorderRadius.circular(Gx.rInput),
          border: focused
              ? Border.all(color: widget.color, width: Gx.borderFocus)
              : null,
        ),
        child: TextField(
          focusNode: _f,
          controller: _ctrl,
          cursorColor: widget.color,
          style: Gx.uiSans(fontSize: 14, color: Gx.textBase),
          decoration: InputDecoration.collapsed(
              hintText: widget.hint,
              hintStyle: Gx.uiSans(fontSize: 14, color: Gx.textBaseMuted)),
        ),
      ),
    );
  }
}

// Desplegable funcional: abre/cierra con animación y glow; al elegir, se cierra.
class GlowDropdown extends StatefulWidget {
  final String label;
  final List<String> options;
  const GlowDropdown({super.key, required this.label, required this.options});
  @override
  State<GlowDropdown> createState() => _GlowDropdownState();
}

class _GlowDropdownState extends State<GlowDropdown> {
  bool _open = false;
  late String _sel = widget.label;
  @override
  // Desplegable con panel semántico expandible/contraíble (_open) y opción seleccionada (_sel).
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        GestureDetector(
          onTap: () => setState(() => _open = !_open),
          child: panelSurface(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
            glow: _open
                ? Gx.glow(Gx.transitionIndigo, blur: 16, opacity: 0.45)
                : null,
            child: Row(mainAxisSize: MainAxisSize.min, children: [
              Flexible(
                  // Texto seleccionado con token dinámico de texto base.
                  child: Text(_sel,
                      style: Gx.uiSans(fontSize: 14, color: Gx.textBase),
                      overflow: TextOverflow.ellipsis)),
              const SizedBox(width: 8),
              AnimatedRotation(
                turns: _open ? 0.5 : 0,
                duration: const Duration(milliseconds: 200),
                // Phosphor caretDown: estética terminal más limpia que el chevron Material.
                child: Icon(Gx.iconChevronDown,
                    // Token dinámico de texto secundario — se adapta a la paleta activa.
                    size: 18, color: Gx.textBaseSecondary),
              ),
            ]),
          ),
        ),
        AnimatedSize(
          duration: const Duration(milliseconds: 220),
          curve: Curves.easeOut,
          child: _open
              ? Padding(
                  padding: const EdgeInsets.only(top: 6),
                  child: panelSurface(
                    padding: const EdgeInsets.symmetric(vertical: 4),
                    child: Column(
                      mainAxisSize: MainAxisSize.min,
                      crossAxisAlignment: CrossAxisAlignment.stretch,
                      children: widget.options
                          .map((o) => InkWell(
                                onTap: () =>
                                    setState(() { _sel = o; _open = false; }),
                                child: Padding(
                                  padding: const EdgeInsets.symmetric(
                                      horizontal: 12, vertical: 8),
                                  // Texto de opción con token dinámico secundario.
                                  child: Text(o, style: Gx.uiSans(fontSize: 14, color: Gx.textBaseSecondary)),
                                ),
                              ))
                          .toList(),
                    ),
                  ),
                )
              : const SizedBox.shrink(),
        ),
      ],
    );
  }
}

// Calendario funcional en grilla: toca un día y se enciende con un anillo de glow. Muestra
// marcadores de evento. Estado local (_sel) de UI, sin datos del Bridge.
class GlowCalendar extends StatefulWidget {
  const GlowCalendar({super.key});
  @override
  State<GlowCalendar> createState() => _GlowCalendarState();
}

class _GlowCalendarState extends State<GlowCalendar> {
  int _sel = 14;
  @override
  // Grilla de 28 días con día seleccionado (_sel) y marcadores de evento; estado local de UI.
  Widget build(BuildContext context) {
    return panelSurface(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Título del calendario: énfasis dinámico para cabeceras de panel.
          Text('Junio 2026', style: Gx.panelTitle.copyWith(color: Gx.accentDynamic)),
          const SizedBox(height: 8),
          Wrap(
            spacing: 6,
            runSpacing: 6,
            children: List.generate(28, (i) {
              final day = i + 1;
              final sel = day == _sel;
              final hasEvent = day % 7 == 3;
              return GestureDetector(
                onTap: () => setState(() => _sel = day),
                child: AnimatedContainer(
                  duration: const Duration(milliseconds: 180),
                  width: 28,
                  height: 28,
                  alignment: Alignment.center,
                  decoration: BoxDecoration(
                    shape: BoxShape.circle,
                    border:
                        sel ? Border.all(color: Gx.optimaCyan, width: 1.5) : null,
                    boxShadow: sel
                        ? Gx.glow(Gx.optimaCyan, blur: 14, opacity: 0.7)
                        : null,
                  ),
                  child: Stack(
                    alignment: Alignment.center,
                    children: [
                      // Día seleccionado: color semántico (estado). No seleccionado: token dinámico.
                      Text('$day',
                          style: Gx.dataMono(
                              fontSize: 11,
                              color: sel ? Gx.optimaCyan : Gx.textBaseSecondary)),
                      if (hasEvent)
                        Positioned(
                          bottom: 3,
                          child: Container(
                              width: 3,
                              height: 3,
                              decoration: const BoxDecoration(
                                  shape: BoxShape.circle,
                                  color: Gx.alertAmber)),
                        ),
                    ],
                  ),
                ),
              );
            }),
          ),
        ],
      ),
    );
  }
}

// Texto con "propagación de luz" (inspiración Reflect): al tocarlo, la
// iluminación se expande del centro hacia afuera como una explosión.
class LightBurstText extends StatefulWidget {
  final String text;
  const LightBurstText(this.text, {super.key});
  @override
  State<LightBurstText> createState() => _LightBurstTextState();
}

class _LightBurstTextState extends State<LightBurstText>
    with SingleTickerProviderStateMixin {
  late final AnimationController _c =
      AnimationController(vsync: this, duration: const Duration(milliseconds: 900));

  @override
  void dispose() {
    _c.dispose();
    super.dispose();
  }

  @override
  // Texto con ShaderMask animado que propaga luz del centro al tocar; animación local _c.
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: () => _c.forward(from: 0),
      child: AnimatedBuilder(
        animation: _c,
        builder: (_, __) {
          final v = _c.value;
          return ShaderMask(
            shaderCallback: (rect) => RadialGradient(
              radius: 0.1 + v * 1.4,
              colors: [Gx.optimaCyan, Gx.transitionIndigo, Gx.textBaseSecondary],
              stops: const [0.0, 0.5, 1.0],
            ).createShader(rect),
            // Gx.pureWhite es el token canónico para blanco puro; necesario
            // para que el ShaderMask pinte los colores del gradiente correctamente.
            child: Text(widget.text,
                style: Gx.body.copyWith(color: Gx.pureWhite)),
          );
        },
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// HoverableChart — envoltorio genérico de hover para CustomPainter.
// Propaga la posición del mouse al painter; en onExit pasa null.
// Úsalo en lugar de CustomPaint directo cuando el painter deba responder al
// cursor: HoverableChart(builder: (h) => MiPainter(hover: h), height: 100).
// ---------------------------------------------------------------------------

typedef HoverPainterBuilder = CustomPainter Function(Offset? hover);

// Widget que entrega la posición local del cursor al builder del painter.
class HoverableChart extends StatefulWidget {
  final HoverPainterBuilder builder;
  final double height;
  const HoverableChart({super.key, required this.builder, this.height = 100});
  @override
  State<HoverableChart> createState() => _HoverableChartState();
}

class _HoverableChartState extends State<HoverableChart> {
  Offset? _hover;
  @override
  // Pasa la posición local del cursor (_hover) al builder del CustomPainter; sin FFI.
  Widget build(BuildContext context) {
    return MouseRegion(
      onHover: (e) => setState(() => _hover = e.localPosition),
      onExit: (_) => setState(() => _hover = null),
      child: SizedBox(
        height: widget.height,
        child: CustomPaint(
          painter: widget.builder(_hover),
          size: Size.infinite,
        ),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// SonarPulseWidget — anillo único que se expande y desaparece (evento discreto).
// ---------------------------------------------------------------------------
// Bug anterior: CustomPaint tomaba el tamaño del hijo (orbe 48×48) y los
// anillos (maxRadius ≥ 44) sobresalían del clip. Fix: el anillo se dibuja en
// un SizedBox propio más grande que el orbe; el orbe queda centrado encima.
// HitTestBehavior.opaque asegura que taps en área transparente se capturen.
class SonarPulseWidget extends StatefulWidget {
  final Widget child;
  final Color color;
  final double maxRadius;
  const SonarPulseWidget({
    super.key,
    required this.child,
    this.color = Gx.optimaCyan,
    this.maxRadius = 52,
  });
  @override
  State<SonarPulseWidget> createState() => _SonarPulseWidgetState();
}

class _SonarPulseWidgetState extends State<SonarPulseWidget>
    with SingleTickerProviderStateMixin {
  late final AnimationController _ctrl = AnimationController(
    vsync: this,
    duration: const Duration(milliseconds: 750),
  );

  @override
  void dispose() {
    _ctrl.dispose();
    super.dispose();
  }

  @override
  // Anillo sonar que se expande al tocar, centrado debajo del hijo; animación local _ctrl.
  Widget build(BuildContext context) {
    // opaque: los taps en el área vacía alrededor del orbe también disparan el pulso.
    return GestureDetector(
      behavior: HitTestBehavior.opaque,
      onTap: () => _ctrl.forward(from: 0),
      child: Stack(
        alignment: Alignment.center,
        children: [
          // Canvas de los anillos: (maxRadius*2 + margen) para que no se corten.
          SizedBox(
            width: widget.maxRadius * 2 + 28,
            height: widget.maxRadius * 2 + 28,
            child: AnimatedBuilder(
              animation: _ctrl,
              builder: (_, __) => CustomPaint(
                painter: _SonarRingPainter(
                  progress: _ctrl.value,
                  color: widget.color,
                  maxRadius: widget.maxRadius,
                ),
                size: Size.infinite,
              ),
            ),
          ),
          widget.child,
        ],
      ),
    );
  }
}

class _SonarRingPainter extends CustomPainter {
  final double progress;
  final Color color;
  final double maxRadius;
  const _SonarRingPainter(
      {required this.progress, required this.color, required this.maxRadius});

  @override
  // Dibuja un anillo de sonar que se expande con halo blando y anillo nítido, alpha decreciente.
  // Sin MaskFilter.blur: se simula el suavizado con círculos de radio ampliado y opacidad baja
  // (DESIGN.md §Performance: sin blur en animaciones — aplica también a one-shot de 750ms).
  void paint(Canvas canvas, Size size) {
    if (progress == 0) return;
    final center = Offset(size.width / 2, size.height / 2);
    final r = maxRadius * progress;
    // Alpha alto al inicio, cae hasta cero al final.
    final alpha = ((1 - progress) * 220).round().clamp(0, 220);
    // Halo exterior difuso (sin blur — círculo más ancho con opacidad baja).
    canvas.drawCircle(center, r + 5, Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 8
      ..color = color.withAlpha((alpha * 0.18).round()));
    // Halo intermedio.
    canvas.drawCircle(center, r, Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 4
      ..color = color.withAlpha((alpha * 0.28).round()));
    // Anillo nítido principal.
    canvas.drawCircle(center, r, Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 2.5
      ..color = color.withAlpha(alpha));
  }

  @override
  bool shouldRepaint(_SonarRingPainter old) => old.progress != progress;
}

// ---------------------------------------------------------------------------
// ScanRingWidget — anillos concéntricos en secuencia (monitoreo sostenido).
// ---------------------------------------------------------------------------
// Mismo fix de tamaño que SonarPulse: SizedBox propio para el lienzo de anillos,
// hijo centrado encima. Anillos visibles: alpha 190, strokeWidth 2.
class ScanRingWidget extends StatefulWidget {
  final Widget child;
  final Color color;
  final double maxRadius;
  final Duration period;
  final bool active;
  const ScanRingWidget({
    super.key,
    required this.child,
    this.color = Gx.optimaCyan,
    this.maxRadius = 52,
    this.period = const Duration(milliseconds: 2800),
    this.active = true,
  });
  @override
  State<ScanRingWidget> createState() => _ScanRingWidgetState();
}

class _ScanRingWidgetState extends State<ScanRingWidget>
    with SingleTickerProviderStateMixin {
  late final AnimationController _ctrl = AnimationController(
    vsync: this,
    duration: widget.period,
  );

  @override
  void initState() {
    super.initState();
    if (widget.active) _ctrl.repeat();
  }

  @override
  void didUpdateWidget(ScanRingWidget old) {
    super.didUpdateWidget(old);
    if (widget.active && !_ctrl.isAnimating) {
      _ctrl.repeat();
    } else if (!widget.active && _ctrl.isAnimating) {
      _ctrl.stop();
    }
  }

  @override
  void dispose() {
    _ctrl.dispose();
    super.dispose();
  }

  @override
  // Apila el lienzo de anillos de scan (SizedBox dedicado) y el widget hijo encima; ambos centrados.
  Widget build(BuildContext context) {
    return Stack(
      alignment: Alignment.center,
      children: [
        SizedBox(
          width: widget.maxRadius * 2 + 28,
          height: widget.maxRadius * 2 + 28,
          child: AnimatedBuilder(
            animation: _ctrl,
            builder: (_, __) => CustomPaint(
              painter: _ScanRingsPainter(
                progress: _ctrl.value,
                color: widget.color,
                maxRadius: widget.maxRadius,
              ),
              size: Size.infinite,
            ),
          ),
        ),
        widget.child,
      ],
    );
  }
}

class _ScanRingsPainter extends CustomPainter {
  final double progress;
  final Color color;
  final double maxRadius;
  const _ScanRingsPainter(
      {required this.progress, required this.color, required this.maxRadius});

  @override
  // Dibuja dos anillos concéntricos en secuencia infinita, cada uno con halo suave y anillo nítido.
  // Sin MaskFilter.blur: animación continua — se simula el suavizado con círculos concéntricos
  // de radio/opacidad decreciente (DESIGN.md §Performance: sin blur en animación).
  void paint(Canvas canvas, Size size) {
    final center = Offset(size.width / 2, size.height / 2);
    for (var ring = 0; ring < 2; ring++) {
      final p = (progress + ring * 0.45) % 1.0;
      final r = maxRadius * p;
      final alpha = ((1 - p) * 190).round().clamp(0, 190);
      // Halo exterior difuso (sin blur — círculo más ancho con opacidad baja).
      canvas.drawCircle(center, r + 4, Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 7
        ..color = color.withAlpha((alpha * 0.12).round()));
      // Halo intermedio (radio normal, opacidad media).
      canvas.drawCircle(center, r, Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 4
        ..color = color.withAlpha((alpha * 0.22).round()));
      // Anillo nítido principal.
      canvas.drawCircle(center, r, Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2.0
        ..color = color.withAlpha(alpha));
    }
  }

  @override
  bool shouldRepaint(_ScanRingsPainter old) => old.progress != progress;
}

// Grafo DAG interactivo: las líneas (CustomPaint) y los nodos (widgets con
// MouseRegion) se encienden al pasar el mouse.
class InteractiveDag extends StatefulWidget {
  const InteractiveDag({super.key});
  @override
  State<InteractiveDag> createState() => _InteractiveDagState();
}

class _InteractiveDagState extends State<InteractiveDag> {
  int? _hover;
  @override
  // Grafo DAG con nodos que se iluminan al hover; geometría local de dagNodes().
  Widget build(BuildContext context) {
    return SizedBox(
      height: 140,
      child: LayoutBuilder(builder: (ctx, box) {
        final size = Size(box.maxWidth, box.maxHeight);
        final nodes = dagNodes(size);
        return Stack(children: [
          // Líneas con glow detrás de los nodos.
          Positioned.fill(child: CustomPaint(painter: DagLinesPainter(_hover))),
          ...List.generate(nodes.length, (i) {
            final n = nodes[i];
            final hov = _hover == i;
            final sel = i == 3;
            final color = (sel || hov) ? Gx.optimaCyan : Gx.transitionIndigo;
            return Positioned(
              left: n.dx - 12,
              top: n.dy - 12,
              child: MouseRegion(
                onEnter: (_) => setState(() => _hover = i),
                onExit: (_) => setState(() => _hover = null),
                child: AnimatedContainer(
                  duration: const Duration(milliseconds: 180),
                  width: 24,
                  height: 24,
                  decoration: BoxDecoration(
                    shape: BoxShape.circle,
                    color: Gx.surfaceCard,
                    border: Border.all(color: color, width: 2),
                    boxShadow: hov
                        ? Gx.glowStrong(color)
                        : Gx.glow(color, blur: 10, opacity: 0.5),
                  ),
                ),
              ),
            );
          }),
        ]);
      }),
    );
  }
}
