// §10 Data-viz — sección de nodos DAG para la galería de componentes.
// Muestra la anatomía de un nodo completo, los 6 estados del nodo,
// los 3 tipos de conexión bezier, y el canvas DAG interactivo completo.
// Sin lógica de negocio ni FFI — solo widgets y CustomPainter visuales.

import 'package:flutter/material.dart';
import '../../theme/gx_tokens.dart';
import '../gallery_fx.dart';
import '../../theme/surfaces.dart';

// ===========================================================================
// DagNodesSection — widget raíz exportable para gallery_tab.dart
// ===========================================================================

// Sección completa de nodos DAG: anatomía, estados, conexiones y canvas.
class DagNodesSection extends StatelessWidget {
  const DagNodesSection({super.key});

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: const [
        // 7a. Anatomía del nodo completo con anotaciones.
        _NodeAnatomyDemo(),
        SizedBox(height: 24),
        // 7b. Matriz de 6 estados del nodo.
        _NodeStatesGrid(),
        SizedBox(height: 24),
        // 7c. Tres tipos de conexión bezier.
        _ConnectionTypesDemo(),
        SizedBox(height: 24),
        // 7d. Canvas DAG interactivo completo (reutiliza InteractiveDag).
        _FullDagCanvas(),
      ],
    );
  }
}

// ===========================================================================
// 7a. Anatomía del nodo (estático con todas sus partes)
// ===========================================================================

// Muestra un nodo completo con header, body key-value, puertos de entrada y
// salida, y glow tenue. Las partes están anotadas con etiquetas.
class _NodeAnatomyDemo extends StatelessWidget {
  const _NodeAnatomyDemo();

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [
        // Título de subsección con énfasis dinámico (token accentDynamic).
        Text('Anatomía del nodo', style: Gx.panelTitle.copyWith(color: Gx.accentDynamic)),
        const SizedBox(height: 10),
        // Stack: el nodo centrado + anotaciones a los lados.
        _AnnotatedNode(),
      ],
    );
  }
}

// Nodo completo con anotaciones usando Stack posicionadas.
class _AnnotatedNode extends StatelessWidget {
  const _AnnotatedNode();

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      height: 180,
      child: Row(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          // Etiqueta del puerto entrada (izquierda).
          _Annotation('Puerto\nentrada', right: true),
          const SizedBox(width: 8),
          // El nodo con sus puertos laterales.
          _DagNodeCard(
            headerColor: Gx.optimaCyan,
            icon: Icons.input,
            name: 'Ingest Pipeline',
            chipLabel: 'ETL',
            keyValues: const [
              ('Fuente', 'CSV / API'),
              ('Intervalo', '5 min'),
              ('Registros', '48.230'),
            ],
            showInputPort: true,
            showOutputPort: true,
            glowColor: Gx.optimaCyan,
          ),
          const SizedBox(width: 8),
          // Etiqueta del puerto salida (derecha).
          _Annotation('Puerto\nsalida', right: false),
        ],
      ),
    );
  }
}

// Etiqueta de anotación simple.
class _Annotation extends StatelessWidget {
  final String text;
  final bool right; // si está a la derecha del elemento que anota

  const _Annotation(this.text, {required this.right});

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: 56,
      child: Text(
        text,
        textAlign: right ? TextAlign.right : TextAlign.left,
        style: Gx.uiSans(fontSize: 10, color: Gx.textBaseMuted),
      ),
    );
  }
}

// ===========================================================================
// Widget base de nodo DAG — _DagNodeCard
// Usado en anatomía, estados y conexiones.
// ===========================================================================

// Nodo DAG de 280×140px según DESIGN.md spec. Incluye:
// • Header con borde izquierdo 3px de color + ícono + nombre + chip
// • Body con filas key-value (dataMono 12px)
// • Puertos laterales (círculos 10px con anillo)
// • Glow tenue en reposo
class _DagNodeCard extends StatelessWidget {
  final Color headerColor;     // color semántico del estado del nodo
  final IconData icon;
  final String name;
  final String chipLabel;
  final List<(String, String)> keyValues; // pares (label, valor)
  final bool showInputPort;
  final bool showOutputPort;
  final Color glowColor;
  final double scale;          // para el estado hover (1.02)
  final double borderWidth;    // 1 en reposo, 2 en seleccionado

