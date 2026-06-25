// Sección STD faltantes — piezas del catálogo §5–§7 pendientes.
// Render-only con estado de UI local. Sin lógica de negocio ni FFI.
// Todas las piezas usan tokens de Gx y siguen el lenguaje
// glow / gradiente / vidrio Apple de DESIGN.md.

import 'package:flutter/material.dart';
import '../gallery_tokens.dart';
import '../gallery_fx.dart';

// ===========================================================================
// §6 INPUTS — CASCADER
// ===========================================================================

// Muestra un selector jerárquico en cascada: columna izquierda (nivel 1),
// columna derecha (nivel 2). Al tocar una opción de nivel 1 aparecen sus
// hijos en nivel 2. Datos hardcodeados de régimen > símbolo.
class GlowCascader extends StatefulWidget {
  const GlowCascader({super.key});

  @override
  State<GlowCascader> createState() => _GlowCascaderState();
}

class _GlowCascaderState extends State<GlowCascader> {
  // Opciones de nivel 1 y sus hijos de nivel 2 (hardcodeados).
  static const _tree = {
    'Óptimo': ['SPX', 'QQQ', 'GLD'],
    'Transición': ['EUR/USD', 'G10'],
    'Alerta': ['VIX', 'OIL'],
  };

  // Índice del ítem seleccionado en nivel 1 (-1 = ninguno).
  int _sel1 = 0;

  @override
  Widget build(BuildContext context) {
    final keys = _tree.keys.toList();
    final children = _tree[keys[_sel1]] ?? [];

    // Contenedor vidrio con dos columnas separadas por hairline.
    return Container(
      height: 130,
      decoration: BoxDecoration(
        color: Gx.glassFill,
        borderRadius: BorderRadius.circular(Gx.rInput),
        border: Border.all(color: Gx.borderPanel),
        boxShadow: Gx.glow(Gx.transitionIndigo, blur: 12, opacity: 0.2),
      ),
      child: Row(
        children: [
          // Columna 1 — categorías principales.
          Expanded(
            child: Column(
              children: keys.asMap().entries.map((e) {
                final isActive = e.key == _sel1;
                return GestureDetector(
                  onTap: () => setState(() => _sel1 = e.key),
                  child: AnimatedContainer(
                    duration: const Duration(milliseconds: 160),
                    padding: const EdgeInsets.symmetric(
                        horizontal: 12, vertical: 10),
                    decoration: BoxDecoration(
                      color: isActive
                          ? Gx.transitionIndigo.withOpacity(0.12)
                          : Colors.transparent,
                      border: isActive
                          ? const Border(
                              right: BorderSide(
                                  color: Gx.transitionIndigo, width: 2))
                          : null,
                    ),
                    child: Row(
                      mainAxisAlignment: MainAxisAlignment.spaceBetween,
                      children: [
                        // Flexible evita desbordamiento cuando el texto
                        // es largo en un Expanded con espacio limitado.
                        Flexible(
                          child: Text(
                            e.value,
                            overflow: TextOverflow.ellipsis,
                            style: Gx.uiSans(
                              fontSize: 13,
                              color: isActive
                                  ? Gx.transitionIndigo
                                  : Gx.textSecondary,
                            ),
                          ),
                        ),
                        // Flecha indicando que hay subnivel.
                        Icon(Gx.iconChevronDown,
                            size: 10,
                            color: isActive
                                ? Gx.transitionIndigo
                                : Gx.textMuted),
                      ],
                    ),
                  ),
                );
              }).toList(),
            ),
          ),
          // Separador vertical entre columnas.
          Container(width: 1, color: Gx.borderPanel),
          // Columna 2 — opciones de nivel 2 del ítem seleccionado.
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: children
                  .map((c) => Padding(
                        padding: const EdgeInsets.symmetric(
                            horizontal: 12, vertical: 10),
                        child: Text(c, style: Gx.dataMono(fontSize: 13)),
                      ))
                  .toList(),
            ),
          ),
        ],
      ),
    );
  }
}

