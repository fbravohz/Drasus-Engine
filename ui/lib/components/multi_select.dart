// multi_select.dart — Componente MultiSelect<T> (ADR-0138 enmienda 2026-06-29).
// Selección múltiple con chips para los ítems seleccionados.
// El estilo lo decide el tema global vía tokens Gx; prohibido hardcodear color.
//
// Nota: importa chip.dart directamente (mismo directorio) para reutilizar
// ui.Chip sin importar el barrel completo. Oculta el Chip de Material para
// evitar colisión de nombres.

import 'package:flutter/material.dart' hide Chip;
import '../gallery/gallery_tokens.dart';
import '../gallery/gallery_fx.dart';
import 'chip.dart'; // Chip y ChipStatus del sistema de diseño

// Ítem de un MultiSelect — asocia un valor tipado a una etiqueta visible.
class MultiSelectItem<T> {
  final T value;
  final String label;
  const MultiSelectItem({required this.value, required this.label});
}

// Selección múltiple con panel de chips.
// Contrato funcional:
//   [selected]  conjunto de valores seleccionados (null = vacío, modo no controlado).
//   [items]     lista de opciones disponibles.
//   [onChanged] callback con el nuevo Set<T> al cambiar la selección.
// En modo no controlado, el componente gestiona [_internalSelected] internamente.
// Los ítems seleccionados se muestran como ui.Chip de estado transition; los
// disponibles como chips neutros con borde estructural.
class MultiSelect<T> extends StatefulWidget {
  final Set<T>? selected;
  final List<MultiSelectItem<T>> items;
  final ValueChanged<Set<T>>? onChanged;

  // No es const: lee getters dinámicos de Gx que cambian con el tema.
  MultiSelect({
    super.key,
    this.selected,
    required this.items,
    this.onChanged,
  });

  @override
  State<MultiSelect<T>> createState() => _MultiSelectState<T>();
}

class _MultiSelectState<T> extends State<MultiSelect<T>> {
  // Estado interno para modo no controlado.
  late Set<T> _internalSelected = widget.selected ?? {};

  // Conjunto efectivo: el externo tiene prioridad sobre el interno.
  Set<T> get _effective => widget.selected ?? _internalSelected;

  // Añade o quita un ítem y notifica al padre.
  void _toggle(T value) {
    final next = Set<T>.of(_effective);
    if (next.contains(value)) {
      next.remove(value);
    } else {
      next.add(value);
    }
    if (widget.selected == null) setState(() => _internalSelected = next);
    widget.onChanged?.call(next);
  }

  @override
  // Panel con chips de seleccionados arriba y disponibles debajo de un separador.
  Widget build(BuildContext context) {
    final selectedItems =
        widget.items.where((i) => _effective.contains(i.value)).toList();
    final availableItems =
        widget.items.where((i) => !_effective.contains(i.value)).toList();

    return panelSurface(
      padding: const EdgeInsets.all(12),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          // Chips de los ítems seleccionados con botón de eliminación (×).
          if (selectedItems.isNotEmpty)
            Wrap(
              spacing: 6,
              runSpacing: 6,
              children: selectedItems.map((item) => GestureDetector(
                    onTap: () => _toggle(item.value),
                    child: Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        // ui.Chip con estado transition como señal visual de "activo".
                        Chip(
                          label: item.label,
                          status: ChipStatus.transition,
                        ),
                        // Icono × junto al chip; toca el Row completo para eliminar.
                        Padding(
                          padding: const EdgeInsets.only(left: 2),
                          child: Icon(
                            Icons.close,
                            size: 12,
                            color: Gx.transitionIndigo,
                          ),
                        ),
                      ],
                    ),
                  )).toList(),
            ),
          // Separador solo visible cuando hay seleccionados Y hay disponibles.
          if (selectedItems.isNotEmpty && availableItems.isNotEmpty) ...[
            const SizedBox(height: 8),
            Divider(color: Gx.borderBase, height: 1),
            const SizedBox(height: 8),
          ],
          // Ítems disponibles para añadir — chips neutros con borde estructural.
          if (availableItems.isNotEmpty)
            Wrap(
              spacing: 6,
              runSpacing: 6,
              children: availableItems
                  .map((item) => GestureDetector(
                        onTap: () => _toggle(item.value),
                        child: Container(
                          padding: const EdgeInsets.symmetric(
                              horizontal: 10, vertical: 4),
                          decoration: BoxDecoration(
                            color: Colors.transparent,
                            borderRadius: BorderRadius.circular(Gx.rChip),
                            border: Border.all(color: Gx.borderBase),
                          ),
                          child: Text(
                            item.label,
                            style: Gx.uiSans(
                                fontSize: 12, color: Gx.textBaseLabel),
                          ),
                        ),
                      ))
                  .toList(),
            ),
        ],
      ),
    );
  }
}
