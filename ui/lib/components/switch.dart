// switch.dart — Componente Switch (ADR-0138 enmienda 2026-06-29).
// Palanca ON/OFF con knob deslizante animado y glow en estado activo.
// El estilo lo decide el tema global vía tokens Gx; prohibido hardcodear color.
//
// Nota: el nombre Switch colisiona con el widget Material del mismo nombre.
// Los consumidores importan con namespace `import ... as ui;` → `ui.Switch`.

import 'package:flutter/material.dart' hide Switch;
import '../gallery/gallery_tokens.dart';

// Palanca de activación con knob animado y glow en ON.
// Contrato funcional: [value] estado actual (null = modo no controlado, empieza en false);
// [onChanged] callback con el nuevo estado al cambiar. El color del glow y del knob
// siguen el énfasis dinámico del tema global — no hay parámetro de color.
class Switch extends StatefulWidget {
  final bool? value;
  final ValueChanged<bool>? onChanged;

  // No es const: lee getters dinámicos de Gx que cambian con el tema.
  Switch({super.key, this.value, this.onChanged});

  @override
  State<Switch> createState() => _SwitchState();
}

class _SwitchState extends State<Switch> {
  // Estado interno para modo no controlado: empieza en OFF.
  bool _internalValue = false;

  // Estado efectivo: el externo tiene prioridad sobre el interno.
  bool get _on => widget.value ?? _internalValue;

  void _toggle() {
    final next = !_on;
    // En modo no controlado, actualiza el estado interno.
    if (widget.value == null) setState(() => _internalValue = next);
    widget.onChanged?.call(next);
  }

  @override
  // Palanca con pista animada (gradiente en ON, sólida en OFF) y knob deslizante.
  // Los colores reaccionan al énfasis dinámico del tema global.
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: _toggle,
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 220),
        curve: Curves.easeOut,
        width: 48,
        height: 26,
        padding: const EdgeInsets.all(3),
        decoration: BoxDecoration(
          // ON: gradiente del énfasis al 40%/15% — efecto cristal encendido.
          gradient: _on
              ? LinearGradient(colors: [
                  Gx.accentDynamic.withOpacity(0.4),
                  Gx.accentDynamic.withOpacity(0.15),
                ])
              : null,
          // OFF: color sólido de la pista — visualmente "apagado".
          color: _on ? null : Gx.gaugeTrack,
          borderRadius: BorderRadius.circular(999),
          // ON: borde del énfasis; OFF: borde estructural global.
          border: Border.all(color: _on ? Gx.accentDynamic : Gx.borderBase),
          // Glow solo en ON para reforzar el estado activo.
          boxShadow:
              _on ? Gx.glow(Gx.accentDynamic, blur: 16, opacity: 0.5) : null,
        ),
        child: AnimatedAlign(
          duration: const Duration(milliseconds: 220),
          curve: Curves.easeOut,
          // El knob se desplaza de izquierda (OFF) a derecha (ON).
          alignment: _on ? Alignment.centerRight : Alignment.centerLeft,
          child: Container(
            width: 18,
            height: 18,
            decoration: BoxDecoration(
              shape: BoxShape.circle,
              // ON: knob con el color de énfasis dinámico; OFF: color muted.
              color: _on ? Gx.accentDynamic : Gx.textBaseMuted,
              boxShadow: _on
                  ? Gx.glow(Gx.accentDynamic, blur: 12, opacity: 0.8)
                  : null,
            ),
          ),
        ),
      ),
    );
  }
}