// ===========================================================================
// §6 INPUTS — TRANSFER / DUAL-LIST
// ===========================================================================

// Muestra dos listas (disponibles / seleccionados) con botones de transferencia
// entre ellas. Los datos son símbolos hardcodeados.
class GlowTransferList extends StatefulWidget {
  const GlowTransferList({super.key});

  @override
  State<GlowTransferList> createState() => _GlowTransferListState();
}

class _GlowTransferListState extends State<GlowTransferList> {
  // Lista izquierda (disponibles) y derecha (seleccionados).
  List<String> _left = ['SPX', 'QQQ', 'GLD', 'VIX', 'OIL'];
  List<String> _right = ['G10', 'EUR/USD'];

  // Conjuntos de ítems marcados en cada lista para mover.
  final Set<String> _checkedLeft = {};
  final Set<String> _checkedRight = {};

  // Mueve los ítems marcados de izquierda a derecha.
  void _moveRight() {
    setState(() {
      _right.addAll(_checkedLeft);
      _left.removeWhere(_checkedLeft.contains);
      _checkedLeft.clear();
    });
  }

  // Mueve los ítems marcados de derecha a izquierda.
  void _moveLeft() {
    setState(() {
      _left.addAll(_checkedRight);
      _right.removeWhere(_checkedRight.contains);
      _checkedRight.clear();
    });
  }

  // Construye una columna de lista con checkboxes.
  Widget _list(List<String> items, Set<String> checked, String header) {
    return Expanded(
      child: Container(
        decoration: BoxDecoration(
          color: Gx.panelSolid,
          borderRadius: BorderRadius.circular(Gx.rPanel),
          border: Border.all(color: Gx.borderPanel),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // Cabecera de la lista.
            Container(
              padding:
                  const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
              decoration: const BoxDecoration(
                border: Border(bottom: BorderSide(color: Gx.borderPanel)),
              ),
              child: Text(header, style: Gx.microLabel),
            ),
            // Ítems con checkbox.
            ...items.map((item) {
              final isChecked = checked.contains(item);
              return GestureDetector(
                onTap: () => setState(() =>
                    isChecked ? checked.remove(item) : checked.add(item)),
                child: AnimatedContainer(
                  duration: const Duration(milliseconds: 140),
                  padding: const EdgeInsets.symmetric(
                      horizontal: 10, vertical: 7),
                  color: isChecked
                      ? Gx.transitionIndigo.withOpacity(0.10)
                      : Colors.transparent,
                  child: Row(children: [
                    // Checkbox mínimo con color de estado.
                    Container(
                      width: 14,
                      height: 14,
                      decoration: BoxDecoration(
                        color: isChecked
                            ? Gx.transitionIndigo
                            : Colors.transparent,
                        borderRadius: BorderRadius.circular(3),
                        border: Border.all(
                          color: isChecked
                              ? Gx.transitionIndigo
                              : Gx.textMuted,
                        ),
                        boxShadow: isChecked
                            ? Gx.glow(Gx.transitionIndigo,
                                blur: 6, opacity: 0.5)
                            : null,
                      ),
                    ),
                    const SizedBox(width: 8),
                    Text(item,
                        style: Gx.dataMono(
                            fontSize: 13,
                            color: isChecked
                                ? Gx.textPrimary
                                : Gx.textSecondary)),
                  ]),
                ),
              );
            }),
          ],
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    // Dos listas con botones de transferencia en el centro.
    return SizedBox(
      height: 200,
      child: Row(
        children: [
          _list(_left, _checkedLeft, 'Disponibles'),
          // Botones de transferencia en columna vertical centrada.
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 8),
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                // Botón mover a derecha.
                _transferBtn(
                    Icons.chevron_right, Gx.optimaCyan, _moveRight),
                const SizedBox(height: 6),
                // Botón mover a izquierda.
                _transferBtn(
                    Icons.chevron_left, Gx.criticalRed, _moveLeft),
              ],
            ),
          ),
          _list(_right, _checkedRight, 'Seleccionados'),
        ],
      ),
    );
  }

  // Botón de transferencia: icono con glow del color de estado.
  Widget _transferBtn(IconData icon, Color c, VoidCallback onTap) {
    return GestureDetector(
      onTap: onTap,
      child: Container(
        padding: const EdgeInsets.all(6),
        decoration: BoxDecoration(
          color: c.withOpacity(0.12),
          borderRadius: BorderRadius.circular(6),
          border: Border.all(color: c.withOpacity(0.5)),
          boxShadow: Gx.glow(c, blur: 8, opacity: 0.3),
        ),
        child: Icon(icon, size: 16, color: c),
      ),
    );
  }
}

