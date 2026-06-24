// Sección §10 Data-viz extendida — heatmap, scatter UMAP, regime-map,
// parallel-coordinates, correlation-matrix, drawdown-curve.
// Todo render-only con CustomPainter nativo; sin lógica financiera en Dart.

import 'dart:math';
import 'package:flutter/material.dart';
import '../gallery_tokens.dart';

// ---------------------------------------------------------------------------
// Heatmap — mapa de calor 7×7 con gradiente semántico
// ---------------------------------------------------------------------------

// Cuadrícula de 7×7 celdas. El valor de cada celda determina su color:
// desde cian óptimo (calor alto) hasta carmesí crítico (frío/negativo).
// Datos hardcodeados; el gradiente es semántico (estado, no decoración).
class HeatmapPainter extends CustomPainter {
  // Semilla reproducible para la demo.
  static final _rng = Random(42);
  // Grilla 7×7 de valores entre -1 y 1.
  static final _data = List.generate(
      49, (_) => (_rng.nextDouble() * 2 - 1));

  @override
  void paint(Canvas canvas, Size size) {
    const cols = 7;
    const rows = 7;
    final cellW = size.width / cols;
    final cellH = size.height / rows;
    final paint = Paint();

    for (var r = 0; r < rows; r++) {
      for (var c = 0; c < cols; c++) {
        final v = _data[r * cols + c]; // entre -1 y 1
        // Interpola color: -1 = carmesí, 0 = índigo, 1 = cian.
        final color = _semanticColor(v);
        paint.color = color.withAlpha(180);
        final rect = Rect.fromLTWH(
            c * cellW + 1, r * cellH + 1, cellW - 2, cellH - 2);
        canvas.drawRRect(
            RRect.fromRectAndRadius(rect, const Radius.circular(3)),
            paint);
      }
    }
  }

  // Retorna el color semántico para un valor entre -1 (crítico) y 1 (óptimo).
  static Color _semanticColor(double v) {
    if (v > 0.3) return Gx.optimaCyan;
    if (v > -0.3) return Gx.transitionIndigo;
    return Gx.criticalCrimson;
  }

  @override
  bool shouldRepaint(HeatmapPainter old) => false;
}

// ---------------------------------------------------------------------------
// Scatter UMAP / PCA — dispersión de clústeres 2D
// ---------------------------------------------------------------------------

// Nube de puntos coloreados por estado semántico, sobre fondo deepSpace.
// Simula la proyección UMAP/PCA de estrategias; sin cálculo real en Dart.
class ScatterPainter extends CustomPainter {
  static final _rng = Random(7);
  // 30 puntos (posición relativa + estado hardcodeado).
  static final _pts = List.generate(30, (i) {
    final x = _rng.nextDouble() * 0.9 + 0.05;
    final y = _rng.nextDouble() * 0.9 + 0.05;
    final state = i % 4; // 0=óptimo, 1=transición, 2=alerta, 3=crítico
    return (x, y, state);
  });

  static const _colors = [
    Gx.optimaCyan,
    Gx.transitionIndigo,
    Gx.alertAmber,
    Gx.criticalCrimson,
  ];

  @override
  void paint(Canvas canvas, Size size) {
    for (final pt in _pts) {
      final c = _colors[pt.$3];
      final paint = Paint()
        ..color = c.withAlpha(200)
        ..style = PaintingStyle.fill;
      final glowPaint = Paint()
        ..color = c.withAlpha(80)
        ..style = PaintingStyle.fill
        ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 6);
      final center = Offset(pt.$1 * size.width, pt.$2 * size.height);
      // Halo de glow.
      canvas.drawCircle(center, 7, glowPaint);
      // Punto central.
      canvas.drawCircle(center, 4, paint);
    }
  }

  @override
  bool shouldRepaint(ScatterPainter old) => false;
}

// ---------------------------------------------------------------------------
// Regime Map — mapa de régimen (no línea de tiempo; bloques de estado)
// ---------------------------------------------------------------------------

// Muestra franjas horizontales de régimen a lo largo del tiempo.
// Cada franja es proporcional a su duración (hardcodeada).
class RegimeMapPainter extends CustomPainter {
  // Segmentos: (fracción del ancho, color de estado).
  static const _segments = [
    (0.3, Gx.optimaCyan),        // Tendencia 30%
    (0.15, Gx.transitionIndigo), // Calmo 15%
    (0.2, Gx.alertAmber),        // Volátil 20%
    (0.25, Gx.optimaCyan),       // Tendencia 25%
    (0.1, Gx.criticalCrimson),   // Fallo 10%
  ];