  const _DagNodeCard({
    required this.headerColor,
    required this.icon,
    required this.name,
    required this.chipLabel,
    required this.keyValues,
    this.showInputPort = false,
    this.showOutputPort = false,
    this.glowColor = Gx.optimaCyan,
    this.scale = 1.0,
    this.borderWidth = 1.0,
  });

  @override
  Widget build(BuildContext context) {
    return Transform.scale(
      scale: scale,
      child: SizedBox(
        width: 280,
        child: Stack(
          clipBehavior: Clip.none,
          children: [
            // Cuerpo del nodo: superficie de tarjeta + borde estructural global dinámico.
            Container(
              decoration: BoxDecoration(
                color: Gx.surfaceCard,
                borderRadius: BorderRadius.circular(Gx.rButton),
                // Borde estructural global (dinámico con énfasis activo).
                border: Border.all(color: Gx.borderBase, width: borderWidth),
                boxShadow: Gx.glow(glowColor, blur: 14, opacity: 0.12),
              ),
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  // Header: borde izquierdo 3px + ícono + nombre + chip.
                  _NodeHeader(
                    color: headerColor,
                    icon: icon,
                    name: name,
                    chipLabel: chipLabel,
                  ),
                  // Body: filas key-value.
                  Padding(
                    padding: const EdgeInsets.fromLTRB(12, 0, 12, 10),
                    child: Column(
                      children: keyValues
                          .map((kv) => _KeyValueRow(label: kv.$1, value: kv.$2))
                          .toList(),
                    ),
                  ),
                ],
              ),
            ),

            // Puerto de entrada (izquierda), centrado verticalmente.
            if (showInputPort)
              Positioned(
                left: -7,
                top: 0,
                bottom: 0,
                child: Center(
                  child: _Port(color: Gx.transitionIndigo),
                ),
              ),

            // Puerto de salida (derecha), centrado verticalmente.
            if (showOutputPort)
              Positioned(
                right: -7,
                top: 0,
                bottom: 0,
                child: Center(
                  child: _Port(color: Gx.optimaCyan),
                ),
              ),
          ],
        ),
      ),
    );
  }
}

// Header del nodo: borde izquierdo de color + ícono + nombre + chip de tipo.
// El borde izquierdo de color se implementa con un Container separado dentro
// de un Row, para evitar el error de Flutter "borderRadius requires uniform
// colors" que ocurre al mezclar colores en Border con borderRadius.
class _NodeHeader extends StatelessWidget {
  final Color color;
  final IconData icon;
  final String name;
  final String chipLabel;

  const _NodeHeader({
    required this.color,
    required this.icon,
    required this.name,
    required this.chipLabel,
  });

  @override
  Widget build(BuildContext context) {
    return Container(
      height: 36,
      // Solo el borde inferior — color uniforme, sin conflicto con borderRadius.
      // No const: Gx.borderBase es un getter dinámico (no const).
      decoration: BoxDecoration(
        border: Border(
          // Separador interno de la cabecera: borde dinámico con el énfasis activo.
          bottom: BorderSide(color: Gx.borderBase, width: Gx.borderHairline),
        ),
        borderRadius: const BorderRadius.only(
          topLeft: Radius.circular(Gx.rButton),
          topRight: Radius.circular(Gx.rButton),
        ),
      ),
      child: Row(
        children: [
          // Borde izquierdo de color semántico (implementado como Container,
          // no como BorderSide, para evitar el error de Flutter con borderRadius).
          // Borde izquierdo de color semántico del estado del nodo.
          Container(
            width: 3,
            decoration: BoxDecoration(
              color: color,
              borderRadius: const BorderRadius.only(
                topLeft: Radius.circular(Gx.rButton),
              ),
            ),
          ),
          const SizedBox(width: 8),
          Icon(icon, size: 14, color: color),
          const SizedBox(width: 6),
          Expanded(
            // Nombre del nodo con token dinámico de texto base.
            child: Text(name,
                style: Gx.displayGrotesque(fontSize: 13, color: Gx.textBase),
                overflow: TextOverflow.ellipsis),
          ),
          // Chip de tipo de nodo.
          Container(
            margin: const EdgeInsets.only(right: 8),
            padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
            decoration: BoxDecoration(
              color: color.withOpacity(0.12),
              border: Border.all(color: color.withOpacity(0.40)),
              borderRadius: BorderRadius.circular(Gx.rChip),
            ),
            child: Text(chipLabel,
                style: Gx.dataMono(fontSize: 10, color: color)),
          ),
        ],
      ),
    );
  }
}

