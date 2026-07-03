// Efectos e interacción de la galería: vidrio Apple (frosted), glow, gradientes
// y micro-animaciones funcionales. Son widgets de UI puros (sin lógica de
// negocio ni FFI): el estado que manejan es local y visual (hover, foco, valor
// de un slider, día seleccionado), permitido en una Cáscara Delgada.

import 'dart:math';
import 'package:flutter/material.dart';
import '../theme/gx_tokens.dart';
import 'gallery_painters.dart';
import '../theme/theme_scope.dart';

// ─── Interacción ───
class HoverGlow extends StatefulWidget {
  final Widget child;
  final Color color;
  final double radius;
  final double scale;
  const HoverGlow(
      {super.key,
      required this.child,
      required this.color,
      this.radius = Gx.rPanel,
      this.scale = 1.03});

  @override
  State<HoverGlow> createState() => _HoverGlowState();
}

class _HoverGlowState extends State<HoverGlow> {
  bool _h = false;
  @override
  // Renderiza el hijo con escala animada al hover y sombra glow; estado local _h.
  Widget build(BuildContext context) {
    return MouseRegion(
      onEnter: (_) => setState(() => _h = true),
      onExit: (_) => setState(() => _h = false),
      child: AnimatedScale(
        scale: _h ? widget.scale : 1.0,
        duration: const Duration(milliseconds: 160),
        curve: Curves.easeOut,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 220),
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(widget.radius),
            boxShadow: _h
                ? Gx.glowStrong(widget.color)
                : Gx.glow(widget.color, blur: 10, opacity: 0.16),
          ),
          child: widget.child,
        ),
      ),
    );
  }
}

// Botón con gradiente, glow potente, hover y "propagación de luz" al pulsar
// (un pulso de glow que estalla del centro hacia afuera, inspiración Reflect).
// Switch funcional de palanca: alterna estado al tocar, con knob deslizante animado, gradiente y
// glow en ON. Estado local (_on) sin dependencia del Bridge.
// Slider funcional: se arrastra, con relleno en gradiente y manija con glow.
// Input funcional: foco real (FocusNode) con borde y glow limpios — sin la
// aberración cromática que quedaba mal. El glow es la señal de foco.
// Desplegable funcional: abre/cierra con animación y glow; al elegir, se cierra.
// Calendario funcional en grilla: toca un día y se enciende con un anillo de glow. Muestra
// marcadores de evento. Estado local (_sel) de UI, sin datos del Bridge.
// Texto con "propagación de luz" (inspiración Reflect): al tocarlo, la
// iluminación se expande del centro hacia afuera como una explosión.
class LightBurstText extends StatefulWidget {
  final String text;
  const LightBurstText(this.text, {super.key});
  @override
  State<LightBurstText> createState() => _LightBurstTextState();
}

