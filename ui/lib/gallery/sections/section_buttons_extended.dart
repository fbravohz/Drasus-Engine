// Sección §7 Botones extendidos — toggle, loading, group, FAB, segmented.
// Render-only con estado de UI local y animación. Sin lógica de negocio ni FFI.
// Tokens globales: Gx.rButton, Gx.rChip, Gx.surfaceFill, Gx.borderBase,
// Gx.borderBase, Gx.accentDynamic, Gx.glowStrong, Gx.gradTransition/gradReactor.

import 'package:flutter/material.dart';
import '../gallery_tokens.dart';
import '../gallery_fx.dart';

// ---------------------------------------------------------------------------
// Toggle Button — botón conmutable on/off con glow
// ---------------------------------------------------------------------------

// Botón que alterna entre dos estados; el estado activo lleva gradiente y glow.
class GlowToggleButton extends StatefulWidget {
  const GlowToggleButton({
    super.key,
    this.label = 'AUTO',
    this.labelOff = 'MANUAL',
    this.initial = false,
  });
  final String label;
  final String labelOff;
  final bool initial;
  @override
  State<GlowToggleButton> createState() => _GlowToggleButtonState();
}

class _GlowToggleButtonState extends State<GlowToggleButton> {
  late bool _on = widget.initial;

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: () => setState(() => _on = !_on),
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 220),
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
        decoration: BoxDecoration(
          gradient: _on ? Gx.linear(Gx.gradTransition) : null,
          color: _on ? null : Gx.surfaceFill,
          borderRadius: BorderRadius.circular(Gx.rButton),
          border: Border.all(
              color: _on ? Gx.transitionIndigo : Gx.borderBase),
          boxShadow: _on ? Gx.glow(Gx.transitionIndigo, blur: 16, opacity: 0.5) : null,
        ),
        child: Text(
          _on ? widget.label : widget.labelOff,
          style: Gx.uiSans(
            fontSize: 13,
            color: _on ? Gx.pureWhite : Gx.textBaseLabel,
            weight: FontWeight.w500,
          ),
        ),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Loading Button — botón con estado de carga (spinner + glow pulsante)
// ---------------------------------------------------------------------------

// Botón que simula un estado de carga al pulsarlo: el label se sustituye por
// un spinner y el glow pulsa en el color del estado activo.
class GlowLoadingButton extends StatefulWidget {
  const GlowLoadingButton({super.key});
  @override
  State<GlowLoadingButton> createState() => _GlowLoadingButtonState();
}

