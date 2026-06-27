// Sección de animaciones universales de DESIGN.md:
//   • Acento Primario A/B — comparativa de dos variantes de accentPrimary candidato
//   • Odómetro numérico — todo número dinámico anima desde 0.0 (regla universal)
//   • Gauge radial con arco animado — sweepAngle 0→final (regla universal)
//   • Equity-curve con path drawing + efecto eléctrico (ElectricScanMixin)
//
// Todos los widgets manejan estado local de UI (AnimationController, valor
// numérico interpolado). Sin lógica de negocio ni FFI — Cáscara Delgada.
// Tokens: superficies via wrappers panelSurface()/cardSurface(),
//   texto via Gx.textBase*, bordes via Gx.borderBase.

import 'dart:math';
import 'package:flutter/material.dart';
import '../gallery_tokens.dart';
import '../gallery_fx.dart';
// Primitivos eléctricos compartidos (migrados a lib/widgets/ — ADR-0138).
import '../../widgets/electric_primitives.dart';

// ---------------------------------------------------------------------------
// AccentAbSection — comparativa side-by-side de dos variantes de acento
// Parámetros: [colorA] y [colorB] con defaults de demostración histórica.
// Los colores son candidatos de "accentPrimary"; se muestran en Chrome
// (borde de foco, tab activo, chip) para evaluar legibilidad.
// Tokens de chrome: deepSpace (fondo del panel), Gx.borderBase (borde panel),
//   Gx.surfaceFill (fondo input), Gx.textBase*/textBaseMuted (texto).
// Los colores A/B son PARÁMETROS de demostración — no son chrome genérico.
// ---------------------------------------------------------------------------

// Muestra dos paneles side-by-side para que el usuario compare dos variantes
// de acento primario candidatas. Cada panel incluye:
//   • Un input simulado con borde de foco en el color de la variante
//   • Un underline de tab activo en el color de la variante
//   • Chip con el hex del color
// [colorA] — primera variante de acento (default: rojo militar #CC2B2B)
// [colorB] — segunda variante de acento (default: neutro frío #B4BFCE)
class AccentAbSection extends StatelessWidget {
  final Color colorA;
  final Color colorB;

  const AccentAbSection({
    super.key,
    // Valores de demostración histórica de las dos candidatas de acento primario.
    // Se exponen como parámetros para que el callsite pueda sustituirlos.
    this.colorA = const Color(0xFFCC2B2B),
    this.colorB = const Color(0xFFB4BFCE),
  });

  @override
  // Renderiza las dos columnas de demostración de acento lado a lado.
  Widget build(BuildContext context) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Expanded(
            child: _AccentPanel(
          color: colorA,
          label: 'A — Rojo Militar',
          hex: '#CC2B2B',
        )),
        SizedBox(width: Gx.space12),
        Expanded(
            child: _AccentPanel(
          color: colorB,
          label: 'B — Neutro Frío',
          hex: '#B4BFCE',
        )),
      ],
    );
  }
}

// ---------------------------------------------------------------------------
// _AccentPanel — panel individual de comparación A/B
// Parámetros: [color] color candidato de acento, [label] nombre, [hex] texto del chip.
// Tokens de chrome: deepSpace (fondo panel — sin glass para ver el color limpio),
//   Gx.borderBase (borde del panel), Gx.surfaceFill (fondo input),
//   Gx.textBaseSecondary (etiqueta), Gx.textBaseMuted (hint input inactivo),
//   Gx.rInput/rPanel/rChip (radios).
// [color] es el parámetro de demostración: se usa en borde de foco, tab y chip.
// ---------------------------------------------------------------------------

// Panel de comparación de una variante de acento.
// Muestra cómo se vería el token en tres usos de chrome: borde de foco,
// tab activo y chip de color.
class _AccentPanel extends StatefulWidget {
  final Color color;
  final String label;
  final String hex;

  const _AccentPanel({
    required this.color,
    required this.label,
    required this.hex,
  });

  @override
  State<_AccentPanel> createState() => _AccentPanelState();
}

class _AccentPanelState extends State<_AccentPanel> {
  // Controla si el input falso está en estado "focus".
  bool _focused = false;