class _LightBurstTextState extends State<LightBurstText>
    with SingleTickerProviderStateMixin {
  late final AnimationController _c =
      AnimationController(vsync: this, duration: const Duration(milliseconds: 900));

  @override
  void dispose() {
    _c.dispose();
    super.dispose();
  }

  @override
  // Texto con ShaderMask animado que propaga luz del centro al tocar; animación local _c.
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: () => _c.forward(from: 0),
      child: AnimatedBuilder(
        animation: _c,
        builder: (_, __) {
          final v = _c.value;
          return ShaderMask(
            shaderCallback: (rect) => RadialGradient(
              radius: 0.1 + v * 1.4,
              colors: [Gx.optimaCyan, Gx.transitionIndigo, Gx.textBaseSecondary],
              stops: const [0.0, 0.5, 1.0],
            ).createShader(rect),
            // Gx.pureWhite es el token canónico para blanco puro; necesario
            // para que el ShaderMask pinte los colores del gradiente correctamente.
            child: Text(widget.text,
                style: Gx.body.copyWith(color: Gx.pureWhite)),
          );
        },
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// HoverableChart — envoltorio genérico de hover para CustomPainter.
// Propaga la posición del mouse al painter; en onExit pasa null.
// Úsalo en lugar de CustomPaint directo cuando el painter deba responder al
// cursor: HoverableChart(builder: (h) => MiPainter(hover: h), height: 100).
// ---------------------------------------------------------------------------

typedef HoverPainterBuilder = CustomPainter Function(Offset? hover);

// Widget que entrega la posición local del cursor al builder del painter.
class HoverableChart extends StatefulWidget {
  final HoverPainterBuilder builder;
  final double height;
  const HoverableChart({super.key, required this.builder, this.height = 100});
  @override
  State<HoverableChart> createState() => _HoverableChartState();
}

class _HoverableChartState extends State<HoverableChart> {
  Offset? _hover;
  @override
  // Pasa la posición local del cursor (_hover) al builder del CustomPainter; sin FFI.
  Widget build(BuildContext context) {
    return MouseRegion(
      onHover: (e) => setState(() => _hover = e.localPosition),
      onExit: (_) => setState(() => _hover = null),
      child: SizedBox(
        height: widget.height,
        child: CustomPaint(
          painter: widget.builder(_hover),
          size: Size.infinite,
        ),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// SonarPulseWidget — anillo único que se expande y desaparece (evento discreto).
// ---------------------------------------------------------------------------
// Bug anterior: CustomPaint tomaba el tamaño del hijo (orbe 48×48) y los
// anillos (maxRadius ≥ 44) sobresalían del clip. Fix: el anillo se dibuja en
// un SizedBox propio más grande que el orbe; el orbe queda centrado encima.
// HitTestBehavior.opaque asegura que taps en área transparente se capturen.
class SonarPulseWidget extends StatefulWidget {
  final Widget child;
  final Color color;
  final double maxRadius;
  const SonarPulseWidget({
    super.key,
    required this.child,
    this.color = Gx.optimaCyan,
    this.maxRadius = 52,
  });
  @override
  State<SonarPulseWidget> createState() => _SonarPulseWidgetState();
}

class _SonarPulseWidgetState extends State<SonarPulseWidget>
    with SingleTickerProviderStateMixin {
  late final AnimationController _ctrl = AnimationController(
    vsync: this,
    duration: const Duration(milliseconds: 750),
  );

  @override
  void dispose() {
    _ctrl.dispose();
    super.dispose();
  }

  @override
  // Anillo sonar que se expande al tocar, centrado debajo del hijo; animación local _ctrl.
  Widget build(BuildContext context) {
    // opaque: los taps en el área vacía alrededor del orbe también disparan el pulso.
    return GestureDetector(
      behavior: HitTestBehavior.opaque,
      onTap: () => _ctrl.forward(from: 0),
      child: Stack(
        alignment: Alignment.center,
        children: [
          // Canvas de los anillos: (maxRadius*2 + margen) para que no se corten.
          SizedBox(
            width: widget.maxRadius * 2 + 28,
            height: widget.maxRadius * 2 + 28,
            child: AnimatedBuilder(
              animation: _ctrl,
              builder: (_, __) => CustomPaint(
                painter: _SonarRingPainter(
                  progress: _ctrl.value,
                  color: widget.color,
                  maxRadius: widget.maxRadius,
                ),
                size: Size.infinite,
              ),
            ),
          ),
          widget.child,
        ],
      ),
    );
  }
}

class _SonarRingPainter extends CustomPainter {
  final double progress;
  final Color color;
  final double maxRadius;
  const _SonarRingPainter(
      {required this.progress, required this.color, required this.maxRadius});

  @override
  // Dibuja un anillo de sonar que se expande con halo blando y anillo nítido, alpha decreciente.
  // Sin MaskFilter.blur: se simula el suavizado con círculos de radio ampliado y opacidad baja
  // (DESIGN.md §Performance: sin blur en animaciones — aplica también a one-shot de 750ms).
  void paint(Canvas canvas, Size size) {
    if (progress == 0) return;
    final center = Offset(size.width / 2, size.height / 2);
    final r = maxRadius * progress;
    // Alpha alto al inicio, cae hasta cero al final.
    final alpha = ((1 - progress) * 220).round().clamp(0, 220);
    // Halo exterior difuso (sin blur — círculo más ancho con opacidad baja).
    canvas.drawCircle(center, r + 5, Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 8
      ..color = color.withAlpha((alpha * 0.18).round()));
    // Halo intermedio.
    canvas.drawCircle(center, r, Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 4
      ..color = color.withAlpha((alpha * 0.28).round()));
    // Anillo nítido principal.
    canvas.drawCircle(center, r, Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 2.5
      ..color = color.withAlpha(alpha));
  }

  @override
  bool shouldRepaint(_SonarRingPainter old) => old.progress != progress;
}

// ---------------------------------------------------------------------------
// ScanRingWidget — anillos concéntricos en secuencia (monitoreo sostenido).
// ---------------------------------------------------------------------------
// Mismo fix de tamaño que SonarPulse: SizedBox propio para el lienzo de anillos,
// hijo centrado encima. Anillos visibles: alpha 190, strokeWidth 2.
class ScanRingWidget extends StatefulWidget {
  final Widget child;
  final Color color;
  final double maxRadius;
  final Duration period;
  final bool active;
  const ScanRingWidget({
    super.key,
    required this.child,
    this.color = Gx.optimaCyan,
    this.maxRadius = 52,
    this.period = const Duration(milliseconds: 2800),
    this.active = true,
  });
  @override
  State<ScanRingWidget> createState() => _ScanRingWidgetState();
}

class _ScanRingWidgetState extends State<ScanRingWidget>
    with SingleTickerProviderStateMixin {
  late final AnimationController _ctrl = AnimationController(
    vsync: this,
    duration: widget.period,
  );

  @override
  void initState() {
    super.initState();
    if (widget.active) _ctrl.repeat();
  }

  @override
  void didUpdateWidget(ScanRingWidget old) {
    super.didUpdateWidget(old);
    if (widget.active && !_ctrl.isAnimating) {
      _ctrl.repeat();
    } else if (!widget.active && _ctrl.isAnimating) {
      _ctrl.stop();
    }
  }

  @override
  void dispose() {
    _ctrl.dispose();
    super.dispose();
  }

  @override
  // Apila el lienzo de anillos de scan (SizedBox dedicado) y el widget hijo encima; ambos centrados.
  Widget build(BuildContext context) {
    return Stack(
      alignment: Alignment.center,
      children: [
        SizedBox(
          width: widget.maxRadius * 2 + 28,
          height: widget.maxRadius * 2 + 28,
          child: AnimatedBuilder(
            animation: _ctrl,
            builder: (_, __) => CustomPaint(
              painter: _ScanRingsPainter(
                progress: _ctrl.value,
                color: widget.color,
                maxRadius: widget.maxRadius,
              ),
              size: Size.infinite,
            ),
          ),
        ),
        widget.child,
      ],
    );
  }
}

class _ScanRingsPainter extends CustomPainter {
  final double progress;
  final Color color;
  final double maxRadius;
  const _ScanRingsPainter(
      {required this.progress, required this.color, required this.maxRadius});

  @override
  // Dibuja dos anillos concéntricos en secuencia infinita, cada uno con halo suave y anillo nítido.
  // Sin MaskFilter.blur: animación continua — se simula el suavizado con círculos concéntricos
  // de radio/opacidad decreciente (DESIGN.md §Performance: sin blur en animación).
  void paint(Canvas canvas, Size size) {
    final center = Offset(size.width / 2, size.height / 2);
    for (var ring = 0; ring < 2; ring++) {
      final p = (progress + ring * 0.45) % 1.0;
      final r = maxRadius * p;
      final alpha = ((1 - p) * 190).round().clamp(0, 190);
      // Halo exterior difuso (sin blur — círculo más ancho con opacidad baja).
      canvas.drawCircle(center, r + 4, Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 7
        ..color = color.withAlpha((alpha * 0.12).round()));
      // Halo intermedio (radio normal, opacidad media).
      canvas.drawCircle(center, r, Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 4
        ..color = color.withAlpha((alpha * 0.22).round()));
      // Anillo nítido principal.
      canvas.drawCircle(center, r, Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2.0
        ..color = color.withAlpha(alpha));
    }
  }

  @override
  bool shouldRepaint(_ScanRingsPainter old) => old.progress != progress;
}

// Grafo DAG interactivo: las líneas (CustomPaint) y los nodos (widgets con
// MouseRegion) se encienden al pasar el mouse.
class InteractiveDag extends StatefulWidget {
  const InteractiveDag({super.key});
  @override
  State<InteractiveDag> createState() => _InteractiveDagState();
}

class _InteractiveDagState extends State<InteractiveDag> {
  int? _hover;
  @override
  // Grafo DAG con nodos que se iluminan al hover; geometría local de dagNodes().
  Widget build(BuildContext context) {
    return SizedBox(
      height: 140,
      child: LayoutBuilder(builder: (ctx, box) {
        final size = Size(box.maxWidth, box.maxHeight);
        final nodes = dagNodes(size);
        return Stack(children: [
          // Líneas con glow detrás de los nodos.
          Positioned.fill(child: CustomPaint(painter: DagLinesPainter(_hover))),
          ...List.generate(nodes.length, (i) {
            final n = nodes[i];
            final hov = _hover == i;
            final sel = i == 3;
            final color = (sel || hov) ? Gx.optimaCyan : Gx.transitionIndigo;
            return Positioned(
              left: n.dx - 12,
              top: n.dy - 12,
              child: MouseRegion(
                onEnter: (_) => setState(() => _hover = i),
                onExit: (_) => setState(() => _hover = null),
                child: AnimatedContainer(
                  duration: const Duration(milliseconds: 180),
                  width: 24,
                  height: 24,
                  decoration: BoxDecoration(
                    shape: BoxShape.circle,
                    color: Gx.surfaceCard,
                    border: Border.all(color: color, width: 2),
                    boxShadow: hov
                        ? Gx.glowStrong(color)
                        : Gx.glow(color, blur: 10, opacity: 0.5),
                  ),
                ),
              ),
            );
          }),
        ]);
      }),
    );
  }
}
