// Shell del Canvas de Drasus Engine.
// Muestra un lienzo interactivo (pan, zoom) donde el usuario puede arrastrar
// features desde el panel lateral y colocarlos como nodos. Pulsar un nodo
// abre el Inspector Panel de esa feature en un panel lateral derecho
// (ADR-0136 FIJO, STORY-073 Etapa 4 criterios #8/#9) — el despacho de QUÉ
// panel abrir vive en `canvas_inspector_registry.dart`, nunca aquí: este
// archivo no conoce ninguna feature por nombre propio. No hay lógica de
// negocio: el Canvas es presentación pura.

import 'dart:ui' as ui;
import 'package:flutter/material.dart';
import '../theme/gx_tokens.dart';
import '../gallery/gallery_painters.dart';
import '../theme/theme_scope.dart';
import '../app_meta.dart';
import 'canvas_inspector_registry.dart';

// ---------------------------------------------------------------------------
// CanvasTab — pestaña del canvas con panel lateral + lienzo interactivo.
// ---------------------------------------------------------------------------

// Layout: Row con panel izquierdo de 200px (lista de features draggables)
// y un Expanded que contiene el InteractiveViewer con el canvas.
class CanvasTab extends StatefulWidget {
  // Costura de prueba (STORY-073 Etapa 4): coloca nodos en el lienzo sin
  // pasar por el gesto real de arrastrar-y-soltar, que `WidgetTester` no
  // simula con coordenadas de `DragTarget` de forma práctica. Un test de
  // widget que verifica "pulsar un nodo abre su panel" (criterios #8/#9) usa
  // esto para llegar al nodo directamente; en producción siempre es `null` y
  // el único origen de nodos es `_addNode()` vía arrastre real.
  final List<String>? debugInitialNodeLabels;

  const CanvasTab({super.key, this.debugInitialNodeLabels});

  @override
  State<CanvasTab> createState() => _CanvasTabState();
}

class _CanvasTabState extends State<CanvasTab> {
  // Lista de nodos colocados por el usuario. Cada nodo registra el nombre
  // de la feature y la posición donde fue soltado en el canvas.
  final List<_CanvasNode> _nodes = [];

  // Controlador del zoom para los botones de la toolbar flotante.
  final TransformationController _transformCtrl = TransformationController();

  // Nombre de la feature cuyo Inspector Panel está abierto — `null` si el
  // panel lateral derecho está cerrado. Un solo panel a la vez (ADR-0136: el
  // panel lateral es un overlay, no una pila de paneles).
  String? _openPanelFeature;

  // Ancho de los dos paneles laterales, ajustable arrastrando su borde.
  //
  // El izquierdo nace `null` y se mide en el primer `build` contra el nombre
  // de feature más largo del catálogo: un ancho fijo a ojo o sobra o corta, y
  // corta justo cuando el catálogo crece, que es cuando más falta hace leer
  // el nombre entero. Una vez que el usuario arrastra, su valor manda y ya no
  // se vuelve a medir.
  double? _sidePanelWidth;
  double _inspectorWidth = _kInspectorPanelDefaultWidth;

  // Aplica un desplazamiento de arrastre a un ancho, dentro de sus límites.
  // Los topes existen para que el panel nunca se coma el lienzo ni se
  // encoja hasta ser impulsable.
  void _resizeSidePanel(double delta) {
    setState(() {
      final base = _sidePanelWidth ?? _measureSidePanelWidth();
      _sidePanelWidth =
          (base + delta).clamp(_kSidePanelMinWidth, _kSidePanelMaxWidth);
    });
  }

  // El inspector crece hacia la IZQUIERDA: arrastrar su manija hacia la
  // izquierda (delta negativo) lo ensancha, por eso el signo va invertido.
  void _resizeInspector(double delta) {
    setState(() {
      _inspectorWidth = (_inspectorWidth - delta)
          .clamp(_kInspectorPanelMinWidth, _kInspectorPanelMaxWidth);
    });
  }

