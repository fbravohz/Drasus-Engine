// tooltip.dart — Componente Tooltip (ADR-0138 enmienda 2026-06-29).
// Tooltip flotante estilizado que aparece al hover sobre el widget hijo.
// El estilo lo decide el tema global vía tokens Gx; prohibido hardcodear color.
//
// Nota: el nombre Tooltip colisiona con el widget Material del mismo nombre.
// Los consumidores importan con namespace `import ... as ui;` → `ui.Tooltip`.
// Implementación propia con OverlayEntry + LayerLink para evitar la colisión
// dentro del archivo (no usa Flutter's Tooltip internamente).

import 'package:flutter/material.dart' hide Tooltip;
import '../theme/gx_tokens.dart';

// Tooltip flotante estilizado sobre el widget hijo.
// Contrato funcional: [message] texto del tooltip; [child] widget sobre el que
// se muestra el tooltip al hacer hover. El popup sigue los tokens del tema global.
class Tooltip extends StatefulWidget {
  final String message;
  final Widget child;

  // No es const: lee getters dinámicos de Gx que cambian con el tema.
  Tooltip({super.key, required this.message, required this.child});

  @override
  State<Tooltip> createState() => _TooltipState();
}

class _TooltipState extends State<Tooltip> {
  // LayerLink conecta el widget hijo con el overlay del tooltip.
  final LayerLink _link = LayerLink();
  OverlayEntry? _entry;

  // Muestra el tooltip sobre el widget hijo usando un OverlayEntry posicionado.
  void _show() {
    // Evita doble inserción si ya está visible.
    if (_entry != null) return;
    _entry = OverlayEntry(
      builder: (_) => Positioned(
        width: 200, // ancho máximo del tooltip; el texto hace wrap dentro
        child: CompositedTransformFollower(
          link: _link,
          // El tooltip aparece centrado arriba del widget hijo.
          targetAnchor: Alignment.topCenter,
          followerAnchor: Alignment.bottomCenter,
          offset: const Offset(0, -6),
          child: Material(
            // Material transparent para que el Container controle el fondo.
            color: Colors.transparent,
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
              decoration: BoxDecoration(
                // Surface de panel para coherencia con el modo global.
                color: Gx.surfacePanel,
                borderRadius: BorderRadius.circular(Gx.rTooltip),
                border: Border.all(color: Gx.borderBase),
                boxShadow: Gx.glow(Gx.accentDynamic, blur: 12, opacity: 0.2),
              ),
              child: Text(
                widget.message,
                style: Gx.uiSans(fontSize: 12, color: Gx.textBaseSecondary),
                textAlign: TextAlign.center,
              ),
            ),
          ),
        ),
      ),
    );
    Overlay.of(context).insert(_entry!);
  }

  // Elimina el tooltip del overlay.
  void _hide() {
    _entry?.remove();
    _entry = null;
  }

  @override
  void dispose() {
    // Limpiar el overlay si el widget se desmonta con el tooltip visible.
    _hide();
    super.dispose();
  }

  @override
  // CompositedTransformTarget ancla el overlay al widget hijo; MouseRegion dispara
  // la visibilidad del tooltip al entrar/salir del área del hijo.
  Widget build(BuildContext context) {
    return CompositedTransformTarget(
      link: _link,
      child: MouseRegion(
        onEnter: (_) => _show(),
        onExit: (_) => _hide(),
        child: widget.child,
      ),
    );
  }
}
