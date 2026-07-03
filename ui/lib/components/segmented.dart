// segmented.dart — Componente Segmented (ADR-0138 enmienda 2026-06-29).
// Control de selección única estilo pill con borde de énfasis dinámico.
// El estilo lo decide el tema global vía tokens Gx; prohibido hardcodear color.

import 'package:flutter/material.dart';
import '../gallery/gallery_tokens.dart';
import '../gallery/gallery_fx.dart';

// Control segmentado de selección única (estilo pill).
// Contrato funcional: [options] etiquetas de las opciones disponibles;
// [selected] índice seleccionado actualmente (null = modo no controlado, empieza en 0);
// [onChanged] callback con el nuevo índice al seleccionar.
class Segmented extends StatefulWidget {
  final List<String> options;
  final int? selected;
  final ValueChanged<int>? onChanged;

  // No es const: lee getters dinámicos de Gx que cambian con el tema.
  Segmented({
    super.key,
    required this.options,
    this.selected,
    this.onChanged,
  });

  @override
  State<Segmented> createState() => _SegmentedState();
}

class _SegmentedState extends State<Segmented> {
  // Estado interno para modo no controlado: empieza en el primer ítem.
  int _internalSel = 0;

  // Índice efectivo: el externo tiene prioridad sobre el interno.
  int get _effectiveSel => widget.selected ?? _internalSel;

  void _select(int index) {
    // En modo no controlado, actualiza el estado interno.
    if (widget.selected == null) setState(() => _internalSel = index);
    widget.onChanged?.call(index);
  }

  @override
  // Fila de pills con indicador de selección: borde de énfasis + fondo translúcido.
  // El contenedor exterior es una panelSurface con radio máximo (pill).
  Widget build(BuildContext context) {
    return panelSurface(
      padding: const EdgeInsets.all(4),
      radius: 999, // pill completo — radio literal permitido para pills
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: widget.options.asMap().entries.map((entry) {
          final isActive = entry.key == _effectiveSel;
          return GestureDetector(
            onTap: () => _select(entry.key),
            child: AnimatedContainer(
              duration: const Duration(milliseconds: 180),
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
              decoration: BoxDecoration(
                // Fondo translúcido en el ítem activo para diferenciarlo visualmente.
                color: isActive
                    ? Gx.accentDynamic.withAlpha(40)
                    : Colors.transparent,
                borderRadius: BorderRadius.circular(999),
                // Borde de énfasis solo en el ítem activo.
                border: isActive
                    ? Border.all(color: Gx.accentDynamic)
                    : null,
                boxShadow: isActive
                    ? Gx.glow(Gx.accentDynamic, blur: 8, opacity: 0.4)
                    : null,
              ),
              child: Text(
                entry.value,
                style: Gx.uiSans(
                  fontSize: 12,
                  // El ítem activo usa el color de énfasis; los demás usan label muted.
                  color: isActive ? Gx.accentDynamic : Gx.textBaseLabel,
                ),
              ),
            ),
          );
        }).toList(),
      ),
    );
  }
}
