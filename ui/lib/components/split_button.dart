// split_button.dart — Componente SplitButton (ADR-0138 enmienda 2026-06-29).
// Botón con acción principal y chevron que despliega opciones secundarias animadas.
// El estilo lo decide el tema global vía tokens Gx; prohibido hardcodear color.

import 'package:flutter/material.dart';
import '../gallery/gallery_tokens.dart';
import '../gallery/gallery_fx.dart';

// Botón dividido en dos partes:
//   - Izquierda: botón de acción principal (gradiente reactor + glow).
//   - Derecha: botón chevron que abre un panel de opciones adicionales.
// El panel de opciones usa panelSurface() — reacciona al modo del tema.
//
// Contrato funcional:
//   [label]            texto de la acción principal.
//   [onPressed]        callback de la acción principal (null = sin acción principal).
//   [actions]          lista de etiquetas de las opciones secundarias.
//   [onActionSelected] callback con la etiqueta de la opción elegida.
class SplitButton extends StatefulWidget {
  final String label;
  final VoidCallback? onPressed;
  final List<String> actions;
  final ValueChanged<String>? onActionSelected;

  // No es const: los getters dinámicos de Gx cambian con el tema.
  SplitButton({
    super.key,
    this.label = 'EJECUTAR',
    this.onPressed,
    this.actions = const [],
    this.onActionSelected,
  });

  @override
  State<SplitButton> createState() => _SplitButtonState();
}

class _SplitButtonState extends State<SplitButton> {
  // Controla la visibilidad del panel de opciones secundarias.
  bool _open = false;

  @override
  // Renderiza el par botón-principal + chevron, y el panel de opciones animado.
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // Fila: acción principal + separador vertical + chevron.
        Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            // Botón de acción principal: gradiente reactor (verde-cian) + glow.
            GestureDetector(
              onTap: widget.onPressed,
              child: Container(
                padding:
                    const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
                decoration: BoxDecoration(
                  gradient: Gx.linear(Gx.gradReactor),
                  borderRadius: BorderRadius.only(
                    topLeft: Radius.circular(Gx.rButton),
                    bottomLeft: Radius.circular(Gx.rButton),
                  ),
                  boxShadow: Gx.glow(Gx.reactorGreen, blur: 12, opacity: 0.5),
                ),
                child: Text(
                  widget.label,
                  style: Gx.uiSans(
                    fontSize: 13,
                    // canvasBase (oscuro) sobre gradiente cian-verde: legibilidad garantizada.
                    color: Gx.canvasBase,
                    weight: FontWeight.w500,
                  ),
                ),
              ),
            ),
            // Separador de 1px entre la acción y el chevron (canvasBase con alpha).
            Container(
              width: 1,
              height: 38,
              color: Gx.canvasBase.withAlpha(100),
            ),
            // Botón chevron: abre / cierra el panel de opciones.
            GestureDetector(
              onTap: widget.actions.isNotEmpty
                  ? () => setState(() => _open = !_open)
                  : null,
              child: AnimatedContainer(
                duration: const Duration(milliseconds: 160),
                padding:
                    const EdgeInsets.symmetric(horizontal: 10, vertical: 10),
                decoration: BoxDecoration(
                  gradient: Gx.linear(Gx.gradReactor),
                  borderRadius: BorderRadius.only(
                    topRight: Radius.circular(Gx.rButton),
                    bottomRight: Radius.circular(Gx.rButton),
                  ),
                  boxShadow: Gx.glow(Gx.reactorGreen, blur: 12, opacity: 0.5),
                ),
                child: AnimatedRotation(
                  duration: const Duration(milliseconds: 200),
                  // El chevron rota 180° al abrir el panel.
                  turns: _open ? -0.5 : 0,
                  child: Icon(Icons.keyboard_arrow_down,
                      size: 16, color: Gx.canvasBase),
                ),
              ),
            ),
          ],
        ),
        // Panel de opciones: animado con AnimatedSize; solo visible si _open y hay opciones.
        AnimatedSize(
          duration: const Duration(milliseconds: 200),
          curve: Curves.easeOut,
          child: _open && widget.actions.isNotEmpty
              ? Padding(
                  padding: EdgeInsets.only(top: Gx.space4),
                  child: panelSurface(
                    // Glow tenue del reactorGreen para conectar visualmente el panel al botón.
                    glow: Gx.glow(Gx.reactorGreen, blur: 10, opacity: 0.2),
                    child: Column(
                      mainAxisSize: MainAxisSize.min,
                      children: widget.actions
                          .map((action) => GestureDetector(
                                onTap: () {
                                  setState(() => _open = false);
                                  widget.onActionSelected?.call(action);
                                },
                                child: Container(
                                  width: double.infinity,
                                  padding: const EdgeInsets.symmetric(
                                      horizontal: Gx.space16 - 2,
                                      vertical: Gx.space8 + 2),
                                  // Texto de opción con token dinámico secundario.
                                  child: Text(
                                    action,
                                    style: Gx.uiSans(
                                      fontSize: 13,
                                      color: Gx.textBaseSecondary,
                                    ),
                                  ),
                                ),
                              ))
                          .toList(),
                    ),
                  ),
                )
              : const SizedBox.shrink(),
        ),
      ],
    );
  }
}