  @override
  void paint(Canvas canvas, Size size) {
    double x = 0;
    for (final seg in _segments) {
      final w = seg.$1 * size.width;
      final paint = Paint()..color = seg.$2;
      final rect = Rect.fromLTWH(x, 0, w - 2, size.height);
      canvas.drawRRect(
          RRect.fromRectAndRadius(rect, const Radius.circular(3)), paint);

      // Glow bajo cada franja.
      final glowPaint = Paint()
        ..color = seg.$2.withAlpha(80)
        ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 8);
      canvas.drawRRect(
          RRect.fromRectAndRadius(rect, const Radius.circular(3)), glowPaint);
      x += w;
    }
  }

  @override
  bool shouldRepaint(RegimeMapPainter old) => false;
}

// ---------------------------------------------------------------------------
// Parallel Coordinates — ejes paralelos de métricas
// ---------------------------------------------------------------------------

// Renderiza 3 ejes verticales con líneas de datos por encima,
// coloreadas por estado semántico de la estrategia.
class ParallelCoordPainter extends CustomPainter {
  static const _axes = ['Sharpe', 'DD', 'Slip'];
  static const _series = [
    // (sharpe%, drawdown%, slippage%) + color de estado
    (0.8, 0.2, 0.1, Gx.optimaCyan),
    (0.4, 0.6, 0.5, Gx.alertAmber),
    (0.1, 0.9, 0.8, Gx.criticalCrimson),
    (0.6, 0.3, 0.2, Gx.transitionIndigo),
  ];

  @override
  void paint(Canvas canvas, Size size) {
    final axisCount = _axes.length;
    final step = size.width / (axisCount - 1);
    final axisPaint = Paint()
      ..color = Gx.borderPanel
      ..strokeWidth = 1;
    final labelStyle = TextStyle(
      color: Gx.textLabel,
      fontSize: 10,
      fontFamily: 'monospace',
    );

    // Dibuja los ejes verticales.
    for (var i = 0; i < axisCount; i++) {
      final x = i * step;
      canvas.drawLine(Offset(x, 20), Offset(x, size.height - 20), axisPaint);

      // Etiqueta del eje.
      final tp = TextPainter(
        text: TextSpan(text: _axes[i], style: labelStyle),
        textDirection: TextDirection.ltr,
      )..layout();
      tp.paint(canvas, Offset(x - tp.width / 2, 4));
    }

    // Dibuja las líneas de cada serie.
    for (final s in _series) {
      final values = [s.$1, s.$2, s.$3];
      final path = Path();
      for (var i = 0; i < axisCount; i++) {
        final x = i * step;
        final y = 20 + (1 - values[i]) * (size.height - 40);
        if (i == 0) {
          path.moveTo(x, y);
        } else {
          path.lineTo(x, y);
        }
      }
      final linePaint = Paint()
        ..color = s.$4.withAlpha(160)
        ..strokeWidth = 1.5
        ..style = PaintingStyle.stroke;
      // Halo de glow.
      final glowPaint = Paint()
        ..color = s.$4.withAlpha(60)
        ..strokeWidth = 4
        ..style = PaintingStyle.stroke
        ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 4);
      canvas.drawPath(path, glowPaint);
      canvas.drawPath(path, linePaint);
    }
  }

  @override
  bool shouldRepaint(ParallelCoordPainter old) => false;
}

// ---------------------------------------------------------------------------
// Correlation Matrix — matriz 4×4 con intensidad de color por correlación
// ---------------------------------------------------------------------------

// Cuadrícula 4×4 de activos; el color de cada celda representa la
// correlación entre el par (datos hardcodeados, rango -1 a 1).
class CorrelationMatrixPainter extends CustomPainter {
  static const _labels = ['SPX', 'QQQ', 'GLD', 'DXY'];
  // Matriz triangular superior simétrica, hardcodeada.
  static const _corr = [
    [1.0, 0.9, -0.3, -0.5],
    [0.9, 1.0, -0.2, -0.4],
    [-0.3, -0.2, 1.0, 0.1],
    [-0.5, -0.4, 0.1, 1.0],
  ];

