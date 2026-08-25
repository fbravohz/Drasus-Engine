// Painters nativos de la galería (CustomPainter/Canvas sobre GPU Impeller).
// Pixel a pixel: telón cósmico ESTÁTICO de ambiente (sin animación), líneas de
// grafo con glow y cono de Monte Carlo con glow. La inspiración supernova/disco
// de acreción vive sobre todo en el glow y los gradientes de los componentes
// (gallery_fx.dart), no en un fondo animado. Prohibido SVG/WebView (ADR-0097).

import 'dart:math';
import 'package:flutter/material.dart';
import '../theme/gx_tokens.dart';

// Telón cósmico estático: resplandor violeta (supernova) + tenue disco de
// acreción + campo de estrellas. Es ambiente sutil; jamás compite con los datos.
class CosmicBackdropPainter extends CustomPainter {
  final Color deepSpace;
  final Color supernovaColor;

  const CosmicBackdropPainter({
    this.deepSpace = Gx.deepSpace,
    this.supernovaColor = Gx.transitionIndigo,
  });

  @override
  void paint(Canvas canvas, Size size) {
    final rect = Offset.zero & size;
    canvas.drawRect(rect, Paint()..color = deepSpace);

    final center = Offset(size.width * 0.5, size.height * 0.34);
    final maxR = size.shortestSide * 0.75;

    // Núcleo violeta (supernova), muy tenue.
    canvas.drawCircle(
      center,
      maxR,
      Paint()
        ..shader = RadialGradient(
          colors: [
            supernovaColor.withOpacity(0.16),
            Gx.transitionPurple.withOpacity(0.06),
            Colors.transparent,
          ],
          stops: const [0.0, 0.4, 1.0],
        ).createShader(Rect.fromCircle(center: center, radius: maxR)),
    );

    // Disco de acreción: anillo elíptico con barrido de color, desenfocado.
    final diskRect = Rect.fromCenter(
        center: center, width: maxR * 1.9, height: maxR * 1.0);
    canvas.drawOval(
      diskRect,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = maxR * 0.14
        ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 30)
        ..shader = const SweepGradient(
          colors: [
            Color(0x0056A8FF),
            Color(0x559A8CFF),
            Color(0x4454E8D0),
            Color(0x668B83E8),
            Color(0x0056A8FF),
          ],
        ).createShader(diskRect),
    );

    // Campo de estrellas estático (semilla fija).
    final rnd = Random(7);
    final star = Paint();
    for (var i = 0; i < 220; i++) {
      final dx = rnd.nextDouble() * size.width;
      final dy = rnd.nextDouble() * size.height;
      final r = rnd.nextDouble() * 1.2 + 0.3;
      star.color = Gx.starField.withOpacity(rnd.nextDouble() * 0.05 + 0.01);
      canvas.drawCircle(Offset(dx, dy), r, star);
    }
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) => false;
}

// Posiciones de los nodos del DAG en función del tamaño (compartidas entre el
// painter de líneas y los nodos interactivos del widget).
List<Offset> dagNodes(Size s) => [
      Offset(s.width * 0.10, s.height * 0.50),
      Offset(s.width * 0.38, s.height * 0.24),
      Offset(s.width * 0.38, s.height * 0.76),
      Offset(s.width * 0.66, s.height * 0.50),
      Offset(s.width * 0.90, s.height * 0.50),
    ];

const dagEdges = [
  [0, 1],
  [0, 2],
  [1, 3],
  [2, 3],
  [3, 4],
];

// Líneas del DAG con halo de glow. Si [hovered] apunta a un nodo, sus aristas
// se encienden con más fuerza (feedback de hover).
class DagLinesPainter extends CustomPainter {
  final int? hovered;
  DagLinesPainter(this.hovered);

  @override
  void paint(Canvas canvas, Size size) {
    final nodes = dagNodes(size);
    for (final e in dagEdges) {
      final lit = hovered != null && (e[0] == hovered || e[1] == hovered);
      final c = lit ? Gx.optimaCyan : Gx.transitionIndigo;
      // Halo de glow (línea ancha y desenfocada).
      canvas.drawLine(
          nodes[e[0]],
          nodes[e[1]],
          Paint()
            ..style = PaintingStyle.stroke
            ..strokeWidth = lit ? 9 : 6
            ..color = c.withOpacity(lit ? 0.4 : 0.16)
            ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 6));
      // Línea núcleo.
      canvas.drawLine(
          nodes[e[0]],
          nodes[e[1]],
          Paint()
            ..style = PaintingStyle.stroke
            ..strokeWidth = 2
            ..color = c.withOpacity(lit ? 1.0 : 0.7));
    }
  }

  @override
  bool shouldRepaint(covariant DagLinesPainter old) => old.hovered != hovered;
}

// Cono de Monte Carlo (Sobre de Expectativa): relleno con degradado y la
// trayectoria central dibujada con glow. Responde a hover: línea más gruesa.
class MonteCarloPainter extends CustomPainter {
  final Offset? hover;
  MonteCarloPainter({this.hover});

  @override
  void paint(Canvas canvas, Size size) {
    final mid = size.height / 2;
    final isHov = hover != null;

    final cone = Path()
      ..moveTo(0, mid)
      ..lineTo(size.width, mid - size.height * 0.42)
      ..lineTo(size.width, mid + size.height * 0.42)
      ..close();
    canvas.drawPath(
      cone,
      Paint()
        ..shader = LinearGradient(
          colors: [
            Gx.optimaTeal.withOpacity(isHov ? 0.08 : 0.05),
            Gx.optimaCyan.withOpacity(isHov ? 0.30 : 0.22),
          ],
        ).createShader(Offset.zero & size),
    );

    const pts = [0.10, 0.05, 0.12, 0.0, -0.08, 0.04, 0.15];
    final traj = Path()..moveTo(0, mid);
    for (var i = 0; i < pts.length; i++) {
      final x = size.width * (i + 1) / pts.length;
      final y = mid - size.height * pts[i];
      traj.lineTo(x, y);
    }
    // Relleno de área bajo la trayectoria central.
    final fillTraj = Path.from(traj)
      ..lineTo(size.width, mid)
      ..close();
    canvas.drawPath(fillTraj, Paint()
      ..shader = LinearGradient(
        begin: Alignment.topCenter, end: Alignment.bottomCenter,
        colors: [Gx.optimaCyan.withAlpha(isHov ? 22 : 14), Colors.transparent],
      ).createShader(Offset.zero & size));

    canvas.drawPath(traj, Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = isHov ? 8 : 5
      ..color = Gx.optimaCyan.withAlpha(isHov ? 80 : 50)
      ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 4));
    canvas.drawPath(traj, Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = isHov ? 2.5 : 1.5
      ..color = Gx.optimaCyan);

    // Cursor vertical en hover.
    if (hover != null) {
      canvas.drawLine(Offset(hover!.dx, 0), Offset(hover!.dx, size.height),
          Paint()..color = Gx.textMuted.withAlpha(50)..strokeWidth = 0.5);
    }
  }

  @override
  bool shouldRepaint(MonteCarloPainter old) => old.hover != hover;
}