// ===========================================================================
// §6 INPUTS — DATE-RANGE PICKER
// ===========================================================================

// Muestra dos campos de fecha (inicio / fin) con un rango hardcodeado
// seleccionado. Cada campo tiene glow en foco y el rango se resalta
// en el mini-calendario debajo.
class GlowDateRangePicker extends StatefulWidget {
  const GlowDateRangePicker({super.key});

  @override
  State<GlowDateRangePicker> createState() => _GlowDateRangePickerState();
}

class _GlowDateRangePickerState extends State<GlowDateRangePicker> {
  // Rango seleccionado hardcodeado: primer y último día del mes activo.
  int _startDay = 3;
  int _endDay = 17;
  // Mes de referencia (junio 2026).
  static const _monthLabel = 'JUN 2026';
  static const _daysInMonth = 30;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: Gx.glassFill,
        borderRadius: BorderRadius.circular(Gx.rInput),
        border: Border.all(color: Gx.borderPanel),
        boxShadow: Gx.glow(Gx.transitionIndigo, blur: 12, opacity: 0.15),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Fila de campos inicio / fin.
          Row(children: [
            Expanded(child: _dateField('Inicio', _startDay)),
            const SizedBox(width: 8),
            const Icon(Icons.arrow_forward, size: 14, color: Gx.textMuted),
            const SizedBox(width: 8),
            Expanded(child: _dateField('Fin', _endDay)),
          ]),
          const SizedBox(height: 10),
          // Etiqueta del mes.
          Text(_monthLabel,
              style: Gx.microLabel.copyWith(color: Gx.textSecondary)),
          const SizedBox(height: 6),
          // Mini-cuadrícula de días del mes.
          _miniCalendar(),
        ],
      ),
    );
  }

  // Campo de fecha con borde neón.
  Widget _dateField(String label, int day) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
      decoration: BoxDecoration(
        color: Gx.panelSolid,
        borderRadius: BorderRadius.circular(Gx.rInput),
        border: Border.all(color: Gx.transitionIndigo.withOpacity(0.6)),
        boxShadow: Gx.glow(Gx.transitionIndigo, blur: 8, opacity: 0.2),
      ),
      child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
        Text(label, style: Gx.microLabel),
        const SizedBox(height: 2),
        Text(
          '$day/06/2026',
          style: Gx.dataMono(fontSize: 13, color: Gx.textPrimary),
        ),
      ]),
    );
  }

  // Mini-cuadrícula del mes con el rango resaltado.
  Widget _miniCalendar() {
    return Wrap(
      spacing: 4,
      runSpacing: 4,
      children: List.generate(_daysInMonth, (i) {
        final day = i + 1;
        // Determina si el día está dentro del rango seleccionado.
        final inRange = day >= _startDay && day <= _endDay;
        final isEdge = day == _startDay || day == _endDay;

        return GestureDetector(
          onTap: () {
            // Al tocar un día, reajusta el rango.
            setState(() {
              if (day <= _startDay) {
                _startDay = day;
              } else if (day >= _endDay) {
                _endDay = day;
              } else {
                // Si está dentro del rango, mueve el extremo más cercano.
                final midpoint = (_startDay + _endDay) ~/ 2;
                if (day <= midpoint) {
                  _startDay = day;
                } else {
                  _endDay = day;
                }
              }
            });
          },
          child: AnimatedContainer(
            duration: const Duration(milliseconds: 140),
            width: 22,
            height: 22,
            decoration: BoxDecoration(
              // Extremos del rango: fondo neón sólido.
              // Interior del rango: fondo tenue.
              // Fuera: transparente.
              color: isEdge
                  ? Gx.transitionIndigo
                  : inRange
                      ? Gx.transitionIndigo.withOpacity(0.18)
                      : Colors.transparent,
              borderRadius: BorderRadius.circular(4),
              boxShadow: isEdge
                  ? Gx.glow(Gx.transitionIndigo, blur: 6, opacity: 0.6)
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
                          ? Gx.transitionIndigo
                          : Gx.textMuted,
                ),
              ),
            ),
          ),
        );
      }),
    );
  }
}