  @override
  // Renderiza el panel de demostración del acento con tres usos de chrome.
  Widget build(BuildContext context) {
    // Panel sólido sobre deepSpace — sin glass para que el color destaque limpio.
    // deepSpace es el fondo idóneo para evaluar el color de acento sin ruido.
    return Container(
      padding: const EdgeInsets.all(Gx.space12 + Gx.space4 / 2),
      decoration: BoxDecoration(
        color: Gx.canvasBase,
        // Borde estructural global dinámico del panel de demostración.
        border: Border.all(color: Gx.borderBase),
        borderRadius: BorderRadius.circular(Gx.rPanel),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          // Etiqueta de la variante con token dinámico secundario.
          Text(widget.label,
              style: Gx.uiSans(
                  fontSize: 12,
                  color: Gx.textBaseSecondary,
                  weight: FontWeight.w500)),
          SizedBox(height: Gx.space8 + Gx.space4),

          // Input demo con borde de foco en el color de la variante.
          GestureDetector(
            onTap: () => setState(() => _focused = !_focused),
            child: AnimatedContainer(
              duration: const Duration(milliseconds: 200),
              padding: const EdgeInsets.symmetric(
                  horizontal: Gx.space8 + Gx.space4, vertical: 9),
              decoration: BoxDecoration(
                color: Gx.surfaceFill,
                borderRadius: BorderRadius.circular(Gx.rInput),
                border: Border.all(
                  // Foco activo: usa el color candidato de la variante.
                  // Reposo: borde estructural global dinámico.
                  color: _focused ? widget.color : Gx.borderBase,
                  width: _focused ? Gx.borderFocus : Gx.borderHairline,
                ),
                boxShadow: _focused
                    ? Gx.glow(widget.color, blur: 18, opacity: 0.40)
                    : null,
              ),
              child: Text(
                _focused ? 'Campo en foco' : 'Toca para enfocar',
                style: Gx.dataMono(
                    fontSize: 12,
                    color: _focused ? widget.color : Gx.textBaseMuted),
              ),
            ),
          ),
          SizedBox(height: Gx.space12),

          // Demo de tab activo con underline en el color de la variante.
          _TabDemo(color: widget.color),
          SizedBox(height: Gx.space12 + Gx.space4 / 2),

          // Chip con el hex del token de acento.
          Container(
            padding: const EdgeInsets.symmetric(
                horizontal: Gx.space8 + Gx.space4, vertical: Gx.space4),
            decoration: BoxDecoration(
              color: widget.color.withOpacity(0.12),
              border:
                  Border.all(color: widget.color.withOpacity(0.50)),
              borderRadius: BorderRadius.circular(Gx.rChip),
            ),
            child: Text(widget.hex,
                style: Gx.dataMono(
                    fontSize: 11, color: widget.color)),
          ),
        ],
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// _TabDemo — demo de tab con underline activo en el color de la variante
// Parámetros: [color] color candidato de acento para el underline activo.
// Tokens de chrome: Colors.transparent (borde inactivo — correcto, es invisibilidad),
//   Gx.textBaseMuted (etiqueta tab inactivo).
// ---------------------------------------------------------------------------

// Demo de tres pestañas; la activa muestra el underline en el color dado.
// Al tocar una pestaña se activa.
class _TabDemo extends StatefulWidget {
  final Color color;
  const _TabDemo({required this.color});
  @override
  State<_TabDemo> createState() => _TabDemoState();
}

class _TabDemoState extends State<_TabDemo> {
  // Índice de la pestaña activa.
  int _active = 0;

  @override
  // Renderiza las tres pestañas con underline animado.
  // El Row usa mainAxisSize.min para no pedir más ancho del que tienen sus hijos
  // y cada pestaña está envuelta en Flexible para que el Row se adapte al espacio
  // disponible sin desbordar cuando el panel es angosto (p.ej. 380px o menos).
  Widget build(BuildContext context) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: List.generate(3, (i) {
        final isActive = i == _active;
        // El último tab no lleva margen derecho para no desperdiciar espacio.
        final isLast = i == 2;
        return Flexible(
          child: GestureDetector(
            onTap: () => setState(() => _active = i),
            child: Container(
              margin: EdgeInsets.only(right: isLast ? 0 : Gx.space8),
              padding: EdgeInsets.only(bottom: Gx.space4),
              decoration: BoxDecoration(
                border: Border(
                  bottom: BorderSide(
                    // Filo neón 2px en variante activa; transparent en inactiva.
                    // Colors.transparent aquí es el "sin borde" del tab inactivo — correcto.
                    color: isActive ? widget.color : Colors.transparent,
                    width: 2,
                  ),
                ),
              ),
              child: Text(
                'Tab ${i + 1}',
                overflow: TextOverflow.ellipsis,
                style: Gx.uiSans(
                    fontSize: 12,
                    // Color del texto: el acento candidato en activa, muted en inactiva.
                    color: isActive ? widget.color : Gx.textBaseMuted),
              ),
            ),
          ),
        );
      }),
    );
  }
}

// ---------------------------------------------------------------------------
// OdometerSection — odómetro numérico (stat-card / KPI)
// Tokens de chrome: superficies via getters dinámicos (surfacePanel/surfaceCard
//   en gradiente), Gx.borderBase (borde de tarjeta), Gx.textBaseLabel (etiqueta),
//   Gx.surfaceFill + Gx.borderBase (botón Replay), Gx.textBaseSecondary (botón).
// Colores de dato: optimaCyan/reactorGreen (señalizan el valor de la KPI).
// ---------------------------------------------------------------------------

// Tres ejemplos de tarjeta KPI con odómetro: entero grande, porcentaje, valor con signo.
class OdometerSection extends StatelessWidget {
  const OdometerSection({super.key});

