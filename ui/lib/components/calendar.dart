// calendar.dart — Componente Calendar (ADR-0138 enmienda 2026-06-29).
// Calendario mensual navegable con marcadores de evento y día seleccionado.
// Diferencia con DatePicker: visualización de mes completo (no selector de formulario);
// soporta [events] para marcar fechas con punto de alerta.
// Migrado de GlowCalendar (gallery/gallery_fx.dart, Batch 4 STORY-025).

import 'package:flutter/material.dart';
import '../gallery/gallery_tokens.dart';
import '../gallery/gallery_fx.dart';

// Nombres de mes en español para el encabezado de navegación.
const _calMonths = [
  'Enero', 'Febrero', 'Marzo', 'Abril', 'Mayo', 'Junio',
  'Julio', 'Agosto', 'Septiembre', 'Octubre', 'Noviembre', 'Diciembre',
];

// Abreviaturas de días de la semana empezando en lunes.
const _calWeekdays = ['Lu', 'Ma', 'Mi', 'Ju', 'Vi', 'Sá', 'Do'];

// Calendario mensual navegable con soporte de marcadores de evento.
// Contrato funcional:
//   [value]     fecha seleccionada inicialmente; null = sin selección (modo no controlado).
//   [onChanged] callback al seleccionar un día; recibe la nueva fecha.
//   [events]    conjunto de fechas con marcador de evento (punto alertAmber debajo del número).
class Calendar extends StatefulWidget {
  final DateTime? value;
  final ValueChanged<DateTime>? onChanged;
  final Set<DateTime>? events;

  // No es const: el build lee getters dinámicos de Gx que cambian con el tema.
  Calendar({super.key, this.value, this.onChanged, this.events});

  @override
  State<Calendar> createState() => _CalendarState();
}

class _CalendarState extends State<Calendar> {
  // Mes/año que muestra el navegador (independiente del día seleccionado).
  late DateTime _viewMonth;
  // Fecha seleccionada en modo no controlado.
  DateTime? _internalValue;

  // Fecha efectiva: la externa (widget.value) tiene prioridad sobre la interna.
  DateTime? get _effectiveValue => widget.value ?? _internalValue;

  @override
  void initState() {
    super.initState();
    // El mes inicial es el de la fecha seleccionada, o el mes actual.
    final ref = widget.value ?? DateTime.now();
    _viewMonth = DateTime(ref.year, ref.month);
  }

  @override
  void didUpdateWidget(Calendar old) {
    super.didUpdateWidget(old);
    // Si el padre actualiza el valor, sincroniza el mes visible.
    if (widget.value != null && widget.value != old.value) {
      setState(() {
        _viewMonth = DateTime(widget.value!.year, widget.value!.month);
      });
    }
  }

  // Avanza o retrocede el mes visible. dir: +1 = siguiente, -1 = anterior.
  void _changeMonth(int dir) {
    setState(() {
      _viewMonth = DateTime(_viewMonth.year, _viewMonth.month + dir);
    });
  }

  // Selecciona un día: actualiza estado interno y llama al callback externo.
  void _select(DateTime date) {
    if (widget.value == null) setState(() => _internalValue = date);
    widget.onChanged?.call(date);
  }

  // Devuelve true si la fecha tiene un marcador de evento registrado en [events].
  bool _hasEvent(DateTime date) {
    if (widget.events == null) return false;
    return widget.events!.any((e) =>
        e.year == date.year && e.month == date.month && e.day == date.day);
  }

  // Número de días del mes visible.
  int _daysInMonth() => DateTime(_viewMonth.year, _viewMonth.month + 1, 0).day;

  // Offset (0=Lun…6=Dom) para alinear el primer día del mes en la grilla semanal.
  int _firstWeekdayOffset() =>
      DateTime(_viewMonth.year, _viewMonth.month, 1).weekday - 1;

