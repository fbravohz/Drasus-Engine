// GlassSurface — primitivo de vidrio Apple (ADR-0138).
// Superficie translúcida con BackdropFilter + relleno + rim-light, que lee
// sus tokens de DrasusGlass vía Theme.of(context).extension<DrasusGlass>().
// Reemplaza al patrón ClipRRect+BackdropFilter+Container duplicado en las
// secciones y el SettingsDrawer.

import 'dart:ui' as ui;
import 'package:flutter/material.dart';
import '../theme/drasus_tokens.dart';
import '../gallery/gallery_tokens.dart';
import '../drasus_theme.dart';

/// Superficie de vidrio Apple reutilizable.
///
/// Params:
/// - [child]: contenido a envolver.
/// - [borderRadius]: radio de las esquinas (default `Gx.rPanel`).
/// - [padding]: padding interior opcional.
class GlassSurface extends StatelessWidget {
  final Widget child;
  final double borderRadius;
  final EdgeInsetsGeometry? padding;

  // No es const: GlassSurface lee el modo global estático y debe poder
  // reconstruirse al cambiar el modo. Un constructor const permitiría
  // instanciación const que congelaría el modo (regla DESIGN.md §Superficie).
  GlassSurface({
    super.key,
    required this.child,
    this.borderRadius = Gx.rPanel,
    this.padding,
  });

  @override
  Widget build(BuildContext context) {
    final mode = DrasusThemeState.globalSurfaceMode;
    final radius = BorderRadius.circular(borderRadius);

    if (mode == DrasusSurfaceMode.solid) {
      return Container(
        padding: padding,
        decoration: BoxDecoration(
          color: Gx.surfacePanel,
          borderRadius: radius,
          border: Border.all(color: Gx.borderPanel),
        ),
        child: child,
      );
    }

    if (mode == DrasusSurfaceMode.tint) {
      return Container(
        padding: padding,
        decoration: BoxDecoration(
          // Color de componentes al 65%: translúcido visible, sin blur.
          color: Gx.surfaceFill.withOpacity(0.65),
          borderRadius: radius,
          border: Border.all(
            color: const Color(0x20A096FF).withOpacity(Gx.glassEdgeOpacity),
          ),
        ),
        child: child,
      );
    }

    // mode == enhancedGlass: gradiente profundo + borde del énfasis dinámico + glow amplio.
    // Replica la receta de glassEnhanced() con Gx.accentDynamic como color de borde,
    // sin importar gallery_fx.dart (evita acoplamiento entre capas de la biblioteca).
    if (mode == DrasusSurfaceMode.enhancedGlass) {
      final accentColor = Gx.accentDynamic;
      final content = Container(
        padding: padding,
        decoration: BoxDecoration(
          // Gradiente desde el color de componentes hasta deepSpace (profundidad tonal).
          gradient: LinearGradient(
            begin: Alignment.topCenter,
            end: Alignment.bottomCenter,
            colors: [Gx.componentBgBase, Gx.deepSpace],
          ),
          borderRadius: radius,
          border: Border.all(color: accentColor.withAlpha(80)),
          boxShadow: Gx.glow(accentColor, blur: 20, opacity: 0.15),
        ),
        child: child,
      );
      return ClipRRect(
        borderRadius: radius,
        child: BackdropFilter(
          filter: ui.ImageFilter.blur(sigmaX: Gx.glassBlur, sigmaY: Gx.glassBlur),
          child: content,
        ),
      );
    }

    // mode == glass: vidrio Apple completo.
    final glass =
        Theme.of(context).extension<DrasusGlass>() ?? DrasusGlass.defaults;

    return ClipRRect(
      borderRadius: radius,
      child: BackdropFilter(
        filter: ui.ImageFilter.blur(
          sigmaX: glass.blurSigma,
          sigmaY: glass.blurSigma,
        ),
        child: Container(
          padding: padding,
          decoration: BoxDecoration(
            color: glass.fill,
            borderRadius: radius,
            // Tinte interior con el color de componentes (sutil, 0.18 de opacidad).
            // Tiñe el vidrio con el tono elegido sin bloquear el BackdropFilter.
            gradient: LinearGradient(
              begin: Alignment.topCenter,
              end: Alignment.bottomCenter,
              colors: [
                Gx.componentBgBase.withOpacity(0.18),
                Colors.transparent,
              ],
            ),
            border: Border.all(
              color: glass.rimColor.withOpacity(glass.edgeOpacity),
            ),
          ),
          child: child,
        ),
      ),
    );
  }
}
