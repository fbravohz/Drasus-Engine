// §10 Data-viz cuantitativa — gráficos financieros con hover interactivo.
// Todos los painters aceptan Offset? hover; úsalos con HoverableChart (gallery_fx.dart).
// Comportamiento de hover común a los gráficos de línea:
//   • Línea entera se engrosa (1.5 → 2.5) y el glow aumenta cuando hay hover.
//   • Cursor vertical tenue en la X del mouse.
//   • Círculo de datos en el punto más cercano.
//   • Relleno de área bajo cada línea (sombra semántica, siempre activa).
// MultiEquityOverlay: áreas apiladas entre curvas ordenadas por Y (DESIGN.md).

import 'dart:math';
import 'package:flutter/material.dart';
import '../gallery_tokens.dart';

// ---------------------------------------------------------------------------
// Helper: cursor hover compartido por todos los painters de línea.
// Dibuja la línea vertical tenue y el punto resaltado en (cx, cy).
// ---------------------------------------------------------------------------
void _hoverCursor(Canvas canvas, Size size, double cx, double cy, Color c) {
  canvas.drawLine(Offset(cx, 0), Offset(cx, size.height), Paint()
    ..color = Gx.textMuted.withAlpha(50)
    ..strokeWidth = 0.5);
  canvas.drawCircle(Offset(cx, cy), 7, Paint()
    ..color = c.withAlpha(70)
    ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 6));
  canvas.drawCircle(Offset(cx, cy), 3.5, Paint()..color = Gx.textPrimary.withAlpha(230));
}

// ---------------------------------------------------------------------------
// EquityCurvePainter — curva de equity acumulada (P&L normalizado)
// ---------------------------------------------------------------------------
class EquityCurvePainter extends CustomPainter {
  final Offset? hover;
  EquityCurvePainter({this.hover});

  static final _pts = [
    0.00, 0.04, 0.08, 0.06, 0.11, 0.16, 0.13, 0.20, 0.25, 0.22,
    0.27, 0.32, 0.28, 0.35, 0.38, 0.35, 0.41, 0.45, 0.42, 0.48,
    0.52, 0.48, 0.45, 0.50, 0.54, 0.58, 0.56, 0.62, 0.66, 0.70,
  ];

  @override
  void paint(Canvas canvas, Size size) {
    final n = _pts.length;
    final dx = size.width / (n - 1);
    final maxV = _pts.reduce(max);
    const pad = 10.0;
    final h = size.height - pad;
    final isHov = hover != null;

    double toY(double v) => pad + h * (1 - v / maxV);

    // Relleno de área bajo la curva (sombra semántica del color de vitalidad).
    final fill = Path()..moveTo(0, toY(_pts[0]));
    for (var i = 1; i < n; i++) fill.lineTo(i * dx, toY(_pts[i]));
    fill..lineTo((n - 1) * dx, size.height)..lineTo(0, size.height)..close();
    canvas.drawPath(fill, Paint()
      ..shader = LinearGradient(
        begin: Alignment.topCenter, end: Alignment.bottomCenter,
        colors: [Gx.optimaCyan.withAlpha(isHov ? 30 : 18), Colors.transparent],
      ).createShader(Offset.zero & size));

    // Línea principal: más gruesa y más glow en hover.
    final line = Path()..moveTo(0, toY(_pts[0]));
    for (var i = 1; i < n; i++) line.lineTo(i * dx, toY(_pts[i]));
    canvas.drawPath(line, Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = isHov ? 7 : 5
      ..color = Gx.optimaCyan.withAlpha(isHov ? 80 : 55)
      ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 5));
    canvas.drawPath(line, Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = isHov ? 2.5 : 1.5
      ..shader = LinearGradient(
        colors: [Gx.optimaTeal, Gx.optimaCyan],
        begin: Alignment.centerLeft, end: Alignment.centerRight,
      ).createShader(Offset.zero & size));

    // Cursor de hover.
    if (hover != null) {
      final idx = (hover!.dx / dx).round().clamp(0, n - 1);
      _hoverCursor(canvas, size, idx * dx, toY(_pts[idx]), Gx.optimaCyan);
    }
  }

  @override
  bool shouldRepaint(EquityCurvePainter old) => old.hover != hover;
}

// ---------------------------------------------------------------------------
// MultiEquityOverlayPainter — superposición de curvas con áreas apiladas
// ---------------------------------------------------------------------------
// Las áreas entre curvas se colorean con el color de la curva superior del par,
// según la descripción del usuario: la sombra de cada línea se extiende hacia
// abajo hasta tocar la siguiente línea; la del siguiente hasta tocar la siguiente,
// y así hasta el eje. Al cruzarse dos curvas, el orden cambia por segmento.
class MultiEquityOverlayPainter extends CustomPainter {
  final Offset? hover;
  MultiEquityOverlayPainter({this.hover});