  @override
  // Muestra encabezado de mes con flechas, cabecera de días de semana y grilla de días.
  Widget build(BuildContext context) {
    final selected = _effectiveValue;
    final offset = _firstWeekdayOffset();
    final total = _daysInMonth();
    final rows = ((offset + total) / 7).ceil();
    final now = DateTime.now();

    return panelSurface(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          // Encabezado de navegación: flecha izquierda · mes/año · flecha derecha.
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              GestureDetector(
                onTap: () => _changeMonth(-1),
                child: Icon(Icons.chevron_left, size: 20, color: Gx.textBaseSecondary),
              ),
              Text(
                '${_calMonths[_viewMonth.month - 1]} ${_viewMonth.year}',
                style: Gx.panelTitle.copyWith(color: Gx.accentDynamic),
              ),
              GestureDetector(
                onTap: () => _changeMonth(1),
                child: Icon(Icons.chevron_right, size: 20, color: Gx.textBaseSecondary),
              ),
            ],
          ),
          const SizedBox(height: 8),
          // Cabecera de días de la semana: Lu Ma Mi Ju Vi Sá Do.
          Row(
            children: _calWeekdays
                .map((d) => Expanded(
                      child: Text(d,
                          textAlign: TextAlign.center,
                          style: Gx.microLabel),
                    ))
                .toList(),
          ),
          const SizedBox(height: 4),
          // Grilla mensual: [rows] filas de 7 columnas de celdas de día.
          ...List.generate(
            rows,
            (rowIdx) => Row(
              children: List.generate(7, (colIdx) {
                final cellIdx = rowIdx * 7 + colIdx;
                final dayNum = cellIdx - offset + 1;

                // Celdas vacías de relleno antes del primer día y tras el último.
                if (dayNum < 1 || dayNum > total) {
                  return const Expanded(child: SizedBox(height: 36));
                }

                final date = DateTime(_viewMonth.year, _viewMonth.month, dayNum);
                final isSel = selected != null &&
                    selected.year == date.year &&
                    selected.month == date.month &&
                    selected.day == date.day;
                final isToday = now.year == date.year &&
                    now.month == date.month &&
                    now.day == date.day;
                final hasEvt = _hasEvent(date);

                return Expanded(
                  child: GestureDetector(
                    onTap: () => _select(date),
                    child: AnimatedContainer(
                      duration: const Duration(milliseconds: 180),
                      height: 36,
                      margin: const EdgeInsets.all(1),
                      alignment: Alignment.center,
                      decoration: BoxDecoration(
                        shape: BoxShape.circle,
                        // Día seleccionado: borde énfasis dinámico + glow.
                        // Hoy (no seleccionado): borde base sutil de referencia.
                        border: isSel
                            ? Border.all(color: Gx.optimaCyan, width: 1.5)
                            : isToday
                                ? Border.all(color: Gx.borderBase)
                                : null,
                        boxShadow: isSel
                            ? Gx.glow(Gx.optimaCyan, blur: 14, opacity: 0.7)
                            : null,
                      ),
                      child: Stack(
                        alignment: Alignment.center,
                        children: [
                          // Número del día con color semántico al seleccionar.
                          Text(
                            '$dayNum',
                            style: Gx.dataMono(
                              fontSize: 11,
                              color: isSel
                                  ? Gx.optimaCyan
                                  : Gx.textBaseSecondary,
                            ),
                          ),
                          // Punto de evento debajo del número: alertAmber, 3px de diámetro.
                          // Radio 3px: decorativo de punto mínimo — sin token equivalente.
                          if (hasEvt)
                            Positioned(
                              bottom: 3,
                              child: Container(
                                width: 3,
                                height: 3,
                                decoration: const BoxDecoration(
                                  shape: BoxShape.circle,
                                  color: Gx.alertAmber,
                                ),
                              ),
                            ),
                        ],
                      ),
                    ),
                  ),
                );
              }),
            ),
          ),
        ],
      ),
    );
  }
}
