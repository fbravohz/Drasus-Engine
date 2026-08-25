// slider.dart — Componente Slider (ADR-0138 enmienda 2026-06-29).
// Slider con track degradado, glow y knob arrastrable en el rango [min, max].
// El estilo lo decide el tema global vía tokens Gx; prohibido hardcodear color.
//
// Nota: el nombre Slider colisiona con el widget Material del mismo nombre.
// Los consumidores importan con namespace `import ... as ui;` → `ui.Slider`.

import 'package:flutter/material.dart' hide Slider;
import '../theme/gx_tokens.dart';

// Slider arrastrable con track degradado y knob con glow.
// Contrato funcional: [value] posición actual en [min, max] (null = no controlado,
// empieza en [initialValue]); [onChanged] callback con el nuevo valor al arrastrar;
// [min]/[max] límites del rango; [initialValue] valor inicial en modo no controlado.
class Slider extends StatefulWidget {
  final double? value;
  final double initialValue;
  final ValueChanged<double>? onChanged;
  final double min;
  final double max;

  // No es const: lee getters dinámicos de Gx que cambian con el tema.
  Slider({
    super.key,
    this.value,
    this.initialValue = 0.5,
    this.onChanged,
    this.min = 0.0,
    this.max = 1.0,
  }) : assert(min < max, 'min debe ser menor que max');

  @override
  State<Slider> createState() => _SliderState();
}

class _SliderState extends State<Slider> {
  // Estado interno para modo no controlado.
  late double _internalValue = widget.initialValue.clamp(widget.min, widget.max);

  // Valor efectivo: el externo tiene prioridad sobre el interno.
  double get _effectiveValue => (widget.value ?? _internalValue)
      .clamp(widget.min, widget.max);

  // Convierte una posición en píxeles al valor en el rango [min, max].
  void _setFromDx(double dx, double trackWidth) {
    final normalized = (dx / trackWidth).clamp(0.0, 1.0);
    final newValue = widget.min + normalized * (widget.max - widget.min);
    if (widget.value == null) setState(() => _internalValue = newValue);
    widget.onChanged?.call(newValue);
  }

  @override
  // Slider con track relleno hasta el knob y glow del color de énfasis dinámico.
  Widget build(BuildContext context) {
    // Factor de normalización para el knob y el track relleno.
    final normalized = (_effectiveValue - widget.min) / (widget.max - widget.min);

    return LayoutBuilder(builder: (ctx, box) {
      final trackWidth = box.maxWidth;
      return GestureDetector(
        onPanDown: (d) => _setFromDx(d.localPosition.dx, trackWidth),
        onPanUpdate: (d) => _setFromDx(d.localPosition.dx, trackWidth),
        child: SizedBox(
          height: 26,
          child: Stack(alignment: Alignment.centerLeft, children: [
            // Pista de fondo: color de la pista de gauge (siempre visible).
            Container(
              height: 5,
              decoration: BoxDecoration(
                color: Gx.gaugeTrack,
                borderRadius: BorderRadius.circular(3), // Radio decorativo fino: 3dp
              ),
            ),
            // Porción rellena de la pista: gradiente de énfasis hasta el valor actual.
            FractionallySizedBox(
              widthFactor: normalized,
              child: Container(
                height: 5,
                decoration: BoxDecoration(
                  // Gradiente de la pista: énfasis dinámico → cian de transición.
                  gradient: LinearGradient(colors: [
                    Gx.accentDynamic,
                    Gx.optimaCyan,
                  ]),
                  borderRadius: BorderRadius.circular(3),
                  boxShadow: Gx.glow(Gx.accentDynamic, blur: 10, opacity: 0.6),
                ),
              ),
            ),
            // Knob: círculo centrado en el valor actual con glow.
            Align(
              alignment: Alignment(normalized * 2 - 1, 0),
              child: Container(
                width: 16,
                height: 16,
                decoration: BoxDecoration(
                  shape: BoxShape.circle,
                  // Color base dinámico para legibilidad en todos los temas (paper/bunker).
                  color: Gx.textBase,
                  boxShadow: Gx.glowStrong(Gx.accentDynamic),
                ),
              ),
            ),
          ]),
        ),
      );
    });
  }
}
