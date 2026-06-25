// ElectricLineChart — primitivo de gráfico de líneas con efecto eléctrico
// universal (ADR-0138). Extraído de section_dataviz_quant.dart para que
// cualquier Cáscara Delgada lo consuma sin duplicar la lógica.
//
// Recorre los puntos de izquierda a derecha: cada segmento recibe ignición
// eléctrica (electricIntensity) justo cuando el scan pasa por él, más la
// cola de cometa y la línea de scan (paintCometTail / paintScanLine).
// RepaintBoundary obligatorio.

import 'dart:math';
import 'package:flutter/material.dart';
import '../theme/drasus_tokens.dart';
import '../gallery/gallery_tokens.dart';
import 'electric_primitives.dart';

/// Gráfico de líneas con scan eléctrico al montarse.
///
/// Params:
/// - [points]: valores 0.0–1.0 a lo largo del ancho del lienzo.
/// - [color]: color semántico de la curva y del scan.
/// - [height]: alto del lienzo (default 96).
/// - [scanEnabled]: si true, ejecuta el scan al montarse (default true).
/// - [duration]: duración del scan (default DrasusMotion.scanMs).
/// - [label]: texto opcional de la cabecera compacta.
class ElectricLineChart extends StatefulWidget {
  final List<double> points;
  final Color color;
  final double height;
  final bool scanEnabled;
  final Duration? duration;
  final String? label;

  const ElectricLineChart({
    super.key,
    required this.points,
    required this.color,
    this.height = 96,
    this.scanEnabled = true,
    this.duration,
    this.label,
  });

  @override
  State<ElectricLineChart> createState() => _ElectricLineChartState();
}

class _ElectricLineChartState extends State<ElectricLineChart>
    with SingleTickerProviderStateMixin {
  // Controller del scan: 0.0–0.8 cruce, 0.8–1.0 fade de la línea de scan.
  late final AnimationController _ctrl;
  late final Animation<double> _curve;
  Duration _duration = const Duration(milliseconds: 1400);

  @override
  void initState() {
    super.initState();
    // Sin Theme.of(context) aquí: context no resuelve heredados en initState.
    _duration = widget.duration ?? const Duration(milliseconds: 1400);
    _ctrl = AnimationController(vsync: this, duration: _duration);
    _curve = CurvedAnimation(parent: _ctrl, curve: Curves.easeInOut);
    if (widget.scanEnabled) _ctrl.forward();
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    if (widget.duration == null) {
      final motion =
          Theme.of(context).extension<DrasusMotion>() ?? DrasusMotion.defaults;
      final resolved = Duration(milliseconds: motion.scanMs);
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

  /// Reinicia el scan desde cero.
  void replay() => _ctrl.forward(from: 0.0);

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(10),
      decoration: BoxDecoration(
        color: Gx.surfacePanel,
        borderRadius: BorderRadius.circular(Gx.rPanel),
        border: Border.all(color: Gx.borderPanel),
        boxShadow: Gx.glow(widget.color, blur: 14, opacity: 0.10),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          Row(children: [
            Icon(Icons.show_chart, size: 12, color: widget.color),
            const SizedBox(width: 6),
            Text(widget.label ?? 'Rolling Metric (eléctrico)',
                style: Gx.uiSans(fontSize: 11, color: Gx.textSecondary)),
            const Spacer(),
            GestureDetector(
              onTap: replay,
              child: Container(
                padding:
                    const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
                decoration: BoxDecoration(
                  color: Gx.surfaceFill,
                  border: Border.all(color: Gx.borderPanel),
                  borderRadius: BorderRadius.circular(Gx.rChip),
                ),
                child: Text('Replay',
                    style: Gx.uiSans(fontSize: 10, color: Gx.textSecondary)),
              ),
            ),
          ]),
          const SizedBox(height: 6),
          AnimatedBuilder(
            animation: _curve,
            builder: (_, __) {
              const crossFraction = 0.8;
              final p = _curve.value;
              final scanProgress =
                  p <= crossFraction ? p / crossFraction : 1.0;
              final scanOpacity = p <= crossFraction
                  ? 1.0
                  : (1.0 - (p - crossFraction) / (1.0 - crossFraction))
                      .clamp(0.0, 1.0);
              return SizedBox(
                height: widget.height,
                child: RepaintBoundary(
                  child: CustomPaint(
                    painter: _ElectricLinePainter(
                      points: widget.points,
                      scanProgress: scanProgress,
                      scanOpacity: scanOpacity,
                      color: widget.color,
                    ),
                    size: Size.infinite,
                  ),
                ),
              );
            },
          ),
        ],
      ),
    );
  }
}

