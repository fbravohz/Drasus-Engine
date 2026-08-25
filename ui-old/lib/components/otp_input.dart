// otp_input.dart — Componente OtpInput (ADR-0138 enmienda 2026-06-29).
// Entrada de código OTP/PIN: N cajas de un dígito con avance automático de foco.
// El estilo lo decide el tema global vía tokens Gx; prohibido hardcodear color.

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import '../theme/gx_tokens.dart';

// Campo OTP/PIN de longitud configurable.
// Contrato funcional:
//   [length]      número de cajas/dígitos (por defecto 6).
//   [onCompleted] callback cuando todos los campos están rellenos (string completo).
//   [onChanged]   callback en cada pulsación (string parcial con la longitud actual).
// Comportamiento de navegación:
//   • Al escribir un carácter, el foco avanza automáticamente a la caja siguiente.
//   • Borrar un campo vacío retrocede el foco a la caja anterior.
//   • Al rellenar el último campo, se llama [onCompleted].
class OtpInput extends StatefulWidget {
  final int length;
  final ValueChanged<String>? onCompleted;
  final ValueChanged<String>? onChanged;

  const OtpInput({
    super.key,
    this.length = 6,
    this.onCompleted,
    this.onChanged,
  });

  @override
  State<OtpInput> createState() => _OtpInputState();
}

class _OtpInputState extends State<OtpInput> {
  late final List<TextEditingController> _controllers;
  late final List<FocusNode> _nodes;

  @override
  void initState() {
    super.initState();
    // Crea un controller y un FocusNode por cada caja.
    _controllers = List.generate(widget.length, (_) => TextEditingController());
    _nodes = List.generate(widget.length, (_) => FocusNode());
    // Redibuja al cambiar el foco para actualizar glow/borde de la caja activa.
    for (final node in _nodes) {
      node.addListener(() => setState(() {}));
    }
  }

  @override
  void dispose() {
    for (final c in _controllers) c.dispose();
    for (final n in _nodes) n.dispose();
    super.dispose();
  }

  // Notifica al padre el valor actual (parcial o completo).
  void _notifyChange() {
    final current = _controllers.map((c) => c.text).join();
    widget.onChanged?.call(current);
    // Llama onCompleted solo cuando todas las cajas tienen exactamente un carácter.
    if (_controllers.every((c) => c.text.length == 1)) {
      widget.onCompleted?.call(current);
    }
  }

  // Construye una caja individual del OTP.
  Widget _box(int i) {
    final isFocused = _nodes[i].hasFocus;
    final hasValue = _controllers[i].text.isNotEmpty;

    return AnimatedContainer(
      duration: const Duration(milliseconds: 180),
      width: 36,
      height: 46,
      alignment: Alignment.center,
      decoration: BoxDecoration(
        // Fondo solo en la caja activa — reposo transparente.
        color: isFocused ? Gx.surfaceFill : Colors.transparent,
        borderRadius: BorderRadius.circular(Gx.rChip),
        border: Border.all(
          color: isFocused ? Gx.accentDynamic : Gx.borderBase,
          width: isFocused ? Gx.borderFocus : Gx.borderHairline,
        ),
        // Glow de énfasis dinámico solo en la caja con foco.
        boxShadow: isFocused
            ? Gx.glow(Gx.accentDynamic, blur: 14, opacity: 0.45)
            : null,
      ),
      child: Stack(alignment: Alignment.center, children: [
        // TextField invisible que captura el input; visualmente solo se muestra el texto.
        SizedBox(
          width: 36,
          height: 46,
          child: TextField(
            controller: _controllers[i],
            focusNode: _nodes[i],
            textAlign: TextAlign.center,
            maxLength: 1,
            // Oculta el contador de caracteres del TextField de Material.
            decoration: const InputDecoration(
              counterText: '',
              border: InputBorder.none,
              contentPadding: EdgeInsets.zero,
            ),
            // Acepta cualquier carácter (alfanumérico o numérico según el uso).
            inputFormatters: [LengthLimitingTextInputFormatter(1)],
            style: Gx.dataMono(fontSize: 17, color: Gx.textBase),
            cursorColor: Gx.accentDynamic,
            onChanged: (val) {
              if (val.isNotEmpty && i < widget.length - 1) {
                // Avanza al siguiente campo al escribir un carácter.
                FocusScope.of(context).requestFocus(_nodes[i + 1]);
              } else if (val.isEmpty && i > 0) {
                // Retrocede al campo anterior al borrar con backspace.
                FocusScope.of(context).requestFocus(_nodes[i - 1]);
              }
              _notifyChange();
            },
          ),
        ),
        // Cursor visual propio cuando el campo está vacío y tiene foco.
        if (isFocused && !hasValue)
          Positioned(
            child: Container(
              width: 1.5,
              height: 18,
              color: Gx.accentDynamic,
            ),
          ),
      ]),
    );
  }

  @override
  // Fila de N cajas OTP con espaciado uniforme.
  Widget build(BuildContext context) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: List.generate(widget.length, (i) {
        return Padding(
          padding: EdgeInsets.only(right: i < widget.length - 1 ? 6 : 0),
          child: _box(i),
        );
      }),
    );
  }
}