  @override
  void paint(Canvas canvas, Size size) {
    final n = _labels.length;
    final cell = min(size.width, size.height) / (n + 1);
    final offset = cell; // primera fila/columna para etiquetas
    final labelStyle = TextStyle(
      color: Gx.textLabel,
      fontSize: 10,
      fontFamily: 'monospace',
    );
    final paint = Paint();

    // Etiquetas de fila y columna.
    for (var i = 0; i < n; i++) {
      final tp = TextPainter(
        text: TextSpan(text: _labels[i], style: labelStyle),
        textDirection: TextDirection.ltr,
      )..layout();
      tp.paint(canvas,
          Offset(offset + i * cell + (cell - tp.width) / 2, 4));
      tp.paint(canvas,
          Offset(4, offset + i * cell + (cell - tp.height) / 2));
    }

    // Celdas de la matriz.
    for (var r = 0; r < n; r++) {
      for (var c = 0; c < n; c++) {
        final v = _corr[r][c]; // entre -1 y 1
        final color = _corrColor(v);
        paint.color = color.withAlpha(180);
        final rect = Rect.fromLTWH(
            offset + c * cell + 1,
            offset + r * cell + 1,
            cell - 2,
            cell - 2);
        canvas.drawRRect(
            RRect.fromRectAndRadius(rect, const Radius.circular(3)),
            paint);
      }
    }
  }

  // Interpola color según correlación: 1=cian, 0=índigo, -1=carmesí.
  static Color _corrColor(double v) {
    if (v > 0.5) return Gx.optimaCyan;
    if (v > 0) return Gx.transitionIndigo;
    if (v > -0.5) return Gx.alertAmber;
    return Gx.criticalCrimson;
  }

  @override
  bool shouldRepaint(CorrelationMatrixPainter old) => false;
}

// ---------------------------------------------------------------------------
// Drawdown Curve — curva de drawdown con coloración por severidad
// ---------------------------------------------------------------------------

// Línea de drawdown (%) a lo largo del tiempo; la zona bajo la curva se
// colorea con el gradiente de la familia crítica.
class DrawdownCurvePainter extends CustomPainter {
  final Offset? hover;
  DrawdownCurvePainter({this.hover});

  static const _pts = [
    0.0, 0.05, 0.08, 0.04, 0.12, 0.09, 0.18, 0.14, 0.08, 0.22,
    0.28, 0.20, 0.15, 0.30, 0.25, 0.18, 0.12, 0.08, 0.04, 0.0,
  ];

  @override
  void paint(Canvas canvas, Size size) {
    final n = _pts.length;
    final stepX = size.width / (n - 1);
    const zeroY = 8.0;
    final h = size.height - zeroY - 4;
    final isHov = hover != null;

    canvas.drawLine(Offset(0, zeroY), Offset(size.width, zeroY),
        Paint()..color = Gx.borderPanel..strokeWidth = 1);

    final path = Path()..moveTo(0, zeroY);
    for (var i = 0; i < n; i++) {
      path.lineTo(i * stepX, zeroY + _pts[i] * h);
    }

    final fill = Path.from(path)..lineTo(size.width, zeroY)..close();
    canvas.drawPath(fill, Paint()
      ..shader = LinearGradient(
        begin: Alignment.topCenter, end: Alignment.bottomCenter,
        colors: [Gx.criticalCrimson.withAlpha(isHov ? 100 : 80), Gx.criticalCrimson.withAlpha(20)],
      ).createShader(Rect.fromLTWH(0, 0, size.width, size.height)));

    canvas.drawPath(path, Paint()
      ..color = Gx.criticalRed.withAlpha(isHov ? 160 : 120)
      ..strokeWidth = isHov ? 5 : 3
      ..style = PaintingStyle.stroke
      ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 4));
    canvas.drawPath(path, Paint()
      ..color = Gx.criticalRed
      ..strokeWidth = isHov ? 2.5 : 1.5
      ..style = PaintingStyle.stroke);

    if (hover != null) {
      final idx = (hover!.dx / stepX).round().clamp(0, n - 1);
      final cy = zeroY + _pts[idx] * h;
      canvas.drawLine(Offset(hover!.dx, 0), Offset(hover!.dx, size.height),
          Paint()..color = Gx.textMuted.withAlpha(50)..strokeWidth = 0.5);
      canvas.drawCircle(Offset(idx * stepX, cy), 6, Paint()
        ..color = Gx.criticalCrimson.withAlpha(70)
        ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 5));
      canvas.drawCircle(Offset(idx * stepX, cy), 3, Paint()..color = Gx.textPrimary.withAlpha(220));
    }
  }

  @override
  bool shouldRepaint(DrawdownCurvePainter old) => old.hover != hover;
}