  static final _series = [
    ([0.0, 0.06, 0.14, 0.10, 0.18, 0.24, 0.20, 0.28, 0.33, 0.38], Gx.optimaCyan),
    ([0.0, 0.03, 0.08, 0.06, 0.11, 0.08, 0.13, 0.10, 0.14, 0.16], Gx.transitionIndigo),
    ([0.0, 0.02, 0.05, 0.03, 0.07, 0.04, 0.06, 0.03, 0.07, 0.09], Gx.alertAmber),
    ([0.0, -0.01, 0.01, -0.02, 0.0, -0.03, -0.05, -0.04, -0.07, -0.09], Gx.criticalCrimson),
  ];

  @override
  void paint(Canvas canvas, Size size) {
    double minV = 0, maxV = 0;
    for (final s in _series) {
      for (final v in s.$1) {
        if (v < minV) minV = v;
        if (v > maxV) maxV = v;
      }
    }
    final range = maxV - minV;
    if (range == 0) return;

    final n = _series[0].$1.length;
    final dx = size.width / (n - 1);
    const pad = 8.0;
    final h = size.height - pad * 2;
    final isHov = hover != null;

    double toY(double v) => pad + h * (1 - (v - minV) / range);

    // Eje cero.
    canvas.drawLine(Offset(0, toY(0)), Offset(size.width, toY(0)),
        Paint()..color = Gx.borderPanel..strokeWidth = 0.5);

    // ── Áreas apiladas entre curvas (por segmento, orden por Y) ─────────────
    // Para cada columna [i, i+1] se ordena las curvas de arriba a abajo (Y menor
    // = más alto en pantalla). El área entre la curva j y j+1 toma el color de j.
    // La última curva rellena hasta size.height (eje inferior).
    for (var i = 0; i < n - 1; i++) {
      final x0 = i * dx;
      final x1 = (i + 1) * dx;
      // Recoger (yIzq, yDer, color) y ordenar de menor Y (arriba) a mayor Y (abajo).
      final band = _series
          .map((s) => (toY(s.$1[i]), toY(s.$1[i + 1]), s.$2))
          .toList()
        ..sort((a, b) => ((a.$1 + a.$2) / 2).compareTo((b.$1 + b.$2) / 2));

      for (var j = 0; j < band.length; j++) {
        final topL = band[j].$1;
        final topR = band[j].$2;
        final botL = j + 1 < band.length ? band[j + 1].$1 : size.height;
        final botR = j + 1 < band.length ? band[j + 1].$2 : size.height;
        final path = Path()
          ..moveTo(x0, topL)
          ..lineTo(x1, topR)
          ..lineTo(x1, botR)
          ..lineTo(x0, botL)
          ..close();
        canvas.drawPath(path, Paint()..color = band[j].$3.withAlpha(28));
      }
    }

    // ── Líneas encima de las áreas ─────────────────────────────────────────
    for (final s in _series) {
      final pts = s.$1;
      final c = s.$2;
      final path = Path()..moveTo(0, toY(pts[0]));
      for (var i = 1; i < n; i++) path.lineTo(i * dx, toY(pts[i]));
      canvas.drawPath(path, Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = isHov ? 6 : 4
        ..color = c.withAlpha(isHov ? 60 : 45)
        ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 4));
      canvas.drawPath(path, Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = isHov ? 2.5 : 1.5
        ..color = c.withAlpha(200));
    }

    // Cursor hover.
    if (hover != null) {
      final idx = (hover!.dx / dx).round().clamp(0, n - 1);
      // Punto en la curva más cercana al mouse Y.
      double bestDist = double.infinity;
      Color bestC = Gx.optimaCyan;
      double bestY = 0;
      for (final s in _series) {
        final y = toY(s.$1[idx]);
        final d = (y - hover!.dy).abs();
        if (d < bestDist) { bestDist = d; bestC = s.$2; bestY = y; }
      }
      _hoverCursor(canvas, size, idx * dx, bestY, bestC);
    }
  }

  @override
  bool shouldRepaint(MultiEquityOverlayPainter old) => old.hover != hover;
}

// ---------------------------------------------------------------------------
// WfaChartPainter — Walk Forward Analysis
// ---------------------------------------------------------------------------
class WfaChartPainter extends CustomPainter {
  final Offset? hover;
  WfaChartPainter({this.hover});

  static const _windows = [
    (0.20, false, Gx.textMuted),
    (0.10, true, Gx.optimaCyan),
    (0.20, false, Gx.textMuted),
    (0.10, true, Gx.alertAmber),
    (0.20, false, Gx.textMuted),
    (0.10, true, Gx.optimaCyan),
    (0.10, false, Gx.textMuted),
  ];

