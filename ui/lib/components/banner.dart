// banner.dart — Componente Banner (ADR-0138 enmienda 2026-06-29).
// Mensaje contextual con icono y borde semántico izquierdo según el tipo.
// El estilo lo decide el tema global vía tokens Gx; prohibido hardcodear color.
//
// Nota: el nombre Banner colisiona con el widget Material del mismo nombre (cinta
// de esquina). Los consumidores importan con namespace `import ... as ui;` → `ui.Banner`.

import 'package:flutter/material.dart' hide Banner;
import '../gallery/gallery_tokens.dart';
import '../gallery/gallery_fx.dart';

// Tipos de mensaje disponibles — cada uno mapea a un color semántico de vitalidad.
enum BannerType { info, success, warning, error }

// Banner de mensaje contextual con icono, borde izquierdo y glow semántico.
// Contrato funcional: [message] texto del mensaje; [type] tipo semántico que
// determina el color e icono (info/success/warning/error).
class Banner extends StatelessWidget {
  final String message;
  final BannerType type;

  // No es const: frosted() lee el modo global estático y debe poder reconstruirse.
  Banner({super.key, required this.message, required this.type});

  // Color primario del banner según el tipo semántico.
  Color _color() => switch (type) {
        BannerType.info    => Gx.transitionIndigo,
        BannerType.success => Gx.optimaCyan,
        BannerType.warning => Gx.alertAmber,
        BannerType.error   => Gx.criticalCrimson,
      };

  // Icono representativo del tipo semántico.
  IconData _icon() => switch (type) {
        BannerType.info    => Gx.iconBolt,
        BannerType.success => Gx.iconCheck,
        BannerType.warning => Gx.iconWarning,
        BannerType.error   => Gx.iconDanger,
      };

  @override
  // Banner con superficie frosted, borde izquierdo del color semántico y glow suave.
  Widget build(BuildContext context) {
    final color = _color();
    return frosted(
      radius: Gx.rPanel,
      padding: const EdgeInsets.all(10),
      glow: Gx.glow(color, blur: 14, opacity: 0.2),
      child: Container(
        decoration: BoxDecoration(
          // Borde izquierdo: señal semántica del tipo de mensaje.
          border: Border(left: BorderSide(color: color, width: 3)),
        ),
        child: Row(children: [
          Icon(_icon(), size: 16, color: color, shadows: Gx.textGlow(color)),
          const SizedBox(width: 8),
          Expanded(
            child: Text(message, style: Gx.bodySecondary),
          ),
        ]),
      ),
    );
  }
}