  @override
  // Renderiza un Wrap de tres tarjetas KPI con odómetro animado.
  Widget build(BuildContext context) {
    return Wrap(
      spacing: Gx.space12,
      runSpacing: Gx.space12,
      children: const [
        _StatCardOdometer(
          label: 'Operaciones totales',
          targetValue: 847293.0,
          decimals: 0,
          color: Gx.optimaCyan,
          prefix: '',
          suffix: '',
        ),
        _StatCardOdometer(
          label: 'Rendimiento anualizado',
          targetValue: 12.47,
          decimals: 2,
          color: Gx.reactorGreen,
          prefix: '',
          suffix: '%',
        ),
        _StatCardOdometer(
          label: 'Alpha vs benchmark',
          targetValue: 4.82,
          decimals: 2,
          color: Gx.optimaCyan,
          prefix: '+',
          suffix: '%',
        ),
      ],
    );
  }
}

// ---------------------------------------------------------------------------
// _StatCardOdometer — tarjeta KPI individual con odómetro animado
// Parámetros: [label] etiqueta, [targetValue] valor destino, [decimals] precisión,
//   [color] color semántico del valor, [prefix]/[suffix] texto adicional.
// Tokens de chrome: gradiente surfacePanel→surfaceCard (superficie dinámica),
//   Gx.borderBase (borde de tarjeta), Gx.textBaseLabel (etiqueta),
//   Gx.surfaceFill + Gx.borderBase + Gx.textBaseSecondary (botón Replay).
// ---------------------------------------------------------------------------

// Tarjeta KPI con odómetro: anima el número desde 0 al valor destino al montarse.
// Botón Replay para repetir la animación.
class _StatCardOdometer extends StatefulWidget {
  final String label;
  final double targetValue;
  final int decimals;
  final Color color;
  final String prefix;
  final String suffix;

  const _StatCardOdometer({
    required this.label,
    required this.targetValue,
    required this.decimals,
    required this.color,
    required this.prefix,
    required this.suffix,
  });