// ===========================================================================
// §6 INPUTS — TIME PICKER
// ===========================================================================

// Muestra un selector de hora con dos columnas deslizables (horas / minutos)
// de estilo rueda. El ítem central está resaltado con glow.
class GlowTimePicker extends StatefulWidget {
  const GlowTimePicker({super.key});

  @override
  State<GlowTimePicker> createState() => _GlowTimePickerState();
}

class _GlowTimePickerState extends State<GlowTimePicker> {
  // Hora y minutos seleccionados (hardcodeados inicialmente).
  int _hour = 9;
  int _minute = 30;

  @override
  Widget build(BuildContext context) {
    return Container(
      height: 120,
      decoration: BoxDecoration(
        color: Gx.glassFill,
        borderRadius: BorderRadius.circular(Gx.rInput),
        border: Border.all(color: Gx.borderPanel),
        boxShadow: Gx.glow(Gx.transitionIndigo, blur: 12, opacity: 0.15),
      ),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          // Columna de horas.
          _wheelColumn(
            values: List.generate(24, (i) => i.toString().padLeft(2, '0')),
            selected: _hour,
            onSelect: (v) => setState(() => _hour = v),
          ),
          // Separador ":".
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 8),
            child: Text(':',
                style: Gx.dataMono(
                    fontSize: 24,
                    color: Gx.transitionIndigo,
                    weight: FontWeight.w500)),
          ),
          // Columna de minutos (de 5 en 5).
          _wheelColumn(
            values: List.generate(12, (i) => (i * 5).toString().padLeft(2, '0')),
            selected: _minute ~/ 5,
            onSelect: (v) => setState(() => _minute = v * 5),
          ),
        ],
      ),
    );
  }

  // Columna tipo rueda: muestra 3 ítems, el central resaltado.
  Widget _wheelColumn({
    required List<String> values,
    required int selected,
    required ValueChanged<int> onSelect,
  }) {
    // Muestra el ítem anterior, el actual y el siguiente.
    final prev = (selected - 1 + values.length) % values.length;
    final next = (selected + 1) % values.length;

    return Column(
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        // Ítem anterior (atenuado).
        GestureDetector(
          onTap: () => onSelect(prev),
          child: Padding(
            padding: const EdgeInsets.symmetric(vertical: 4),
            child: Text(values[prev],
                style: Gx.dataMono(fontSize: 16, color: Gx.textMuted)),
          ),
        ),
        // Ítem central (seleccionado, con glow).
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
          decoration: BoxDecoration(
            color: Gx.transitionIndigo.withOpacity(0.15),
            borderRadius: BorderRadius.circular(8),
            border: Border.all(color: Gx.transitionIndigo.withOpacity(0.6)),
            boxShadow: Gx.glow(Gx.transitionIndigo, blur: 8, opacity: 0.4),
          ),
          child: Text(
            values[selected],
            style: Gx.dataMono(
                fontSize: 22,
                color: Gx.transitionIndigo,
                weight: FontWeight.w500),
          ),
        ),
        // Ítem siguiente (atenuado).
        GestureDetector(
          onTap: () => onSelect(next),
          child: Padding(
            padding: const EdgeInsets.symmetric(vertical: 4),
            child: Text(values[next],
                style: Gx.dataMono(fontSize: 16, color: Gx.textMuted)),
          ),
        ),
      ],
    );
  }
}

