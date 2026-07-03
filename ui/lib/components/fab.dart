// fab.dart — Componente Fab (Floating Action Button) (ADR-0138 enmienda 2026-06-29).
// Botón circular flotante con gradiente reactor y glow. Hover: escala y glow se intensifican.
// El estilo lo decide el tema global vía tokens Gx; prohibido hardcodear color.

import 'package:flutter/material.dart';
import '../gallery/gallery_tokens.dart';

// Botón flotante de acción (FAB). Circular, con gradiente reactor (verde → cian).
// En hover escala a 1.05 y el glow se intensifica. Sin callback = deshabilitado.
//
// Contrato funcional:
//   [icon]      ícono central (por defecto: Icons.add).
//   [onPressed] callback al pulsar (null = deshabilitado, sin glow ni hover).
//   [tooltip]   texto descriptivo al hacer hover (opcional, usa Tooltip de Material).
class Fab extends StatefulWidget {
  final IconData icon;
  final VoidCallback? onPressed;
  final String? tooltip;

  // No es const: los getters dinámicos de Gx cambian con el tema.
  Fab({
    super.key,
    this.icon = Icons.add,
    this.onPressed,
    this.tooltip,
  });

  @override
  State<Fab> createState() => _FabState();
}

class _FabState extends State<Fab> {
  // Estado de hover para animar escala y glow.
  bool _hover = false;

  // Interactivo solo si tiene callback registrado.
  bool get _interactive => widget.onPressed != null;

  @override
  // Botón circular: gradiente reactor (verde-cian), glow reactorGreen.
  // En hover escala 1.05 con AnimatedScale; el glow pasa de 0.75 a 1.3.
  // Con tooltip: envuelto en el widget Tooltip de Material.
  Widget build(BuildContext context) {
    Widget button = MouseRegion(
      onEnter: _interactive ? (_) => setState(() => _hover = true) : null,
      onExit: _interactive ? (_) => setState(() => _hover = false) : null,
      cursor:
          _interactive ? SystemMouseCursors.click : SystemMouseCursors.basic,
      child: GestureDetector(
        onTap: _interactive ? widget.onPressed : null,
        child: AnimatedScale(
          scale: _hover && _interactive ? 1.05 : 1.0,
          duration: const Duration(milliseconds: 160),
          curve: Curves.easeOut,
          child: AnimatedContainer(
            duration: const Duration(milliseconds: 220),
            width: 52,
            height: 52,
            decoration: BoxDecoration(
              shape: BoxShape.circle,
              gradient: Gx.linear(Gx.gradReactor),
              // Glow intensificado en hover; base en reposo; sin glow si deshabilitado.
              boxShadow: _interactive
                  ? Gx.glowStrong(Gx.reactorGreen, _hover ? 1.3 : 0.75)
                  : null,
            ),
            child: Icon(
              widget.icon,
              size: 22,
              // canvasBase (oscuro) sobre gradiente cian-verde: legibilidad garantizada.
              color: Gx.canvasBase,
            ),
          ),
        ),
      ),
    );

    // Envuelve en el Tooltip de Material solo si se provee texto.
    if (widget.tooltip != null) {
      button = Tooltip(message: widget.tooltip!, child: button);
    }

    // Opacidad reducida cuando el botón está deshabilitado.
    return _interactive ? button : Opacity(opacity: 0.45, child: button);
  }
}
