// time_picker.dart — Componente TimePicker (ADR-0138 enmienda 2026-06-29).
// Selector de hora con dos ruedas de desplazamiento (horas y minutos).
// El ítem central está resaltado con borde de énfasis dinámico y glow.

import 'package:flutter/material.dart';
import '../theme/gx_tokens.dart';
import '../theme/surfaces.dart';

// Selector de hora con estilo rueda de desplazamiento.
// Muestra tres ítems visibles por columna (anterior / seleccionado / siguiente).
// Modo no controlado: arranca a las 09:30 si no se pasa value.
class TimePicker extends StatefulWidget {
  // value: hora inicial seleccionada; null arranca en 09:30.
  final TimeOfDay? value;
  // onChanged: se llama con la nueva hora al cambiar horas o minutos.
  final ValueChanged<TimeOfDay>? onChanged;

  const TimePicker({super.key, this.value, this.onChanged});

  @override
  State<TimePicker> createState() => _TimePickerState();
}

class _TimePickerState extends State<TimePicker> {
  // Hora y minuto seleccionados internamente.
  late int _hour;
  // _minute almacena el índice de la columna de minutos (0..11 para pasos de 5).
  late int _minuteIdx;

  @override
  void initState() {
    super.initState();
    final val = widget.value ?? const TimeOfDay(hour: 9, minute: 30);
    _hour = val.hour;
    // Convierte minutos a índice en la columna de pasos de 5 (30 min → índice 6).
    _minuteIdx = (val.minute ~/ 5).clamp(0, 11);
  }

  // Minutos reales para el valor actual (índice × 5).
  int get _minute => _minuteIdx * 5;

  // Notifica al padre con la nueva hora seleccionada.
  void _notify() {
    widget.onChanged?.call(TimeOfDay(hour: _hour, minute: _minute));
  }

  @override
  // Fila con columna de horas, separador ":" y columna de minutos.
  Widget build(BuildContext context) {
    return panelSurface(
      glow: Gx.glow(Gx.accentDynamic, blur: 12, opacity: 0.15),
      child: SizedBox(
        height: 120,
        child: Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            // Columna de horas: 0–23.
            _wheelColumn(
              values: List.generate(24, (i) => i.toString().padLeft(2, '0')),
              selected: _hour,
              onSelect: (v) {
                setState(() => _hour = v);
                _notify();
              },
            ),
            // Separador ":".
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 8),
              child: Text(
                ':',
                style: Gx.dataMono(
                  fontSize: 24,
                  color: Gx.accentDynamic,
                  weight: FontWeight.w500,
                ),
              ),
            ),
            // Columna de minutos: pasos de 5 (00, 05, 10 … 55).
            _wheelColumn(
              values: List.generate(12, (i) => (i * 5).toString().padLeft(2, '0')),
              selected: _minuteIdx,
              onSelect: (v) {
                setState(() => _minuteIdx = v);
                _notify();
              },
            ),
          ],
        ),
      ),
    );
  }

  // Columna de rueda: muestra el ítem anterior, el seleccionado y el siguiente.
  // El ítem seleccionado (central) lleva borde de énfasis y glow.
  Widget _wheelColumn({
    required List<String> values,
    required int selected,
    required ValueChanged<int> onSelect,
  }) {
    // Calcula índices circular para el ítem anterior y siguiente al seleccionado.
    final prev = (selected - 1 + values.length) % values.length;
    final next = (selected + 1) % values.length;

    return Column(
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        // Ítem anterior: atenuado con token dinámico muted.
        GestureDetector(
          onTap: () => onSelect(prev),
          child: Padding(
            padding: const EdgeInsets.symmetric(vertical: Gx.space4),
            child: Text(
              values[prev],
              style: Gx.dataMono(fontSize: 16, color: Gx.textBaseMuted),
            ),
          ),
        ),
        // Ítem central (seleccionado): borde de énfasis + glow del tema.
        Container(
          padding: const EdgeInsets.symmetric(
              horizontal: Gx.space12, vertical: Gx.space4 + 2),
          decoration: BoxDecoration(
            color: Gx.accentDynamic.withAlpha(38),
            borderRadius: BorderRadius.circular(Gx.rInput),
            border: Border.all(color: Gx.accentDynamic.withAlpha(153)),
            boxShadow: Gx.glow(Gx.accentDynamic, blur: 8, opacity: 0.4),
          ),
          child: Text(
            values[selected],
            style: Gx.dataMono(
              fontSize: 22,
              color: Gx.accentDynamic,
              weight: FontWeight.w500,
            ),
          ),
        ),
        // Ítem siguiente: atenuado con token dinámico muted.
        GestureDetector(
          onTap: () => onSelect(next),
          child: Padding(
            padding: const EdgeInsets.symmetric(vertical: Gx.space4),
            child: Text(
              values[next],
              style: Gx.dataMono(fontSize: 16, color: Gx.textBaseMuted),
            ),
          ),
        ),
      ],
    );
  }
}
