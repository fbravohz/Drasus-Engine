// Shell del Canvas de Drasus Engine.
// Muestra un lienzo interactivo (pan, zoom) donde el usuario puede arrastrar
// features desde el panel lateral y colocarlos como nodos. Los nodos son
// rectangulos vacíos sin lógica de conexión (futura story). No hay lógica
// de negocio: el Canvas es presentación pura.

import 'dart:ui' as ui;
import 'package:flutter/material.dart';
import '../gallery/gallery_tokens.dart';
import '../gallery/gallery_painters.dart';
import '../drasus_theme.dart';

// ---------------------------------------------------------------------------
// CanvasTab — pestaña del canvas con panel lateral + lienzo interactivo.
// ---------------------------------------------------------------------------

// Layout: Row con panel izquierdo de 200px (lista de features draggables)
// y un Expanded que contiene el InteractiveViewer con el canvas.
class CanvasTab extends StatefulWidget {
  const CanvasTab({super.key});

  @override
  State<CanvasTab> createState() => _CanvasTabState();
}

class _CanvasTabState extends State<CanvasTab> {
  // Lista de nodos colocados por el usuario. Cada nodo registra el nombre
  // de la feature y la posición donde fue soltado en el canvas.
  final List<_CanvasNode> _nodes = [];

  // Controlador del zoom para los botones de la toolbar flotante.
  final TransformationController _transformCtrl = TransformationController();

  @override
  void dispose() {
    _transformCtrl.dispose();
    super.dispose();
  }

  // Agrega un nuevo nodo al canvas en la posición local indicada.
  void _addNode(String featureName, Offset localPosition) {
    setState(() {
      _nodes.add(_CanvasNode(label: featureName, position: localPosition));
    });
  }

  // Elimina todos los nodos del canvas.
  void _clearNodes() {
    setState(() => _nodes.clear());
  }

  // Reencuadra el canvas a escala 1:1, centrado en el origen.
  void _fitCanvas() {
    _transformCtrl.value = Matrix4.identity();
  }

  // Aplica un zoom incremental multiplicando la escala actual.
  // factor > 1 → acercar; factor < 1 → alejar.
  // scaleByDouble(sx, sy, sz, sw): escalamos X e Y por factor, Z y W se dejan en 1.
  void _zoom(double factor) {
    final current = _transformCtrl.value.clone();
    current.scaleByDouble(factor, factor, 1.0, 1.0);
    _transformCtrl.value = current;
  }

  @override
  Widget build(BuildContext context) {
    final surfaces = DrasusTheme.surfaceFor(context);
    // panelSolid: color de fondo del panel lateral de features.
    final panelBg = surfaces?.panelSolid ?? Gx.panelSolid;

    return Row(
      children: [
        // ---- Panel lateral izquierdo: lista de features draggables ----
        _FeatureSidePanel(panelBg: panelBg),

        // ---- Canvas principal: InteractiveViewer + DragTarget ----
        Expanded(
          child: Stack(
            children: [
              // DragTarget captura las features soltadas desde el panel lateral.
              DragTarget<String>(
                onAcceptWithDetails: (details) {
                  // Convertimos la posición global de drop a posición local del canvas.
                  final box = context.findRenderObject() as RenderBox?;
                  if (box != null) {
                    final local = box.globalToLocal(details.offset);
                    _addNode(details.data, local);
                  }
                },
                builder: (ctx, candidateData, rejectedData) {
                  // Resaltado tenue cuando hay un elemento en arrastre sobre el canvas.
                  final isDraggingOver = candidateData.isNotEmpty;
                  return _CanvasViewport(
                    controller: _transformCtrl,
                    nodes: _nodes,
                    isDraggingOver: isDraggingOver,
                  );
                },
              ),

              // ---- Breadcrumb flotante (arriba izquierda) ----
              Positioned(
                top: 16,
                left: 16,
                child: _BreadcrumbPill(),
              ),

              // ---- Toolbar flotante (arriba derecha) ----
              Positioned(
                top: 16,
                right: 16,
                child: _CanvasToolbar(
                  onZoomIn: () => _zoom(1.2),
                  onZoomOut: () => _zoom(1 / 1.2),
                  onFit: _fitCanvas,
                  onClear: _clearNodes,
                ),
              ),
            ],
          ),
        ),
      ],
    );
  }
}