// Fila key-value del body del nodo.
class _KeyValueRow extends StatelessWidget {
  final String label;
  final String value;

  const _KeyValueRow({required this.label, required this.value});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 3),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          // Etiqueta de la fila: token de texto etiqueta dinámico.
          Text(label,
              style: Gx.dataMono(fontSize: 11, color: Gx.textBaseLabel)),
          // Valor de la fila: token de texto secundario dinámico.
          Text(value,
              style: Gx.dataMono(fontSize: 12, color: Gx.textBaseSecondary)),
        ],
      ),
    );
  }
}

// Puerto de conexión lateral: círculo 10px con anillo de color.
class _Port extends StatelessWidget {
  final Color color;

  const _Port({required this.color});

  @override
  Widget build(BuildContext context) {
    return Container(
      width: 14,
      height: 14,
      decoration: BoxDecoration(
        shape: BoxShape.circle,
        color: Gx.surfaceCard,
        border: Border.all(color: color, width: 1.5),
        boxShadow: Gx.glow(color, blur: 6, opacity: 0.4),
      ),
    );
  }
}

// ===========================================================================
// 7b. Matriz de 6 estados del nodo
// ===========================================================================

// Grid Wrap de 6 nodos, uno por estado, con etiqueta descriptiva.
class _NodeStatesGrid extends StatelessWidget {
  const _NodeStatesGrid();

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [
        // Título de subsección con énfasis dinámico.
        Text('Estados del nodo', style: Gx.panelTitle.copyWith(color: Gx.accentDynamic)),
        const SizedBox(height: 12),
        Wrap(
          spacing: 24,
          runSpacing: 20,
          children: const [
            _LabeledNodeState(label: '1. Reposo', child: _NodeStateRest()),
            _LabeledNodeState(label: '2. Hover', child: _NodeStateHover()),
            _LabeledNodeState(label: '3. Seleccionado', child: _NodeStateSelected()),
            _LabeledNodeState(label: '4. Procesando', child: _NodeStateProcessing()),
            _LabeledNodeState(label: '5. Recibe datos', child: _NodeStateReceiving()),
            _LabeledNodeState(label: '6. Error', child: _NodeStateError()),
          ],
        ),
      ],
    );
  }
}

// Envoltorio con etiqueta debajo del nodo.
class _LabeledNodeState extends StatelessWidget {
  final String label;
  final Widget child;

  const _LabeledNodeState({required this.label, required this.child});

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        child,
        const SizedBox(height: 6),
        // Etiqueta descriptiva con token muted dinámico.
        Text(label, style: Gx.uiSans(fontSize: 11, color: Gx.textBaseMuted)),
      ],
    );
  }
}

// Estado 1: reposo — borde 1px borderPanel, glow tenue.
class _NodeStateRest extends StatelessWidget {
  const _NodeStateRest();

  @override
  Widget build(BuildContext context) {
    return _DagNodeCard(
      headerColor: Gx.transitionIndigo,
      icon: Icons.data_object,
      name: 'Feature Node',
      chipLabel: 'CORE',
      keyValues: const [('Estado', 'Reposo'), ('Datos', 'Listos')],
      showInputPort: true,
      showOutputPort: true,
      glowColor: Gx.transitionIndigo,
    );
  }
}

// Estado 2: hover — glowStrong + scale 1.02 (StatefulWidget con MouseRegion).
class _NodeStateHover extends StatefulWidget {
  const _NodeStateHover();

  @override
  State<_NodeStateHover> createState() => _NodeStateHoverState();
}

