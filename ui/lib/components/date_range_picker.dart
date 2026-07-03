// date_range_picker.dart — Componente DateRangePicker (ADR-0138 enmienda 2026-06-29).
// Selector de rango de fechas con campos de inicio/fin y mini-calendario interactivo.
// Estilo 100% por tema; prohibido hardcodear color.

import 'package:flutter/material.dart';
import '../gallery/gallery_tokens.dart';
import '../gallery/gallery_fx.dart';

// Par de fechas de inicio y fin de un rango seleccionado.
class DateRange {
  final DateTime start;
  final DateTime end;
  const DateRange({required this.start, required this.end});
}

// Selector de rango de fechas con mini-calendario interactivo.
// Modo no controlado: arranca con el rango de los días 3-17 del mes actual
// y gestiona el estado internamente. [onChanged] se llama al ajustar el rango.
class DateRangePicker extends StatefulWidget {
  // range: rango inicial seleccionado (null = día 3-17 del mes actual).
  final DateRange? range;
  // onChanged: se llama con el nuevo DateRange al ajustar los extremos.
  final ValueChanged<DateRange>? onChanged;

  const DateRangePicker({super.key, this.range, this.onChanged});

  @override
  State<DateRangePicker> createState() => _DateRangePickerState();
}

class _DateRangePickerState extends State<DateRangePicker> {
  late DateTime _refMonth; // mes de referencia para el mini-calendario
  late int _startDay;      // día de inicio del rango (dentro del mes de referencia)
  late int _endDay;        // día de fin del rango

  @override
  void initState() {
    super.initState();
    final now = DateTime.now();
    _refMonth = widget.range?.start ?? DateTime(now.year, now.month);
    _startDay = widget.range?.start.day ?? 3;
    _endDay = widget.range?.end.day ?? 17;
  }

  // Número de días del mes de referencia actual.
  int get _daysInMonth =>
      DateTime(_refMonth.year, _refMonth.month + 1, 0).day;

  // Etiqueta del mes y año para la cabecera del mini-calendario.
  String get _monthLabel {
    const months = [
      'ENE', 'FEB', 'MAR', 'ABR', 'MAY', 'JUN',
      'JUL', 'AGO', 'SEP', 'OCT', 'NOV', 'DIC',
    ];
    return '${months[_refMonth.month - 1]} ${_refMonth.year}';
  }

  // Notifica al padre con el rango actualizado.
  void _notify() {
    widget.onChanged?.call(DateRange(
      start: DateTime(_refMonth.year, _refMonth.month, _startDay),
      end: DateTime(_refMonth.year, _refMonth.month, _endDay),
    ));
  }

  @override
  // Panel con campos de fecha + cabecera de mes + mini-cuadrícula de días.
  Widget build(BuildContext context) {
    return panelSurface(
      padding: const EdgeInsets.all(12),
      glow: Gx.glow(Gx.accentDynamic, blur: 12, opacity: 0.15),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Fila con campos de fecha inicio y fin.
          Row(children: [
            Expanded(child: _dateField('Inicio', _startDay)),
            const SizedBox(width: Gx.space8),
            Icon(Icons.arrow_forward, size: 14, color: Gx.textBaseMuted),
            const SizedBox(width: 8),
            Expanded(child: _dateField('Fin', _endDay)),
          ]),
          const SizedBox(height: 10),
          // Navegación de mes con flechas y etiqueta.
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              GestureDetector(
                onTap: () => setState(() =>
                    _refMonth = DateTime(_refMonth.year, _refMonth.month - 1)),
                child: Icon(Icons.chevron_left, size: 16, color: Gx.textBaseSecondary),
              ),
              Text(
                _monthLabel,
                style: Gx.microLabel.copyWith(color: Gx.textBaseSecondary),
              ),
              GestureDetector(
                onTap: () => setState(() =>
                    _refMonth = DateTime(_refMonth.year, _refMonth.month + 1)),
                child: Icon(Icons.chevron_right, size: 16, color: Gx.textBaseSecondary),
              ),
            ],
          ),
          const SizedBox(height: 6),
          // Mini-cuadrícula de días del mes con el rango resaltado.
          _miniCalendar(),
        ],
      ),
    );
  }

  // Campo de fecha con borde de foco semántico (énfasis dinámico) y glow suave.
  Widget _dateField(String label, int day) {
    return Container(
      padding: const EdgeInsets.symmetric(
          horizontal: Gx.space8 + 2, vertical: Gx.space8),
      decoration: BoxDecoration(
        color: Gx.surfacePanel,
        borderRadius: BorderRadius.circular(Gx.rInput),
        // Borde de estado de foco con énfasis dinámico del tema.
        border: Border.all(color: Gx.accentDynamic.withAlpha(153)),
        boxShadow: Gx.glow(Gx.accentDynamic, blur: 8, opacity: 0.2),
      ),
      child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
        Text(label, style: Gx.microLabel.copyWith(color: Gx.textBaseLabel)),
        const SizedBox(height: 2),
        Text(
          '$day/${_refMonth.month.toString().padLeft(2, '0')}/${_refMonth.year}',
          style: Gx.dataMono(fontSize: 13, color: Gx.textBase),
        ),
      ]),
    );
  }

  // Mini-cuadrícula del mes: resalta extremos del rango y el interior.
  Widget _miniCalendar() {
    return Wrap(
      spacing: 4,
      runSpacing: 4,
      children: List.generate(_daysInMonth, (i) {
        final day = i + 1;
        final inRange = day >= _startDay && day <= _endDay;
        // Los extremos del rango llevan fondo sólido del énfasis.
        final isEdge = day == _startDay || day == _endDay;

        return GestureDetector(
          onTap: () {
            setState(() {
              // Ajusta el extremo más cercano al día tocado.
              if (day <= _startDay) {
                _startDay = day;
              } else if (day >= _endDay) {
                _endDay = day;
              } else {
                final midpoint = (_startDay + _endDay) ~/ 2;
                if (day <= midpoint) {
                  _startDay = day;
                } else {
                  _endDay = day;
                }
              }
            });
            _notify();
          },
          child: AnimatedContainer(
            duration: const Duration(milliseconds: 140),
            width: 22,
            height: 22,
            decoration: BoxDecoration(
              // Extremos: fondo sólido del énfasis. Interior: fondo tenue. Fuera: transparente.
              color: isEdge
                  ? Gx.accentDynamic
                  : inRange
                      ? Gx.accentDynamic.withAlpha(46)
                      : Colors.transparent,
              // Radio 4: decorativo para celda de calendario 22px (sin token a esta escala).
              borderRadius: BorderRadius.circular(4),
              boxShadow: isEdge
                  ? Gx.glow(Gx.accentDynamic, blur: 6, opacity: 0.6)
                  : null,
            ),
            child: Center(
              child: Text(
                '$day',
                style: Gx.uiSans(
                  fontSize: 10,
                  color: isEdge
                      ? Gx.pureWhite
                      : inRange
                          ? Gx.accentDynamic
                          : Gx.textBaseMuted,
                ),
              ),
            ),
          ),
        );
      }),
    );
  }
}
