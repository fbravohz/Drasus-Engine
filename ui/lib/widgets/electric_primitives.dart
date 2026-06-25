// Primitivos eléctricos reutilizables (ADR-0138).
// Funciones puras extraídas de section_dataviz_new.dart para que cualquier
// gráfico de líneas de la galería o de las Cáscaras Delgadas las consuma
// sin duplicarlas. Los valores (decaimiento 8.0, cola 120px, etc.) son los
// canónicos originales — no se inventó ninguno.

import 'dart:math';
import 'package:flutter/material.dart';

// Calcula la intensidad de ignición eléctrica para un punto en X dado el
// progreso actual del scan. Retorna 0.0–1.0 donde 1.0 = máxima ignición
// (el scan acaba de pasar por ese punto). Decae exponencialmente con
// scanDecay (default 8.0 — DrasusMotion.scanDecay).
double electricIntensity(
  double pointX,
  double scanX,
  double width, {
  double decay = 8.0,
}) {
  if (scanX < pointX) return 0.0; // el scan aún no llegó a este punto
  final timeSinceScan = (scanX - pointX) / width;
  return exp(-timeSinceScan * decay).clamp(0.0, 1.0);
}

// Pinta la cola de cometa (comet tail) del scan line: gradiente de 120px
// que va de transparente hasta el color de acento al 25% de opacidad.
// [scanX] posición X actual del scan. [canvasSize] tamaño del lienzo.
void paintCometTail(
  Canvas canvas,
  double scanX,
  Size canvasSize,
  Color accentColor,
) {
  const tailWidth = 120.0;
  final tailLeft = (scanX - tailWidth).clamp(0.0, canvasSize.width);
  final tailRect =
      Rect.fromLTWH(tailLeft, 0, scanX - tailLeft, canvasSize.height);
  if (tailRect.width <= 0) return;
  final tailPaint = Paint()
    ..shader = LinearGradient(
      colors: [Colors.transparent, accentColor.withOpacity(0.25)],
    ).createShader(tailRect);
  canvas.drawRect(tailRect, tailPaint);
}

// Pinta la línea vertical del scanner: halo ancho semitransparente + línea nítida.
// Sin MaskFilter.blur por consistencia con el resto de líneas eléctricas.
void paintScanLine(
  Canvas canvas,
  double scanX,
  double height,
  Color accentColor,
  double opacity,
) {
  if (opacity <= 0) return;
  // Halo sin blur: línea gruesa a baja opacidad.
  canvas.drawLine(
    Offset(scanX, 0),
    Offset(scanX, height),
    Paint()
      ..color = accentColor.withOpacity(0.15 * opacity)
      ..strokeWidth = 20.0,
  );
  // Línea media.
  canvas.drawLine(
    Offset(scanX, 0),
    Offset(scanX, height),
    Paint()
      ..color = accentColor.withOpacity(0.25 * opacity)
      ..strokeWidth = 6.0,
  );
  // Línea nítida central.
  canvas.drawLine(
    Offset(scanX, 0),
    Offset(scanX, height),
    Paint()
      ..color = accentColor.withOpacity(opacity)
      ..strokeWidth = 1.5,
  );
}