class _NodeStateHoverState extends State<_NodeStateHover> {
  bool _hovered = false;

  @override
  // Nodo en estado hover: escala 1.02 y glowStrong al posicionar el cursor encima.
  Widget build(BuildContext context) {
    return MouseRegion(
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 160),
        decoration: BoxDecoration(
          borderRadius: BorderRadius.circular(Gx.rButton),
          boxShadow: _hovered
              ? Gx.glowStrong(Gx.optimaCyan)
              : [],
        ),
        child: _DagNodeCard(
          headerColor: Gx.optimaCyan,
          icon: Icons.search,
          name: 'Hover State',
          chipLabel: 'HOVER',
          keyValues: const [('Cursor', 'encima'), ('Glow', 'fuerte')],
          showInputPort: true,
          showOutputPort: true,
          glowColor: Gx.optimaCyan,
          scale: _hovered ? 1.02 : 1.0,
        ),
      ),
    );
  }
}

// Estado 3: seleccionado — borde 2px optimaCyan + glow(optimaCyan).
class _NodeStateSelected extends StatelessWidget {
  const _NodeStateSelected();

  @override
  // Nodo seleccionado: glow optimaCyan y borde 2px activo.
  Widget build(BuildContext context) {
    return Container(
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(Gx.rButton),
        boxShadow: Gx.glow(Gx.optimaCyan, blur: 16, opacity: 0.45),
      ),
      child: _DagNodeCard(
        headerColor: Gx.optimaCyan,
        icon: Icons.check_circle_outline,
        name: 'Seleccionado',
        chipLabel: 'SEL',
        keyValues: const [('Foco', 'activo'), ('Borde', '2px neón')],
        showInputPort: true,
        showOutputPort: true,
        glowColor: Gx.optimaCyan,
        borderWidth: 2.0,
      ),
    );
  }
}

// Estado 4: procesando — chip "PROC" + scanRing animado en el puerto de salida.
class _NodeStateProcessing extends StatefulWidget {
  const _NodeStateProcessing();

  @override
  State<_NodeStateProcessing> createState() => _NodeStateProcessingState();
}

class _NodeStateProcessingState extends State<_NodeStateProcessing>
    with SingleTickerProviderStateMixin {
  late AnimationController _ringCtrl;

  @override
  void initState() {
    super.initState();
    // Ring que pulsa indefinidamente mientras el nodo procesa.
    _ringCtrl = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 1800),
    )..repeat();
  }

  @override
  void dispose() {
    _ringCtrl.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Stack(
      clipBehavior: Clip.none,
      children: [
        _DagNodeCard(
          headerColor: Gx.alertAmber,
          icon: Icons.settings,
          name: 'Procesando',
          chipLabel: 'PROC',
          keyValues: const [('CPU', '78%'), ('ETA', '4.2s')],
          showInputPort: true,
          showOutputPort: false, // el puerto derecho lo dibuja el ring
          glowColor: Gx.alertAmber,
        ),
        // Ring pulsante en el puerto de salida (derecha, centro vertical).
        Positioned(
          right: -7,
          top: 0,
          bottom: 0,
          child: Center(
            child: AnimatedBuilder(
              animation: _ringCtrl,
              builder: (_, __) => _ScanRingPort(
                progress: _ringCtrl.value,
                color: Gx.alertAmber,
              ),
            ),
          ),
        ),
      ],
    );
  }
}

// Anillo que se expande y desvanece en el puerto de salida (efecto scanRing).
class _ScanRingPort extends StatelessWidget {
  final double progress; // 0.0–1.0
  final Color color;

  const _ScanRingPort({required this.progress, required this.color});

  @override
  Widget build(BuildContext context) {
    // El anillo se expande de radio 7 a 20, y se desvanece al final.
    final ringRadius = 7.0 + progress * 13.0;
    final ringOpacity = (1.0 - progress).clamp(0.0, 1.0);

    return SizedBox(
      width: 40,
      height: 40,
      child: CustomPaint(
        painter: _RingPainter(
          radius: ringRadius,
          color: color.withOpacity(ringOpacity * 0.7),
        ),
        child: Center(
          child: _Port(color: color),
        ),
      ),
    );
  }
}

