// card.dart — Componente Card (ADR-0138 enmienda 2026-06-29).
// Tarjeta de contenido genérica con superficie cardSurface y glow opcional.
// El estilo lo decide el tema global vía tokens Gx; prohibido hardcodear color.
//
// Nota: el nombre Card colisiona con el widget Material del mismo nombre.
// Los consumidores importan con namespace `import ... as ui;` → `ui.Card`.

import 'package:flutter/material.dart' hide Card;
import '../theme/gx_tokens.dart';
import '../theme/surfaces.dart';

// Tarjeta de contenido genérica con superficie de card.
// Contrato funcional: [child] contenido de la tarjeta; [padding] padding interno
// (default 10dp via cardSurface); [radius] radio de esquinas (default Gx.rPanel);
// [glow] sombra de glow opcional para tarjetas con énfasis semántico.
class Card extends StatelessWidget {
  final Widget child;
  final EdgeInsets? padding;
  final double radius;
  final List<BoxShadow>? glow;

  // No es const: cardSurface() lee el modo global estático y debe poder reconstruirse.
  Card({
    super.key,
    required this.child,
    this.padding,
    this.radius = Gx.rPanel,
    this.glow,
  });

  @override
  // Delega a cardSurface() para aplicar la receta de card según el modo global.
  // La card es un nivel de profundidad por encima del panel (más claro en solid).
  Widget build(BuildContext context) {
    return cardSurface(
      child: child,
      padding: padding,
      radius: radius,
      glow: glow,
    );
  }
}
