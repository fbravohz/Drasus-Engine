// transfer_list.dart — Componente TransferList (ADR-0138 enmienda 2026-06-29).
// Lista de transferencia dual: mueve ítems entre "Disponibles" y "Seleccionados"
// con checkboxes de selección individual y botones de flecha de transferencia.
// Estilo 100% por tema; prohibido hardcodear color.

import 'package:flutter/material.dart';
import '../theme/gx_tokens.dart';
import '../theme/surfaces.dart';

// Lista de transferencia dual (disponibles ↔ seleccionados).
// Modo no controlado: [available] y [selected] son listas iniciales que el
// widget gestiona internamente. [onChanged] se llama al mover ítems.
class TransferList extends StatefulWidget {
  // available: lista inicial de ítems en el panel izquierdo (disponibles).
  final List<String> available;
  // selected: lista inicial de ítems en el panel derecho (seleccionados).
  final List<String> selected;
  // onChanged: se llama con las dos listas actualizadas tras cada transferencia.
  final void Function(List<String> available, List<String> selected)? onChanged;

  const TransferList({
    super.key,
    required this.available,
    required this.selected,
    this.onChanged,
  });

  @override
  State<TransferList> createState() => _TransferListState();
}

class _TransferListState extends State<TransferList> {
  // Listas mutables de ítems en cada panel.
  late List<String> _left;
  late List<String> _right;
  // Conjuntos de ítems marcados con checkbox en cada panel.
  final Set<String> _checkedLeft = {};
  final Set<String> _checkedRight = {};

  @override
  void initState() {
    super.initState();
    // Copia defensiva para no mutar las listas del padre.
    _left = List.of(widget.available);
    _right = List.of(widget.selected);
  }

  // Mueve los ítems marcados en la lista izquierda a la derecha.
  void _moveRight() {
    setState(() {
      _right.addAll(_checkedLeft);
      _left.removeWhere(_checkedLeft.contains);
      _checkedLeft.clear();
    });
    widget.onChanged?.call(List.of(_left), List.of(_right));
  }

  // Mueve los ítems marcados en la lista derecha a la izquierda.
  void _moveLeft() {
    setState(() {
      _left.addAll(_checkedRight);
      _right.removeWhere(_checkedRight.contains);
      _checkedRight.clear();
    });
    widget.onChanged?.call(List.of(_left), List.of(_right));
  }

  // Construye un panel de lista con cabecera, ítems y checkboxes.
  Widget _list(List<String> items, Set<String> checked, String header) {
    return Expanded(
      child: PanelFromDecoration(
        decoration: BoxDecoration(
          color: Gx.surfacePanel,
          borderRadius: BorderRadius.circular(Gx.rPanel),
          border: Border.all(color: Gx.borderBase),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // Cabecera del panel con borde inferior.
            Container(
              padding: const EdgeInsets.symmetric(
                horizontal: Gx.space8 + 2,
                vertical: Gx.space4 + 2,
              ),
              decoration: BoxDecoration(
                border: Border(bottom: BorderSide(color: Gx.borderBase)),
              ),
              child: Text(
                header,
                style: Gx.microLabel.copyWith(color: Gx.textBaseLabel),
              ),
            ),
            // Ítems con checkbox de selección individual.
            ...items.map((item) {
              final isChecked = checked.contains(item);
              return GestureDetector(
                onTap: () => setState(() =>
                    isChecked ? checked.remove(item) : checked.add(item)),
                child: AnimatedContainer(
                  duration: const Duration(milliseconds: 140),
                  padding: const EdgeInsets.symmetric(
                      horizontal: Gx.space8 + 2, vertical: 7),
                  // Fondo tenue del énfasis en el ítem seleccionado.
                  color: isChecked
                      ? Gx.accentDynamic.withAlpha(26)
                      : Colors.transparent,
                  child: Row(children: [
                    // Checkbox visual: cuadrado 14px con estado checked/unchecked.
                    Container(
                      width: 14,
                      height: 14,
                      decoration: BoxDecoration(
                        color: isChecked ? Gx.accentDynamic : Colors.transparent,
                        // Radio decorativo intencional para checkbox pequeño (14px).
                        borderRadius: BorderRadius.circular(3),
                        border: Border.all(
                          color: isChecked ? Gx.accentDynamic : Gx.textBaseMuted,
                        ),
                        boxShadow: isChecked
                            ? Gx.glow(Gx.accentDynamic, blur: 6, opacity: 0.5)
                            : null,
                      ),
                    ),
                    const SizedBox(width: Gx.space8),
                    // Texto del ítem: base si está marcado, secundario si no.
                    Text(
                      item,
                      style: Gx.dataMono(
                        fontSize: 13,
                        color: isChecked ? Gx.textBase : Gx.textBaseSecondary,
                      ),
                    ),
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
  // Dos paneles de lista con botones de transferencia en el centro.
  Widget build(BuildContext context) {
    return SizedBox(
      height: 200,
      child: Row(
        children: [
          _list(_left, _checkedLeft, 'Disponibles'),
          // Columna central con dos botones de dirección.
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 8),
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                // Botón mover a la derecha (añadir a seleccionados).
                _transferBtn(Icons.chevron_right, Gx.optimaCyan, _moveRight),
                const SizedBox(height: 6),
                // Botón mover a la izquierda (quitar de seleccionados).
                _transferBtn(Icons.chevron_left, Gx.criticalRed, _moveLeft),
              ],
            ),
          ),
          _list(_right, _checkedRight, 'Seleccionados'),
        ],
      ),
    );
  }

  // Botón de flecha de transferencia con glow del color de estado semántico.
  // color: color semántico (éxito para añadir, crítico para quitar).
  Widget _transferBtn(IconData icon, Color color, VoidCallback onTap) {
    return GestureDetector(
      onTap: onTap,
      child: Container(
        padding: const EdgeInsets.all(6),
        decoration: BoxDecoration(
          color: color.withAlpha(31),
          borderRadius: BorderRadius.circular(Gx.rChip),
          border: Border.all(color: color.withAlpha(128)),
          boxShadow: Gx.glow(color, blur: 8, opacity: 0.3),
        ),
        child: Icon(icon, size: 16, color: color),
      ),
    );
  }
}
