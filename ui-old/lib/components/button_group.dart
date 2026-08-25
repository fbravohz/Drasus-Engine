// button_group.dart — Componente ButtonGroup (ADR-0138 enmienda 2026-06-29).
// Fila de botones unidos con selección única; el activo lleva gradiente + glow.
// El estilo lo decide el tema global vía tokens Gx; prohibido hardcodear color.

import 'package:flutter/material.dart';
import '../theme/gx_tokens.dart';

// Fila de botones agrupados horizontalmente. Solo uno puede estar activo.
// El botón seleccionado lleva gradiente de transición + glow; los demás usan
// la superficie de relleno. Los extremos del grupo tienen radio redondeado;
// los interiores comparten borde plano.
//
// Contrato funcional:
//   [items]    lista de etiquetas de los botones (mínimo 1).
//   [selected] índice del activo (null = modo no controlado; arranca en 0).
//   [onChanged] callback con el índice del botón pulsado.
class ButtonGroup extends StatefulWidget {
  final List<String> items;
  final int? selected;
  final ValueChanged<int>? onChanged;

  // No es const: los getters dinámicos de Gx cambian con el tema.
  ButtonGroup({
    super.key,
    required this.items,
    this.selected,
    this.onChanged,
  });

  @override
  State<ButtonGroup> createState() => _ButtonGroupState();
}

class _ButtonGroupState extends State<ButtonGroup> {
  // Índice activo interno para modo no controlado (arranca en 0).
  int _internalSelected = 0;

  // Índice efectivo: el externo tiene prioridad sobre el interno.
  int get _active => widget.selected ?? _internalSelected;

  // Selecciona un botón; en modo no controlado actualiza el estado interno.
  void _select(int index) {
    if (widget.selected == null) setState(() => _internalSelected = index);
    widget.onChanged?.call(index);
  }

  @override
  // Fila de botones animados; el radio se aplica solo a las esquinas exteriores del grupo.
  Widget build(BuildContext context) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: widget.items.asMap().entries.map((e) {
        final isActive = e.key == _active;
        final isFirst = e.key == 0;
        final isLast = e.key == widget.items.length - 1;
        return GestureDetector(
          onTap: () => _select(e.key),
          child: AnimatedContainer(
            duration: const Duration(milliseconds: 180),
            padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 9),
            decoration: BoxDecoration(
              gradient: isActive ? Gx.linear(Gx.gradTransition) : null,
              color: isActive ? null : Gx.surfaceFill,
              // Solo las esquinas externas del grupo llevan radio; las interiores son planas.
              borderRadius: BorderRadius.only(
                topLeft: Radius.circular(isFirst ? Gx.rButton : 0),
                bottomLeft: Radius.circular(isFirst ? Gx.rButton : 0),
                topRight: Radius.circular(isLast ? Gx.rButton : 0),
                bottomRight: Radius.circular(isLast ? Gx.rButton : 0),
              ),
              border: Border.all(color: Gx.borderBase),
              boxShadow: isActive
                  ? Gx.glow(Gx.transitionIndigo, blur: 12, opacity: 0.4)
                  : null,
            ),
            child: Text(
              e.value,
              style: Gx.dataMono(
                fontSize: 12,
                // Texto: blanco sobre gradiente (activo); token de etiqueta (inactivo).
                color: isActive ? Gx.pureWhite : Gx.textBaseLabel,
              ),
            ),
          ),
        );
      }).toList(),
    );
  }
}