  @override
  void paint(Canvas canvas, Size size) {
    double x = 0;
    final barH = size.height * 0.55;
    final barY = (size.height - barH) / 2;

    // Ventana hovereada (la que contiene hover.dx).
    int? hovIdx;
    if (hover != null) {
      double cx = 0;
      for (var k = 0; k < _windows.length; k++) {
        final w = _windows[k].$1 * size.width;
        if (hover!.dx >= cx && hover!.dx < cx + w) { hovIdx = k; break; }
        cx += w;
      }
    }

    for (var k = 0; k < _windows.length; k++) {
      final win = _windows[k];
      final w = win.$1 * size.width - 1;
      final isOos = win.$2;
      final c = win.$3;
      final hov = hovIdx == k;
      final rect = Rect.fromLTWH(x, barY, w, barH);

      if (isOos) {
        canvas.drawRRect(RRect.fromRectAndRadius(rect, const Radius.circular(3)),
            Paint()..color = c.withAlpha(hov ? 40 : 22));
        canvas.drawRRect(RRect.fromRectAndRadius(rect, const Radius.circular(3)),
            Paint()
              ..style = PaintingStyle.stroke
              ..strokeWidth = hov ? 6 : 4
              ..color = c.withAlpha(hov ? 80 : 55)
              ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 4));
        canvas.drawRRect(RRect.fromRectAndRadius(rect, const Radius.circular(3)),
            Paint()
              ..style = PaintingStyle.stroke
              ..strokeWidth = hov ? 2 : 1.5
              ..color = c.withAlpha(hov ? 240 : 200));
        final dotCenter = Offset(x + w / 2, barY - 8);
        canvas.drawCircle(dotCenter, hov ? 7 : 5, Paint()
          ..color = c.withAlpha(hov ? 120 : 80)
          ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 5));
        canvas.drawCircle(dotCenter, hov ? 3.5 : 2.5, Paint()..color = c);
      } else {
        canvas.drawRRect(RRect.fromRectAndRadius(rect, const Radius.circular(3)),
            Paint()..color = hov ? Gx.surfaceRaised : Gx.surfacePanel);
        canvas.drawRRect(RRect.fromRectAndRadius(rect, const Radius.circular(3)),
            Paint()
              ..style = PaintingStyle.stroke
              ..strokeWidth = 0.5
              ..color = hov ? Gx.textMuted : Gx.borderPanel);
        _label(canvas, 'IS', x + w / 2, barY + barH / 2);
      }
      x += win.$1 * size.width;
    }
  }

  void _label(Canvas canvas, String t, double cx, double cy) {
    final tp = TextPainter(
      text: TextSpan(text: t, style: TextStyle(fontFamily: Gx.fontMono, fontSize: 9, color: Gx.textMuted)),
      textDirection: TextDirection.ltr,
    )..layout();
    tp.paint(canvas, Offset(cx - tp.width / 2, cy - tp.height / 2));
  }

  @override
  bool shouldRepaint(WfaChartPainter old) => old.hover != hover;
}

// ---------------------------------------------------------------------------
// TradeTimelinePainter — entrada/salida de trades sobre eje temporal
// ---------------------------------------------------------------------------
class TradeTimelinePainter extends CustomPainter {
  final Offset? hover;
  TradeTimelinePainter({this.hover});

  static const _trades = [
    (0.06, 0), (0.13, 3), (0.22, 1), (0.31, 2),
    (0.40, 0), (0.47, 0), (0.54, 3), (0.63, 1),
    (0.71, 3), (0.80, 0), (0.89, 2),
  ];
  static const _colors = [
    Gx.optimaCyan, Gx.transitionIndigo, Gx.criticalCrimson, Gx.reactorGreen,
  ];

  @override
  void paint(Canvas canvas, Size size) {
    final midY = size.height / 2;
    final markH = size.height * 0.35;

    canvas.drawLine(Offset(0, midY), Offset(size.width, midY),
        Paint()..color = Gx.borderPanel..strokeWidth = 1);

    for (final t in _trades) {
      final x = t.$1 * size.width;
      final c = _colors[t.$2];
      final goUp = t.$2 == 0 || t.$2 == 3;
      final p1 = Offset(x, midY);
      final p2 = Offset(x, goUp ? midY - markH : midY + markH);
      // Hover: la marca más cercana al cursor X se agranda.
      final nearHov = hover != null && (hover!.dx - x).abs() < 12;
      canvas.drawLine(p1, p2, Paint()
        ..color = c.withAlpha(nearHov ? 120 : 80)
        ..strokeWidth = nearHov ? 6 : 4
        ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 4));
      canvas.drawLine(p1, p2, Paint()
        ..color = c.withAlpha(nearHov ? 240 : 200)
        ..strokeWidth = nearHov ? 2 : 1.5);
      canvas.drawCircle(p2, nearHov ? 4 : 2.5, Paint()
        ..color = c
        ..maskFilter = nearHov ? const MaskFilter.blur(BlurStyle.normal, 4) : null);
    }
  }

  @override
  bool shouldRepaint(TradeTimelinePainter old) => old.hover != hover;
}