  @override
  void initState() {
    super.initState();
    // Costura de prueba: precarga nodos si el test los proveyó (ver
    // doc-comment de `debugInitialNodeLabels`).
    final debugLabels = widget.debugInitialNodeLabels;
    if (debugLabels != null) {
      for (final label in debugLabels) {
        _nodes.add(_CanvasNode(label: label, position: const Offset(40, 40)));
      }
    }
  }

  @override
  void dispose() {
    _transformCtrl.dispose();
    super.dispose();
  }

  // Abre el Inspector Panel de la feature pulsada — el mecanismo GENÉRICO
  // (criterio de aceptación #8): no importa qué feature sea, el lienzo
  // siempre sabe abrir SU panel consultando el registro.
  void _openPanel(String featureLabel) {
    setState(() => _openPanelFeature = featureLabel);
  }

  void _closePanel() {
    setState(() => _openPanelFeature = null);
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
    final surfaces = ThemeScope.surfaceFor(context);
    // panelSolid: color de fondo del panel lateral de features.
    final panelBg = surfaces?.panelSolid ?? Gx.surfacePanel;

    // Medición perezosa: solo la primera vez, y nunca después de que el
    // usuario haya elegido su propio ancho.
    final sideWidth = _sidePanelWidth ??= _measureSidePanelWidth();

    return Row(
      children: [
        // ---- Panel lateral izquierdo: lista de features draggables ----
        _FeatureSidePanel(
          key: const ValueKey('canvas-side-panel'),
          panelBg: panelBg,
          width: sideWidth,
        ),

        // Manija de ajuste del panel izquierdo: vive FUERA del panel para que
        // arrastrarla no compita con el arrastre de las features de dentro.
        _ResizeHandle(
          key: const ValueKey('canvas-side-resize'),
          tooltip: 'Ajustar el ancho del panel de features',
          onDrag: _resizeSidePanel,
        ),

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
                    // Mecanismo GENÉRICO (criterio #8): cualquier nodo, de
                    // cualquier feature, abre SU panel al pulsarlo.
                    onNodeTap: _openPanel,
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

              // ---- Inspector Panel lateral derecho (ADR-0136 FIJO) ----
              // Un solo overlay para TODAS las features: el contenido lo
              // resuelve `resolveCanvasPanelBuilder` contra el registro
              // genérico, este widget nunca importa una feature por nombre.
              if (_openPanelFeature != null)
                Positioned(
                  top: 0,
                  right: 0,
                  bottom: 0,
                  child: _InspectorPanelOverlay(
                    key: const ValueKey('canvas-inspector-panel'),
                    featureLabel: _openPanelFeature!,
                    width: _inspectorWidth,
                    onClose: _closePanel,
                    onResize: _resizeInspector,
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
  final ValueChanged<String> onNodeTap;

  const _CanvasViewport({
    required this.controller,
    required this.nodes,
    required this.isDraggingOver,
    required this.onNodeTap,
  });

  @override
  Widget build(BuildContext context) {
    return AnimatedContainer(
      duration: const Duration(milliseconds: 200),
      decoration: BoxDecoration(
        // Indicador visual de zona de soltar: borde tenue cuando se arrastra.
        border: isDraggingOver
            ? Border.all(
                color: Gx.transitionIndigo.withOpacity(0.4), width: 1.5)
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
              // Key propia (no el texto del label): el panel lateral de
              // features usa EL MISMO texto para su propio ítem de lista, así
              // que `find.text(label)` en un test de widget encontraría dos
              // candidatos — la Key distingue el nodo del lienzo sin
              // ambigüedad.
              ...nodes.map((node) => Positioned(
                    left: node.position.dx,
                    top: node.position.dy,
                    child: _FeatureNodeCard(
                      key: ValueKey(
                          'canvas-node-${node.label}-${node.position.dx}-${node.position.dy}'),
                      label: node.label,
                      onTap: () => onNodeTap(node.label),
                    ),
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
// Pulsable (STORY-073 Etapa 4, criterio #8): `onTap` abre el Inspector Panel
// de esta feature — cursor de mano en hover para señalar la interacción.
class _FeatureNodeCard extends StatelessWidget {
  final String label;
  final VoidCallback onTap;
  const _FeatureNodeCard({super.key, required this.label, required this.onTap});

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      child: GestureDetector(
        onTap: onTap,
        child: Container(
          width: 240,
          height: 80,
          padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 10),
          decoration: BoxDecoration(
            color: Gx.surfaceCard,
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
                  style: Gx.uiSans(
                      fontSize: 13,
                      color: Gx.textBase,
                      weight: FontWeight.w500),
                  overflow: TextOverflow.ellipsis,
                ),
              ),
              const SizedBox(width: 8),
              // Chip "Vacío": indica que el nodo no tiene configuración ni datos aún.
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 7, vertical: 3),
                decoration: BoxDecoration(
                  color: Gx.surfaceCard,
                  borderRadius: BorderRadius.circular(Gx.rChip),
                  border: Border.all(color: Gx.borderPanel),
                ),
                child: Text(
                  'Vacío',
                  style: Gx.dataMono(fontSize: 10, color: Gx.textBaseMuted),
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
// Anchos de los paneles laterales y su medición
// ---------------------------------------------------------------------------

// Topes del panel de features. El mínimo deja sitio al punto de estado y a
// unas pocas letras; el máximo impide que el panel se coma el lienzo.
const double _kSidePanelMinWidth = 150;
const double _kSidePanelMaxWidth = 460;

// Topes del Inspector Panel. El mínimo es el ancho con el que se diseñó su
// contenido (formularios de dos columnas); por debajo se rompe el layout.
const double _kInspectorPanelDefaultWidth = 420;
const double _kInspectorPanelMinWidth = 340;
const double _kInspectorPanelMaxWidth = 760;

// Espacio que consume la fila de una feature alrededor del texto del nombre:
// sangrado de la lista (8 a cada lado), relleno de la fila (10 a cada lado),
// punto de estado (6) y su separación (8).
const double _kFeatureRowChrome = 8 * 2 + 10 * 2 + 6 + 8;

// Mide el ancho que necesita el nombre de feature más largo del catálogo, con
// la MISMA tipografía con que se dibuja. Un ancho fijo escrito a mano deja de
// ser correcto en cuanto el catálogo crece — y crece a cada Story.
double _measureSidePanelWidth() {
  var widest = 0.0;
  for (final feature in _kFeatureList) {
    final painter = TextPainter(
      text: TextSpan(
        text: feature.name,
        style: Gx.uiSans(fontSize: 12, color: Gx.textBase),
      ),
      textDirection: TextDirection.ltr,
      maxLines: 1,
    )..layout();
    if (painter.width > widest) widest = painter.width;
  }
  return (widest + _kFeatureRowChrome)
      .clamp(_kSidePanelMinWidth, _kSidePanelMaxWidth);
}

// ---------------------------------------------------------------------------
// _ResizeHandle — franja vertical que ajusta el ancho de un panel al arrastrar.
// ---------------------------------------------------------------------------

// Cumple la regla irrompible de `docs/DESIGN.md`: pasar el cursor cambia SOLO
// el color, nunca el tamaño. Una manija que se ensancha al apuntarla se
// mueve por debajo del propio cursor y produce el parpadeo que ya costó una
// sesión de pruebas en la zona de carga del asistente de importación.
class _ResizeHandle extends StatefulWidget {
  final String tooltip;

  // Recibe el desplazamiento horizontal del arrastre, en píxeles lógicos.
  final void Function(double delta) onDrag;

  const _ResizeHandle(
      {super.key, required this.tooltip, required this.onDrag});

  @override
  State<_ResizeHandle> createState() => _ResizeHandleState();
}

class _ResizeHandleState extends State<_ResizeHandle> {
  bool _hover = false;
  bool _dragging = false;

  @override
  Widget build(BuildContext context) {
    final active = _hover || _dragging;
    return MouseRegion(
      cursor: SystemMouseCursors.resizeLeftRight,
      onEnter: (_) => setState(() => _hover = true),
      onExit: (_) => setState(() => _hover = false),
      child: Tooltip(
        message: widget.tooltip,
        waitDuration: const Duration(milliseconds: 600),
        child: GestureDetector(
          behavior: HitTestBehavior.opaque,
          onHorizontalDragStart: (_) => setState(() => _dragging = true),
          onHorizontalDragEnd: (_) => setState(() => _dragging = false),
          onHorizontalDragCancel: () => setState(() => _dragging = false),
          onHorizontalDragUpdate: (details) => widget.onDrag(details.delta.dx),
          child: SizedBox(
            // Ancho constante en los tres estados. Seis píxeles es estrecho
            // de ver pero cómodo de agarrar; el realce va por color.
            width: 6,
            height: double.infinity,
            child: Center(
              child: Container(
                width: 2,
                height: double.infinity,
                color: active ? Gx.transitionIndigo : Colors.transparent,
              ),
            ),
          ),
        ),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// _FeatureSidePanel — panel lateral con features draggables, ancho ajustable.
// ---------------------------------------------------------------------------

// Superficie panelSolid, borde derecho borderPanel 1px.
// Cada item es un Draggable<String> que transporta el nombre de la feature.
class _FeatureSidePanel extends StatelessWidget {
  final Color panelBg;
  final double width;
  const _FeatureSidePanel(
      {super.key, required this.panelBg, required this.width});

  @override
  Widget build(BuildContext context) {
    return Container(
      width: width,
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
              style: Gx.displayGrotesque(
                  fontSize: 14, color: Gx.textBaseSecondary),
            ),
          ),
          // Lista de features draggables.
          Expanded(
            child: ListView(
              padding: const EdgeInsets.symmetric(horizontal: 8),
              children: _kFeatureList
                  .map((f) => _FeatureListItem(feature: f))
                  .toList(),
            ),
          ),
        ],
      ),
    );
  }
}

// Datos de cada feature en el panel lateral: nombre, qué hace, y si está activa.
class _FeatureInfo {
  final String name;

  // Qué hace la feature, en una o dos líneas. Se muestra bajo el nombre y,
  // completa, al posar el cursor. **Debe distinguir qué CONSUME de qué
  // PRODUCE** (DEBT-045): ese es el eje que confundió las dos features de
  // datos —una lee un archivo del disco, la otra trabaja sobre lo ya
  // importado— y es además el eje del pipeline, así que sirve para todas.
  final String description;

  final bool active;
  const _FeatureInfo(this.name,
      {required this.description, required this.active});
}

// Catálogo de features disponibles en el panel del canvas (EPIC-0 + EPIC-1).
// Las dos últimas son las que STORY-073 hace alcanzables por primera vez —
// antes de esta Story no había forma de arrastrarlas al lienzo, ni forma de
// que ningún nodo abriera nada al pulsarlo.
const List<_FeatureInfo> _kFeatureList = [
  _FeatureInfo('Reloj Determinista',
      description: 'Da la hora oficial del sistema. Todo lo que se registra '
          'se fecha aquí, para que dos ejecuciones sean comparables.',
      active: true),
  _FeatureInfo('Async Jobs',
      description: 'Ejecuta trabajos largos en segundo plano y reporta su '
          'avance sin bloquear la aplicación.',
      active: true),
  _FeatureInfo('Audit Log',
      description: 'Registro sellado de todo lo que ocurre. Solo se añade, '
          'nunca se borra ni se corrige.',
      active: true),
  _FeatureInfo('Telemetría',
      description: 'Mide el estado del motor y lo reporta al puesto de mando '
          'central.',
      active: false),
  _FeatureInfo('MCP Gateway',
      description: 'Puerta de entrada para herramientas externas, con '
          'permisos declarados operación por operación.',
      active: false),
  _FeatureInfo('Asistente de Importación',
      description: 'Toma un archivo de tu disco (CSV, TXT o TSV) y lo '
          'convierte en datos de mercado listos para usar.',
      active: true),
  _FeatureInfo('Transformador de Datos de Mercado',
      description: 'Toma datos YA importados y los reagrega a otra '
          'temporalidad (de 1 minuto a 1 hora). No lee archivos.',
      active: true),
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
            color: Gx.surfaceCard.withOpacity(0.9),
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

  // Fila visual: ícono de estado + nombre + candado si está bloqueada, con
  // la descripción corta debajo del nombre.
  //
  // Por qué la descripción va en los DOS sitios —debajo del título y al posar
  // el cursor— y no en uno solo: debajo del título se lee sin hacer nada, que
  // es lo que hace falta para elegir entre features vecinas de un vistazo;
  // pero ahí se recorta a dos líneas, y el panel puede estrecharse hasta
  // recortarla más. El tooltip da el texto completo sin depender del ancho.
  Widget _buildContent() {
    final content = Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
        decoration: BoxDecoration(
          borderRadius: BorderRadius.circular(Gx.rPanel),
          // Fondo tenue al estar activa.
          color: feature.active
              ? Gx.optimaCyan.withOpacity(0.05)
              : Colors.transparent,
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisSize: MainAxisSize.min,
          children: [
            Row(
              children: [
                // Punto de estado: verde activo, gris bloqueado.
                Container(
                  width: 6,
                  height: 6,
                  decoration: BoxDecoration(
                    shape: BoxShape.circle,
                    color: feature.active ? Gx.optimaCyan : Gx.textBaseMuted,
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
                      color:
                          feature.active ? Gx.optimaCyan : Gx.textBaseMuted,
                    ),
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
                // Candado para las features bloqueadas.
                if (!feature.active)
                  Icon(Icons.lock_outline, size: 12, color: Gx.textBaseMuted),
              ],
            ),
            const SizedBox(height: 3),
            // Descripción: alineada bajo el nombre (el sangrado de 14 px
            // corresponde al punto de estado más su separación), en color de
            // etiqueta para que no compita con el nombre.
            Padding(
              padding: const EdgeInsets.only(left: 14),
              child: Text(
                feature.description,
                style: Gx.uiSans(fontSize: 10, color: Gx.textBaseLabel),
                maxLines: 2,
                overflow: TextOverflow.ellipsis,
              ),
            ),
          ],
        ),
      ),
    );

    // Texto completo al posar el cursor, sin depender de cuánto se haya
    // estrechado el panel.
    return Tooltip(
      message: '${feature.name}\n\n${feature.description}',
      waitDuration: const Duration(milliseconds: 400),
      child: content,
    );
  }
}

// ---------------------------------------------------------------------------
// _BreadcrumbPill — pill de cristal "Canvas · <kCanvasName>" arriba izquierda.
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
            color: Gx.surfaceFill,
            borderRadius: BorderRadius.circular(20),
            border: Gx.rimLight(0.15),
          ),
          child: Text(
            'Canvas · $kCanvasName',
            style: Gx.dataMono(fontSize: 12, color: Gx.textBaseSecondary),
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
            color: Gx.surfaceFill,
            borderRadius: BorderRadius.circular(Gx.rChrome),
            border: Gx.rimLight(0.15),
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              _ToolbarBtn(icon: Icons.add, tooltip: 'Acercar', onTap: onZoomIn),
              _ToolbarBtn(
                  icon: Icons.remove, tooltip: 'Alejar', onTap: onZoomOut),
              _ToolbarBtn(
                  icon: Icons.fit_screen, tooltip: 'Encuadrar', onTap: onFit),
              _ToolbarBtn(
                  icon: Icons.delete_outline,
                  tooltip: 'Limpiar',
                  onTap: onClear),
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
  const _ToolbarBtn(
      {required this.icon, required this.tooltip, required this.onTap});

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
              color: _hover
                  ? Gx.surfaceRaised.withOpacity(0.5)
                  : Colors.transparent,
              borderRadius: BorderRadius.circular(8),
            ),
            child: Icon(widget.icon,
                size: 16, color: _hover ? Gx.textBase : Gx.textBaseLabel),
          ),
        ),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// _InspectorPanelOverlay — chrome GENÉRICO del panel lateral derecho
// (ADR-0136 FIJO: un feature node siempre abre panel lateral, nunca pantalla
// completa). Se construye UNA vez para todas las features — el contenido lo
// resuelve `resolveCanvasPanelBuilder` contra el registro; adoptar una
// feature nueva (criterio #9: el transformador híbrido deja de estar
// huérfano) nunca implica tocar este widget, solo el registro.
// ---------------------------------------------------------------------------

// El ancho es ajustable arrastrando el borde izquierdo, entre los topes
// declarados arriba. La Cáscara Visual de `data-import-wizard` pedía ancho
// fijo para que el panel no invadiera el lienzo; el tope máximo cumple esa
// intención sin negarle al usuario ensanchar cuando la tabla de vista previa
// no cabe. El scroll-area interno de cada panel sigue igual.
class _InspectorPanelOverlay extends StatelessWidget {
  final String featureLabel;
  final double width;
  final VoidCallback onClose;
  final void Function(double delta) onResize;

  const _InspectorPanelOverlay({
    super.key,
    required this.featureLabel,
    required this.width,
    required this.onClose,
    required this.onResize,
  });

  @override
  // Superficie de vidrio Apple (glassFill + blur + rim, mismo tratamiento
  // que _BreadcrumbPill/_CanvasToolbar) con cabecera fija (nombre de la
  // feature + botón de cierre) y el contenido del panel resuelto por el
  // registro genérico, dentro de un Expanded para que cada panel gestione
  // su propio scroll interno.
  Widget build(BuildContext context) {
    final builder = resolveCanvasPanelBuilder(featureLabel);
    return SizedBox(
      width: width,
      // La manija va en una Row junto al panel, no superpuesta a su borde:
      // superpuesta robaría los gestos del contenido pegado al margen
      // izquierdo (un campo de texto, un botón).
      child: Row(
        children: [
          _ResizeHandle(
            key: const ValueKey('canvas-inspector-resize'),
            tooltip: 'Ajustar el ancho del panel',
            onDrag: onResize,
          ),
          Expanded(child: _buildPanelSurface(context, builder)),
        ],
      ),
    );
  }

  // Superficie de cristal del panel: cabecera con el nombre de la feature,
  // separador, y el contenido que resuelve el registro genérico.
  Widget _buildPanelSurface(BuildContext context, CanvasPanelBuilder builder) {
    return ClipRRect(
        borderRadius:
            const BorderRadius.horizontal(left: Radius.circular(Gx.rChrome)),
        child: BackdropFilter(
          filter: ui.ImageFilter.blur(sigmaX: 24, sigmaY: 24),
          child: Container(
            decoration: BoxDecoration(
              color: Gx.surfaceFill,
              border: Border(left: BorderSide(color: Gx.borderPanel, width: 1)),
              boxShadow: Gx.glow(Gx.transitionIndigo, blur: 20, opacity: 0.12),
            ),
            padding: const EdgeInsets.all(Gx.space16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                Row(
                  children: [
                    Expanded(
                      child: Text(
                        featureLabel,
                        style: Gx.uiSans(
                            fontSize: 14,
                            color: Gx.textBase,
                            weight: FontWeight.w600),
                        overflow: TextOverflow.ellipsis,
                      ),
                    ),
                    _ToolbarBtn(
                        icon: Icons.close, tooltip: 'Cerrar', onTap: onClose),
                  ],
                ),
                const SizedBox(height: Gx.space12),
                Container(height: 1, color: Gx.divider),
                const SizedBox(height: Gx.space12),
                Expanded(child: builder(context)),
              ],
            ),
          ),
        ),
    );
  }
}