// Painter del anillo expandido para el estado procesando.
class _RingPainter extends CustomPainter {
  final double radius;
  final Color color;

  const _RingPainter({required this.radius, required this.color});

  @override
  void paint(Canvas canvas, Size size) {
    final center = Offset(size.width / 2, size.height / 2);
    canvas.drawCircle(center, radius, Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.5
      ..color = color);
  }

  @override
  bool shouldRepaint(_RingPainter old) =>
      old.radius != radius || old.color != color;
}

// Estado 5: recibe datos — sonarPulse en el puerto de entrada.
class _NodeStateReceiving extends StatefulWidget {
  const _NodeStateReceiving();

  @override
  State<_NodeStateReceiving> createState() => _NodeStateReceivingState();
}

class _NodeStateReceivingState extends State<_NodeStateReceiving>
    with TickerProviderStateMixin {
  // Controller del pulso sonar (700ms, una pasada por pulso).
  late AnimationController _sonarCtrl;
  // Controller de espera entre pulsos (2000ms, sin UI visible).
  // Usar AnimationController como "timer" cancelable en vez de dart:async Timer
  // evita timers huérfanos que el test framework detecta como '!timersPending'.
  late AnimationController _delayCtrl;

  @override
  void initState() {
    super.initState();
    _sonarCtrl = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 700),
    );
    _delayCtrl = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 2000),
    );
    // Lanza el primer ciclo: espera 2s, luego sonar, luego repite.
    _delayCtrl.forward(from: 0.0).then((_) => _fireAndLoop());
  }

  // Dispara el pulso sonar y programa el siguiente ciclo al terminar.
  void _fireAndLoop() {
    if (!mounted) return;
    _sonarCtrl.forward(from: 0.0).then((_) {
      if (!mounted) return;
      _delayCtrl.forward(from: 0.0).then((_) => _fireAndLoop());
    });
  }

  @override
  void dispose() {
    _sonarCtrl.dispose();  // detiene el ticker del sonar
    _delayCtrl.dispose();  // detiene el ticker del delay
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Stack(
      clipBehavior: Clip.none,
      children: [
        _DagNodeCard(
          headerColor: Gx.transitionBlue,
          icon: Icons.download,
          name: 'Recibe datos',
          chipLabel: 'IN',
          keyValues: const [('Flujo', 'activo'), ('Rate', '120/s')],
          showInputPort: false, // el puerto izquierdo lo dibuja el sonar
          showOutputPort: true,
          glowColor: Gx.transitionBlue,
        ),
        // Sonar pulse en el puerto de entrada (izquierda, centro vertical).
        Positioned(
          left: -7,
          top: 0,
          bottom: 0,
          child: Center(
            child: AnimatedBuilder(
              animation: _sonarCtrl,
              builder: (_, __) => _ScanRingPort(
                progress: _sonarCtrl.value,
                color: Gx.transitionBlue,
              ),
            ),
          ),
        ),
      ],
    );
  }
}

// Estado 6: error — borde criticalCrimson + glow parpadeante.
class _NodeStateError extends StatefulWidget {
  const _NodeStateError();

  @override
  State<_NodeStateError> createState() => _NodeStateErrorState();
}

class _NodeStateErrorState extends State<_NodeStateError>
    with SingleTickerProviderStateMixin {
  late AnimationController _blinkCtrl;

  @override
  void initState() {
    super.initState();
    _blinkCtrl = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 800),
    )..repeat(reverse: true);
  }

  @override
  void dispose() {
    _blinkCtrl.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return AnimatedBuilder(
      animation: _blinkCtrl,
      builder: (_, __) => Container(
        decoration: BoxDecoration(
          // Radio del contenedor glow = Gx.rButton para alinearse con el nodo interior.
          borderRadius: BorderRadius.circular(Gx.rButton),
          boxShadow: Gx.glow(Gx.criticalCrimson,
              blur: 20, opacity: 0.30 + _blinkCtrl.value * 0.25),
        ),
        child: _DagNodeCard(
          headerColor: Gx.criticalCrimson,
          icon: Icons.error_outline,
          name: 'Error crítico',
          chipLabel: 'ERR',
          keyValues: const [
            ('Código', 'E-0x4F'),
            ('Causa', 'Timeout FFI'),
          ],
          showInputPort: true,
          showOutputPort: true,
          glowColor: Gx.criticalCrimson,
          borderWidth: 1.5,
        ),
      ),
    );
  }
}

