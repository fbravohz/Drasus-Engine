// toggle_button.dart — Componente ToggleButton (ADR-0138 enmienda 2026-06-29).
// Botón conmutable ON/OFF con gradiente semántico (transición) y glow en estado activo.
// El estilo lo decide el tema global vía tokens Gx; prohibido hardcodear color.

import 'package:flutter/material.dart';
import '../gallery/gallery_tokens.dart';

// Botón que alterna entre dos estados (ON/OFF). El activo lleva gradiente de
// transición (índigo) con glow; el inactivo usa la superficie de relleno del tema.
//
// Contrato funcional:
//   [value]    estado actual (null = modo no controlado; arranca en [initial]).
//   [onChanged] callback con el nuevo bool al pulsar.
//   [label]    texto visible cuando ON.
//   [labelOff] texto visible cuando OFF.
//   [icon]     ícono opcional mostrado junto al label.
//   [initial]  valor inicial en modo no controlado (por defecto false).
class ToggleButton extends StatefulWidget {
  final bool? value;
  final ValueChanged<bool>? onChanged;
  final String label;
  final String labelOff;
  final IconData? icon;
  final bool initial;

  // No es const: los getters dinámicos de Gx cambian con el tema.
  ToggleButton({
    super.key,
    this.value,
    this.onChanged,
    this.label = 'ON',
    this.labelOff = 'OFF',
    this.icon,
    this.initial = false,
  });

  @override
  State<ToggleButton> createState() => _ToggleButtonState();
}

class _ToggleButtonState extends State<ToggleButton> {
  // Estado interno para modo no controlado; en modo controlado se ignora.
  late bool _internalOn = widget.initial;

  // Valor efectivo: el externo (value) tiene prioridad sobre el interno.
  bool get _on => widget.value ?? _internalOn;

  // Alterna el estado; en modo no controlado también actualiza el estado interno.
  void _toggle() {
    final next = !_on;
    if (widget.value == null) setState(() => _internalOn = next);
    widget.onChanged?.call(next);
  }

  @override
  // Botón animado: ON = gradiente transitionIndigo + glow; OFF = surfaceFill + borde neutro.
  // El ícono opcional se muestra antes del label.
  Widget build(BuildContext context) {
    final on = _on;
    return GestureDetector(
      onTap: _toggle,
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 220),
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
        decoration: BoxDecoration(
          gradient: on ? Gx.linear(Gx.gradTransition) : null,
          color: on ? null : Gx.surfaceFill,
          borderRadius: BorderRadius.circular(Gx.rButton),
          border: Border.all(
            // Borde semántico (activo) vs estructural global (inactivo).
            color: on ? Gx.transitionIndigo : Gx.borderBase,
          ),
          boxShadow: on
              ? Gx.glow(Gx.transitionIndigo, blur: 16, opacity: 0.5)
              : null,
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            if (widget.icon != null) ...[
              // Ícono: blanco sobre gradiente (activo); token de etiqueta (inactivo).
              Icon(widget.icon, size: 14,
                  color: on ? Gx.pureWhite : Gx.textBaseLabel),
              const SizedBox(width: 6),
            ],
            Text(
              on ? widget.label : widget.labelOff,
              style: Gx.uiSans(
                fontSize: 13,
                // Texto: blanco sobre gradiente (activo); token dinámico (inactivo).
                color: on ? Gx.pureWhite : Gx.textBaseLabel,
                weight: FontWeight.w500,
              ),
            ),
          ],
        ),
      ),
    );
  }
}