  @override
  State<_StatCardOdometer> createState() => _StatCardOdometerState();
}

class _StatCardOdometerState extends State<_StatCardOdometer>
    with SingleTickerProviderStateMixin {
  late final AnimationController _ctrl;
  late final Animation<double> _anim;

  @override
  void initState() {
    super.initState();
    // 500ms, easeOut — regla universal del odómetro.
    _ctrl = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 500),
    );
    _anim = CurvedAnimation(parent: _ctrl, curve: Curves.easeOut);
    // Arranca al montarse.
    _ctrl.forward();
  }

  @override
  void dispose() {
    _ctrl.dispose();
    super.dispose();
  }

  // Reinicia la animación desde cero (para el botón Replay).
  void _replay() => _ctrl.forward(from: 0.0);

  // Formatea el valor interpolado con la precisión del destino.
  // Usa separador de miles solo cuando la magnitud supera 9999 y no tiene unidad.
  String _format(double v) {
    if (widget.decimals == 0) {
      final n = v.toInt();
      if (n >= 10000) {
        // Formatea con puntos como separador de miles.
        final s = n.toString();
        final buf = StringBuffer();
        var count = 0;
        for (var i = s.length - 1; i >= 0; i--) {
          if (count > 0 && count % 3 == 0) buf.write('.');
          buf.write(s[i]);
          count++;
        }
        return buf.toString().split('').reversed.join();
      }
      return n.toString();
    }
    return v.toStringAsFixed(widget.decimals);
  }

  @override
  // Tarjeta KPI con gradiente dinámico de superficie, borde estructural y odómetro animado.
  Widget build(BuildContext context) {
    return SizedBox(
      width: 180,
      child: PanelFromDecoration(
        decoration: BoxDecoration(
          gradient: LinearGradient(
            begin: Alignment.topCenter,
            end: Alignment.bottomCenter,
            // Getters dinámicos: reaccionan al modo glass/tint/solid.
            colors: [Gx.surfacePanel, Gx.surfaceCard],
          ),
          // Borde estructural global dinámico.
          border: Border.all(color: Gx.borderBase),
          borderRadius: BorderRadius.circular(Gx.rPanel),
          boxShadow: Gx.glow(widget.color, blur: 20, opacity: 0.10),
        ),
        padding: const EdgeInsets.all(Gx.space12 + Gx.space4 / 2),
        child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          // Etiqueta de la métrica con token label dinámico.
          Text(widget.label,
              style: Gx.uiSans(fontSize: 12, color: Gx.textBaseLabel)),
          SizedBox(height: Gx.space8),

          // Número animado con odómetro y glow semántico.
          AnimatedBuilder(
            animation: _anim,
            builder: (_, __) {
              final v = _anim.value * widget.targetValue;
              return Text(
                '${widget.prefix}${_format(v)}${widget.suffix}',
                style: Gx.dataMono(
                  fontSize: 28,
                  height: 1.1,
                  color: widget.color,
                ).copyWith(shadows: Gx.textGlow(widget.color)),
              );
            },
          ),
          SizedBox(height: Gx.space8 + Gx.space4),

          // Botón Replay con borde estructural global.
          GestureDetector(
            onTap: _replay,
            child: Container(
              padding: const EdgeInsets.symmetric(
                  horizontal: Gx.space8 + Gx.space4, vertical: Gx.space4),
              decoration: BoxDecoration(
                color: Gx.surfaceFill,
                border: Border.all(color: Gx.borderBase),
                borderRadius: BorderRadius.circular(Gx.rChip),
              ),
              child: Text('Replay',
                  style: Gx.uiSans(
                      fontSize: 11, color: Gx.textBaseSecondary)),
            ),
          ),
        ],
      ),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// GaugeSection — cuatro gauges radiales animados con colores semánticos
// Tokens de chrome: superficies vía getters dinámicos, Gx.borderBase,
//   Gx.textBaseLabel (etiqueta), Gx.textBaseSecondary (botón Replay).
// Colores de dato: los cuatro pares semánticos (óptimo/trans/alerta/crítico).
// ---------------------------------------------------------------------------

// Cuatro versiones del gauge radial con colores semánticos: óptimo, transición,
// alerta y crítico. Cada uno anima su arco y número central al montarse.
class GaugeSection extends StatelessWidget {
  const GaugeSection({super.key});

  @override
  // Renderiza un Wrap de cuatro gauges radiales.
  Widget build(BuildContext context) {
    return Wrap(
      spacing: Gx.space16,
      runSpacing: Gx.space16,
      children: const [
        _AnimatedGauge(
          label: 'Sharpe Ratio',
          value: 0.82,
          displayValue: '2.41',
          color: Gx.optimaCyan,
          gradColors: Gx.gradOptima,
        ),
        _AnimatedGauge(
          label: 'Exposición',
          value: 0.54,
          displayValue: '54%',
          color: Gx.transitionIndigo,
          gradColors: Gx.gradTransition,
        ),
        _AnimatedGauge(
          label: 'Volatilidad',
          value: 0.68,
          displayValue: '68%',
          color: Gx.alertAmber,
          gradColors: Gx.gradAlert,
        ),
        _AnimatedGauge(
          label: 'Drawdown máx.',
          value: 0.91,
          displayValue: '−22.3%',
          color: Gx.criticalCrimson,
          gradColors: Gx.gradCritical,
        ),
      ],
    );
  }
}

// ---------------------------------------------------------------------------
// _AnimatedGauge — gauge radial individual con arco animado y odómetro central
// Parámetros: [label] etiqueta, [value] fracción 0–1, [displayValue] texto central,
//   [color] color semántico, [gradColors] colores del gradiente del arco.
// Tokens de chrome: gradiente dinámico surfacePanel→surfaceCard (superficie),
//   Gx.borderBase (borde), Gx.textBaseLabel (etiqueta), Gx.textBaseSecondary (Replay).
// ---------------------------------------------------------------------------

// Gauge radial con arco que barre desde 0 hasta value*totalSweep al montarse.
// El número central también interpola desde 0% con odómetro simultáneo.
class _AnimatedGauge extends StatefulWidget {
  final String label;
  final double value;
  final String displayValue;
  final Color color;
  final List<Color> gradColors;

  const _AnimatedGauge({
    required this.label,
    required this.value,
    required this.displayValue,
    required this.color,
    required this.gradColors,
  });

  @override
  State<_AnimatedGauge> createState() => _AnimatedGaugeState();
}

class _AnimatedGaugeState extends State<_AnimatedGauge>
    with SingleTickerProviderStateMixin {
  late final AnimationController _ctrl;
  late final Animation<double> _anim;

  @override
  void initState() {
    super.initState();
    // 600ms, easeOut — regla universal del arco animado.
    _ctrl = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 600),
    );
    _anim = CurvedAnimation(parent: _ctrl, curve: Curves.easeOut);
    _ctrl.forward();
  }

  @override
  void dispose() {
    _ctrl.dispose();
    super.dispose();
  }

  // Reinicia la animación desde cero.
  void _replay() => _ctrl.forward(from: 0.0);

  @override
  // Tarjeta de gauge con gradiente dinámico, arco CustomPainter y botón Replay.
  Widget build(BuildContext context) {
    return SizedBox(
      width: 150,
      child: PanelFromDecoration(
        decoration: BoxDecoration(
          gradient: LinearGradient(
            begin: Alignment.topCenter,
            end: Alignment.bottomCenter,
            // Getters dinámicos: reaccionan al modo global.
            colors: [Gx.surfacePanel, Gx.surfaceCard],
          ),
          border: Border.all(color: Gx.borderBase),
          borderRadius: BorderRadius.circular(Gx.rPanel),
          boxShadow: Gx.glow(widget.color, blur: 16, opacity: 0.12),
        ),
        padding: const EdgeInsets.all(Gx.space12 + Gx.space4 / 2),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          // Lienzo del gauge con arco animado y número central.
          AnimatedBuilder(
            animation: _anim,
            builder: (_, __) {
              return SizedBox(
                width: 110,
                height: 110,
                child: CustomPaint(
                  painter: _GaugePainter(
                    progress: _anim.value,
                    targetFraction: widget.value,
                    color: widget.color,
                    gradColors: widget.gradColors,
                    displayValue: widget.displayValue,
                  ),
                ),
              );
            },
          ),
          SizedBox(height: Gx.space8),

          // Etiqueta del gauge con token label dinámico.
          Text(widget.label,
              style:
                  Gx.uiSans(fontSize: 11, color: Gx.textBaseLabel),
              textAlign: TextAlign.center),
          SizedBox(height: Gx.space8),

          // Botón Replay con borde estructural global.
          GestureDetector(
            onTap: _replay,
            child: Container(
              padding: const EdgeInsets.symmetric(
                  horizontal: Gx.space8 + Gx.space4, vertical: 3),
              decoration: BoxDecoration(
                color: Gx.surfaceFill,
                border: Border.all(color: Gx.borderBase),
                borderRadius: BorderRadius.circular(Gx.rChip),
              ),
              child: Text('Replay',
                  style: Gx.uiSans(
                      fontSize: 10,
                      color: Gx.textBaseSecondary)),
            ),
          ),
        ],
      ),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// _GaugePainter — painter del arco radial del gauge
// Recibe [progress] (0.0–1.0) de la animación y traza el arco hasta
// (progress × targetFraction × totalSweep). El número central también
// se interpola desde 0 hasta targetFraction.
// Tokens de chrome: Gx.divider (riel del gauge).
// El glow del arco usa MaskFilter.blur — aceptado porque está en CustomPainter
// accionado por AnimationController (no en bucle de hover perpetuo).
// ---------------------------------------------------------------------------

// Painter del arco radial animado con riel de fondo y número central interpolado.
class _GaugePainter extends CustomPainter {
  final double progress; // progreso de la animación (0.0–1.0)
  final double targetFraction; // fracción destino del gauge (0.0–1.0)
  final Color color;
  final List<Color> gradColors;
  final String displayValue;

  const _GaugePainter({
    required this.progress,
    required this.targetFraction,
    required this.color,
    required this.gradColors,
    required this.displayValue,
  });

  @override
  // Dibuja el riel del gauge, el arco animado con glow y el número central interpolado.
  void paint(Canvas canvas, Size size) {
    final center = Offset(size.width / 2, size.height / 2);
    final radius = size.shortestSide / 2 - 8;

    // Ángulo total del arco: de −200° a +20° (240° de barrido).
    const startAngle = -200.0 * pi / 180;
    const totalSweep = 240.0 * pi / 180;

    // Riel del gauge: divider es el token de separadores — apropiado para el fondo.
    canvas.drawArc(
      Rect.fromCircle(center: center, radius: radius),
      startAngle,
      totalSweep,
      false,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 8
        ..strokeCap = StrokeCap.round
        ..color = Gx.divider,
    );

    // Arco animado: sweepAngle va de 0 hasta (progress × targetFraction × totalSweep).
    final currentSweep = progress * targetFraction * totalSweep;
    if (currentSweep > 0.01) {
      // Glow del arco — MaskFilter.blur en CustomPainter accionado por animación: aceptado.
      canvas.drawArc(
        Rect.fromCircle(center: center, radius: radius),
        startAngle,
        currentSweep,
        false,
        Paint()
          ..style = PaintingStyle.stroke
          ..strokeWidth = 8
          ..strokeCap = StrokeCap.round
          ..color = color.withAlpha(70)
          ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 6),
      );
      // Arco nítido con degradado semántico.
      canvas.drawArc(
        Rect.fromCircle(center: center, radius: radius),
        startAngle,
        currentSweep,
        false,
        Paint()
          ..style = PaintingStyle.stroke
          ..strokeWidth = 6
          ..strokeCap = StrokeCap.round
          ..shader = SweepGradient(
            startAngle: startAngle,
            endAngle: startAngle + currentSweep,
            colors: gradColors,
          ).createShader(Rect.fromCircle(center: center, radius: radius)),
      );
    }

    // Número central interpolado con odómetro simultáneo.
    // Interpola el porcentaje de 0 a (progress × targetFraction × 100).
    final pct =
        (progress * targetFraction * 100).toStringAsFixed(0);
    final tp = TextPainter(
      text: TextSpan(
        text: '$pct%',
        style: Gx.dataMono(
          fontSize: 20,
          height: 1.1,
          color: color,
        ).copyWith(shadows: Gx.textGlow(color)),
      ),
      textDirection: TextDirection.ltr,
    )..layout();
    tp.paint(
      canvas,
      Offset(
          center.dx - tp.width / 2, center.dy - tp.height / 2),
    );
  }

  @override
  bool shouldRepaint(_GaugePainter old) =>
      old.progress != progress || old.targetFraction != targetFraction;
}