// Nota: el radio del glow del nodo error usa Gx.rButton para mantener
// la consistencia con el radio del cuerpo del nodo (que también usa Gx.rButton).

// ===========================================================================
// 7c. Tipos de conexión bezier (3 ejemplos)
// ===========================================================================

// Tres pares de nodos mini (150×60px) con bezier S-curve entre ellos.
class _ConnectionTypesDemo extends StatelessWidget {
  const _ConnectionTypesDemo();

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [
        Text('Tipos de conexión', style: Gx.panelTitle),
        const SizedBox(height: 12),
        Wrap(
          spacing: 24,
          runSpacing: 16,
          children: const [
            // Conexión normal.
            _LabeledConnection(
              label: '1. Normal',
              child: _ConnectionNormal(),
            ),
            // Conexión en hover con tooltip.
            _LabeledConnection(
              label: '2. Hover + tooltip',
              child: _ConnectionHover(),
            ),
            // Conexión inválida parpadeante.
            _LabeledConnection(
              label: '3. Inválida',
              child: _ConnectionInvalid(),
            ),
          ],
        ),
      ],
    );
  }
}

// Envoltorio con etiqueta.
class _LabeledConnection extends StatelessWidget {
  final String label;
  final Widget child;

  const _LabeledConnection({required this.label, required this.child});

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        child,
        const SizedBox(height: 6),
        // Etiqueta descriptiva con token muted dinámico.
        Text(label, style: Gx.uiSans(fontSize: 11, color: Gx.textBaseMuted)),
      ],
    );
  }
}

// Nodo mini para los ejemplos de conexión (150×60px).
class _MiniNode extends StatelessWidget {
  final String name;
  final Color color;
  final bool showLeft;
  final bool showRight;

  const _MiniNode({
    required this.name,
    required this.color,
    this.showLeft = false,
    this.showRight = false,
  });

  @override
  Widget build(BuildContext context) {
    return Stack(
      clipBehavior: Clip.none,
      children: [
        Container(
          width: 150,
          height: 60,
          padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
          decoration: BoxDecoration(
            color: Gx.surfaceCard,
            borderRadius: BorderRadius.circular(Gx.rButton),
            border: Border.all(color: Gx.borderBase),
            boxShadow: Gx.glow(color, blur: 10, opacity: 0.10),
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Container(
                width: 60,
                height: 3,
                decoration: BoxDecoration(
                  color: color,
                  // barra de color del mini-nodo (3px alto): radio decorativo
                  borderRadius: BorderRadius.circular(2),
                ),
              ),
              const SizedBox(height: 5),
              Text(name,
                  style:
                      Gx.displayGrotesque(fontSize: 12, color: Gx.textBaseSecondary)),
            ],
          ),
        ),
        if (showLeft)
          Positioned(
            left: -7,
            top: 0,
            bottom: 0,
            child: Center(child: _Port(color: color)),
          ),
        if (showRight)
          Positioned(
            right: -7,
            top: 0,
            bottom: 0,
            child: Center(child: _Port(color: color)),
          ),
      ],
    );
  }
}

// Painter de la curva bezier S entre dos nodos.
// [startX, startY] = posición del puerto de salida del nodo izquierdo.
// [endX, endY] = posición del puerto de entrada del nodo derecho.
class _BezierConnectionPainter extends CustomPainter {
  final Color color;
  final double strokeWidth;
  final double glowBlur;
  final double glowOpacity;

  const _BezierConnectionPainter({
    required this.color,
    this.strokeWidth = 2.0,
    this.glowBlur = 4.0,
    this.glowOpacity = 0.4,
  });

