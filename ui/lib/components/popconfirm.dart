// popconfirm.dart — Componente Popconfirm (ADR-0138 enmienda 2026-06-29).
// Panel compacto de confirmación inline que aparece al pulsar el widget ancla.
// El estilo lo decide el tema global vía tokens Gx; prohibido hardcodear color.

import 'package:flutter/material.dart';
import '../gallery/gallery_tokens.dart';
import '../gallery/gallery_fx.dart';

// Panel de confirmación inline. Al pulsar el [child] (widget ancla) aparece
// un panel debajo con pregunta, descripción opcional, y dos botones: uno
// destructor (gradiente crítico) y uno de cancelación (superficie neutra).
// El color crítico es señalización interna de la acción destructiva — correcto.
//
// Contrato funcional:
//   [message]     pregunta de confirmación (texto principal).
//   [description] explicación de la acción destructiva (opcional).
//   [confirmLabel] etiqueta del botón destructor (por defecto 'Confirmar').
//   [cancelLabel]  etiqueta del botón secundario (por defecto 'Cancelar').
//   [onConfirm]   callback al confirmar.
//   [onCancel]    callback al cancelar.
//   [child]       widget ancla que abre el panel al pulsarlo.
class Popconfirm extends StatefulWidget {
  final String message;
  final String? description;
  final String confirmLabel;
  final String cancelLabel;
  final VoidCallback? onConfirm;
  final VoidCallback? onCancel;
  final Widget child;

  // No es const: los getters dinámicos de Gx cambian con el tema.
  Popconfirm({
    super.key,
    required this.message,
    this.description,
    this.confirmLabel = 'Confirmar',
    this.cancelLabel = 'Cancelar',
    this.onConfirm,
    this.onCancel,
    required this.child,
  });

  @override
  State<Popconfirm> createState() => _PopconfirmState();
}

class _PopconfirmState extends State<Popconfirm> {
  // Visibilidad del panel de confirmación.
  bool _visible = false;

  // Abre el panel al pulsar el ancla.
  void _open() => setState(() => _visible = true);

  // Cierra el panel y ejecuta onConfirm.
  void _confirm() {
    setState(() => _visible = false);
    widget.onConfirm?.call();
  }

  // Cierra el panel y ejecuta onCancel.
  void _cancel() {
    setState(() => _visible = false);
    widget.onCancel?.call();
  }

  @override
  // Columna: widget ancla + panel de confirmación animado debajo.
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // Widget ancla: GestureDetector para abrir el panel al pulsarlo.
        GestureDetector(onTap: _open, child: widget.child),
        // Panel de confirmación: animado con AnimatedSize.
        AnimatedSize(
          duration: const Duration(milliseconds: 200),
          child: _visible
              ? Padding(
                  padding: EdgeInsets.only(top: Gx.space8),
                  child: panelSurface(
                    padding: const EdgeInsets.all(Gx.space12),
                    // Glow crítico tenue: señaliza que la acción es destructiva.
                    glow: Gx.glow(Gx.criticalCrimson, blur: 14, opacity: 0.15),
                    child: Container(
                      decoration: BoxDecoration(
                        // Borde izquierdo crítico: señalización interna de acción destructiva.
                        border: Border(
                            left: BorderSide(
                                color: Gx.criticalCrimson, width: 3)),
                      ),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        mainAxisSize: MainAxisSize.min,
                        children: [
                          // Pregunta principal: token dinámico base, peso semibold.
                          Text(
                            widget.message,
                            style: Gx.uiSans(
                              fontSize: 13,
                              color: Gx.textBase,
                              weight: FontWeight.w500,
                            ),
                          ),
                          if (widget.description != null) ...[
                            SizedBox(height: Gx.space4),
                            // Descripción: token dinámico secundario (más tenue).
                            Text(
                              widget.description!,
                              style: Gx.uiSans(
                                fontSize: 12,
                                color: Gx.textBaseSecondary,
                              ),
                            ),
                          ],
                          SizedBox(height: Gx.space8 + Gx.space4),
                          Row(children: [
                            // Botón destructor: gradiente crítico (carmesí-rojo) + glow.
                            GestureDetector(
                              onTap: _confirm,
                              child: Container(
                                padding: const EdgeInsets.symmetric(
                                    horizontal: Gx.space12, vertical: 7),
                                decoration: BoxDecoration(
                                  gradient: Gx.linear(Gx.gradCritical),
                                  borderRadius:
                                      BorderRadius.circular(Gx.rButton),
                                  boxShadow: Gx.glow(Gx.criticalCrimson,
                                      blur: 10, opacity: 0.5),
                                ),
                                // pureWhite: texto visible sobre gradiente crítico oscuro.
                                child: Text(
                                  widget.confirmLabel,
                                  style: Gx.uiSans(
                                      fontSize: 12, color: Gx.pureWhite),
                                ),
                              ),
                            ),
                            SizedBox(width: Gx.space8),
                            // Botón de cancelación: superficie de relleno + borde estructural global.
                            GestureDetector(
                              onTap: _cancel,
                              child: Container(
                                padding: const EdgeInsets.symmetric(
                                    horizontal: Gx.space12, vertical: 7),
                                decoration: BoxDecoration(
                                  color: Gx.surfaceFill,
                                  borderRadius:
                                      BorderRadius.circular(Gx.rButton),
                                  border: Border.all(color: Gx.borderBase),
                                ),
                                // textBaseLabel: etiqueta discreta en acción secundaria.
                                child: Text(
                                  widget.cancelLabel,
                                  style: Gx.uiSans(
                                      fontSize: 12,
                                      color: Gx.textBaseLabel),
                                ),
                              ),
                            ),
                          ]),
                        ],
                      ),
                    ),
                  ),
                )
              : const SizedBox.shrink(),
        ),
      ],
    );
  }
}
