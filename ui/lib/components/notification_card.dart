// notification_card.dart — Componente NotificationCard (ADR-0138 enmienda 2026-06-29).
// Tarjeta de notificación con borde semántico izquierdo e indicador "no leída".
// El estilo lo decide el tema global vía tokens Gx; prohibido hardcodear color.

import 'package:flutter/material.dart';
import '../gallery/gallery_tokens.dart';
import '../gallery/gallery_fx.dart';

// Tipo semántico de la notificación. Mapea a los cuatro estados de vitalidad del sistema.
// Determina el color del borde lateral y del punto "no leída".
enum NotificationCardType {
  info,    // transitionIndigo — mensaje informativo
  success, // optimaCyan      — operación completada
  warning, // alertAmber      — atención requerida
  error,   // criticalCrimson — fallo o estado crítico
}

// Tarjeta de notificación con borde izquierdo semántico y punto indicador de lectura.
// En modo no controlado, al tocar la tarjeta pasa a "leída" internamente.
// El botón de descarte (X) es opcional.
//
// Contrato funcional:
//   [title]    texto principal de la notificación.
//   [message]  texto secundario o timestamp.
//   [type]     tipo semántico que determina el color (info / success / warning / error).
//   [read]     si true, el punto "no leída" no aparece (null = no controlado, arranca no leída).
//   [onTap]    callback al pulsar la tarjeta (p.ej. marcar como leída externamente).
//   [onDismiss] callback al pulsar el botón de descarte (null = botón no visible).
class NotificationCard extends StatefulWidget {
  final String title;
  final String message;
  final NotificationCardType type;
  final bool? read;
  final VoidCallback? onTap;
  final VoidCallback? onDismiss;

  // No es const: los getters dinámicos de Gx cambian con el tema.
  NotificationCard({
    super.key,
    required this.title,
    required this.message,
    this.type = NotificationCardType.info,
    this.read,
    this.onTap,
    this.onDismiss,
  });

  @override
  State<NotificationCard> createState() => _NotificationCardState();
}

class _NotificationCardState extends State<NotificationCard> {
  // Estado interno de lectura para modo no controlado (arranca como "no leída").
  bool _internalRead = false;

  // Estado efectivo: el externo (read) tiene prioridad sobre el interno.
  bool get _isRead => widget.read ?? _internalRead;

  // Devuelve el color semántico según el tipo de notificación.
  Color get _semColor => switch (widget.type) {
        NotificationCardType.info    => Gx.transitionIndigo,
        NotificationCardType.success => Gx.optimaCyan,
        NotificationCardType.warning => Gx.alertAmber,
        NotificationCardType.error   => Gx.criticalCrimson,
      };

  // Al tocar la tarjeta: en modo no controlado pasa a "leída"; siempre ejecuta onTap.
  void _handleTap() {
    if (widget.read == null && !_internalRead) {
      setState(() => _internalRead = true);
    }
    widget.onTap?.call();
  }

  @override
  // Tarjeta sobre panelSurface con borde izquierdo semántico.
  // El glow tenue desaparece al leer la notificación.
  Widget build(BuildContext context) {
    final semColor = _semColor;
    return GestureDetector(
      onTap: _handleTap,
      child: panelSurface(
        padding: const EdgeInsets.all(Gx.space12),
        // Glow tenue del color semántico mientras no leída; se apaga al leer.
        glow: !_isRead ? Gx.glow(semColor, blur: 14, opacity: 0.15) : null,
        child: Container(
          decoration: BoxDecoration(
            // Borde izquierdo semántico: señaliza el tipo de notificación.
            border: Border(left: BorderSide(color: semColor, width: 3)),
          ),
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              // Punto "no leída": lleva color semántico y glow; transparente cuando leída.
              AnimatedContainer(
                duration: const Duration(milliseconds: 300),
                width: Gx.space8,
                height: Gx.space8,
                margin: EdgeInsets.only(
                    top: Gx.space4, right: Gx.space8 + Gx.space4),
                decoration: BoxDecoration(
                  shape: BoxShape.circle,
                  // Colors.transparent es el estado visual "leída" del punto.
                  color: !_isRead ? semColor : Colors.transparent,
                  boxShadow: !_isRead
                      ? Gx.glow(semColor, blur: 8, opacity: 0.7)
                      : null,
                ),
              ),
              // Columna de contenido: título + mensaje.
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    // Título: token base dinámico; peso semibold mientras no leída.
                    Text(
                      widget.title,
                      style: Gx.uiSans(
                        fontSize: 13,
                        color: Gx.textBase,
                        weight:
                            !_isRead ? FontWeight.w500 : FontWeight.w400,
                      ),
                    ),
                    const SizedBox(height: 2),
                    // Mensaje/timestamp: token muted dinámico en tipografía mono.
                    Text(
                      widget.message,
                      style: Gx.dataMono(
                          fontSize: 11, color: Gx.textBaseMuted),
                    ),
                  ],
                ),
              ),
              // Botón de descarte: visible solo si onDismiss está registrado.
              if (widget.onDismiss != null)
                GestureDetector(
                  onTap: widget.onDismiss,
                  child: Padding(
                    padding: EdgeInsets.only(left: Gx.space8),
                    child: Icon(Icons.close,
                        size: 14, color: Gx.textBaseMuted),
                  ),
                ),
            ],
          ),
        ),
      ),
    );
  }
}
