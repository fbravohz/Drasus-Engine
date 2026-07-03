// button.dart — Componente Button (ADR-0138 enmienda 2026-06-29).
// Botón de acción con cuatro variantes semánticas, hover, down y carga.
// El estilo lo decide el tema global vía tokens Gx; prohibido hardcodear color.

import 'dart:math' show sin, pi;
import 'package:flutter/material.dart';
import '../theme/gx_tokens.dart';
import '../theme/surfaces.dart';

// Variantes semánticas del botón — mapean a los estados de vitalidad del sistema.
enum ButtonVariant {
  primary,   // acción de confirmación/éxito (gradOptima — cian)
  secondary, // acción de transición/incubación (gradTransition — índigo)
  danger,    // acción destructiva (gradCritical — carmesí)
  ghost,     // acción neutro/secundaria (superficie frosted, sin gradiente)
}

// Botón con gradiente, glow y animación de pulso de luz al pulsar.
// Contrato funcional: [label] texto visible; [onPressed] callback de acción
// (null = deshabilitado); [variant] variante semántica; [enabled]/[loading]
// estados de interacción. El loading muestra un spinner junto al label.
class Button extends StatefulWidget {
  final String label;
  final VoidCallback? onPressed;
  final ButtonVariant variant;
  final bool enabled;
  final bool loading;

  // No es const: el estilo de superficie lee getters dinámicos de Gx que
  // cambian con el tema — un const congelaría el modo activo.
  Button({
    super.key,
    required this.label,
    this.onPressed,
    this.variant = ButtonVariant.primary,
    this.enabled = true,
    this.loading = false,
  });

  @override
  State<Button> createState() => _ButtonState();
}

class _ButtonState extends State<Button> with SingleTickerProviderStateMixin {
  // Controla el pulso de glow al soltar el botón (explosión de luz).
  late final AnimationController _burst = AnimationController(
    vsync: this,
    duration: const Duration(milliseconds: 460),
  );
  bool _hover = false;
  bool _down = false;

  // El botón es interactivo solo si está habilitado, no está en carga y tiene callback.
  bool get _interactive =>
      widget.enabled && !widget.loading && widget.onPressed != null;

  // Gradiente según variante semántica.
  List<Color> _gradient() => switch (widget.variant) {
        ButtonVariant.primary   => Gx.gradOptima,
        ButtonVariant.secondary => Gx.gradTransition,
        ButtonVariant.danger    => Gx.gradCritical,
        ButtonVariant.ghost     => [],
      };

  // Color de glow según variante semántica.
  Color _glowColor() => switch (widget.variant) {
        ButtonVariant.primary   => Gx.optimaCyan,
        ButtonVariant.secondary => Gx.transitionIndigo,
        ButtonVariant.danger    => Gx.criticalCrimson,
        ButtonVariant.ghost     => Gx.accentDynamic,
      };

  // Color del texto según variante — legible sobre cada gradiente.
  Color _textColor() => switch (widget.variant) {
        ButtonVariant.primary   => Gx.canvasBase,   // oscuro sobre cian claro
        ButtonVariant.secondary => Gx.pureWhite,    // blanco sobre índigo
        ButtonVariant.danger    => Gx.pureWhite,    // blanco sobre carmesí
        ButtonVariant.ghost     => Gx.textBase,     // dinámico sobre superficie
      };

  @override
  void dispose() {
    _burst.dispose();
    super.dispose();
  }

  @override
  // Botón estilizado con animaciones de hover, down y burst; estados loading y disabled.
  // Para variante ghost usa frosted(); el resto usa gradiente + glow escalonado.
  Widget build(BuildContext context) {
    final glowColor = _glowColor();
    final textColor = _textColor();
    final isGhost = widget.variant == ButtonVariant.ghost;

    // Contenido interno: spinner + label en loading, solo label en idle.
    final Widget labelWidget = widget.loading
        ? Row(mainAxisSize: MainAxisSize.min, children: [
            SizedBox(
              width: 14,
              height: 14,
              // Spinner en el color del texto del botón (legible sobre gradiente).
              child: CircularProgressIndicator(strokeWidth: 1.5, color: textColor),
            ),
            const SizedBox(width: 8),
            Text(widget.label,
                style: Gx.uiSans(
                        fontSize: 13, weight: FontWeight.w600, color: textColor)
                    .copyWith(letterSpacing: 0.3)),
          ])
        : Text(widget.label,
            style:
                Gx.uiSans(fontSize: 13, weight: FontWeight.w600, color: textColor)
                    .copyWith(letterSpacing: 0.3));

    // Superficie del botón: ghost usa frosted(), el resto usa Container con gradiente.
    final Widget surface = isGhost
        ? frosted(
            padding:
                const EdgeInsets.symmetric(horizontal: 18, vertical: 11),
            glow: _hover && _interactive
                ? Gx.glow(glowColor, blur: 14, opacity: 0.3)
                : null,
            child: labelWidget,
          )
        : AnimatedBuilder(
            animation: _burst,
            builder: (_, child) {
              // Pulso: 0 → pico → 0 durante la animación de burst.
              final burst = sin(_burst.value * pi);
              final k = (_hover && _interactive ? 1.2 : 0.75) + burst * 1.3;
              return Container(
                padding:
                    const EdgeInsets.symmetric(horizontal: 18, vertical: 11),
                decoration: BoxDecoration(
                  gradient: Gx.linear(_gradient()),
                  borderRadius: BorderRadius.circular(Gx.rButton),
                  // Sin glow cuando no es interactivo (disabled / loading sin callback).
                  boxShadow:
                      _interactive ? Gx.glowStrong(glowColor, k) : null,
                ),
                child: child,
              );
            },
            child: labelWidget,
          );

    Widget button = MouseRegion(
      onEnter: _interactive ? (_) => setState(() => _hover = true) : null,
      onExit: _interactive ? (_) => setState(() => _hover = false) : null,
      cursor:
          _interactive ? SystemMouseCursors.click : SystemMouseCursors.basic,
      child: GestureDetector(
        onTapDown: _interactive ? (_) => setState(() => _down = true) : null,
        onTapUp: _interactive
            ? (_) {
                setState(() => _down = false);
                // Solo dispara burst en variantes con gradiente (no ghost).
                if (!isGhost) _burst.forward(from: 0);
                widget.onPressed?.call();
              }
            : null,
        onTapCancel:
            _interactive ? () => setState(() => _down = false) : null,
        child: AnimatedScale(
          scale: _down ? 0.96 : 1.0,
          duration: const Duration(milliseconds: 110),
          child: surface,
        ),
      ),
    );

    // Opacidad reducida cuando el botón no puede interactuar.
    return _interactive ? button : Opacity(opacity: 0.45, child: button);
  }
}
