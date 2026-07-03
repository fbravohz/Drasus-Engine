// surface.dart — Componente Surface (ADR-0138 enmienda 2026-06-29).
// Wrapper de superficie neutral que consume el modo global de tema.
// El estilo lo decide el tema global; prohibido hardcodear color o modo.

import 'package:flutter/material.dart';
import '../theme/gx_tokens.dart';
import '../theme/surfaces.dart';

// Wrapper de superficie que aplica vidrio/tint/solid según el modo global.
// Delega a panelSurface() y reacciona automáticamente a cambios de tema.
// Contrato funcional: [child] contenido a envolver; [padding] padding interno
// (default 12dp); [radius] radio de esquinas (default Gx.rPanel).
// NO admite parámetro de modo o estilo — el modo lo dicta el tema global.
class Surface extends StatelessWidget {
  final Widget child;
  final EdgeInsets? padding;
  final double radius;
  final List<BoxShadow>? glow;

  // No es const: panelSurface() lee el modo global estático en cada build.
  Surface({
    super.key,
    required this.child,
    this.padding,
    this.radius = Gx.rPanel,
    this.glow,
  });

  @override
  // Delega a panelSurface() para aplicar la receta de superficie correcta
  // según el modo global (glass / tint / solid / enhancedGlass).
  Widget build(BuildContext context) {
    return panelSurface(
      child: child,
      padding: padding,
      radius: radius,
      glow: glow,
    );
  }
}
