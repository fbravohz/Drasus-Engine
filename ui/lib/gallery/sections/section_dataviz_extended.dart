// Sección §10 Data-viz extendida — heatmap, scatter UMAP, regime-map,
// parallel-coordinates, correlation-matrix, drawdown-curve.
// Todo render-only con CustomPainter nativo; sin lógica financiera en Dart.

import 'dart:math';
import 'package:flutter/material.dart';
import '../../theme/gx_tokens.dart';

// ---------------------------------------------------------------------------
// HeatmapPainter — mapa de calor 7×7 con gradiente semántico.
// Cuadrícula estática de 7×7 celdas; el valor de cada celda (-1…1) determina
// su color semántico (óptimo/transición/crítico). Sin parámetros de entrada.
// Tokens de dato (se conservan): optimaCyan, transitionIndigo, criticalCrimson.
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
  // Renderiza la grilla de 49 celdas con color semántico según el valor de cada celda.
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
// ScatterPainter — dispersión UMAP/PCA de clústeres 2D.
// Simula la proyección de estrategias en 2D con 30 puntos semánticos.
// Sin parámetros de entrada (datos y semilla hardcodeados para demo).
// Tokens de dato (se conservan): optimaCyan, transitionIndigo, alertAmber,
//   criticalCrimson. MaskFilter.blur fuera de bucle de animación → aceptado.
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
  // Dibuja 30 puntos con halo de glow semántico y punto central nítido.
  void paint(Canvas canvas, Size size) {
    for (final pt in _pts) {
      final c = _colors[pt.$3];
      final paint = Paint()
        ..color = c.withAlpha(200)
        ..style = PaintingStyle.fill;
      // Halo de glow: painter estático (no animado) → MaskFilter.blur aceptado.
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
// RegimeMapPainter — mapa de régimen de mercado en franjas horizontales.
// Cada franja es proporcional a su duración; el color es semántico (estado).
// Sin parámetros de entrada (datos hardcodeados para demo).
// Tokens de dato (se conservan): optimaCyan, transitionIndigo, alertAmber,
//   criticalCrimson. MaskFilter.blur fuera de bucle de animación → aceptado.
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
  // Dibuja franjas de régimen con glow bajo cada segmento.
  void paint(Canvas canvas, Size size) {
    double x = 0;
    for (final seg in _segments) {
      final w = seg.$1 * size.width;
      final paint = Paint()..color = seg.$2;
      final rect = Rect.fromLTWH(x, 0, w - 2, size.height);
      canvas.drawRRect(
          RRect.fromRectAndRadius(rect, const Radius.circular(3)), paint);

      // Glow bajo cada franja: painter estático (no animado) → MaskFilter.blur aceptado.
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
// ParallelCoordPainter — coordenadas paralelas de métricas de estrategia.
// Renderiza 3 ejes verticales (Sharpe, DD, Slip) con 4 series coloreadas
// por estado semántico. Sin parámetros de entrada.
// Tokens de chrome: borderBase (ejes estructurales), textBaseLabel (etiquetas
//   de eje), fontMono (tipografía de datos).
// Tokens de dato (se conservan): optimaCyan, alertAmber, criticalCrimson,
//   transitionIndigo.
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
  // Dibuja los ejes, etiquetas y líneas de cada serie con halo de glow.
  void paint(Canvas canvas, Size size) {
    final axisCount = _axes.length;
    final step = size.width / (axisCount - 1);
    // Borde estructural del eje → borderBase (énfasis dinámico al 35% de opacidad).
    final axisPaint = Paint()
      ..color = Gx.borderBase
      ..strokeWidth = 1;
    // Etiqueta del eje: textBaseLabel reacciona a la paleta activa (paper/bunker).
    final labelStyle = TextStyle(
      color: Gx.textBaseLabel,
      fontSize: 10,
      fontFamily: Gx.fontMono,
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

    // Dibuja las líneas de cada serie con halo de glow.
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
      // Halo de glow: painter estático (no animado) → MaskFilter.blur aceptado.
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
// CorrelationMatrixPainter — matriz de correlación 4×4 (SPX/QQQ/GLD/DXY).
// El color de cada celda representa la correlación entre el par (rango -1…1).
// Sin parámetros de entrada (datos hardcodeados para demo).
// Tokens de chrome: textBaseLabel (etiquetas), fontMono (tipografía de datos).
// Tokens de dato (se conservan): optimaCyan, transitionIndigo, alertAmber,
//   criticalCrimson.
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
  // Renderiza etiquetas de fila/columna y celdas de la matriz con color semántico.
  void paint(Canvas canvas, Size size) {
    final n = _labels.length;
    final cell = min(size.width, size.height) / (n + 1);
    final offset = cell; // primera fila/columna para etiquetas
    // Etiqueta de eje: textBaseLabel reacciona a la paleta activa (paper/bunker).
    final labelStyle = TextStyle(
      color: Gx.textBaseLabel,
      fontSize: 10,
      fontFamily: Gx.fontMono,
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
// DrawdownCurvePainter — curva de drawdown (%) con coloración semántica.
// Línea de drawdown a lo largo del tiempo; la zona bajo la curva se colorea
// con el gradiente de la familia crítica. Recibe: Offset? hover (posición
// del cursor; null si no hay hover).
// Tokens de chrome: borderBase (eje cero), textBaseMuted (cursor vertical),
//   textBase (dot de hover).
// Tokens de dato (se conservan): criticalCrimson, criticalRed (familia
//   crítica — codifican severidad del drawdown).
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
  // Dibuja curva de drawdown con relleno semántico y cursor de hover dinámico.
  // MaskFilter.blur condicionado a hover (no en bucle de AnimationController) → aceptado.
  void paint(Canvas canvas, Size size) {
    final n = _pts.length;
    final stepX = size.width / (n - 1);
    const zeroY = 8.0;
    final h = size.height - zeroY - 4;
    final isHov = hover != null;

    // Eje cero: borde estructural → borderBase (énfasis dinámico al 35% de opacidad).
    canvas.drawLine(Offset(0, zeroY), Offset(size.width, zeroY),
        Paint()..color = Gx.borderBase..strokeWidth = 1);

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
      // Cursor vertical: textBaseMuted reacciona a la paleta activa.
      canvas.drawLine(Offset(hover!.dx, 0), Offset(hover!.dx, size.height),
          Paint()..color = Gx.textBaseMuted.withAlpha(50)..strokeWidth = 0.5);
      canvas.drawCircle(Offset(idx * stepX, cy), 6, Paint()
        ..color = Gx.criticalCrimson.withAlpha(70)
        ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 5));
      // Dot central: textBase reacciona a la paleta activa.
      canvas.drawCircle(Offset(idx * stepX, cy), 3, Paint()..color = Gx.textBase.withAlpha(220));
    }
  }

  @override
  bool shouldRepaint(DrawdownCurvePainter old) => old.hover != hover;
}
