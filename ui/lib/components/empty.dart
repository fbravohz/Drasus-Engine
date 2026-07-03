// empty.dart — Componente Empty (ADR-0138 enmienda 2026-06-29).
// Estado vacío con orbe/icono tenue y mensaje descriptivo.
// El estilo lo decide el tema global vía tokens Gx; prohibido hardcodear color.

import 'package:flutter/material.dart';
import '../gallery/gallery_tokens.dart';

// Estado vacío: icono (o orbe de cristal latente) + mensaje + subtítulo opcional.
// Contrato funcional: [message] texto principal del estado vacío; [icon] icono
// a mostrar (null = muestra orbe de cristal latente por defecto); [subtitle]
// texto secundario descriptivo opcional.
class Empty extends StatelessWidget {
  final String message;
  final IconData? icon;
  final String? subtitle;

  // No es const: lee getters dinámicos de Gx que cambian con el tema.
  Empty({super.key, required this.message, this.icon, this.subtitle});

  @override
  // Columna centrada con orbe/icono tenue, mensaje en muted y subtítulo opcional.
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        // Representación visual del estado vacío.
        if (icon != null)
          // Icono provisto: muestra el icono en color muted con tamaño grande.
          Icon(icon!, size: 40, color: Gx.textBaseMuted)
        else
          // Orbe de cristal latente: gradiente radial tenue, sin glow fuerte.
          Container(
            width: 48,
            height: 48,
            decoration: BoxDecoration(
              shape: BoxShape.circle,
              // Gradiente del énfasis dinámico → color de tarjeta: orbe "dormido".
              gradient: RadialGradient(
                colors: [Gx.accentDynamic.withAlpha(80), Gx.surfaceCard],
              ),
              border: Border.all(color: Gx.borderBase),
            ),
          ),
        const SizedBox(height: 12),
        // Mensaje principal del estado vacío.
        Text(
          message,
          style: Gx.uiSans(fontSize: 14, color: Gx.textBaseMuted),
          textAlign: TextAlign.center,
        ),
        // Subtítulo opcional: instrucción o contexto adicional.
        if (subtitle != null) ...[
          const SizedBox(height: 6),
          Text(
            subtitle!,
            style: Gx.uiSans(fontSize: 12, color: Gx.textBaseMuted),
            textAlign: TextAlign.center,
          ),
        ],
      ],
    );
  }
}