class _GlowLoadingButtonState extends State<GlowLoadingButton>
    with SingleTickerProviderStateMixin {
  bool _loading = false;
  // Controlador de la animación de pulso del glow durante la carga.
  late final AnimationController _pulse = AnimationController(
    vsync: this,
    duration: const Duration(milliseconds: 900),
  )..repeat(reverse: true);

  @override
  void dispose() {
    _pulse.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: _loading
          ? null
          : () async {
              setState(() => _loading = true);
              // Simula trabajo de 2s; en prod esto vendría del Bridge.
              await Future.delayed(const Duration(seconds: 2));
              if (mounted) setState(() => _loading = false);
            },
      child: AnimatedBuilder(
        animation: _pulse,
        builder: (_, child) {
          // El multiplicador de glow varía entre 0.7 y 1.5 durante la carga.
          final k = _loading ? 0.7 + _pulse.value * 0.8 : 0.75;
          return Container(
            padding:
                const EdgeInsets.symmetric(horizontal: 18, vertical: 11),
            decoration: BoxDecoration(
              gradient: _loading
                  ? Gx.linear(Gx.gradTransition)
                  : Gx.linear(Gx.gradReactor),
              borderRadius: BorderRadius.circular(Gx.rButton),
              boxShadow: Gx.glowStrong(
                _loading ? Gx.transitionIndigo : Gx.reactorGreen, k),
            ),
            child: child,
          );
        },
        child: Row(mainAxisSize: MainAxisSize.min, children: [
          if (_loading) ...[
            const SizedBox(
              width: 14,
              height: 14,
              child: CircularProgressIndicator(
                  // Token dinámico: spinner siempre blanco puro (legible sobre gradiente oscuro).
                  strokeWidth: 1.5, color: Gx.pureWhite),
            ),
            const SizedBox(width: 8),
          ],
          Text(
            _loading ? 'Cargando…' : 'LANZAR',
            style: Gx.uiSans(
              fontSize: 13,
              color: Gx.canvasBase,
              weight: FontWeight.w600,
            ),
          ),
        ]),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Button Group — fila de botones unidos con borde compartido
// ---------------------------------------------------------------------------

// Grupo de tres botones estilo segmented-control; solo uno puede estar activo.
class GlowButtonGroup extends StatefulWidget {
  const GlowButtonGroup({super.key});
  @override
  State<GlowButtonGroup> createState() => _GlowButtonGroupState();
}

class _GlowButtonGroupState extends State<GlowButtonGroup> {
  // Índice del botón activo.
  int _active = 1;
  static const _labels = ['1D', '1W', '1M'];

  @override
  Widget build(BuildContext context) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: _labels.asMap().entries.map((e) {
        final isActive = e.key == _active;
        // Determina el radio de esquinas: redondeado solo en extremos.
        final isFirst = e.key == 0;
        final isLast = e.key == _labels.length - 1;
        return GestureDetector(
          onTap: () => setState(() => _active = e.key),
          child: AnimatedContainer(
            duration: const Duration(milliseconds: 180),
            padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 9),
            decoration: BoxDecoration(
              gradient: isActive ? Gx.linear(Gx.gradTransition) : null,
              color: isActive ? null : Gx.surfaceFill,
              borderRadius: BorderRadius.only(
                topLeft: Radius.circular(isFirst ? Gx.rButton : 0),
                bottomLeft: Radius.circular(isFirst ? Gx.rButton : 0),
                topRight: Radius.circular(isLast ? Gx.rButton : 0),
                bottomRight: Radius.circular(isLast ? Gx.rButton : 0),
              ),
              border: Border.all(color: Gx.borderBase),
              boxShadow: isActive
                  ? Gx.glow(Gx.transitionIndigo, blur: 12, opacity: 0.4)
                  : null,
            ),
            child: Text(
              e.value,
              style: Gx.dataMono(
                  fontSize: 12,
                  color: isActive ? Gx.pureWhite : Gx.textBaseLabel),
            ),
          ),
        );
      }).toList(),
    );
  }
}

// ---------------------------------------------------------------------------
// FAB — botón flotante de acción con glow potente
// ---------------------------------------------------------------------------

// Botón circular flotante; al hover escala levemente y su glow se intensifica.
class GlowFab extends StatelessWidget {
  const GlowFab({super.key});

  @override
  Widget build(BuildContext context) {
    return HoverGlow(
      color: Gx.reactorGreen,
      radius: 999,
      child: Container(
        width: 52,
        height: 52,
        decoration: BoxDecoration(
          shape: BoxShape.circle,
          gradient: Gx.linear(Gx.gradReactor),
          boxShadow: Gx.glowStrong(Gx.reactorGreen),
        ),
        child: Icon(Gx.iconAdd, size: 22, color: Gx.canvasBase),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Segmented Control — conmutador de opciones tipo chip
// ---------------------------------------------------------------------------

// Control de selección única estilo pill/chip; el seleccionado lleva filo neón.
class GlowSegmented extends StatefulWidget {
  const GlowSegmented({super.key});
  @override
  State<GlowSegmented> createState() => _GlowSegmentedState();
}

class _GlowSegmentedState extends State<GlowSegmented> {
  int _sel = 0;
  static const _opts = ['Tend.', 'Rango', 'Vol.'];

  @override
  Widget build(BuildContext context) {
    return panelSurface(
      padding: const EdgeInsets.all(4),
      radius: 999,
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: _opts.asMap().entries.map((e) {
          final isActive = e.key == _sel;
          return GestureDetector(
            onTap: () => setState(() => _sel = e.key),
            child: AnimatedContainer(
              duration: const Duration(milliseconds: 180),
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
              decoration: BoxDecoration(
                color: isActive ? Gx.transitionIndigo.withAlpha(40) : Colors.transparent,
                borderRadius: BorderRadius.circular(999),
                border: isActive
                    ? Border.all(color: Gx.transitionIndigo)
                    : null,
                boxShadow: isActive
                    ? Gx.glow(Gx.transitionIndigo, blur: 8, opacity: 0.4)
                    : null,
              ),
              child: Text(
                e.value,
                style: Gx.uiSans(
                  fontSize: 12,
                  color: isActive ? Gx.transitionIndigo : Gx.textBaseLabel,
                ),
              ),
            ),
          );
        }).toList(),
      ),
    );
  }
}