  @override
  void paint(Canvas canvas, Size size) {
    // Los nodos están en los extremos del canvas; la curva va de izq a der.
    final start = Offset(0, size.height / 2);
    final end = Offset(size.width, size.height / 2);
    final cp1 = Offset(size.width * 0.4, start.dy);
    final cp2 = Offset(size.width * 0.6, end.dy);

    final path = Path()
      ..moveTo(start.dx, start.dy)
      ..cubicTo(cp1.dx, cp1.dy, cp2.dx, cp2.dy, end.dx, end.dy);

    // Halo de glow.
    canvas.drawPath(path, Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = strokeWidth + 4
      ..color = color.withOpacity(glowOpacity)
      ..maskFilter = MaskFilter.blur(BlurStyle.normal, glowBlur));

    // Línea nítida.
    canvas.drawPath(path, Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = strokeWidth
      ..color = color);

    // Punta de flecha en el extremo destino.
    _drawArrow(canvas, cp2, end, color, strokeWidth);
  }

  // Dibuja una punta de flecha de 8px en el destino de la curva.
  void _drawArrow(Canvas canvas, Offset cp, Offset end, Color c, double sw) {
    final dir = (end - cp).normalize() * 8.0;
    final perp = Offset(-dir.dy, dir.dx) * 0.4;
    final tip = end;
    final left = tip - dir + perp;
    final right = tip - dir - perp;
    canvas.drawPath(
      Path()
        ..moveTo(tip.dx, tip.dy)
        ..lineTo(left.dx, left.dy)
        ..lineTo(right.dx, right.dy)
        ..close(),
      Paint()..color = c,
    );
  }

  @override
  bool shouldRepaint(_BezierConnectionPainter old) =>
      old.color != color || old.strokeWidth != strokeWidth;
}

// Extensión auxiliar para normalizar un Offset (vector unitario).
extension _OffsetNorm on Offset {
  Offset normalize() {
    final d = distance;
    return d == 0 ? this : this / d;
  }
}

// Conexión tipo 1: normal — strokeWidth 2, transitionIndigo, glow blur 4.
class _ConnectionNormal extends StatelessWidget {
  const _ConnectionNormal();

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: 360,
      height: 80,
      child: Stack(
        children: [
          // Nodo origen (izquierda).
          Positioned(
            left: 0,
            top: 10,
            child: _MiniNode(
              name: 'Origen',
              color: Gx.transitionIndigo,
              showRight: true,
            ),
          ),
          // Curva bezier en el espacio entre nodos.
          Positioned(
            left: 150,
            top: 10,
            child: SizedBox(
              width: 60,
              height: 60,
              child: CustomPaint(
                painter: const _BezierConnectionPainter(
                  color: Gx.transitionIndigo,
                  strokeWidth: 2.0,
                  glowBlur: 4.0,
                  glowOpacity: 0.3,
                ),
                size: const Size(60, 60),
              ),
            ),
          ),
          // Nodo destino (derecha).
          Positioned(
            right: 0,
            top: 10,
            child: _MiniNode(
              name: 'Destino',
              color: Gx.transitionIndigo,
              showLeft: true,
            ),
          ),
        ],
      ),
    );
  }
}

// Conexión tipo 2: hover — strokeWidth 3, glow intensificado + tooltip vidrio.
class _ConnectionHover extends StatefulWidget {
  const _ConnectionHover();

  @override
  State<_ConnectionHover> createState() => _ConnectionHoverState();
}

