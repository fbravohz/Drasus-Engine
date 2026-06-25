// AnimatedArc — primitivo de arco animado (ADR-0138).
// Arco radial que crece desde 0 hasta el ángulo final (definido por
// [progress] 0..1) al montarse, con Curves.easeOutCubic. Lee la duración
// por defecto de DrasusMotion.arcMs vía Theme.of(context).
// Extraído del patrón de _QuantRadialGauge / _QuantGaugePainter.

import 'dart:math';
import 'package:flutter/material.dart';
import '../theme/drasus_tokens.dart';
import '../gallery/gallery_tokens.dart';

/// Arco radial animado desde 0° hasta [progress] × barrido total.
///
/// Params:
/// - [progress]: fracción destino 0.0–1.0.
/// - [color]: color del arco (default DrasusPalette.accentColor).
/// - [size]: lado del lienzo cuadrado.
/// - [strokeWidth]: grosor del trazo (default 6).
/// - [duration]: duración (default DrasusMotion.arcMs).
/// - [gradColors]: degradado semántico opcional del arco.
/// - [startAngle] / [totalSweep]: geometría del arco (defaults del gauge).
/// - [trackColor]: color del riel de fondo (default Gx.divider).
class AnimatedArc extends StatefulWidget {
  final double progress;
  final Color? color;
  final double size;
  final double strokeWidth;
  final Duration? duration;
  final List<Color>? gradColors;
  final double startAngle;
  final double totalSweep;
  final Color trackColor;

  const AnimatedArc({
    super.key,
    required this.progress,
    this.color,
    this.size = 110,
    this.strokeWidth = 6,
    this.duration,
    this.gradColors,
    this.startAngle = -200.0 * pi / 180,
    this.totalSweep = 240.0 * pi / 180,
    this.trackColor = Gx.divider,
  });

  @override
  State<AnimatedArc> createState() => _AnimatedArcState();
}

class _AnimatedArcState extends State<AnimatedArc>
    with SingleTickerProviderStateMixin {
  late final AnimationController _ctrl;
  late final Animation<double> _anim;
  // Duración efectiva: la resuelve didChangeDependencies desde el theme.
  Duration _duration = const Duration(milliseconds: 1000);

  @override
  void initState() {
    super.initState();
    // Sin Theme.of(context) aquí: context no resuelve heredados en initState.
    // Usamos el valor explícito del widget si lo pasaron, o un default
    // constante que didChangeDependencies ajustará desde DrasusMotion.
    _duration = widget.duration ?? const Duration(milliseconds: 1000);
    _ctrl = AnimationController(vsync: this, duration: _duration);
    _anim = CurvedAnimation(parent: _ctrl, curve: Curves.easeOutCubic);
    _ctrl.forward();
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    // Aquí context ya resuelve Theme: ajustamos la duración desde el token
    // si el caller no pasó una explícita.
    if (widget.duration == null) {
      final motion =
          Theme.of(context).extension<DrasusMotion>() ?? DrasusMotion.defaults;
      final resolved = Duration(milliseconds: motion.arcMs);
      if (resolved != _duration) {
        _duration = resolved;
        _ctrl.duration = resolved;
      }
    }
  }

  @override
  void dispose() {
    _ctrl.dispose();
    super.dispose();
  }

  /// Reinicia el arco desde 0.
  void replay() => _ctrl.forward(from: 0.0);

  @override
  Widget build(BuildContext context) {
    final palette =
        Theme.of(context).extension<DrasusPalette>() ?? DrasusPalette.defaults;
    final color = widget.color ?? palette.accentColor;
    return SizedBox(
      width: widget.size,
      height: widget.size,
      child: RepaintBoundary(
        child: AnimatedBuilder(
          animation: _anim,
          builder: (_, __) => CustomPaint(
            painter: _AnimatedArcPainter(
              progress: _anim.value,
              targetFraction: widget.progress,
              color: color,
              gradColors: widget.gradColors,
              strokeWidth: widget.strokeWidth,
              startAngle: widget.startAngle,
              totalSweep: widget.totalSweep,
              trackColor: widget.trackColor,
            ),
          ),
        ),
      ),
    );
  }
}

class _AnimatedArcPainter extends CustomPainter {
  final double progress;
  final double targetFraction;
  final Color color;
  final List<Color>? gradColors;
  final double strokeWidth;
  final double startAngle;
  final double totalSweep;
  final Color trackColor;

  const _AnimatedArcPainter({
    required this.progress,
    required this.targetFraction,
    required this.color,
    required this.gradColors,
    required this.strokeWidth,
    required this.startAngle,
    required this.totalSweep,
    required this.trackColor,
  });

  @override
  void paint(Canvas canvas, Size size) {
    final center = Offset(size.width / 2, size.height / 2);
    final radius = size.shortestSide / 2 - 8;
    final rect = Rect.fromCircle(center: center, radius: radius);

    // Riel de fondo.
    canvas.drawArc(
      rect,
      startAngle,
      totalSweep,
      false,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = strokeWidth + 2
        ..strokeCap = StrokeCap.round
        ..color = trackColor,
    );

    final currentSweep = progress * targetFraction * totalSweep;
    if (currentSweep > 0.01) {
      // Glow difuso del arco.
      canvas.drawArc(
        rect,
        startAngle,
        currentSweep,
        false,
        Paint()
          ..style = PaintingStyle.stroke
          ..strokeWidth = strokeWidth + 2
          ..strokeCap = StrokeCap.round
          ..color = color.withAlpha(70)
          ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 6),
      );
      // Arco nítido con degradado semántico si se proveyó.
      final arcPaint = Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = strokeWidth
        ..strokeCap = StrokeCap.round;
      if (gradColors != null && gradColors!.isNotEmpty) {
        arcPaint.shader = SweepGradient(
          startAngle: startAngle,
          endAngle: startAngle + currentSweep,
          colors: gradColors!,
        ).createShader(rect);
      } else {
        arcPaint.color = color;
      }
      canvas.drawArc(rect, startAngle, currentSweep, false, arcPaint);
    }
  }

  @override
  bool shouldRepaint(_AnimatedArcPainter old) =>
      old.progress != progress || old.targetFraction != targetFraction;
}