// ---------------------------------------------------------------------------
// ReturnsCalendarPainter — heatmap de rentabilidad mensual
// ---------------------------------------------------------------------------
class ReturnsCalendarPainter extends CustomPainter {
  final Offset? hover;
  ReturnsCalendarPainter({this.hover});

  static const _data = [
     0.04,  0.02, -0.01,  0.03,  0.06,  0.01,
    -0.02,  0.05,  0.03, -0.04,  0.02,  0.04,
     0.08,  0.03,  0.01,  0.05, -0.02,  0.07,
     0.01, -0.03,  0.04,  0.02,  0.09,  0.03,
  ];

  @override
  void paint(Canvas canvas, Size size) {
    const cols = 6;
    const rows = 4;
    final cellW = size.width / cols;
    final cellH = size.height / rows;

    final hovCol = hover != null ? (hover!.dx / cellW).floor().clamp(0, cols - 1) : -1;
    final hovRow = hover != null ? (hover!.dy / cellH).floor().clamp(0, rows - 1) : -1;

    for (var r = 0; r < rows; r++) {
      for (var c = 0; c < cols; c++) {
        final v = _data[r * cols + c];
        final color = _colorFor(v);
        final hov = r == hovRow && c == hovCol;
        final alpha = hov
            ? 220
            : (60 + (v.abs() / 0.10 * 160).clamp(0.0, 160.0)).round();
        final rect = Rect.fromLTWH(c * cellW + 2, r * cellH + 2, cellW - 4, cellH - 4);
        canvas.drawRRect(
          RRect.fromRectAndRadius(rect, const Radius.circular(4)),
          Paint()..color = color.withAlpha(alpha),
        );
        if (hov) {
          canvas.drawRRect(
            RRect.fromRectAndRadius(rect, const Radius.circular(4)),
            Paint()
              ..style = PaintingStyle.stroke
              ..strokeWidth = 1.5
              ..color = color.withAlpha(200),
          );
        }
      }
    }
  }

  static Color _colorFor(double v) {
    if (v > 0.02) return Gx.optimaCyan;
    if (v > -0.02) return Gx.transitionIndigo;
    return Gx.criticalCrimson;
  }

  @override
  bool shouldRepaint(ReturnsCalendarPainter old) => old.hover != hover;
}

// ---------------------------------------------------------------------------
// FitnessEvolutionPainter — curva de fitness del algoritmo genético
// ---------------------------------------------------------------------------
class FitnessEvolutionPainter extends CustomPainter {
  final Offset? hover;
  FitnessEvolutionPainter({this.hover});

  static const _pts = [
    0.10, 0.11, 0.13, 0.12, 0.15, 0.16, 0.14, 0.18, 0.20, 0.19,
    0.23, 0.25, 0.24, 0.27, 0.26, 0.26, 0.28, 0.29, 0.30, 0.30,
    0.31, 0.34, 0.38, 0.42, 0.46, 0.50, 0.55, 0.60, 0.65, 0.68,
    0.70, 0.72, 0.73, 0.74, 0.75, 0.76, 0.77, 0.78, 0.78, 0.79,
    0.80, 0.81, 0.81, 0.82, 0.83, 0.83, 0.84, 0.85, 0.85, 0.85,
  ];

  @override
  void paint(Canvas canvas, Size size) {
    final n = _pts.length;
    final dx = size.width / (n - 1);
    const pad = 10.0;
    final h = size.height - pad;
    final isHov = hover != null;

    double toY(double v) => pad + h * (1 - v);

    // Relleno de área bajo la curva (gradiente por fitness: índigo→cian).
    final fill = Path()..moveTo(0, toY(_pts[0]));
    for (var i = 1; i < n; i++) fill.lineTo(i * dx, toY(_pts[i]));
    fill..lineTo((n - 1) * dx, size.height)..lineTo(0, size.height)..close();
    canvas.drawPath(fill, Paint()
      ..shader = LinearGradient(
        begin: Alignment.topCenter, end: Alignment.bottomCenter,
        colors: [Gx.transitionIndigo.withAlpha(isHov ? 22 : 14), Colors.transparent],
      ).createShader(Offset.zero & size));

    // Línea segmento a segmento con color según nivel de fitness.
    for (var i = 0; i < n - 1; i++) {
      final vMid = (_pts[i] + _pts[i + 1]) / 2;
      final c = _colorForFitness(vMid);
      final p0 = Offset(i * dx, toY(_pts[i]));
      final p1 = Offset((i + 1) * dx, toY(_pts[i + 1]));
      canvas.drawLine(p0, p1, Paint()
        ..strokeWidth = isHov ? 7 : 4
        ..color = c.withAlpha(isHov ? 80 : 50)
        ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 4));
      canvas.drawLine(p0, p1, Paint()
        ..strokeWidth = isHov ? 2.5 : 1.5
        ..color = c.withAlpha(200));
    }

