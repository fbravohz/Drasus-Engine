// Helpers de superficie (vidrio Apple / panel / card) — cimiento compartido
// entre la galería y la librería de componentes. Movidos desde
// gallery/gallery_fx.dart (STORY-026) para que `components/` no dependa de
// `gallery/`: la galería es CONSUMIDORA de este cimiento, no su dueña.
// Son wrappers de superficie puros (sin lógica de negocio ni FFI).

import 'dart:ui' as ui;
import 'package:flutter/material.dart';
import 'gx_tokens.dart';
import 'theme_scope.dart';

// Vidrio Apple — o lo que indique el modo global de superficie.
// Lee ThemeState.globalSurfaceMode para decidir la receta:
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
  final mode = ThemeState.globalSurfaceMode;

  if (mode == SurfaceMode.solid) {
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

  if (mode == SurfaceMode.tint) {
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
  if (mode == SurfaceMode.enhancedGlass) {
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
    final mode = ThemeState.globalSurfaceMode;

    if (mode == SurfaceMode.solid) {
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
  final mode = ThemeState.globalSurfaceMode;

  // En solid: color de componentes directo; en glass/tint/enhancedGlass: mismo color
  // (los wrappers aplican la opacidad adecuada al renderizar).
  final fill = mode == SurfaceMode.solid ? Gx.surfacePanel : Gx.componentBgBase;

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
  if (mode == SurfaceMode.glass || mode == SurfaceMode.enhancedGlass) {
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