// ---------------------------------------------------------------------------
// EquityCurveAnimated — equity-curve con path drawing y efecto eléctrico
// Tokens de chrome: gradiente dinámico surfacePanel→surfaceCard (superficie),
//   Gx.borderBase (borde panel), Gx.textBaseSecondary (ícono cabecera),
//   Gx.surfaceFill + Gx.borderBase + Gx.textBaseSecondary (botón Replay).
// Colores de dato: optimaCyan (línea de equity — señal alcista, se conserva).
// ---------------------------------------------------------------------------

// Widget que muestra la curva de equity con animación de path drawing al
// montarse: la línea se traza progresivamente de izquierda a derecha
// en 800ms con Curves.easeOut. El área de relleno progresa junto con la línea.
// Incluye botón "Replay".
class EquityCurveAnimated extends StatefulWidget {
  const EquityCurveAnimated({super.key});

  @override
  State<EquityCurveAnimated> createState() => _EquityCurveAnimatedState();
}

class _EquityCurveAnimatedState extends State<EquityCurveAnimated>
    with SingleTickerProviderStateMixin {
  // Controller único de 1200ms:
  // • 0.0–0.8: path drawing + scan eléctrico (progress 0→1 del scan)
  // • 0.8–1.0: scan desaparece (fade opacity 1→0)
  late final AnimationController _ctrl;

  @override
  void initState() {
    super.initState();
    _ctrl = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 1200),
    );
    _ctrl.forward();
  }

  @override
  void dispose() {
    _ctrl.dispose();
    super.dispose();
  }

  // Reinicia la animación desde cero.
  void _replay() => _ctrl.forward(from: 0.0);

  @override
  // Panel con gradiente dinámico que contiene el lienzo de la curva y el botón Replay.
  Widget build(BuildContext context) {
    return SizedBox(
      width: 360,
      child: PanelFromDecoration(
        decoration: BoxDecoration(
          gradient: LinearGradient(
            begin: Alignment.topCenter,
            end: Alignment.bottomCenter,
            // Getters dinámicos: reaccionan al modo global.
            colors: [Gx.surfacePanel, Gx.surfaceCard],
          ),
          border: Border.all(color: Gx.borderBase),
          borderRadius: BorderRadius.circular(Gx.rPanel),
          boxShadow: Gx.glow(Gx.optimaCyan, blur: 20, opacity: 0.08),
        ),
        padding: const EdgeInsets.all(Gx.space12),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          // Cabecera del panel con ícono y título.
          Row(children: [
            // Icon Material de chart — no tiene token propio; color secundario dinámico.
            Icon(Icons.show_chart, size: 14, color: Gx.textBaseSecondary),
            SizedBox(width: Gx.space4 + Gx.space4 / 2),
            Text('Equity Curve',
                style: Gx.panelTitle
                    .copyWith(color: Gx.textBaseSecondary)),
          ]),
          SizedBox(height: Gx.space8),

          // Lienzo del gráfico con path drawing + efecto eléctrico.
          AnimatedBuilder(
            animation: _ctrl,
            builder: (_, __) {
              const crossFraction = 0.8;
              final p = _ctrl.value;
              // scanProgress: fracción del cruce del scan (0→1 durante el 80% inicial).
              final scanProgress =
                  p <= crossFraction ? p / crossFraction : 1.0;
              // scanOpacity: 1.0 mientras cruza, luego fade hasta 0.
              final scanOpacity = p <= crossFraction
                  ? 1.0
                  : 1.0 -
                      ((p - crossFraction) / (1.0 - crossFraction));
              return SizedBox(
                height: 120,
                child: CustomPaint(
                  painter: _EquityCurveElectricPainter(
                    scanProgress: scanProgress,
                    scanOpacity: scanOpacity.clamp(0.0, 1.0),
                  ),
                  size: Size.infinite,
                ),
              );
            },
          ),
          SizedBox(height: Gx.space8),

          // Botón Replay con borde estructural global.
          GestureDetector(
            onTap: _replay,
            child: Container(
              padding: const EdgeInsets.symmetric(
                  horizontal: Gx.space8 + Gx.space4, vertical: Gx.space4),
              decoration: BoxDecoration(
                color: Gx.surfaceFill,
                border: Border.all(color: Gx.borderBase),
                borderRadius: BorderRadius.circular(Gx.rChip),
              ),
              child: Text('Replay',
                  style: Gx.uiSans(
                      fontSize: 11,
                      color: Gx.textBaseSecondary)),
            ),
          ),
        ],
      ),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// _EquityCurveElectricPainter — painter de equity curve con efecto eléctrico
// Recibe [scanProgress] (0.0–1.0): el scan avanza y revela la línea mientras
// la ilumina con ignición eléctrica (intensidad que decae exponencialmente).
// Tras la línea: comet tail + scan line con [scanOpacity].
// Tokens de dato: optimaCyan (curva de equity alcista — se conserva).
// MaskFilter.blur condicionado a intensity > umbral — no en bucle perpetuo; aceptado.
// ---------------------------------------------------------------------------

// Painter de la equity curve con efecto eléctrico integrado al avance del scan.
class _EquityCurveElectricPainter extends CustomPainter {
  final double scanProgress; // fracción 0→1 del cruce del scan
  final double scanOpacity; // 1.0 mientras cruza, fade a 0 en los últimos 200ms

  // Puntos de la curva de equity (sintéticos, siempre los mismos).
  static const _pts = [
    0.00, 0.04, 0.08, 0.06, 0.11, 0.16, 0.13, 0.20, 0.25, 0.22,
    0.27, 0.32, 0.28, 0.35, 0.38, 0.35, 0.41, 0.45, 0.42, 0.48,
    0.52, 0.48, 0.45, 0.50, 0.54, 0.58, 0.56, 0.62, 0.66, 0.70,
  ];

  const _EquityCurveElectricPainter({
    required this.scanProgress,
    required this.scanOpacity,
  });

  @override
  // Dibuja el área de relleno progresiva, los segmentos de línea con ignición eléctrica,
  // el comet tail y la línea de scan con fade al final.
  void paint(Canvas canvas, Size size) {
    final n = _pts.length;
    final dx = size.width / (n - 1);
    const pad = 10.0;
    final h = size.height - pad;
    final maxV = _pts.reduce(max);
    final scanX = scanProgress * size.width; // posición X actual del scanner

    double toY(double v) => pad + h * (1 - v / maxV);

    // Relleno de área bajo la curva (solo hasta scanX para seguir el scan).
    final visibleCount = max(1, (scanProgress * n).floor());
    if (visibleCount > 1) {
      final fill = Path()..moveTo(0, toY(_pts[0]));
      for (var i = 1; i < visibleCount; i++) {
        fill.lineTo(i * dx, toY(_pts[i]));
      }
      fill
        ..lineTo((visibleCount - 1) * dx, size.height)
        ..lineTo(0, size.height)
        ..close();
      canvas.drawPath(
        fill,
        Paint()
          ..shader = LinearGradient(
            begin: Alignment.topCenter,
            end: Alignment.bottomCenter,
            // optimaCyan: color semántico de la curva alcista (dato — se conserva).
            colors: [Gx.optimaCyan.withAlpha(22), Colors.transparent],
          ).createShader(Offset.zero & size),
      );
    }

    // Dibuja cada segmento de la línea con ignición eléctrica basada en proximidad al scan.
    for (var i = 0; i < n - 1; i++) {
      final x0 = i * dx;
      if (x0 >= scanX) break;

      // Intensidad de ignición: máxima en el frente del scan, decae exponencialmente.
      final intensity = electricIntensity(x0, scanX, size.width);
      final effectiveOpacity = 0.7 + intensity * 0.3;
      final extraStroke = intensity * 2.0;

      final y0 = toY(_pts[i]);
      final y1 = toY(_pts[i + 1]);

      // Glow difuso eléctrico — MaskFilter.blur condicionado a intensity > 0.05: aceptado.
      if (intensity > 0.05) {
        canvas.drawLine(
          Offset(x0, y0),
          Offset(x0 + dx, y1),
          Paint()
            ..color = Gx.optimaCyan.withOpacity(intensity * 0.45)
            ..strokeWidth = 1.5 + extraStroke + 4
            ..maskFilter =
                MaskFilter.blur(BlurStyle.normal, 5 + intensity * 14),
        );
      }

      // Línea nítida con color del dato (optimaCyan).
      canvas.drawLine(
        Offset(x0, y0),
        Offset(x0 + dx, y1),
        Paint()
          ..color = Gx.optimaCyan.withOpacity(effectiveOpacity)
          ..strokeWidth = 1.5 + extraStroke,
      );
    }

    // Comet tail y scan line con efecto eléctrico del mixin compartido.
    paintCometTail(canvas, scanX, size, Gx.optimaCyan);
    paintScanLine(
        canvas, scanX, size.height, Gx.optimaCyan, scanOpacity);
  }

  @override
  bool shouldRepaint(_EquityCurveElectricPainter old) =>
      old.scanProgress != scanProgress || old.scanOpacity != scanOpacity;
}