    // Punto de convergencia.
    final last = Offset((n - 1) * dx, toY(_pts.last));
    canvas.drawCircle(last, 6, Paint()
      ..color = Gx.optimaCyan.withAlpha(80)
      ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 8));
    canvas.drawCircle(last, 3, Paint()..color = Gx.optimaCyan);

    // Cursor hover.
    if (hover != null) {
      final idx = (hover!.dx / dx).round().clamp(0, n - 1);
      _hoverCursor(canvas, size, idx * dx, toY(_pts[idx]), _colorForFitness(_pts[idx]));
    }
  }

  static Color _colorForFitness(double v) {
    if (v < 0.35) return Gx.transitionIndigo;
    if (v < 0.60) return Gx.alertAmber;
    return Gx.optimaCyan;
  }

  @override
  bool shouldRepaint(FitnessEvolutionPainter old) => old.hover != hover;
}

// ---------------------------------------------------------------------------
// RollingMetricPainter — métricas rolling en el tiempo
// ---------------------------------------------------------------------------
class RollingMetricPainter extends CustomPainter {
  final Offset? hover;
  RollingMetricPainter({this.hover});

  static final _series = [
    ([0.6, 0.7, 0.65, 0.75, 0.8, 0.72, 0.78, 0.85, 0.80, 0.90, 0.88, 0.95], Gx.optimaCyan),
    ([0.3, 0.35, 0.4, 0.38, 0.5, 0.55, 0.45, 0.4, 0.6, 0.7, 0.65, 0.55], Gx.alertAmber),
    ([0.1, 0.15, 0.2, 0.18, 0.25, 0.35, 0.3, 0.28, 0.4, 0.45, 0.38, 0.3], Gx.criticalCrimson),
  ];

  @override
  void paint(Canvas canvas, Size size) {
    const pad = 8.0;
    final h = size.height - pad * 2;
    final isHov = hover != null;
    double toY(double v) => pad + h * (1 - v);

    for (final s in _series) {
      final pts = s.$1;
      final c = s.$2;
      final n = pts.length;
      final dx = size.width / (n - 1);

      // Relleno de área bajo cada métrica.
      final fill = Path()..moveTo(0, toY(pts[0]));
      for (var i = 1; i < n; i++) fill.lineTo(i * dx, toY(pts[i]));
      fill..lineTo((n - 1) * dx, size.height)..lineTo(0, size.height)..close();
      canvas.drawPath(fill, Paint()
        ..shader = LinearGradient(
          begin: Alignment.topCenter, end: Alignment.bottomCenter,
          colors: [c.withAlpha(isHov ? 20 : 12), Colors.transparent],
        ).createShader(Offset.zero & size));

      final path = Path()..moveTo(0, toY(pts[0]));
      for (var i = 1; i < n; i++) path.lineTo(i * dx, toY(pts[i]));
      canvas.drawPath(path, Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = isHov ? 6 : 4
        ..color = c.withAlpha(isHov ? 60 : 40)
        ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 4));
      canvas.drawPath(path, Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = isHov ? 2.5 : 1.5
        ..color = c.withAlpha(200));

      // Cursor hover: punto en la métrica más cercana verticalmente.
      if (hover != null) {
        final dx2 = size.width / (n - 1);
        final idx = (hover!.dx / dx2).round().clamp(0, n - 1);
        final cy = toY(pts[idx]);
        if ((cy - hover!.dy).abs() < 16) {
          _hoverCursor(canvas, size, idx * dx2, cy, c);
        }
      }
    }
  }

  @override
  bool shouldRepaint(RollingMetricPainter old) => old.hover != hover;
}

// ---------------------------------------------------------------------------
// UnderwaterPlotPainter — drawdown bajo el eje cero
// ---------------------------------------------------------------------------
class UnderwaterPlotPainter extends CustomPainter {
  final Offset? hover;
  UnderwaterPlotPainter({this.hover});

  static const _pts = [
    0.0, 0.05, 0.10, 0.08, 0.15, 0.12, 0.22, 0.18, 0.28, 0.35,
    0.30, 0.25, 0.18, 0.38, 0.32, 0.25, 0.20, 0.14, 0.08, 0.04,
    0.0, 0.06, 0.12, 0.08, 0.18, 0.15, 0.10, 0.05, 0.02, 0.0,
  ];

  @override
  void paint(Canvas canvas, Size size) {
    final n = _pts.length;
    final dx = size.width / (n - 1);
    const zeroY = 8.0;
    final h = size.height - zeroY - 4;
    final isHov = hover != null;

    canvas.drawLine(Offset(0, zeroY), Offset(size.width, zeroY),
        Paint()..color = Gx.borderPanel..strokeWidth = 1);

    final path = Path()..moveTo(0, zeroY);
    for (var i = 0; i < n; i++) path.lineTo(i * dx, zeroY + _pts[i] * h);

    final fill = Path.from(path)
      ..lineTo((n - 1) * dx, zeroY)
      ..close();
    canvas.drawPath(fill, Paint()
      ..shader = LinearGradient(
        begin: Alignment.topCenter, end: Alignment.bottomCenter,
        colors: [Gx.criticalCrimson.withAlpha(isHov ? 55 : 40), Gx.criticalRed.withAlpha(12)],
      ).createShader(Offset.zero & size));

    canvas.drawPath(path, Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = isHov ? 7 : 4
      ..color = Gx.criticalCrimson.withAlpha(isHov ? 80 : 55)
      ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 4));
    canvas.drawPath(path, Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = isHov ? 2.5 : 1.5
      ..color = Gx.criticalRed.withAlpha(220));

