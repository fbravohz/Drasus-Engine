// mention_input.dart — Componente MentionInput (ADR-0138 enmienda 2026-06-29).
// Campo de texto con detección de @menciones y dropdown de sugerencias.
// El estilo lo decide el tema global vía tokens Gx; prohibido hardcodear color.

import 'package:flutter/material.dart';
import '../gallery/gallery_tokens.dart';
import '../gallery/gallery_fx.dart';

// Campo de texto con soporte de @menciones.
// Contrato funcional:
//   [controller]  controller externo (null = modo no controlado, se crea uno interno).
//   [suggestions] lista de usuarios/entidades mencionables (p.ej. '@quant-01').
//   [onChanged]   callback al cambiar el contenido.
//   [hint]        texto guía cuando el campo está vacío.
// Comportamiento de menciones:
//   • Al detectar "@" seguido de texto sin espacio, muestra el dropdown de sugerencias.
//   • Al seleccionar una sugerencia, la inserta en lugar del fragmento "@<parcial>".
//   • El dropdown se cierra al seleccionar o al dejar de escribir "@".
class MentionInput extends StatefulWidget {
  final TextEditingController? controller;
  final List<String> suggestions;
  final ValueChanged<String>? onChanged;
  final String? hint;

  // No es const: lee getters dinámicos de Gx que cambian con el tema.
  MentionInput({
    super.key,
    this.controller,
    required this.suggestions,
    this.onChanged,
    this.hint,
  });

  @override
  State<MentionInput> createState() => _MentionInputState();
}

class _MentionInputState extends State<MentionInput> {
  final FocusNode _focus = FocusNode();
  // Controller interno para modo no controlado.
  TextEditingController? _internalCtrl;
  bool _showSuggestions = false;

  // Devuelve el controller efectivo: el externo si fue provisto, el interno si no.
  TextEditingController get _ctrl =>
      widget.controller ?? (_internalCtrl ??= TextEditingController());

  @override
  void initState() {
    super.initState();
    // Detecta cambios en el texto para mostrar/ocultar el dropdown.
    _ctrl.addListener(_onTextChange);
    _focus.addListener(() => setState(() {}));
  }

  // Determina si el cursor está dentro de un fragmento "@<parcial>" sin espacio.
  // Muestra sugerencias solo en ese caso — el dropdown se oculta al completar o borrar "@".
  void _onTextChange() {
    final text = _ctrl.text;
    // Busca si el texto termina en "@" o "@<texto-sin-espacio>".
    final lastAt = text.lastIndexOf('@');
    if (lastAt == -1) {
      if (_showSuggestions) setState(() => _showSuggestions = false);
      return;
    }
    final afterAt = text.substring(lastAt + 1);
    final isMentioning = !afterAt.contains(' ');
    if (isMentioning != _showSuggestions) {
      setState(() => _showSuggestions = isMentioning);
    }
    widget.onChanged?.call(text);
  }

  // Inserta la mención seleccionada en el texto, reemplazando el fragmento "@<parcial>".
  void _insertMention(String suggestion) {
    final text = _ctrl.text;
    final lastAt = text.lastIndexOf('@');
    if (lastAt == -1) return;
    // Reemplaza desde el "@" hasta el cursor con la sugerencia completa + espacio.
    final newText = '${text.substring(0, lastAt)}$suggestion ';
    _ctrl.text = newText;
    // Mueve el cursor al final del texto insertado.
    _ctrl.selection = TextSelection.collapsed(offset: newText.length);
    setState(() => _showSuggestions = false);
    widget.onChanged?.call(newText);
  }

  @override
  void dispose() {
    _ctrl.removeListener(_onTextChange);
    _focus.dispose();
    // Solo libera el controller interno — el externo es responsabilidad del padre.
    _internalCtrl?.dispose();
    super.dispose();
  }

  @override
  // Campo de texto con glow de foco + panel de sugerencias animado bajo él.
  Widget build(BuildContext context) {
    final hasFocus = _focus.hasFocus;
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        // Campo de texto: misma estética que Input con glow de énfasis dinámico.
        panelSurface(
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
          radius: Gx.rInput,
          glow: hasFocus
              ? Gx.glow(Gx.accentDynamic, blur: 18, opacity: 0.40)
              : null,
          child: AnimatedContainer(
            duration: const Duration(milliseconds: 200),
            decoration: BoxDecoration(
              borderRadius: BorderRadius.circular(Gx.rInput),
              border: hasFocus
                  ? Border.all(
                      color: Gx.accentDynamic, width: Gx.borderFocus)
                  : null,
            ),
            child: TextField(
              controller: _ctrl,
              focusNode: _focus,
              style: Gx.uiSans(fontSize: 14, color: Gx.textBase),
              cursorColor: Gx.accentDynamic,
              decoration: InputDecoration.collapsed(
                hintText: widget.hint,
                hintStyle:
                    Gx.uiSans(fontSize: 14, color: Gx.textBaseMuted),
              ),
            ),
          ),
        ),
        // Panel de sugerencias: aparece bajo el campo al detectar "@".
        AnimatedSize(
          duration: const Duration(milliseconds: 180),
          curve: Curves.easeOut,
          child: _showSuggestions
              ? Container(
                  margin: const EdgeInsets.only(top: 4),
                  decoration: BoxDecoration(
                    color: Gx.surfaceFill,
                    borderRadius: BorderRadius.circular(Gx.rPanel),
                    // Borde estructural global (énfasis dinámico).
                    border: Border.all(color: Gx.borderBase),
                    boxShadow:
                        Gx.glow(Gx.accentDynamic, blur: 12, opacity: 0.25),
                  ),
                  child: Column(
                    mainAxisSize: MainAxisSize.min,
                    children: widget.suggestions
                        .map((suggestion) => InkWell(
                              onTap: () => _insertMention(suggestion),
                              child: Container(
                                width: double.infinity,
                                padding: const EdgeInsets.symmetric(
                                    horizontal: 12, vertical: 9),
                                child: Row(children: [
                                  // Ícono de persona con el color de énfasis dinámico.
                                  Icon(Icons.alternate_email,
                                      size: 14, color: Gx.accentDynamic),
                                  const SizedBox(width: 8),
                                  // Nombre de usuario con token dinámico secundario.
                                  Text(
                                    suggestion,
                                    style: Gx.uiSans(
                                        fontSize: 13,
                                        color: Gx.textBaseSecondary),
                                  ),
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
