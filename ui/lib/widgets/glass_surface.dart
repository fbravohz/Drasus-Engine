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

  const GlassSurface({
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
          color: Gx.panelSolid,
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
          color: Gx.glassFill,
          borderRadius: radius,
          border: Border.all(
            color: const Color(0x20A096FF).withOpacity(Gx.glassEdgeOpacity),
          ),
        ),
        child: child,
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
            // Tinte interior (milk glass) como capa superpuesta.
            gradient: LinearGradient(
              begin: Alignment.topCenter,
              end: Alignment.bottomCenter,
              colors: [
                glass.tint,
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