    if (hover != null) {
      final idx = (hover!.dx / dx).round().clamp(0, n - 1);
      _hoverCursor(canvas, size, idx * dx, zeroY + _pts[idx] * h, Gx.criticalCrimson);
    }
  }

  @override
  bool shouldRepaint(UnderwaterPlotPainter old) => old.hover != hover;
}

// ---------------------------------------------------------------------------
// RiskReturnScatterPainter — frontera de eficiencia
// ---------------------------------------------------------------------------
class RiskReturnScatterPainter extends CustomPainter {
  final Offset? hover;
  RiskReturnScatterPainter({this.hover});

  static const _pts = [
    (0.15, 0.85, 0), (0.20, 0.78, 0), (0.30, 0.80, 0),
    (0.35, 0.60, 1), (0.50, 0.55, 1), (0.25, 0.50, 1),
    (0.60, 0.50, 2), (0.70, 0.40, 2),
    (0.80, 0.30, 3), (0.45, 0.30, 3),
  ];
  static const _colors = [
    Gx.optimaCyan, Gx.transitionIndigo, Gx.alertAmber, Gx.criticalCrimson,
  ];
  static const _frontier = [0, 1, 2, 4, 3];

  @override
  void paint(Canvas canvas, Size size) {
    const pad = 12.0;
    final w = size.width - pad * 2;
    final h = size.height - pad * 2;

    canvas.drawLine(Offset(pad, pad), Offset(pad, size.height - pad),
        Paint()..color = Gx.borderPanel..strokeWidth = 0.5);
    canvas.drawLine(Offset(pad, size.height - pad),
        Offset(size.width - pad, size.height - pad),
        Paint()..color = Gx.borderPanel..strokeWidth = 0.5);

    // Frontera de Pareto.
    final fp = Path();
    for (var i = 0; i < _frontier.length; i++) {
      final pt = _pts[_frontier[i]];
      final px = pad + pt.$1 * w;
      final py = pad + (1 - pt.$2) * h;
      if (i == 0) fp.moveTo(px, py); else fp.lineTo(px, py);
    }
    canvas.drawPath(fp, Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 3
      ..color = Gx.optimaCyan.withAlpha(50)
      ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 4));
    canvas.drawPath(fp, Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1
      ..shader = LinearGradient(colors: Gx.gradOptima).createShader(Offset.zero & size));

    // Puntos con hover en el más cercano al cursor.
    int nearIdx = -1;
    if (hover != null) {
      double minDist = double.infinity;
      for (var i = 0; i < _pts.length; i++) {
        final px = pad + _pts[i].$1 * w;
        final py = pad + (1 - _pts[i].$2) * h;
        final d = (Offset(px, py) - hover!).distance;
        if (d < minDist) { minDist = d; nearIdx = i; }
      }
    }

    for (var i = 0; i < _pts.length; i++) {
      final pt = _pts[i];
      final px = pad + pt.$1 * w;
      final py = pad + (1 - pt.$2) * h;
      final c = _colors[pt.$3];
      final hov = i == nearIdx;
      canvas.drawCircle(Offset(px, py), hov ? 10 : 7, Paint()
        ..color = c.withAlpha(hov ? 90 : 55)
        ..maskFilter = MaskFilter.blur(BlurStyle.normal, hov ? 8 : 5));
      canvas.drawCircle(Offset(px, py), hov ? 6 : 3.5, Paint()..color = c.withAlpha(220));
    }
  }

  @override
  bool shouldRepaint(RiskReturnScatterPainter old) => old.hover != hover;
}

// ---------------------------------------------------------------------------
// TradeDistributionPainter — histograma de P&L por trade
// ---------------------------------------------------------------------------
class TradeDistributionPainter extends CustomPainter {
  final Offset? hover;
  TradeDistributionPainter({this.hover});

  static const _bins = [
    (-0.05, 3), (-0.04, 5), (-0.03, 8), (-0.02, 12), (-0.01, 9),
    (0.01, 14),  (0.02, 18), (0.03, 11), (0.04, 7),  (0.05, 4),
  ];