// ===========================================================================
// §6 INPUTS — COLOR PICKER
// ===========================================================================

// Muestra una paleta de colores del espectro de vitalidad + una muestra
// del color seleccionado. Sin rueda HSV — la paleta de Drasus es semántica.
class GlowColorPicker extends StatefulWidget {
  const GlowColorPicker({super.key});

  @override
  State<GlowColorPicker> createState() => _GlowColorPickerState();
}

class _GlowColorPickerState extends State<GlowColorPicker> {
  // Colores del espectro de vitalidad del sistema de diseño.
  static const _palette = [
    Gx.optimaCyan,
    Gx.optimaTeal,
    Gx.reactorGreen,
    Gx.transitionIndigo,
    Gx.transitionBlue,
    Gx.transitionPurple,
    Gx.alertAmber,
    Gx.alertOrange,
    Gx.criticalRed,
    Gx.criticalCrimson,
  ];

  // Color seleccionado actualmente.
  Color _selected = Gx.optimaCyan;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: Gx.glassFill,
        borderRadius: BorderRadius.circular(Gx.rInput),
        border: Border.all(color: Gx.borderPanel),
        boxShadow: Gx.glow(_selected, blur: 12, opacity: 0.2),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Muestra del color seleccionado + valor hex.
          Row(children: [
            // Swatch grande del color activo.
            Container(
              width: 36,
              height: 36,
              decoration: BoxDecoration(
                color: _selected,
                borderRadius: BorderRadius.circular(8),
                boxShadow: Gx.glowStrong(_selected),
              ),
            ),
            const SizedBox(width: 10),
            // Etiqueta del token o valor.
            Text(
              _tokenName(_selected),
              style: Gx.dataMono(fontSize: 12, color: Gx.textSecondary),
            ),
          ]),
          const SizedBox(height: 10),
          // Grilla de swatches de la paleta semántica.
          Wrap(
            spacing: 6,
            runSpacing: 6,
            children: _palette.map((c) {
              final isSelected = c == _selected;
              return GestureDetector(
                onTap: () => setState(() => _selected = c),
                child: AnimatedContainer(
                  duration: const Duration(milliseconds: 140),
                  width: 24,
                  height: 24,
                  decoration: BoxDecoration(
                    color: c,
                    borderRadius: BorderRadius.circular(6),
                    border: Border.all(
                      color: isSelected ? Gx.pureWhite : Colors.transparent,
                      width: 2,
                    ),
                    boxShadow: isSelected
                        ? Gx.glowStrong(c)
                        : Gx.glow(c, blur: 6, opacity: 0.3),
                  ),
                ),
              );
            }).toList(),
          ),
        ],
      ),
    );
  }

  // Retorna el nombre del token para el color dado (hardcodeado para la galería).
  // No usa mapa const con Color como clave porque Color sobrescribe == y no es
  // apto para claves de mapa constante en Dart — se usa if/else en su lugar.
  String _tokenName(Color c) {
    if (c == Gx.optimaCyan) return 'optimaCyan';
    if (c == Gx.optimaTeal) return 'optimaTeal';
    if (c == Gx.reactorGreen) return 'reactorGreen';
    if (c == Gx.transitionIndigo) return 'transitionIndigo';
    if (c == Gx.transitionBlue) return 'transitionBlue';
    if (c == Gx.transitionPurple) return 'transitionPurple';
    if (c == Gx.alertAmber) return 'alertAmber';
    if (c == Gx.alertOrange) return 'alertOrange';
    if (c == Gx.criticalRed) return 'criticalRed';
    if (c == Gx.criticalCrimson) return 'criticalCrimson';
    // Color desconocido: muestra el valor ARGB en hex.
    return '#${c.toARGB32().toRadixString(16).padLeft(8, '0').toUpperCase()}';
  }
}

