// Shell del Dashboard de Drasus Engine.
// Muestra el lienzo vacío del tablero con celdas bento-grid glassmorfismo,
// y un FAB que abre el catálogo de widgets disponibles (actualmente todos
// "Próximamente"). No contiene lógica de negocio: es pura presentación.

import 'dart:ui' as ui;
import 'package:flutter/material.dart';
import 'package:iconsax_plus/iconsax_plus.dart';
import '../gallery/gallery_tokens.dart';
import '../gallery/gallery_painters.dart';
import '../drasus_theme.dart';
import 'dashboard_registry.dart';

// ---------------------------------------------------------------------------
// DashboardTab — pestaña principal del tablero.
// ---------------------------------------------------------------------------

// Muestra un header con título + subtítulo, una bento-grid de 6 celdas
// placeholder y un FAB para abrir el catálogo de widgets.
class DashboardTab extends StatelessWidget {
  const DashboardTab({super.key});

  @override
  Widget build(BuildContext context) {
    // deepSpace de la paleta activa; fallback al bunker si el tema no está montado.
    final surfaces = DrasusTheme.surfaceFor(context);
    final bg = surfaces?.deepSpace ?? Gx.deepSpace;

    return Scaffold(
      // Fondo del deepSpace de la paleta activa.
      backgroundColor: bg,
      body: Stack(
        children: [
          // Telón cósmico: da contenido al blur del vidrio Apple.
          Positioned.fill(child: CustomPaint(painter: CosmicBackdropPainter())),
          Padding(
            padding: const EdgeInsets.all(24),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                // ---- Header ----
                Text('Dashboard', style: Gx.displayGrotesque(fontSize: 22, color: Gx.textPrimary)),
                const SizedBox(height: 4),
                Text(
                  'Sin widgets activos — próximamente',
                  style: Gx.uiSans(fontSize: 13, color: Gx.textMuted),
                ),
                const SizedBox(height: 24),

                // ---- Bento-grid glassmorfismo ----
                Expanded(
                  child: SingleChildScrollView(
                    child: _BentoGrid(),
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
      // FAB en la esquina inferior derecha: abre el catálogo de widgets.
      floatingActionButton: FloatingActionButton(
        onPressed: () => _showWidgetCatalog(context),
        // Fondo con gradiente reactorGreen→optimaCyan más glow.
        backgroundColor: Colors.transparent,
        elevation: 0,
        child: Container(
          width: 56,
          height: 56,
          decoration: BoxDecoration(
            gradient: Gx.linear(Gx.gradReactor),
            shape: BoxShape.circle,
            boxShadow: Gx.glowStrong(Gx.reactorGreen),
          ),
          child: const Icon(IconsaxPlusLinear.add_square, color: Gx.deepSpace, size: 24),
        ),
      ),
    );
  }

  // Abre el BottomSheet de catálogo de widgets de vidrio Apple.
  void _showWidgetCatalog(BuildContext context) {
    showModalBottomSheet(
      context: context,
      // isScrollControlled permite que el sheet alcance 400px exactos.
      isScrollControlled: true,
      backgroundColor: Colors.transparent,
      builder: (_) => const _WidgetCatalogSheet(),
    );
  }
}

// ---------------------------------------------------------------------------
// _BentoGrid — layout bento variado con glassmorfismo Apple real.
// ---------------------------------------------------------------------------

// Celda de vidrio con BackdropFilter blur 40 + fill blanco 13% + borde 20%.
// Muestra ícono + título + subtítulo "Sin datos — próximamente".
class GlassBentoCard extends StatelessWidget {
  final IconData icon;
  final String title;
  final double height;

  const GlassBentoCard({
    super.key,
    required this.icon,
    required this.title,
    this.height = 200,
  });

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      height: height,
      child: ClipRRect(
        borderRadius: BorderRadius.circular(16),
        child: BackdropFilter(
          filter: ui.ImageFilter.blur(sigmaX: 40, sigmaY: 40),
          child: Container(
            decoration: BoxDecoration(
              color: const Color(0x22FFFFFF),
              borderRadius: BorderRadius.circular(16),
              border: Border.all(
                color: const Color(0x33FFFFFF),
                width: 1.0,
              ),
              boxShadow: [
                BoxShadow(
                  color: Colors.black.withOpacity(0.15),
                  blurRadius: 20,
                  offset: const Offset(0, 8),
                ),
              ],
            ),
            child: Padding(
              padding: const EdgeInsets.all(20),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Icon(icon, size: 22, color: Gx.textSecondary),
                  const SizedBox(height: 10),
                  Text(
                    title,
                    style: const TextStyle(
                      fontFamily: 'Rajdhani',
                      fontSize: 15,
                      fontWeight: FontWeight.w600,
                      color: Color(0xFFE6ECF8),
                      letterSpacing: 0.5,
                    ),
                  ),
                  const Spacer(),
                  Text(
                    'Sin datos — próximamente',
                    style: Gx.uiSans(fontSize: 11, color: Gx.textMuted),
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}

// Layout bento: dos columnas con alturas variadas.
// flex:2 (izquierda) — Portafolio Principal + fila Rendimiento/Drawdown.
// flex:1 (derecha) — Estrategias Activas + Pipeline.
class _BentoGrid extends StatelessWidget {
  const _BentoGrid();

  @override
  Widget build(BuildContext context) {
    return IntrinsicHeight(
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // ---- Columna izquierda (flex 2) ----
          Expanded(
            flex: 2,
            child: Column(
              children: [
                GlassBentoCard(
                  icon: IconsaxPlusLinear.chart,
                  title: 'Portafolio Principal',
                  height: 280,
                ),
                const SizedBox(height: 12),
                Row(
                  children: const [
                    Expanded(
                      child: GlassBentoCard(
                        icon: IconsaxPlusLinear.chart_1,
                        title: 'Rendimiento',
                        height: 180,
                      ),
                    ),
                    SizedBox(width: 12),
                    Expanded(
                      child: GlassBentoCard(
                        icon: IconsaxPlusLinear.warning_2,
                        title: 'Drawdown',
                        height: 180,
                      ),
                    ),
                  ],
                ),
              ],
            ),
          ),
          const SizedBox(width: 12),
          // ---- Columna derecha (flex 1) ----
          Expanded(
            flex: 1,
            child: Column(
              children: const [
                GlassBentoCard(
                  icon: IconsaxPlusLinear.element_1,
                  title: 'Estrategias Activas',
                  height: 240,
                ),
                SizedBox(height: 12),
                GlassBentoCard(
                  icon: IconsaxPlusLinear.element,
                  title: 'Pipeline',
                  height: 220,
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// _PlaceholderCell — celda bento vacía con borde punteado.
// ---------------------------------------------------------------------------

// 240×160px. Muestra un ícono "+" y el texto "Sin widget" en el centro.
// El borde es punteado 1.5px con patrón 6/4, pintado via CustomPainter
// porque BoxDecoration no soporta dash nativo.
class _PlaceholderCell extends StatelessWidget {
  const _PlaceholderCell();

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: 240,
      height: 160,
      child: CustomPaint(
        painter: _DashedBorderPainter(
          color: Gx.borderPanel,
          strokeWidth: 1.5,
          dashWidth: 6,
          gapWidth: 4,
          radius: Gx.rPanel,
        ),
        child: Center(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              const Icon(IconsaxPlusLinear.add, size: 24, color: Gx.textMuted),
              const SizedBox(height: 6),
              Text('Sin widget', style: Gx.uiSans(fontSize: 12, color: Gx.textMuted)),
            ],
          ),
        ),
      ),
    );
  }
}

// Pinta un rectángulo redondeado con borde punteado sobre el canvas.
// dashWidth: longitud de cada trazo. gapWidth: longitud del hueco entre trazos.
// radius: radio de las esquinas redondeadas.
class _DashedBorderPainter extends CustomPainter {
  final Color color;
  final double strokeWidth;
  final double dashWidth;
  final double gapWidth;
  final double radius;

  const _DashedBorderPainter({
    required this.color,
    required this.strokeWidth,
    required this.dashWidth,
    required this.gapWidth,
    required this.radius,
  });

  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()
      ..color = color
      ..strokeWidth = strokeWidth
      ..style = PaintingStyle.stroke;

    // Construimos el path del rectángulo redondeado.
    final path = Path()
      ..addRRect(RRect.fromRectAndRadius(
        Rect.fromLTWH(0, 0, size.width, size.height),
        Radius.circular(radius),
      ));

    // Extraemos la longitud total del contorno para avanzar con dashWidth/gapWidth.
    final metrics = path.computeMetrics().toList();
    for (final metric in metrics) {
      var distance = 0.0;
      while (distance < metric.length) {
        // Extraemos un segmento de longitud dashWidth y lo dibujamos.
        final extractPath = metric.extractPath(
          distance,
          (distance + dashWidth).clamp(0, metric.length),
        );
        canvas.drawPath(extractPath, paint);
        // Avanzamos el puntero un trazo + un hueco.
        distance += dashWidth + gapWidth;
      }
    }
  }

  @override
  bool shouldRepaint(_DashedBorderPainter old) =>
      old.color != color ||
      old.strokeWidth != strokeWidth ||
      old.dashWidth != dashWidth ||
      old.gapWidth != gapWidth;
}

// ---------------------------------------------------------------------------
// _WidgetCatalogSheet — BottomSheet de vidrio con el catálogo de widgets.
// ---------------------------------------------------------------------------

// Altura fija 400px. Vidrio Apple: BackdropFilter blur 36 + glassFill.
// Lista los widgets del kDashboardRegistry; todos muestran el chip "Próximamente".
class _WidgetCatalogSheet extends StatelessWidget {
  const _WidgetCatalogSheet();

  @override
  Widget build(BuildContext context) {
    return ClipRRect(
      borderRadius: const BorderRadius.vertical(top: Radius.circular(20)),
      child: BackdropFilter(
        filter: ui.ImageFilter.blur(sigmaX: 36, sigmaY: 36),
        child: Container(
          height: 400,
          decoration: BoxDecoration(
            color: Gx.glassFill,
            borderRadius: const BorderRadius.vertical(top: Radius.circular(20)),
            border: Border(
              top: BorderSide(color: Gx.textPrimary.withOpacity(Gx.glassEdgeOpacity), width: 1),
            ),
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              // Tirador visual del sheet.
              Center(
                child: Padding(
                  padding: const EdgeInsets.only(top: 12),
                  child: Container(
                    width: 36,
                    height: 4,
                    decoration: BoxDecoration(
                      color: Gx.borderPanel,
                      borderRadius: BorderRadius.circular(2),
                    ),
                  ),
                ),
              ),
              Padding(
                padding: const EdgeInsets.fromLTRB(20, 16, 20, 8),
                child: Text(
                  'Agregar Widget',
                  style: Gx.displayGrotesque(fontSize: 18, color: Gx.textPrimary),
                ),
              ),
              // Lista scrolleable de entradas del catálogo.
              Expanded(
                child: ListView.separated(
                  padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
                  itemCount: kDashboardRegistry.length,
                  separatorBuilder: (_, __) => const SizedBox(height: 2),
                  itemBuilder: (ctx, i) =>
                      _GlowWidgetCatalogItem(meta: kDashboardRegistry[i]),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// _GlowWidgetCatalogItem — fila del catálogo con ícono, nombre, descripción
// y chip "Próximamente".
// ---------------------------------------------------------------------------

// Sin acción al tap por ahora (available siempre false en el registry inicial).
class _GlowWidgetCatalogItem extends StatelessWidget {
  final DashboardWidgetMeta meta;
  const _GlowWidgetCatalogItem({required this.meta});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 8, horizontal: 4),
      child: Row(
        children: [
          // Ícono del widget en un fondo de vidrio.
          Container(
            width: 36,
            height: 36,
            decoration: BoxDecoration(
              color: Gx.cardInner,
              borderRadius: BorderRadius.circular(Gx.rChip),
              border: Border.all(color: Gx.borderPanel),
            ),
            child: Icon(meta.icon, size: 18, color: Gx.textSecondary),
          ),
          const SizedBox(width: 12),
          // Nombre + descripción del widget.
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(meta.name, style: Gx.uiSans(fontSize: 13, color: Gx.textPrimary, weight: FontWeight.w500)),
                const SizedBox(height: 2),
                Text(meta.description, style: Gx.uiSans(fontSize: 11, color: Gx.textMuted)),
              ],
            ),
          ),
          const SizedBox(width: 8),
          // Chip "Próximamente" — visible cuando el widget no está disponible.
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
            decoration: BoxDecoration(
              color: Gx.cardInner,
              borderRadius: BorderRadius.circular(Gx.rChip),
              border: Border.all(color: Gx.borderPanel),
            ),
            child: Text(
              'Próximamente',
              style: Gx.dataMono(fontSize: 10, color: Gx.textMuted),
            ),
          ),
        ],
      ),
    );
  }
}