class _ConnectionHoverState extends State<_ConnectionHover> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: 360,
      height: 80,
      child: Stack(
        children: [
          Positioned(
            left: 0,
            top: 10,
            child: _MiniNode(
              name: 'Ingest',
              color: Gx.optimaCyan,
              showRight: true,
            ),
          ),
          // Área de hover sobre la curva.
          Positioned(
            left: 150,
            top: 0,
            child: MouseRegion(
              onEnter: (_) => setState(() => _hovered = true),
              onExit: (_) => setState(() => _hovered = false),
              child: SizedBox(
                width: 60,
                height: 80,
                child: Stack(
                  children: [
                    // Curva con intensidad variable.
                    CustomPaint(
                      painter: _BezierConnectionPainter(
                        color: _hovered ? Gx.optimaCyan : Gx.transitionIndigo,
                        strokeWidth: _hovered ? 3.0 : 2.0,
                        glowBlur: _hovered ? 8.0 : 4.0,
                        glowOpacity: _hovered ? 0.55 : 0.3,
                      ),
                      size: const Size(60, 80),
                    ),
                    // Tooltip vidrio Apple en hover.
                    if (_hovered)
                      Positioned(
                        top: 0,
                        left: 0,
                        child: Container(
                          padding: const EdgeInsets.symmetric(
                              horizontal: 7, vertical: 4),
                          decoration: BoxDecoration(
                            color: Gx.surfaceFill,
                            border: Border.all(
                                // Borde del tooltip: token dinámico de texto base al 12%.
                            color: Gx.textBase.withOpacity(0.12)),
                            borderRadius: BorderRadius.circular(Gx.rTooltip),
                          ),
                          child: Text(
                            'DataStream<Float32>',
                            style:
                                Gx.dataMono(fontSize: 10, color: Gx.textBase),
                          ),
                        ),
                      ),
                  ],
                ),
              ),
            ),
          ),
          Positioned(
            right: 0,
            top: 10,
            child: _MiniNode(
              name: 'Validate',
              color: Gx.optimaCyan,
              showLeft: true,
            ),
          ),
        ],
      ),
    );
  }
}

// Conexión tipo 3: inválida — criticalCrimson, strokeWidth 1.5, parpadeante.
class _ConnectionInvalid extends StatefulWidget {
  const _ConnectionInvalid();

  @override
  State<_ConnectionInvalid> createState() => _ConnectionInvalidState();
}

class _ConnectionInvalidState extends State<_ConnectionInvalid>
    with SingleTickerProviderStateMixin {
  late AnimationController _blinkCtrl;

  @override
  void initState() {
    super.initState();
    _blinkCtrl = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 700),
    )..repeat(reverse: true);
  }

  @override
  void dispose() {
    _blinkCtrl.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: 360,
      height: 80,
      child: Stack(
        children: [
          Positioned(
            left: 0,
            top: 10,
            child: _MiniNode(
              name: 'Broken',
              color: Gx.criticalCrimson,
              showRight: true,
            ),
          ),
          Positioned(
            left: 150,
            top: 10,
            child: AnimatedBuilder(
              animation: _blinkCtrl,
              builder: (_, __) => SizedBox(
                width: 60,
                height: 60,
                child: CustomPaint(
                  painter: _BezierConnectionPainter(
                    color: Gx.criticalCrimson
                        .withOpacity(0.5 + _blinkCtrl.value * 0.5),
                    strokeWidth: 1.5,
                    glowBlur: 6.0 + _blinkCtrl.value * 6.0,
                    glowOpacity: 0.3 + _blinkCtrl.value * 0.3,
                  ),
                  size: const Size(60, 60),
                ),
              ),
            ),
          ),
          Positioned(
            right: 0,
            top: 10,
            child: _MiniNode(
              name: 'Target',
              color: Gx.criticalCrimson,
              showLeft: true,
            ),
          ),
        ],
      ),
    );
  }
}

// ===========================================================================
// 7d. Canvas DAG interactivo completo — reutiliza InteractiveDag de gallery_fx
// ===========================================================================

// Canvas completo del DAG (700×400px) usando el widget InteractiveDag existente.
class _FullDagCanvas extends StatelessWidget {
  const _FullDagCanvas();

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [
        Text('Canvas DAG — interactivo', style: Gx.panelTitle),
        const SizedBox(height: 10),
        // SizedBox con dimensiones fijas para que InteractiveDag reciba
        // restricciones finitas dentro del SingleChildScrollView.
        SizedBox(
          width: 700,
          height: 400,
          child: panelSurface(
            radius: Gx.rPanel,
            glow: Gx.glow(Gx.transitionIndigo, blur: 20, opacity: 0.10),
            child: ClipRRect(
              borderRadius: BorderRadius.circular(Gx.rPanel),
              // InteractiveDag implementa el canvas nodal completo con
              // hover, nodos, puertos y conexiones bezier (gallery_fx.dart).
              child: InteractiveDag(),
            ),
          ),
        ),
      ],
    );
  }
}