  @override
  void paint(Canvas canvas, Size size) {
    final maxCount = _bins.map((b) => b.$2).reduce(max).toDouble();
    final n = _bins.length;
    final barW = size.width / (n + 1);
    final maxH = size.height - 14.0;

    final hovIdx = hover != null
        ? ((hover!.dx / barW) - 0.5).round().clamp(0, n - 1)
        : -1;

    for (var i = 0; i < n; i++) {
      final isGain = _bins[i].$1 >= 0;
      final c = isGain ? Gx.optimaCyan : Gx.criticalCrimson;
      final hov = i == hovIdx;
      // La barra hovereada crece 2px extra hacia arriba.
      final barH = (_bins[i].$2 / maxCount) * maxH + (hov ? 2 : 0);
      final x = barW * 0.5 + i * barW;
      final rect = Rect.fromLTWH(x - barW * 0.38, size.height - barH, barW * 0.76, barH);
      canvas.drawRRect(RRect.fromRectAndRadius(rect.inflate(hov ? 2 : 1), const Radius.circular(3)),
          Paint()
            ..color = c.withAlpha(hov ? 90 : 50)
            ..maskFilter = MaskFilter.blur(BlurStyle.normal, hov ? 6 : 4));
      canvas.drawRRect(RRect.fromRectAndRadius(rect, const Radius.circular(2)),
          Paint()..color = c.withAlpha(hov ? 220 : 190));
    }

    canvas.drawLine(Offset(size.width / 2, 0), Offset(size.width / 2, size.height),
        Paint()
          ..color = Gx.optimaTeal.withAlpha(160)
          ..strokeWidth = 1
          ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 2));
  }

  @override
  bool shouldRepaint(TradeDistributionPainter old) => old.hover != hover;
}

// ---------------------------------------------------------------------------
// ParameterSensitivityPainter — robustez por parámetro
// ---------------------------------------------------------------------------
class ParameterSensitivityPainter extends CustomPainter {
  final Offset? hover;
  ParameterSensitivityPainter({this.hover});

  static const _params = [
    ('StopLoss %', 0.85),
    ('Lookback', 0.62),
    ('SMA Fast', 0.45),
    ('SMA Slow', 0.78),
  ];

  @override
  void paint(Canvas canvas, Size size) {
    final n = _params.length;
    final rowH = size.height / n;
    const labelW = 52.0;
    const barOffset = labelW + 4;
    final barMaxW = size.width - barOffset;

    final hovRow = hover != null
        ? (hover!.dy / rowH).floor().clamp(0, n - 1)
        : -1;

    for (var i = 0; i < n; i++) {
      final param = _params[i];
      final c = _colorFor(param.$2);
      final barW = param.$2 * barMaxW;
      final barY = i * rowH + rowH * 0.28;
      final barH = rowH * 0.44;
      final hov = i == hovRow;

      canvas.drawRRect(
        RRect.fromRectAndRadius(Rect.fromLTWH(barOffset, barY, barMaxW, barH), const Radius.circular(3)),
        Paint()..color = Gx.gaugeTrack,
      );
      canvas.drawRRect(
        RRect.fromRectAndRadius(Rect.fromLTWH(barOffset, barY, barW, barH).inflate(hov ? 2 : 1), const Radius.circular(4)),
        Paint()
          ..color = c.withAlpha(hov ? 90 : 55)
          ..maskFilter = MaskFilter.blur(BlurStyle.normal, hov ? 6 : 4),
      );
      canvas.drawRRect(
        RRect.fromRectAndRadius(Rect.fromLTWH(barOffset, barY, barW, barH), const Radius.circular(3)),
        Paint()..color = c.withAlpha(hov ? 220 : 200),
      );
    }
  }

  static Color _colorFor(double v) {
    if (v >= 0.70) return Gx.optimaCyan;
    if (v >= 0.45) return Gx.alertAmber;
    return Gx.criticalCrimson;
  }

  @override
  bool shouldRepaint(ParameterSensitivityPainter old) => old.hover != hover;
}

// ---------------------------------------------------------------------------
// RegimeTimelinePainter — línea de tiempo de régimen
// ---------------------------------------------------------------------------
class RegimeTimelinePainter extends CustomPainter {
  final Offset? hover;
  RegimeTimelinePainter({this.hover});

  static const _segments = [
    (0.18, Gx.optimaCyan, 'Trend'),
    (0.10, Gx.transitionIndigo, 'Calm'),
    (0.14, Gx.alertAmber, 'Vol'),
    (0.22, Gx.optimaCyan, 'Trend'),
    (0.08, Gx.criticalCrimson, 'Crash'),
    (0.16, Gx.transitionIndigo, 'Calm'),
    (0.12, Gx.alertAmber, 'Vol'),
  ];

