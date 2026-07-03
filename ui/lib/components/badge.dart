// badge.dart — Componente Badge (ADR-0138 enmienda 2026-06-29).
// Indicador numérico o de etiqueta, típicamente superpuesto sobre otro widget
// para señalar notificaciones o estados nuevos.
// El estilo lo decide el tema global vía tokens Gx; prohibido hardcodear color.

import 'package:flutter/material.dart' hide Badge;
import '../gallery/gallery_tokens.dart';

// Badge: pequeño indicador superpuesto con número, etiqueta o punto.
// Contrato funcional: [count] número a mostrar (null = sin número);
// [label] texto alternativo al número (null = solo punto o número);
// [child] widget sobre el que se superpone el badge (null = standalone).
// Si [count] y [label] son null → muestra solo un punto/dot.
// Si [child] es provisto → superpone el badge en la esquina superior derecha.
class Badge extends StatelessWidget {
  final int? count;
  final String? label;
  final Widget? child;

  // No es const: lee getters dinámicos de Gx que cambian con el tema.
  Badge({super.key, this.count, this.label, this.child});

  // Construye la pastilla del badge con el contenido apropiado.
  Widget _pill() {
    final text = count != null
        ? (count! > 99 ? '99+' : '$count')
        : label;

    if (text == null) {
      // Punto sin texto: dot de notificación.
      return Container(
        width: 8,
        height: 8,
        decoration: BoxDecoration(
          shape: BoxShape.circle,
          color: Gx.accentDynamic,
          boxShadow: Gx.glow(Gx.accentDynamic, blur: 8, opacity: 0.7),
        ),
      );
    }

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 5, vertical: 2),
      decoration: BoxDecoration(
        // Color de énfasis dinámico — el badge es siempre el color de acento del tema.
        color: Gx.accentDynamic,
        borderRadius: BorderRadius.circular(999), // pill completo
        boxShadow: Gx.glow(Gx.accentDynamic, blur: 8, opacity: 0.6),
      ),
      child: Text(
        text,
        style: Gx.uiSans(
          fontSize: 10,
          weight: FontWeight.w600,
          color: Gx.canvasBase,
        ),
      ),
    );
  }

  @override
  // Si hay [child], superpone el badge en la esquina superior derecha del hijo.
  // Si no hay [child], muestra el badge de forma independiente (standalone).
  Widget build(BuildContext context) {
    final pill = _pill();
    if (child == null) return pill;

    return Stack(
      clipBehavior: Clip.none,
      children: [
        child!,
        Positioned(
          // Badge en esquina superior derecha del hijo.
          top: -4,
          right: -4,
          child: pill,
        ),
      ],
    );
  }
}
