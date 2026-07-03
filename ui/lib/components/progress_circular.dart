// progress_circular.dart — Componente ProgressCircular (ADR-0138 enmienda 2026-06-29).
// Anillo de progreso circular con glow del color activo del tema.
// Modo determinado (value != null): muestra el porcentaje en el centro.
// Modo indeterminado (value == null): gira continuamente sin texto.

import 'package:flutter/material.dart';
import '../gallery/gallery_tokens.dart';

// Indicador circular de progreso determinado o indeterminado.
// Uso determinado: ProgressCircular(value: 0.68)
// Uso spinner:    ProgressCircular()  (sin value)
class ProgressCircular extends StatefulWidget {
  // value: fracción 0.0–1.0 del progreso completado; null = spinner giratorio.
  final double? value;
  // size: diámetro del anillo en píxeles lógicos.
  final double size;
  // color: color del arco activo; usa el énfasis dinámico del tema cuando es null.
  final Color? color;

  const ProgressCircular({
    super.key,
    this.value,
    this.size = 60.0,
    this.color,
  });

  @override
  State<ProgressCircular> createState() => _ProgressCircularState();
}

class _ProgressCircularState extends State<ProgressCircular> {
  @override
  // Dibuja la pista de fondo + el arco activo + el porcentaje central (si determinado).
  Widget build(BuildContext context) {
    // Si no se pasa color, usa el énfasis dinámico que cambia con la paleta del usuario.
    final activeColor = widget.color ?? Gx.accentDynamic;
    return SizedBox(
      width: widget.size,
      height: widget.size,
      child: Stack(alignment: Alignment.center, children: [
        // Pista de fondo: círculo completo en gris neutro de riel de gauge.
        CircularProgressIndicator(
          value: 1.0,
          strokeWidth: 4,
          color: Gx.gaugeTrack,
        ),
        // Arco activo: determinado (value) o giratorio continuo (null).
        CircularProgressIndicator(
          value: widget.value,
          strokeWidth: 4,
          color: activeColor,
          strokeCap: StrokeCap.round,
        ),
        // Porcentaje en el centro: solo visible cuando el modo es determinado.
        if (widget.value != null)
          Text(
            '${(widget.value! * 100).round()}%',
            style: Gx.dataMono(fontSize: 11, color: activeColor),
          ),
      ]),
    );
  }
}
