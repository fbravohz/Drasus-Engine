// input.dart — Componente Input (ADR-0138 enmienda 2026-06-29).
// Campo de texto con foco, glow, error y estado deshabilitado.
// El estilo lo decide el tema global vía tokens Gx; prohibido hardcodear color.

import 'package:flutter/material.dart';
import '../gallery/gallery_tokens.dart';
import '../gallery/gallery_fx.dart';

// Campo de texto estilizado con foco glow y estado de error.
// Contrato funcional: [controller] controlador externo (null = modo no controlado,
// el componente crea y gestiona su propio controller); [hint] texto guía;
// [onChanged] callback al cambiar el valor; [errorText] mensaje de error visible
// debajo del campo; [enabled] activa/desactiva la interacción.
class Input extends StatefulWidget {
  final TextEditingController? controller;
  final String? hint;
  final ValueChanged<String>? onChanged;
  final String? errorText;
  final bool enabled;

  // No es const: lee getters dinámicos de Gx que cambian con el tema.
  Input({
    super.key,
    this.controller,
    this.hint,
    this.onChanged,
    this.errorText,
    this.enabled = true,
  });

  @override
  State<Input> createState() => _InputState();
}

class _InputState extends State<Input> {
  final FocusNode _focus = FocusNode();
  // Controller interno para modo no controlado (cuando el padre no pasa uno).
  TextEditingController? _internalCtrl;

  // Devuelve el controller efectivo: el externo si fue provisto, el interno si no.
  TextEditingController get _ctrl =>
      widget.controller ?? (_internalCtrl ??= TextEditingController());

  @override
  void initState() {
    super.initState();
    // Redibuja al ganar/perder foco para animar el glow de borde.
    _focus.addListener(() => setState(() {}));
  }

  @override
  void dispose() {
    _focus.dispose();
    // Solo libera el controller interno — el externo es responsabilidad del padre.
    _internalCtrl?.dispose();
    super.dispose();
  }

  @override
  // Campo con superficie panelSurface, glow de foco y mensaje de error opcional abajo.
  Widget build(BuildContext context) {
    final focused = _focus.hasFocus;
    final hasError = widget.errorText != null && widget.errorText!.isNotEmpty;

    // Borde: rojo en error, color de énfasis dinámico en foco, ninguno en reposo.
    final borderColor = hasError
        ? Gx.criticalCrimson
        : focused
            ? Gx.accentDynamic
            : null;
    final glowColor = hasError ? Gx.criticalCrimson : Gx.accentDynamic;

    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        panelSurface(
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 11),
          radius: Gx.rInput,
          // Glow solo cuando hay foco (o error): señal visual clara del estado activo.
          glow: (focused || hasError)
              ? Gx.glow(glowColor, blur: 18, opacity: 0.45)
              : null,
          child: AnimatedContainer(
            duration: const Duration(milliseconds: 200),
            decoration: BoxDecoration(
              borderRadius: BorderRadius.circular(Gx.rInput),
              // Borde animado: aparece al ganar foco o en estado de error.
              border: borderColor != null
                  ? Border.all(color: borderColor, width: Gx.borderFocus)
                  : null,
            ),
            child: TextField(
              focusNode: _focus,
              controller: _ctrl,
              enabled: widget.enabled,
              onChanged: widget.onChanged,
              cursorColor: Gx.accentDynamic,
              style: Gx.uiSans(fontSize: 14, color: Gx.textBase),
              decoration: InputDecoration.collapsed(
                hintText: widget.hint,
                hintStyle: Gx.uiSans(fontSize: 14, color: Gx.textBaseMuted),
              ),
            ),
          ),
        ),
        // Mensaje de error bajo el campo: solo visible cuando errorText no es nulo.
        if (hasError) ...[
          const SizedBox(height: 4),
          Text(
            widget.errorText!,
            style: Gx.uiSans(fontSize: 12, color: Gx.criticalCrimson),
          ),
        ],
      ],
    );
  }
}
