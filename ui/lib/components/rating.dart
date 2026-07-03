// rating.dart — Componente Rating (ADR-0138 enmienda 2026-06-29).
// Valoración visual con N indicadores circulares neón; activos brillan en alertAmber.
// El estilo lo decide el tema global vía tokens Gx; prohibido hardcodear color.

import 'package:flutter/material.dart';
import '../theme/gx_tokens.dart';

// Valoración por indicadores circulares (estilo Drasus: neón encendido en vez de estrellas).
// Contrato funcional:
//   [value]     puntuación actual (null = modo no controlado; empieza en 0).
//   [onChanged] callback con la nueva puntuación al tocar un indicador.
//   [max]       número total de indicadores (por defecto 5).
// Un toque sobre el indicador i+1 establece [value] = i+1.
// Tocar el indicador ya seleccionado lo deselecciona (value pasa a 0).
class Rating extends StatefulWidget {
  final int? value;
  final ValueChanged<int>? onChanged;
  final int max;

  // No es const: lee getters dinámicos de Gx que cambian con el tema.
  Rating({
    super.key,
    this.value,
    this.onChanged,
    this.max = 5,
  });

  @override
  State<Rating> createState() => _RatingState();
}

class _RatingState extends State<Rating> {
  // Valor interno para modo no controlado (empieza en 0 = sin calificar).
  int _internalValue = 0;

  // Valor efectivo: el externo tiene prioridad sobre el interno.
  int get _effective => widget.value ?? _internalValue;

  // Cambia la puntuación: tocar el mismo indicador lo deselecciona (→ 0).
  void _setRating(int tapped) {
    final next = tapped == _effective ? 0 : tapped;
    if (widget.value == null) setState(() => _internalValue = next);
    widget.onChanged?.call(next);
  }

  @override
  // Fila de [max] indicadores circulares animados; los activos brillan en alertAmber.
  Widget build(BuildContext context) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: List.generate(widget.max, (i) {
        // El indicador está activo si su índice es menor que la puntuación.
        final active = i < _effective;
        return GestureDetector(
          onTap: () => _setRating(i + 1),
          child: AnimatedContainer(
            duration: const Duration(milliseconds: 160),
            width: 22,
            height: 22,
            margin: const EdgeInsets.symmetric(horizontal: 3),
            decoration: BoxDecoration(
              shape: BoxShape.circle,
              // Fondo tenue cuando activo; transparente en reposo.
              color: active ? Gx.alertAmber.withAlpha(40) : Colors.transparent,
              border: Border.all(
                // Borde semántico alertAmber cuando activo; borde estructural cuando inactivo.
                color: active ? Gx.alertAmber : Gx.borderBase,
              ),
              // Glow semántico solo en indicadores activos.
              boxShadow: active
                  ? Gx.glow(Gx.alertAmber, blur: 10, opacity: 0.55)
                  : null,
            ),
          ),
        );
      }),
    );
  }
}
