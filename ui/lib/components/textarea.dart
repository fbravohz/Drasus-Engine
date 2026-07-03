// textarea.dart — Componente Textarea (ADR-0138 enmienda 2026-06-29).
// Campo de texto multilínea con glow de foco; equivale a un Input de N líneas.
// El estilo lo decide el tema global vía tokens Gx; prohibido hardcodear color.

import 'package:flutter/material.dart';
import '../gallery/gallery_tokens.dart';
import '../gallery/gallery_fx.dart';

// Campo de texto multilínea con foco-glow y soporte de controller externo.
// Contrato funcional:
//   [controller] controlador externo (null = modo no controlado, se crea uno interno).
//   [hint]       texto guía cuando el campo está vacío.
//   [onChanged]  callback al cambiar el contenido.
//   [maxLines]   número máximo de líneas visibles (por defecto 3).
//   [enabled]    activa/desactiva la interacción (por defecto true).
class Textarea extends StatefulWidget {
  final TextEditingController? controller;
  final String? hint;
  final ValueChanged<String>? onChanged;
  final int maxLines;
  final bool enabled;

  // No es const: lee getters dinámicos de Gx que cambian con el tema.
  Textarea({
    super.key,
    this.controller,
    this.hint,
    this.onChanged,
    this.maxLines = 3,
    this.enabled = true,
  });

  @override
  State<Textarea> createState() => _TextareaState();
}

class _TextareaState extends State<Textarea> {
  final FocusNode _focus = FocusNode();
  // Controller interno para modo no controlado.
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
  // Campo multilínea con superficie panelSurface y borde de énfasis al ganar foco.
  Widget build(BuildContext context) {
    final focused = _focus.hasFocus;

    return panelSurface(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
      radius: Gx.rInput,
      // Glow solo cuando hay foco; señal visual consistente con Input.
      glow: focused
          ? Gx.glow(Gx.accentDynamic, blur: 18, opacity: 0.45)
          : null,
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 200),
        decoration: BoxDecoration(
          borderRadius: BorderRadius.circular(Gx.rInput),
          // Borde de énfasis dinámico al ganar foco.
          border: focused
              ? Border.all(color: Gx.accentDynamic, width: Gx.borderFocus)
              : null,
        ),
        child: TextField(
          focusNode: _focus,
          controller: _ctrl,
          enabled: widget.enabled,
          maxLines: widget.maxLines,
          // minLines igual a maxLines para alto fijo; el texto hace scroll si supera.
          minLines: widget.maxLines,
          onChanged: widget.onChanged,
          style: Gx.uiSans(fontSize: 14, color: Gx.textBase),
          cursorColor: Gx.accentDynamic,
          decoration: InputDecoration.collapsed(
            hintText: widget.hint,
            hintStyle: Gx.uiSans(fontSize: 14, color: Gx.textBaseMuted),
          ),
        ),
      ),
    );
  }
}
