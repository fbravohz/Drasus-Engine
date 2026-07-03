// Shell del Dashboard de Drasus Engine.
// Muestra el lienzo vacío del tablero con celdas bento-grid glassmorfismo,
// y un FAB que abre el catálogo de widgets disponibles (actualmente todos
// "Próximamente"). No contiene lógica de negocio: es pura presentación.

import 'package:flutter/material.dart';
import 'package:iconsax_plus/iconsax_plus.dart';
import '../gallery/gallery_tokens.dart';
import '../gallery/gallery_painters.dart';
import '../theme/theme_scope.dart';
import '../components/components.dart' as ui;
import 'dashboard_registry.dart';
import 'sovereign_data_fetcher_dashboard_widget.dart';

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
    final surfaces = ThemeScope.surfaceFor(context);
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
                Text('Dashboard', style: Gx.displayGrotesque(fontSize: 22, color: Gx.textBase)),
                const SizedBox(height: 4),
                Text(
                  'Sin widgets activos — próximamente',
                  style: Gx.uiSans(fontSize: 13, color: Gx.textBaseMuted),
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

  // Abre el catálogo de widgets en un Sheet de vidrio (ui.Sheet vía showAppSheet).
  void _showWidgetCatalog(BuildContext context) {
    ui.showAppSheet(
      context,
      height: 400,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(
                Gx.space16, Gx.space8, Gx.space16, Gx.space8),
            child: Text(
              'Agregar Widget',
              style: Gx.displayGrotesque(fontSize: 18, color: Gx.textBase),
            ),
          ),
          // Lista scrolleable de entradas del catálogo de widgets del dashboard.
          Expanded(
            child: ListView.separated(
              padding: const EdgeInsets.symmetric(
                  horizontal: Gx.space16, vertical: Gx.space4),
              itemCount: kDashboardRegistry.length,
              separatorBuilder: (_, __) => const SizedBox(height: 2),
              itemBuilder: (ctx, i) =>
                  _GlowWidgetCatalogItem(meta: kDashboardRegistry[i]),
            ),
          ),
        ],
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// _BentoGrid — layout bento variado con glassmorfismo reactivo al tema.
// GlassBentoCard migrado a ui.BentoCard (Batch 4 STORY-025).
// ---------------------------------------------------------------------------

// Layout bento: dos columnas con alturas variadas.
// flex:2 (izquierda) — Portafolio Principal + fila Rendimiento/Drawdown.
// flex:1 (derecha) — Estrategias Activas + Pipeline + widget STORY-024.
class _BentoGrid extends StatelessWidget {
  const _BentoGrid();

  @override
  // Construye el layout bento de dos columnas usando ui.BentoCard como celda.
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
                // ui.BentoCard reemplaza GlassBentoCard; no es const (lee Gx dinámico).
                ui.BentoCard(
                  icon: IconsaxPlusLinear.chart,
                  title: 'Portafolio Principal',
                  height: 280,
                ),
                const SizedBox(height: 12),
                Row(
                  children: [
                    Expanded(
                      child: ui.BentoCard(
                        icon: IconsaxPlusLinear.chart_1,
                        title: 'Rendimiento',
                        height: 180,
                      ),
                    ),
                    const SizedBox(width: 12),
                    Expanded(
                      child: ui.BentoCard(
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
              children: [
                ui.BentoCard(
                  icon: IconsaxPlusLinear.element_1,
                  title: 'Estrategias Activas',
                  height: 240,
                ),
                const SizedBox(height: 12),
                ui.BentoCard(
                  icon: IconsaxPlusLinear.element,
                  title: 'Pipeline',
                  height: 220,
                ),
                const SizedBox(height: 12),
                // Widget real del Sovereign Data Fetcher (STORY-024).
                // Muestra el último registro de sovereign_download_records por FFI.
                const SovereignDataFetcherDashboardWidget(),
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
              Icon(IconsaxPlusLinear.add, size: 24, color: Gx.textBaseMuted),
              const SizedBox(height: 6),
              Text('Sin widget', style: Gx.uiSans(fontSize: 12, color: Gx.textBaseMuted)),
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
              color: Gx.surfaceCard,
              borderRadius: BorderRadius.circular(Gx.rChip),
              border: Border.all(color: Gx.borderPanel),
            ),
            child: Icon(meta.icon, size: 18, color: Gx.textBaseSecondary),
          ),
          const SizedBox(width: 12),
          // Nombre + descripción del widget.
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(meta.name, style: Gx.uiSans(fontSize: 13, color: Gx.textBase, weight: FontWeight.w500)),
                const SizedBox(height: 2),
                Text(meta.description, style: Gx.uiSans(fontSize: 11, color: Gx.textBaseMuted)),
              ],
            ),
          ),
          const SizedBox(width: 8),
          // Chip "Próximamente" — visible cuando el widget no está disponible.
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
            decoration: BoxDecoration(
              color: Gx.surfaceCard,
              borderRadius: BorderRadius.circular(Gx.rChip),
              border: Border.all(color: Gx.borderPanel),
            ),
            child: Text(
              'Próximamente',
              style: Gx.dataMono(fontSize: 10, color: Gx.textBaseMuted),
            ),
          ),
        ],
      ),
    );
  }
}
