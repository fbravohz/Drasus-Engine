// dialog.dart — Componente Dialog (ADR-0138 enmienda 2026-06-29).
// Diálogo modal con superficie panelSurface, título, contenido y acciones.
// Incluye el helper showAppDialog() para mostrar el dialog como overlay centrado.
//
// Nota: el nombre Dialog colisiona con el widget Material del mismo nombre.
// Los consumidores importan con namespace `import ... as ui;` → `ui.Dialog`.
// Dentro de este archivo se oculta Dialog de Material con `hide Dialog`.

// ignore: undefined_hidden_name — Dialog se oculta para evitar colisión de nombres.
import 'package:flutter/material.dart' hide Dialog;
import '../theme/gx_tokens.dart';
import '../theme/surfaces.dart';

// Contenedor estilizado de diálogo modal.
// Contrato funcional:
//   [title]   texto del encabezado del diálogo.
//   [content] widget del cuerpo (texto, formulario, etc.).
//   [actions] lista de botones de acción; aparecen alineados a la derecha.
//   [maxWidth] ancho máximo en píxeles lógicos; default: 480.
class Dialog extends StatelessWidget {
  final String title;
  final Widget content;
  final List<Widget>? actions;
  final double maxWidth;

  // No es const: el build lee getters dinámicos de Gx (textBase, accentDynamic…).
  Dialog({
    super.key,
    required this.title,
    required this.content,
    this.actions,
    this.maxWidth = 480.0,
  });

  @override
  // Muestra el contenido del diálogo en una tarjeta panelSurface centrada.
  // showAppDialog() envuelve este widget en el overlay de showDialog.
  Widget build(BuildContext context) {
    return ConstrainedBox(
      constraints: BoxConstraints(maxWidth: maxWidth),
      child: panelSurface(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // Título del diálogo: énfasis dinámico para el encabezado del panel.
            Text(
              title,
              style: Gx.panelTitle.copyWith(color: Gx.textBase),
            ),
            const SizedBox(height: Gx.space12),
            // Cuerpo del diálogo: cualquier widget (texto, formulario, etc.).
            content,
            // Fila de acciones solo si se proporcionaron botones.
            if (actions != null && actions!.isNotEmpty) ...[
              const SizedBox(height: Gx.space16),
              Row(
                mainAxisAlignment: MainAxisAlignment.end,
                // Espacio horizontal entre los botones de acción.
                children: actions!
                    .expand((a) => [a, const SizedBox(width: Gx.space8)])
                    .take(actions!.length * 2 - 1)
                    .toList(),
              ),
            ],
          ],
        ),
      ),
    );
  }
}

// Muestra el Dialog como overlay modal centrado en la pantalla.
// Devuelve el valor que el widget de dialog pase a Navigator.pop<T>().
// [title]    encabezado del diálogo.
// [content]  cuerpo del diálogo.
// [actions]  lista de botones de acción (usualmente Cancelar + Confirmar).
Future<T?> showAppDialog<T>(
  BuildContext context, {
  required String title,
  required Widget content,
  List<Widget>? actions,
}) {
  return showDialog<T>(
    context: context,
    // Scrim semitransparente: oscurece el contenido detrás del diálogo.
    barrierColor: Colors.black.withOpacity(0.5),
    builder: (_) => Dialog(
      title: title,
      content: content,
      actions: actions,
    ),
  );
}