  @override
  void paint(Canvas canvas, Size size) {
    final barH = size.height * 0.55;
    final barY = (size.height - barH) / 2;
    double x = 0;

    int? hovIdx;
    if (hover != null) {
      double cx = 0;
      for (var k = 0; k < _segments.length; k++) {
        final w = _segments[k].$1 * size.width;
        if (hover!.dx >= cx && hover!.dx < cx + w) { hovIdx = k; break; }
        cx += w;
      }
    }

    for (var k = 0; k < _segments.length; k++) {
      final seg = _segments[k];
      final w = seg.$1 * size.width - 1;
      final c = seg.$2;
      final hov = k == hovIdx;
      final rect = Rect.fromLTWH(x, barY, w, barH);

      canvas.drawRRect(RRect.fromRectAndRadius(rect, const Radius.circular(3)),
          Paint()..color = c.withAlpha(hov ? 70 : 40));
      canvas.drawRRect(RRect.fromRectAndRadius(rect, const Radius.circular(3)),
          Paint()
            ..style = PaintingStyle.stroke
            ..strokeWidth = hov ? 2 : 1
            ..color = c.withAlpha(hov ? 240 : 160));
      if (hov) {
        canvas.drawRRect(RRect.fromRectAndRadius(rect.inflate(1), const Radius.circular(4)),
            Paint()
              ..style = PaintingStyle.stroke
              ..strokeWidth = 4
              ..color = c.withAlpha(80)
              ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 4));
      }

      if (w > 28) {
        final tp = TextPainter(
          text: TextSpan(text: seg.$3,
              style: TextStyle(fontFamily: Gx.fontMono, fontSize: 8,
                  color: c.withAlpha(hov ? 255 : 200))),
          textDirection: TextDirection.ltr,
        )..layout(maxWidth: w);
        if (tp.width < w - 4) {
          tp.paint(canvas, Offset(x + (w - tp.width) / 2, barY + (barH - tp.height) / 2));
        }
      }
      x += seg.$1 * size.width;
    }
  }

  @override
  bool shouldRepaint(RegimeTimelinePainter old) => old.hover != hover;
}

// ---------------------------------------------------------------------------
// OptimizationContourPainter — fitness landscape 2D
// ---------------------------------------------------------------------------
class OptimizationContourPainter extends CustomPainter {
  final Offset? hover;
  OptimizationContourPainter({this.hover});

  static final _rng = Random(99);
  static final _data = List.generate(256, (i) {
    final row = i ~/ 16;
    final col = i % 16;
    final dr = (row - 6).toDouble();
    final dc = (col - 8).toDouble();
    final v = exp(-(dr * dr + dc * dc) / 12) + _rng.nextDouble() * 0.08;
    return v.clamp(0.0, 1.0);
  });

  @override
  void paint(Canvas canvas, Size size) {
    const n = 16;
    final cellW = size.width / n;
    final cellH = size.height / n;

    final hovCol = hover != null ? (hover!.dx / cellW).floor().clamp(0, n - 1) : -1;
    final hovRow = hover != null ? (hover!.dy / cellH).floor().clamp(0, n - 1) : -1;

    for (var r = 0; r < n; r++) {
      for (var c = 0; c < n; c++) {
        final v = _data[r * n + c];
        final color = _colorFor(v);
        final hov = r == hovRow || c == hovCol;
        final alpha = hov
            ? (v * 230 + 30).round().clamp(30, 255)
            : (v * 200 + 25).round().clamp(25, 225);
        canvas.drawRect(
          Rect.fromLTWH(c * cellW, r * cellH, cellW - 0.5, cellH - 0.5),
          Paint()..color = color.withAlpha(alpha),
        );
        if (r == hovRow && c == hovCol) {
          canvas.drawRect(
            Rect.fromLTWH(c * cellW, r * cellH, cellW, cellH),
            Paint()
              ..style = PaintingStyle.stroke
              ..strokeWidth = 1.5
              ..color = color.withAlpha(220),
          );
        }
      }
    }

    // Crosshair en hover.
    if (hover != null) {
      canvas.drawLine(Offset(hover!.dx, 0), Offset(hover!.dx, size.height),
          Paint()..color = Gx.textMuted.withAlpha(40)..strokeWidth = 0.5);
      canvas.drawLine(Offset(0, hover!.dy), Offset(size.width, hover!.dy),
          Paint()..color = Gx.textMuted.withAlpha(40)..strokeWidth = 0.5);
    }

    // Punto óptimo.
    final maxIdx = _data.indexOf(_data.reduce(max));
    final optRow = maxIdx ~/ n;
    final optCol = maxIdx % n;
    final cx = optCol * cellW + cellW / 2;
    final cy = optRow * cellH + cellH / 2;
    canvas.drawCircle(Offset(cx, cy), 9, Paint()
      ..color = Gx.optimaCyan.withAlpha(80)
      ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 10));
    canvas.drawCircle(Offset(cx, cy), 4, Paint()..color = Gx.optimaCyan);
  }

  static Color _colorFor(double v) {
    if (v > 0.65) return Gx.optimaCyan;
    if (v > 0.35) return Gx.transitionIndigo;
    if (v > 0.15) return Gx.alertAmber;
    return Gx.criticalCrimson;
  }

  @override
  bool shouldRepaint(OptimizationContourPainter old) => old.hover != hover;
}
