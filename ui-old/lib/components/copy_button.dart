// copy_button.dart — Botón de copiado al portapapeles, reutilizable.
//
// Por qué existe (2026-08-01, reporte del propietario probando STORY-073):
// un mensaje de error apareció en pantalla y NO se pudo copiar — hubo que
// transcribirlo a mano para reportarlo. Ese es el peor momento posible para
// obligar a copiar a mano: el texto que el usuario más necesita mover fuera
// de la aplicación es justamente el que describe un fallo.
//
// La regla que fija este componente, y que vale para toda la interfaz:
//   - Etiquetas, títulos y textos de botón: NO seleccionables. Seleccionarlos
//     por accidente al arrastrar es ruido, y no aportan nada fuera de la app.
//   - Texto que el usuario podría necesitar FUERA de la aplicación —mensajes
//     de error, identificadores, huellas, rutas, valores calculados—: siempre
//     seleccionable, y con un botón de copiado a un clic cuando el texto es
//     largo o difícil de seleccionar con precisión.
//
// Se ofrece en dos formas, por eso vive en su propio archivo y no dentro de
// `banner.dart`: `CopyButton` (el icono suelto, para componer donde haga
// falta) y `CopyableText` (texto seleccionable + icono, ya emparejados).

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import '../theme/gx_tokens.dart';

// Cuánto dura la confirmación visual antes de volver al icono de copiar.
const Duration _kConfirmDuration = Duration(seconds: 2);

// Icono de copiado. Al pulsarlo copia [text] al portapapeles y muestra una
// marca de verificación durante dos segundos — confirmación visible, porque
// copiar al portapapeles no produce ningún efecto observable por sí mismo y
// sin acuse el usuario no sabe si funcionó.
//
// Contrato funcional:
//   [text]     contenido exacto que va al portapapeles.
//   [tooltip]  texto de ayuda al posar el cursor; default: "Copiar".
//   [size]     tamaño del icono en píxeles lógicos; default 16.
class CopyButton extends StatefulWidget {
  final String text;
  final String? tooltip;
  final double size;

  const CopyButton({
    super.key,
    required this.text,
    this.tooltip,
    this.size = 16,
  });

  @override
  State<CopyButton> createState() => _CopyButtonState();
}

class _CopyButtonState extends State<CopyButton> {
  bool _justCopied = false;

  Future<void> _copy() async {
    await Clipboard.setData(ClipboardData(text: widget.text));
    if (!mounted) return;
    setState(() => _justCopied = true);
    await Future.delayed(_kConfirmDuration);
    // El widget pudo desmontarse durante la espera (panel cerrado, paso
    // avanzado): sin esta guarda, `setState` sobre un State muerto lanza.
    if (!mounted) return;
    setState(() => _justCopied = false);
  }

  @override
  Widget build(BuildContext context) {
    final color = _justCopied ? Gx.optimaCyan : Gx.textBaseMuted;
    return Tooltip(
      message: _justCopied ? 'Copiado' : (widget.tooltip ?? 'Copiar'),
      child: InkWell(
        onTap: _copy,
        borderRadius: BorderRadius.circular(Gx.rChip),
        // Área de toque cómoda sin agrandar el icono: el padding es constante
        // en ambos estados para que el acuse no mueva el layout vecino
        // (misma regla que corrigió el parpadeo de `dropzone.dart`).
        child: Padding(
          padding: const EdgeInsets.all(4),
          child: Icon(
            _justCopied ? Gx.iconCheck : Gx.iconCopy,
            size: widget.size,
            color: color,
          ),
        ),
      ),
    );
  }
}

// Texto seleccionable con su botón de copiado al lado. Es la forma que se
// usa para mensajes de error, identificadores y huellas: el usuario puede
// seleccionar un fragmento con el ratón, o llevarse el texto entero de un
// clic.
//
// Contrato funcional:
//   [text]     contenido mostrado y copiado (el mismo, sin recortes).
//   [style]    estilo del texto; default: `Gx.bodySecondary`.
//   [tooltip]  ayuda del botón de copiado.
class CopyableText extends StatelessWidget {
  final String text;
  final TextStyle? style;
  final String? tooltip;

  const CopyableText({
    super.key,
    required this.text,
    this.style,
    this.tooltip,
  });

  @override
  Widget build(BuildContext context) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Expanded(
          child: SelectableText(text, style: style ?? Gx.bodySecondary),
        ),
        const SizedBox(width: 6),
        CopyButton(text: text, tooltip: tooltip),
      ],
    );
  }
}