// ---------------------------------------------------------------------------
// _CanvasNode — datos de un nodo colocado en el canvas.
// ---------------------------------------------------------------------------
class _CanvasNode {
  final String label;
  final Offset position;
  const _CanvasNode({required this.label, required this.position});
}

// ---------------------------------------------------------------------------
// _CanvasViewport — el lienzo con fondo cósmico + dot-grid + nodos.
// ---------------------------------------------------------------------------

// InteractiveViewer permite pan y zoom entre 0.1× y 5×.
// Sobre el fondo se superpone un CustomPaint de dot-grid y los nodos.
class _CanvasViewport extends StatelessWidget {
  final TransformationController controller;
  final List<_CanvasNode> nodes;
  final bool isDraggingOver;

  const _CanvasViewport({
    required this.controller,
    required this.nodes,
    required this.isDraggingOver,
  });

  @override
  Widget build(BuildContext context) {
    return AnimatedContainer(
      duration: const Duration(milliseconds: 200),
      decoration: BoxDecoration(
        // Indicador visual de zona de soltar: borde tenue cuando se arrastra.
        border: isDraggingOver
            ? Border.all(color: Gx.transitionIndigo.withOpacity(0.4), width: 1.5)
            : null,
      ),
      child: InteractiveViewer(
        transformationController: controller,
        minScale: 0.1,
        maxScale: 5.0,
        boundaryMargin: const EdgeInsets.all(double.infinity),
        child: SizedBox(
          // Canvas interno de 4000×4000px para que haya espacio para nodos.
          width: 4000,
          height: 4000,
          child: Stack(
            children: [
              // Fondo cósmico (CosmicBackdropPainter de gallery_painters.dart).
              Positioned.fill(
                child: CustomPaint(
                  painter: CosmicBackdropPainter(),
                ),
              ),
              // Dot-grid sobre el fondo: puntos 1.5px en borderPanel cada 20px.
              Positioned.fill(
                child: CustomPaint(
                  painter: _DotGridPainter(),
                ),
              ),
              // Nodos colocados por el usuario (rectangulos de feature).
              ...nodes.map((node) => Positioned(
                    left: node.position.dx,
                    top: node.position.dy,
                    child: _FeatureNodeCard(label: node.label),
                  )),
            ],
          ),
        ),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// _DotGridPainter — cuadrícula de puntos 1.5px spaced a 20px.
// ---------------------------------------------------------------------------

// El color es borderPanel (hairline sutil, no distrae del contenido).
class _DotGridPainter extends CustomPainter {
  @override
  void paint(Canvas canvas, Size size) {
    const spacing = 20.0;
    const dotRadius = 0.75; // 1.5px de diámetro
    final paint = Paint()..color = Gx.borderPanel;

    for (var x = 0.0; x < size.width; x += spacing) {
      for (var y = 0.0; y < size.height; y += spacing) {
        canvas.drawCircle(Offset(x, y), dotRadius, paint);
      }
    }
  }

  @override
  bool shouldRepaint(_DotGridPainter old) => false;
}

// ---------------------------------------------------------------------------
// _FeatureNodeCard — nodo 240×80px que representa una feature en el canvas.
// ---------------------------------------------------------------------------

// Fondo cardInner, borde borderPanel 1px, nombre de la feature + chip "Vacío".
class _FeatureNodeCard extends StatelessWidget {
  final String label;
  const _FeatureNodeCard({required this.label});

  @override
  Widget build(BuildContext context) {
    return Container(
      width: 240,
      height: 80,
      padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 10),
      decoration: BoxDecoration(
        color: Gx.cardInner,
        borderRadius: BorderRadius.circular(Gx.rPanel),
        border: Border.all(color: Gx.borderPanel, width: 1),
        boxShadow: Gx.glow(Gx.transitionIndigo, blur: 8, opacity: 0.15),
      ),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          Expanded(
            child: Text(
              label,
              style: Gx.uiSans(fontSize: 13, color: Gx.textPrimary, weight: FontWeight.w500),
              overflow: TextOverflow.ellipsis,
            ),
          ),
          const SizedBox(width: 8),
          // Chip "Vacío": indica que el nodo no tiene configuración ni datos aún.
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 7, vertical: 3),
            decoration: BoxDecoration(
              color: Gx.cardInner,
              borderRadius: BorderRadius.circular(Gx.rChip),
              border: Border.all(color: Gx.borderPanel),
            ),
            child: Text(
              'Vacío',
              style: Gx.dataMono(fontSize: 10, color: Gx.textMuted),
            ),
          ),
        ],
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// _FeatureSidePanel — panel lateral 200px con features draggables.
// ---------------------------------------------------------------------------

// Superficie panelSolid, borde derecho borderPanel 1px.
// Cada item es un Draggable<String> que transporta el nombre de la feature.
class _FeatureSidePanel extends StatelessWidget {
  final Color panelBg;
  const _FeatureSidePanel({required this.panelBg});

  @override
  Widget build(BuildContext context) {
    return Container(
      width: 200,
      decoration: BoxDecoration(
        color: panelBg,
        border: const Border(
          right: BorderSide(color: Gx.borderPanel, width: 1),
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Encabezado del panel.
          Padding(
            padding: const EdgeInsets.fromLTRB(16, 20, 16, 12),
            child: Text(
              'Features',
              style: Gx.displayGrotesque(fontSize: 14, color: Gx.textSecondary),
            ),
          ),
          // Lista de features draggables.
          Expanded(
            child: ListView(
              padding: const EdgeInsets.symmetric(horizontal: 8),
              children: _kFeatureList.map((f) => _FeatureListItem(feature: f)).toList(),
            ),
          ),
        ],
      ),
    );
  }
}

// Datos de cada feature en el panel lateral: nombre + si está activa.
class _FeatureInfo {
  final String name;
  final bool active;
  const _FeatureInfo(this.name, {required this.active});
}

// Catálogo de features disponibles en el panel del canvas (EPIC-0).
const List<_FeatureInfo> _kFeatureList = [
  _FeatureInfo('Reloj Determinista', active: true),
  _FeatureInfo('Async Jobs', active: true),
  _FeatureInfo('Audit Log', active: true),
  _FeatureInfo('Telemetría', active: false),
  _FeatureInfo('MCP Gateway', active: false),
];

// ---------------------------------------------------------------------------
// _FeatureListItem — fila draggable de una feature en el panel lateral.
// ---------------------------------------------------------------------------

// Activo: texto en optimaCyan + glow tenue. Locked: texto textMuted + candado.
// Al iniciar el arrastre se muestra una tarjeta fantasma siguiendo el cursor.
class _FeatureListItem extends StatelessWidget {
  final _FeatureInfo feature;
  const _FeatureListItem({required this.feature});

  @override
  Widget build(BuildContext context) {
    // Solo las features activas son draggables.
    final content = _buildContent();

    if (!feature.active) {
      // Feature no disponible: solo muestra la fila sin arrastre.
      return content;
    }

    return Draggable<String>(
      // El dato transportado es el nombre de la feature.
      data: feature.name,
      // Feedback: tarjeta semitransparente que sigue el cursor.
      feedback: Material(
        color: Colors.transparent,
        child: Container(
          width: 180,
          height: 44,
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
          decoration: BoxDecoration(
            color: Gx.cardInner.withOpacity(0.9),
            borderRadius: BorderRadius.circular(Gx.rPanel),
            border: Border.all(color: Gx.transitionIndigo.withOpacity(0.6)),
            boxShadow: Gx.glowStrong(Gx.transitionIndigo),
          ),
          child: Text(
            feature.name,
            style: Gx.uiSans(fontSize: 12, color: Gx.optimaCyan),
            overflow: TextOverflow.ellipsis,
          ),
        ),
      ),
      // childWhenDragging: la fila original se vuelve semitransparente.
      childWhenDragging: Opacity(opacity: 0.3, child: content),
      child: content,
    );
  }

  // Fila visual: ícono de estado + nombre + candado si está bloqueada.
  Widget _buildContent() {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
        decoration: BoxDecoration(
          borderRadius: BorderRadius.circular(Gx.rPanel),
          // Fondo tenue al estar activa.
          color: feature.active ? Gx.optimaCyan.withOpacity(0.05) : Colors.transparent,
        ),
        child: Row(
          children: [
            // Punto de estado: verde activo, gris bloqueado.
            Container(
              width: 6,
              height: 6,
              decoration: BoxDecoration(
                shape: BoxShape.circle,
                color: feature.active ? Gx.optimaCyan : Gx.textMuted,
                boxShadow: feature.active
                    ? Gx.glow(Gx.optimaCyan, blur: 6, opacity: 0.6)
                    : null,
              ),
            ),
            const SizedBox(width: 8),
            Expanded(
              child: Text(
                feature.name,
                style: Gx.uiSans(
                  fontSize: 12,
                  color: feature.active ? Gx.optimaCyan : Gx.textMuted,
                ),
                overflow: TextOverflow.ellipsis,
              ),
            ),
            // Candado para las features bloqueadas.
            if (!feature.active)
              const Icon(Icons.lock_outline, size: 12, color: Gx.textMuted),
          ],
        ),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// _BreadcrumbPill — pill de cristal "Canvas · Forge" arriba izquierda.
// ---------------------------------------------------------------------------

// Superficie de vidrio Apple: glassFill + blur 16 + borde rim-light.
class _BreadcrumbPill extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return ClipRRect(
      borderRadius: BorderRadius.circular(20),
      child: BackdropFilter(
        filter: ui.ImageFilter.blur(sigmaX: 16, sigmaY: 16),
        child: Container(
          padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 6),
          decoration: BoxDecoration(
            color: Gx.glassFill,
            borderRadius: BorderRadius.circular(20),
            border: Gx.rimLight(0.15),
          ),
          child: Text(
            'Canvas · Forge',
            style: Gx.dataMono(fontSize: 12, color: Gx.textSecondary),
          ),
        ),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// _CanvasToolbar — botones flotantes de zoom+, zoom-, fit y clear.
// ---------------------------------------------------------------------------

// Cada botón es un chip de vidrio Apple con ícono.
class _CanvasToolbar extends StatelessWidget {
  final VoidCallback onZoomIn;
  final VoidCallback onZoomOut;
  final VoidCallback onFit;
  final VoidCallback onClear;

  const _CanvasToolbar({
    required this.onZoomIn,
    required this.onZoomOut,
    required this.onFit,
    required this.onClear,
  });

  @override
  Widget build(BuildContext context) {
    return ClipRRect(
      borderRadius: BorderRadius.circular(Gx.rChrome),
      child: BackdropFilter(
        filter: ui.ImageFilter.blur(sigmaX: 16, sigmaY: 16),
        child: Container(
          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
          decoration: BoxDecoration(
            color: Gx.glassFill,
            borderRadius: BorderRadius.circular(Gx.rChrome),
            border: Gx.rimLight(0.15),
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              _ToolbarBtn(icon: Icons.add, tooltip: 'Acercar', onTap: onZoomIn),
              _ToolbarBtn(icon: Icons.remove, tooltip: 'Alejar', onTap: onZoomOut),
              _ToolbarBtn(icon: Icons.fit_screen, tooltip: 'Encuadrar', onTap: onFit),
              _ToolbarBtn(icon: Icons.delete_outline, tooltip: 'Limpiar', onTap: onClear),
            ],
          ),
        ),
      ),
    );
  }
}

// Botón individual de la toolbar: ícono con tooltip y hover sutil.
class _ToolbarBtn extends StatefulWidget {
  final IconData icon;
  final String tooltip;
  final VoidCallback onTap;
  const _ToolbarBtn({required this.icon, required this.tooltip, required this.onTap});

  @override
  State<_ToolbarBtn> createState() => _ToolbarBtnState();
}

class _ToolbarBtnState extends State<_ToolbarBtn> {
  bool _hover = false;

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      onEnter: (_) => setState(() => _hover = true),
      onExit: (_) => setState(() => _hover = false),
      cursor: SystemMouseCursors.click,
      child: Tooltip(
        message: widget.tooltip,
        child: GestureDetector(
          onTap: widget.onTap,
          child: AnimatedContainer(
            duration: const Duration(milliseconds: 140),
            width: 32,
            height: 32,
            decoration: BoxDecoration(
              color: _hover ? Gx.surfaceRaised.withOpacity(0.5) : Colors.transparent,
              borderRadius: BorderRadius.circular(8),
            ),
            child: Icon(widget.icon, size: 16, color: _hover ? Gx.textPrimary : Gx.textLabel),
          ),
        ),
      ),
    );
  }
}
