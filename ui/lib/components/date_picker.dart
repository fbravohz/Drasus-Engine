// date_picker.dart — Componente DatePicker (ADR-0138 enmienda 2026-06-29).
// Selector de fecha compacto con grilla mensual y navegación de mes.
// El estilo lo decide el tema global vía tokens Gx; prohibido hardcodear color.

import 'package:flutter/material.dart';
import '../theme/gx_tokens.dart';
import '../theme/surfaces.dart';

// Nombres de meses en español para el encabezado de navegación.
const _meses = [
  'Enero', 'Febrero', 'Marzo', 'Abril', 'Mayo', 'Junio',
  'Julio', 'Agosto', 'Septiembre', 'Octubre', 'Noviembre', 'Diciembre',
];

// Abreviaturas de días de la semana empezando en lunes.
const _diasSem = ['Lu', 'Ma', 'Mi', 'Ju', 'Vi', 'Sá', 'Do'];

// Selector de fecha compacto con grilla mensual, navegación y estado seleccionado.
// Contrato funcional: [value] fecha actualmente seleccionada (null = modo no controlado,
// sin selección inicial); [onChanged] callback con la nueva fecha al seleccionar;
// [firstDate] fecha mínima seleccionable; [lastDate] fecha máxima seleccionable.
class DatePicker extends StatefulWidget {
  final DateTime? value;
  final ValueChanged<DateTime>? onChanged;
  final DateTime? firstDate;
  final DateTime? lastDate;

  // No es const: lee getters dinámicos de Gx que cambian con el tema.
  DatePicker({
    super.key,
    this.value,
    this.onChanged,
    this.firstDate,
    this.lastDate,
  });

  @override
  State<DatePicker> createState() => _DatePickerState();
}

class _DatePickerState extends State<DatePicker> {
  // Mes/año que se muestra actualmente en el navegador.
  late DateTime _viewMonth;
  // Fecha seleccionada en modo no controlado.
  DateTime? _internalValue;

  // Fecha efectiva: la externa tiene prioridad sobre la interna.
  DateTime? get _effectiveValue => widget.value ?? _internalValue;

  @override
  void initState() {
    super.initState();
    // El mes inicial es el de la fecha seleccionada o el mes actual.
    final ref = widget.value ?? DateTime.now();
    _viewMonth = DateTime(ref.year, ref.month);
  }

  @override
  void didUpdateWidget(DatePicker old) {
    super.didUpdateWidget(old);
    // Si el padre actualiza el valor, sincroniza el mes visible.
    if (widget.value != null && widget.value != old.value) {
      _viewMonth = DateTime(widget.value!.year, widget.value!.month);
    }
  }

  // Avanza o retrocede el mes visible. dir: +1 = siguiente, -1 = anterior.
  void _changeMonth(int dir) {
    setState(() {
      _viewMonth = DateTime(_viewMonth.year, _viewMonth.month + dir);
    });
  }

  // Comprueba si una fecha está dentro del rango permitido.
  bool _isInRange(DateTime date) {
    if (widget.firstDate != null && date.isBefore(widget.firstDate!)) return false;
    if (widget.lastDate != null && date.isAfter(widget.lastDate!)) return false;
    return true;
  }

  // Selecciona una fecha: actualiza el estado interno y llama al callback.
  void _selectDate(DateTime date) {
    if (!_isInRange(date)) return;
    if (widget.value == null) setState(() => _internalValue = date);
    widget.onChanged?.call(date);
  }

  // Calcula el primer día de la semana del mes visible (0=Lun…6=Dom en _diasSem).
  int _firstWeekdayOffset() {
    // weekday en Dart: 1=Lun, 7=Dom. Offset para empezar en lunes = weekday - 1.
    return DateTime(_viewMonth.year, _viewMonth.month, 1).weekday - 1;
  }

  // Número de días del mes visible.
  int _daysInMonth() {
    return DateTime(_viewMonth.year, _viewMonth.month + 1, 0).day;
  }

  @override
  // Grilla mensual con encabezado de navegación, cabecera de días y celdas de día.
  Widget build(BuildContext context) {
    final selected = _effectiveValue;
    final offset = _firstWeekdayOffset();
    final total = _daysInMonth();
    // Total de celdas = offset de inicio + días del mes (redondeado a semanas completas).
    final cellCount = offset + total;
    final rows = (cellCount / 7).ceil();

    return panelSurface(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          // Encabezado de navegación: mes/año + flechas anterior/siguiente.
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              GestureDetector(
                onTap: () => _changeMonth(-1),
                child: Icon(Icons.chevron_left,
                    size: 20, color: Gx.textBaseSecondary),
              ),
              Text(
                '${_meses[_viewMonth.month - 1]} ${_viewMonth.year}',
                style: Gx.panelTitle.copyWith(color: Gx.accentDynamic),
              ),
              GestureDetector(
                onTap: () => _changeMonth(1),
                child: Icon(Icons.chevron_right,
                    size: 20, color: Gx.textBaseSecondary),
              ),
            ],
          ),
          const SizedBox(height: 8),
          // Cabecera de días de la semana (Lun … Dom).
          Row(
            children: _diasSem.map((d) => Expanded(
              child: Text(d,
                  textAlign: TextAlign.center,
                  style: Gx.microLabel),
            )).toList(),
          ),
          const SizedBox(height: 4),
          // Grilla de celdas de día: [rows] filas de 7 columnas.
          ...List.generate(rows, (rowIdx) {
            return Row(
              children: List.generate(7, (colIdx) {
                final cellIdx = rowIdx * 7 + colIdx;
                final dayNum = cellIdx - offset + 1; // número del día (1-base)

                // Celdas de relleno antes del primer día y después del último.
                if (dayNum < 1 || dayNum > total) {
                  return const Expanded(child: SizedBox(height: 32));
                }

                final date = DateTime(_viewMonth.year, _viewMonth.month, dayNum);
                final isSel = selected != null &&
                    selected.year == date.year &&
                    selected.month == date.month &&
                    selected.day == date.day;
                final isToday = () {
                  final now = DateTime.now();
                  return now.year == date.year &&
                      now.month == date.month &&
                      now.day == date.day;
                }();
                final inRange = _isInRange(date);

                return Expanded(
                  child: GestureDetector(
                    onTap: inRange ? () => _selectDate(date) : null,
                    child: AnimatedContainer(
                      duration: const Duration(milliseconds: 180),
                      height: 32,
                      margin: const EdgeInsets.all(1),
                      alignment: Alignment.center,
                      decoration: BoxDecoration(
                        shape: BoxShape.circle,
                        // Día seleccionado: borde de énfasis dinámico con glow.
                        border: isSel
                            ? Border.all(color: Gx.optimaCyan, width: 1.5)
                            : isToday
                                ? Border.all(color: Gx.borderBase)
                                : null,
                        boxShadow: isSel
                            ? Gx.glow(Gx.optimaCyan, blur: 14, opacity: 0.7)
                            : null,
                      ),
                      child: Text(
                        '$dayNum',
                        style: Gx.dataMono(
                          fontSize: 11,
                          color: isSel
                              ? Gx.optimaCyan          // seleccionado: color semántico
                              : !inRange
                                  ? Gx.textBaseMuted   // fuera de rango: muy tenue
                                  : Gx.textBaseSecondary, // disponible: secundario
                        ),
                      ),
                    ),
                  ),
                );
              }),
            );
          }),
        ],
      ),
    );
  }
}