class _ElectricLinePainter extends CustomPainter {
  final List<double> points;
  final double scanProgress;
  final double scanOpacity;
  final Color color;

  const _ElectricLinePainter({
    required this.points,
    required this.scanProgress,
    required this.scanOpacity,
    required this.color,
  });

  @override
  void paint(Canvas canvas, Size size) {
    final n = points.length;
    if (n < 2) return;
    final dx = size.width / (n - 1);
    const pad = 6.0;
    final h = size.height - pad * 2;
    final scanX = scanProgress * size.width;

    double toY(double v) => pad + h * (1 - v);

    // Relleno de área revelada por el scan (hasta scanX).
    final visibleCount = max(1, (scanProgress * n).floor());
    if (visibleCount > 1) {
      final fill = Path()..moveTo(0, toY(points[0]));
      for (var i = 1; i < visibleCount; i++) {
        fill.lineTo(i * dx, toY(points[i]));
      }
      fill
        ..lineTo((visibleCount - 1) * dx, size.height)
        ..lineTo(0, size.height)
        ..close();
      canvas.drawPath(
        fill,
        Paint()
          ..shader = LinearGradient(
            begin: Alignment.topCenter,
            end: Alignment.bottomCenter,
            colors: [color.withAlpha(22), Colors.transparent],
          ).createShader(Offset.zero & size),
      );
    }

    // Segmentos con ignición eléctrica: intensidad alta justo tras el scan.
    for (var i = 0; i < n - 1; i++) {
      final x0 = i * dx;
      if (x0 >= scanX) break;
      final intensity = electricIntensity(x0, scanX, size.width);
      final effOpacity = 0.7 + intensity * 0.3;
      final extraStroke = intensity * 2.0;
      final y0 = toY(points[i]);
      final y1 = toY(points[i + 1]);
      if (intensity > 0.05) {
        canvas.drawLine(
          Offset(x0, y0),
          Offset(x0 + dx, y1),
          Paint()
            ..color = color.withOpacity(intensity * 0.45)
            ..strokeWidth = 1.5 + extraStroke + 4
            ..maskFilter = MaskFilter.blur(BlurStyle.normal, 5 + intensity * 14),
        );
      }
      canvas.drawLine(
        Offset(x0, y0),
        Offset(x0 + dx, y1),
        Paint()
          ..color = color.withOpacity(effOpacity)
          ..strokeWidth = 1.5 + extraStroke,
      );
    }

    // Punto de cabeza de la curva (donde está el scan) con glow.
    if (scanProgress > 0 && scanProgress < 1.0) {
      final headIdx = (scanProgress * (n - 1)).clamp(0, n - 1).toInt();
      final head = Offset(headIdx * dx, toY(points[headIdx]));
      canvas.drawCircle(head, 7, Paint()
        ..color = color.withOpacity(0.5 * scanOpacity)
        ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 6));
      canvas.drawCircle(head, 3.5,
          Paint()..color = color.withOpacity(scanOpacity));
    }

    // Comet tail y scan line — primitivos eléctricos universales.
    paintCometTail(canvas, scanX, size, color);
    paintScanLine(canvas, scanX, size.height, color, scanOpacity);
  }

  @override
  bool shouldRepaint(_ElectricLinePainter old) =>
      old.scanProgress != scanProgress ||
      old.scanOpacity != scanOpacity ||
      old.points != points;
}
