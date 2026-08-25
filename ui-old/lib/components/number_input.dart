// number_input.dart — Componente NumberInput (ADR-0138 enmienda 2026-06-29).
// Campo numérico con botones + y − que respetan rango y paso configurables.
// El estilo lo decide el tema global vía tokens Gx; prohibido hardcodear color.

import 'package:flutter/material.dart';
import '../theme/gx_tokens.dart';
import '../theme/surfaces.dart';

// Input numérico con botones de incremento/decremento.
// Contrato funcional:
//   [value]     valor actual (null = modo no controlado, usa [initialValue]).
//   [initialValue] valor inicial en modo no controlado (por defecto: [min]).
//   [onChanged] callback con el nuevo valor al cambiar.
//   [min]       valor mínimo permitido (por defecto: 0).
//   [max]       valor máximo permitido (por defecto: 100).
//   [step]      incremento/decremento por cada pulsación (por defecto: 1.0).
// Los botones se deshabilitan visualmente cuando el valor toca el límite.
class NumberInput extends StatefulWidget {
  final double? value;
  final double? initialValue;
  final ValueChanged<double>? onChanged;
  final double min;
  final double max;
  final double step;

  // No es const: lee getters dinámicos de Gx que cambian con el tema.
  NumberInput({
    super.key,
    this.value,
    this.initialValue,
    this.onChanged,
    this.min = 0,
    this.max = 100,
    this.step = 1.0,
  });

  @override
  State<NumberInput> createState() => _NumberInputState();
}

class _NumberInputState extends State<NumberInput> {
  // Valor interno para modo no controlado.
  late double _internalValue;

  @override
  void initState() {
    super.initState();
    // Inicializa con el valor externo, el initialValue, o el mínimo.
    _internalValue = widget.value ?? widget.initialValue ?? widget.min;
  }

  // Valor efectivo: el externo tiene prioridad sobre el interno.
  double get _effective => widget.value ?? _internalValue;

  // Formatea el número: entero si no tiene parte decimal, dos decimales si la tiene.
  String _format(double v) {
    if (v == v.truncateToDouble()) return v.truncate().toString();
    return v.toStringAsFixed(2);
  }

  // Cambia el valor en [delta] (positivo o negativo), respetando [min] y [max].
  void _change(double delta) {
    final next = (_effective + delta).clamp(widget.min, widget.max);
    if (widget.value == null) setState(() => _internalValue = next);
    widget.onChanged?.call(next);
  }

  // Construye un botón de control (+ o −) con fondo de superficie y glow suave al tocar.
  Widget _btn(IconData icon, VoidCallback? onTap) => GestureDetector(
        onTap: onTap,
        child: Container(
          width: 28,
          height: 28,
          alignment: Alignment.center,
          decoration: BoxDecoration(
            color: Gx.surfaceFill,
            borderRadius: BorderRadius.circular(Gx.rChip),
          ),
          // Opacidad reducida cuando el botón está deshabilitado (límite alcanzado).
          child: Opacity(
            opacity: onTap != null ? 1.0 : 0.35,
            child: Icon(icon, size: 14, color: Gx.textBaseSecondary),
          ),
        ),
      );

  @override
  // Fila compacta: [−] [valor] [+] con superficie panelSurface.
  Widget build(BuildContext context) {
    final atMin = _effective <= widget.min;
    final atMax = _effective >= widget.max;

    return panelSurface(
      padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 4),
      child: Row(mainAxisSize: MainAxisSize.min, children: [
        // Botón − deshabilitado cuando el valor ya está en el mínimo.
        _btn(
          Icons.remove,
          atMin ? null : () => _change(-widget.step),
        ),
        // Valor centrado con fuente monoespaciada de datos.
        SizedBox(
          width: 52,
          child: Text(
            _format(_effective),
            textAlign: TextAlign.center,
            style: Gx.dataMono(fontSize: 14, color: Gx.textBase),
          ),
        ),
        // Botón + deshabilitado cuando el valor ya está en el máximo.
        _btn(
          Icons.add,
          atMax ? null : () => _change(widget.step),
        ),
      ]),
    );
  }
}