// ===========================================================================
// §6 INPUTS — FILE UPLOAD / DROPZONE
// ===========================================================================

// Muestra una zona de arrastre de archivos con estado: reposo, arrastrando
// (activado al pasar el mouse), y "cargando" (simulado al tocar).
class GlowDropzone extends StatefulWidget {
  const GlowDropzone({super.key});

  @override
  State<GlowDropzone> createState() => _GlowDropzoneState();
}

class _GlowDropzoneState extends State<GlowDropzone> {
  // Estado de la dropzone: reposo, hover (arrastrando), cargando.
  _DropState _state = _DropState.idle;

  @override
  Widget build(BuildContext context) {
    // Color del estado activo.
    final stateColor = _state == _DropState.loading
        ? Gx.optimaCyan
        : _state == _DropState.hover
            ? Gx.transitionIndigo
            : Gx.textMuted;

    return MouseRegion(
      onEnter: (_) {
        if (_state == _DropState.idle) {
          setState(() => _state = _DropState.hover);
        }
      },
      onExit: (_) {
        if (_state == _DropState.hover) {
          setState(() => _state = _DropState.idle);
        }
      },
      child: GestureDetector(
        // Al tocar simula la carga y luego vuelve al reposo.
        onTap: () async {
          setState(() => _state = _DropState.loading);
          await Future.delayed(const Duration(seconds: 2));
          if (mounted) setState(() => _state = _DropState.idle);
        },
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 200),
          padding: const EdgeInsets.all(24),
          decoration: BoxDecoration(
            // Borde discontinuo simulado con border coloreado.
            color: _state == _DropState.hover
                ? Gx.transitionIndigo.withOpacity(0.08)
                : _state == _DropState.loading
                    ? Gx.optimaCyan.withOpacity(0.06)
                    : Gx.panelSolid,
            borderRadius: BorderRadius.circular(Gx.rPanel),
            border: Border.all(
              color: stateColor.withOpacity(
                  _state == _DropState.idle ? 0.4 : 0.8),
              // Ancho ligeramente mayor en estados activos.
              width: _state == _DropState.idle ? 1.0 : 1.5,
            ),
            boxShadow: _state != _DropState.idle
                ? Gx.glow(stateColor, blur: 14, opacity: 0.25)
                : null,
          ),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              // Icono central con glow del estado.
              Icon(
                _state == _DropState.loading
                    ? Gx.iconRefresh
                    : Gx.iconAdd,
                size: 28,
                color: stateColor,
                shadows: _state != _DropState.idle
                    ? Gx.textGlow(stateColor, 12)
                    : null,
              ),
              const SizedBox(height: 8),
              // Mensaje según estado.
              Text(
                _state == _DropState.loading
                    ? 'Cargando…'
                    : _state == _DropState.hover
                        ? 'Suelta aquí'
                        : 'Arrastra o toca para cargar',
                style:
                    Gx.uiSans(fontSize: 13, color: stateColor),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

// Estados internos de la dropzone.
enum _DropState { idle, hover, loading }

// ===========================================================================
// §6 INPUTS — MENTION INPUT
// ===========================================================================

// Input de texto con detección de @menciones. Al escribir "@" aparece
// una lista de sugerencias. Datos hardcodeados (usuarios simulados).
class GlowMentionInput extends StatefulWidget {
  const GlowMentionInput({super.key});

  @override
  State<GlowMentionInput> createState() => _GlowMentionInputState();
}

class _GlowMentionInputState extends State<GlowMentionInput> {
  final _ctrl = TextEditingController(text: 'Revisar con @');
  final _focus = FocusNode();
  // Usuarios disponibles para mencionar (hardcodeados).
  static const _users = [
    '@quant-01',
    '@alpha-desk',
    '@risk-mgr',
    '@ops-lead',
  ];
  bool _showSuggestions = false;

  @override
  void initState() {
    super.initState();
    // Detecta si el cursor está justo después de un "@" para mostrar sugerencias.
    _ctrl.addListener(_updateSuggestions);
    _focus.addListener(() => setState(() {}));
  }

  void _updateSuggestions() {
    final text = _ctrl.text;
    // Muestra sugerencias si el texto termina en "@" o "@<parcial>".
    final match = text.endsWith('@') ||
        (text.contains('@') &&
            !text.substring(text.lastIndexOf('@')).contains(' '));
    setState(() => _showSuggestions = match);
  }

  @override
  void dispose() {
    _ctrl.dispose();
    _focus.dispose();
    super.dispose();
  }

  // Inserta la mención seleccionada en el texto.
  void _insertMention(String user) {
    final text = _ctrl.text;
    final lastAt = text.lastIndexOf('@');
    _ctrl.text = '${text.substring(0, lastAt)}$user ';
    _ctrl.selection = TextSelection.collapsed(offset: _ctrl.text.length);
    setState(() => _showSuggestions = false);
  }

  @override
  Widget build(BuildContext context) {
    final hasFocus = _focus.hasFocus;
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        // Campo de texto con glow en foco.
        AnimatedContainer(
          duration: const Duration(milliseconds: 200),
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
          decoration: BoxDecoration(
            color: Gx.glassFill,
            borderRadius: BorderRadius.circular(Gx.rInput),
            border: Border.all(
              color: hasFocus ? Gx.transitionIndigo : Gx.borderPanel,
              width: hasFocus ? 1.5 : 1,
            ),
            boxShadow: hasFocus
                ? Gx.glow(Gx.transitionIndigo, blur: 18, opacity: 0.35)
                : null,
          ),
          child: TextField(
            controller: _ctrl,
            focusNode: _focus,
            style: Gx.body,
            cursorColor: Gx.transitionIndigo,
            decoration: const InputDecoration.collapsed(hintText: ''),
          ),
        ),
        // Lista de sugerencias animada que aparece al detectar "@".
        AnimatedSize(
          duration: const Duration(milliseconds: 180),
          curve: Curves.easeOut,
          child: _showSuggestions
              ? Container(
                  margin: const EdgeInsets.only(top: 4),
                  decoration: BoxDecoration(
                    color: Gx.glassFill,
                    borderRadius: BorderRadius.circular(Gx.rPanel),
                    border: Border.all(color: Gx.borderPanel),
                    boxShadow: Gx.glow(Gx.transitionIndigo,
                        blur: 12, opacity: 0.25),
                  ),
                  child: Column(
                    mainAxisSize: MainAxisSize.min,
                    children: _users
                        .map((u) => GestureDetector(
                              onTap: () => _insertMention(u),
                              child: Container(
                                width: double.infinity,
                                padding: const EdgeInsets.symmetric(
                                    horizontal: 12, vertical: 9),
                                child: Row(children: [
                                  Icon(Gx.iconAudit,
                                      size: 14,
                                      color: Gx.transitionIndigo),
                                  const SizedBox(width: 8),
                                  Text(u,
                                      style: Gx.uiSans(
                                          fontSize: 13,
                                          color: Gx.textSecondary)),
                                ]),
                              ),
                            ))
                        .toList(),
                  ),
                )
              : const SizedBox.shrink(),
        ),
      ],
    );
  }
}

// ===========================================================================
// §7 BOTONES — SPLIT BUTTON
// ===========================================================================

// Botón con una acción principal y un chevron que despliega opciones
// adicionales. El dropdown es vidrio Apple con glow.
class GlowSplitButton extends StatefulWidget {
  const GlowSplitButton({super.key});

  @override
  State<GlowSplitButton> createState() => _GlowSplitButtonState();
}

class _GlowSplitButtonState extends State<GlowSplitButton> {
  // Estado del dropdown secundario.
  bool _open = false;

  // Opciones adicionales del dropdown.
  static const _options = [
    'Ejecutar ahora',
    'Programar',
    'Ejecutar en dry-run',
  ];

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // Parte principal: botón con gradiente + separador + chevron.
        Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            // Botón de acción principal.
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
              decoration: BoxDecoration(
                gradient: Gx.linear(Gx.gradReactor),
                borderRadius: const BorderRadius.only(
                  topLeft: Radius.circular(Gx.rButton),
                  bottomLeft: Radius.circular(Gx.rButton),
                ),
                boxShadow: Gx.glow(Gx.reactorGreen, blur: 12, opacity: 0.5),
              ),
              child: Text(
                'EJECUTAR',
                style: Gx.uiSans(
                    fontSize: 13,
                    color: Gx.deepSpace,
                    weight: FontWeight.w500),
              ),
            ),
            // Separador vertical de 1px entre acción y chevron.
            Container(
                width: 1,
                height: 38,
                color: Gx.deepSpace.withOpacity(0.4)),
            // Botón chevron que abre el dropdown.
            GestureDetector(
              onTap: () => setState(() => _open = !_open),
              child: AnimatedContainer(
                duration: const Duration(milliseconds: 160),
                padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 10),
                decoration: BoxDecoration(
                  gradient: Gx.linear(Gx.gradReactor),
                  borderRadius: const BorderRadius.only(
                    topRight: Radius.circular(Gx.rButton),
                    bottomRight: Radius.circular(Gx.rButton),
                  ),
                  boxShadow: Gx.glow(Gx.reactorGreen, blur: 12, opacity: 0.5),
                ),
                child: AnimatedRotation(
                  duration: const Duration(milliseconds: 200),
                  turns: _open ? -0.5 : 0,
                  child: const Icon(Icons.keyboard_arrow_down,
                      size: 16, color: Gx.deepSpace),
                ),
              ),
            ),
          ],
        ),
        // Dropdown de opciones adicionales, animado.
        AnimatedSize(
          duration: const Duration(milliseconds: 200),
          curve: Curves.easeOut,
          child: _open
              ? Container(
                  margin: const EdgeInsets.only(top: 4),
                  decoration: BoxDecoration(
                    color: Gx.glassFill,
                    borderRadius: BorderRadius.circular(Gx.rPanel),
                    border: Border.all(color: Gx.borderPanel),
                    boxShadow:
                        Gx.glow(Gx.reactorGreen, blur: 10, opacity: 0.2),
                  ),
                  child: Column(
                    mainAxisSize: MainAxisSize.min,
                    children: _options
                        .map((opt) => GestureDetector(
                              onTap: () =>
                                  setState(() => _open = false),
                              child: Container(
                                width: double.infinity,
                                padding: const EdgeInsets.symmetric(
                                    horizontal: 14, vertical: 10),
                                child: Text(opt,
                                    style: Gx.uiSans(
                                        fontSize: 13,
                                        color: Gx.textSecondary)),
                              ),
                            ))
                        .toList(),
                  ),
                )
              : const SizedBox.shrink(),
        ),
      ],
    );
  }
}

// ===========================================================================
// §5 NAVEGACIÓN — BACK TO TOP
// ===========================================================================

// Botón flotante de "volver arriba" con vidrio Apple y glow.
// En la galería se muestra como cáscara estática (sin scroll real).
class GlowBackToTop extends StatelessWidget {
  const GlowBackToTop({super.key});

  @override
  Widget build(BuildContext context) {
    return Align(
      alignment: Alignment.bottomRight,
      child: Container(
        width: 42,
        height: 42,
        decoration: BoxDecoration(
          color: Gx.glassFill,
          shape: BoxShape.circle,
          border: Border.all(color: Gx.borderPanel),
          boxShadow: Gx.glow(Gx.transitionIndigo, blur: 14, opacity: 0.35),
        ),
        child: const Icon(
          Icons.keyboard_arrow_up,
          size: 20,
          color: Gx.textSecondary,
        ),
      ),
    );
  }
}
