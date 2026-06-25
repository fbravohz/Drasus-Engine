// Sección de animaciones universales de DESIGN.md:
//   • Acento Primario A/B — elección de variante de accentPrimary
//   • Odómetro numérico — todo número dinámico anima desde 0.0 (regla universal)
//   • Gauge radial con arco animado — sweepAngle 0→final (regla universal)
//   • Equity-curve con path drawing + efecto eléctrico (ElectricScanMixin)
//
// Todos los widgets manejan estado local de UI (AnimationController, valor
// numérico interpolado). Sin lógica de negocio ni FFI — Cáscara Delgada.

import 'dart:math';
import 'package:flutter/material.dart';
import '../gallery_tokens.dart';
// Primitivos eléctricos compartidos (migrados a lib/widgets/ — ADR-0138).
import '../../widgets/electric_primitives.dart';

// ---------------------------------------------------------------------------
// Tarea 2 — Sección "Acento Primario A/B"
// ---------------------------------------------------------------------------

// Muestra dos paneles side-by-side para que el usuario compare la variante A
// (Rojo Militar) y la variante B (Neutro Frío). Cada panel incluye:
//   • Un input simulado con borde de foco en el color de la variante
//   • Un underline de tab activo en el color de la variante
//   • Etiqueta y chip con el hex
// Fondo de ambos paneles: deepSpace.
class AccentAbSection extends StatelessWidget {
  const AccentAbSection({super.key});

  @override
  Widget build(BuildContext context) {
    // Muestra las dos variantes de acento primario en columnas iguales.
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Expanded(child: _AccentPanel(
          color: const Color(0xFFCC2B2B),
          label: 'A — Rojo Militar',
          hex: '#CC2B2B',
        )),
        const SizedBox(width: 12),
        Expanded(child: _AccentPanel(
          color: const Color(0xFFB4BFCE),
          label: 'B — Neutro Frío',
          hex: '#B4BFCE',
        )),
      ],
    );
  }
}

// Panel individual de comparación A/B. Muestra los usos del token en Chrome:
// borde de foco, tab activo, chip con el hex.
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
  Widget build(BuildContext context) {
    // Panel sólido sobre deepSpace — sin glass para que el color destaque limpio.
    return Container(
      padding: const EdgeInsets.all(14),
      decoration: BoxDecoration(
        color: Gx.deepSpace,
        border: Border.all(color: Gx.borderPanel),
        borderRadius: BorderRadius.circular(Gx.rPanel),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          // Etiqueta de variante.
          Text(widget.label,
              style: Gx.uiSans(
                  fontSize: 12,
                  color: Gx.textSecondary,
                  weight: FontWeight.w500)),
          const SizedBox(height: 10),

          // Input con borde de foco en el color de la variante.
          GestureDetector(
            onTap: () => setState(() => _focused = !_focused),
            child: AnimatedContainer(
              duration: const Duration(milliseconds: 200),
              padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 9),
              decoration: BoxDecoration(
                color: Gx.surfaceFill,
                borderRadius: BorderRadius.circular(Gx.rInput),
                border: Border.all(
                  color: _focused ? widget.color : Gx.borderPanel,
                  width: _focused ? 1.5 : 1.0,
                ),
                // Glow limpio de foco (sin aberración cromática).
                boxShadow: _focused
                    ? Gx.glow(widget.color, blur: 18, opacity: 0.40)
                    : null,
              ),
              child: Text(
                _focused ? 'Campo en foco' : 'Toca para enfocar',
                style: Gx.dataMono(
                    fontSize: 12,
                    color: _focused ? widget.color : Gx.textMuted),
              ),
            ),
          ),
          const SizedBox(height: 12),

          // Tab activo: underline de 2px en el color de la variante.
          _TabDemo(color: widget.color),
          const SizedBox(height: 14),

          // Chip con el hex del token.
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
            decoration: BoxDecoration(
              color: widget.color.withOpacity(0.12),
              border: Border.all(color: widget.color.withOpacity(0.50)),
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

// Demo de tab con underline activo en el color de la variante.
// Muestra tres pestañas; la primera siempre está activa.
class _TabDemo extends StatefulWidget {
  final Color color;
  const _TabDemo({required this.color});
  @override
  State<_TabDemo> createState() => _TabDemoState();
}

class _TabDemoState extends State<_TabDemo> {
  int _active = 0;
  @override
  Widget build(BuildContext context) {
    return Row(
      children: List.generate(3, (i) {
        final isActive = i == _active;
        return GestureDetector(
          onTap: () => setState(() => _active = i),
          child: Container(
            margin: const EdgeInsets.only(right: 12),
            padding: const EdgeInsets.only(bottom: 4),
            decoration: BoxDecoration(
              border: Border(
                bottom: BorderSide(
                  // Filo neón 2px en variante activa, transparente en inactiva.
                  color: isActive ? widget.color : Colors.transparent,
                  width: 2,
                ),
              ),
            ),
            child: Text('Tab ${i + 1}',
                style: Gx.uiSans(
                    fontSize: 12,
                    color: isActive ? widget.color : Gx.textMuted)),
          ),
        );
      }),
    );
  }
}

// ---------------------------------------------------------------------------
// Tarea 3 — Odómetro numérico (stat-card / kpi)
// ---------------------------------------------------------------------------

// Widget que anima un número desde 0.0 hasta el valor destino usando
// AnimationController + Tween<double> + CurvedAnimation(Curves.easeOut).
// El texto formatea el double interpolado con la misma precisión decimal
// que el valor destino, nunca salta al valor final.
// Incluye botón "Replay" para repetir el efecto.
class OdometerSection extends StatelessWidget {
  const OdometerSection({super.key});

  @override
  Widget build(BuildContext context) {
    // Tres ejemplos: entero grande, porcentaje, valor con signo.
    return Wrap(
      spacing: 12,
      runSpacing: 12,
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

// Tarjeta KPI individual con odómetro. Anima al montarse.
// [label] — etiqueta de la métrica (sans 12px en textLabel)
// [targetValue] — valor destino de la animación
// [decimals] — decimales a mostrar durante toda la animación
// [color] — color semántico del valor (neón con textGlow)
// [prefix] / [suffix] — texto antes/después del número (signo, unidad)
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
  // Usa separador de miles solo cuando el destino no tiene sufijo de unidad
  // y la magnitud supera 9999.
  String _format(double v) {
    if (widget.decimals == 0) {
      // Entero — separador de miles con espacio fino para legibilidad.
      final n = v.toInt();
      if (n >= 10000) {
        // Formatea manualmente con puntos como separador de miles.
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
  Widget build(BuildContext context) {
    // Tarjeta KPI sólida: panelSolid con hairline y glow tenue del estado.
    return Container(
      width: 180,
      padding: const EdgeInsets.all(14),
      decoration: BoxDecoration(
        gradient: LinearGradient(
          begin: Alignment.topCenter,
          end: Alignment.bottomCenter,
          colors: [Gx.surfacePanel, Gx.surfaceCard],
        ),
        border: Border.all(color: Gx.borderPanel),
        borderRadius: BorderRadius.circular(Gx.rPanel),
        boxShadow: Gx.glow(widget.color, blur: 20, opacity: 0.10),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          // Etiqueta de la métrica.
          Text(widget.label,
              style: Gx.uiSans(fontSize: 12, color: Gx.textLabel)),
          const SizedBox(height: 8),

          // Número animado con odómetro.
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
          const SizedBox(height: 10),

          // Botón Replay — reinicia el odómetro desde cero.
          GestureDetector(
            onTap: _replay,
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
              decoration: BoxDecoration(
                color: Gx.surfaceFill,
                border: Border.all(color: Gx.borderPanel),
                borderRadius: BorderRadius.circular(Gx.rChip),
              ),
              child: Text('Replay',
                  style: Gx.uiSans(fontSize: 11, color: Gx.textSecondary)),
            ),
          ),
        ],
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Tarea 4 — Gauge radial con arco animado
// ---------------------------------------------------------------------------

// Muestra cuatro versiones del gauge radial con colores semánticos:
// óptimo (optimaCyan), transición (transitionIndigo), alerta (alertAmber),
// crítico (criticalCrimson). Cada gauge anima su arco y su número central
// simultáneamente al montarse. Incluye botón Replay.
class GaugeSection extends StatelessWidget {
  const GaugeSection({super.key});

  @override
  Widget build(BuildContext context) {
    return Wrap(
      spacing: 16,
      runSpacing: 16,
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

// Gauge radial individual con arco animado y odómetro central simultáneo.
// [value] — fracción 0.0–1.0 que determina el ángulo final del arco
// [displayValue] — texto que aparece en el centro (el odómetro usa [value])
// [color] — color semántico del estado
// [gradColors] — colores para el degradado del arco
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

  void _replay() => _ctrl.forward(from: 0.0);

  @override
  Widget build(BuildContext context) {
    return Container(
      width: 150,
      padding: const EdgeInsets.all(14),
      decoration: BoxDecoration(
        gradient: LinearGradient(
          begin: Alignment.topCenter,
          end: Alignment.bottomCenter,
          colors: [Gx.surfacePanel, Gx.surfaceCard],
        ),
        border: Border.all(color: Gx.borderPanel),
        borderRadius: BorderRadius.circular(Gx.rPanel),
        boxShadow: Gx.glow(widget.color, blur: 16, opacity: 0.12),
      ),
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
          const SizedBox(height: 8),

          // Etiqueta del gauge.
          Text(widget.label,
              style: Gx.uiSans(fontSize: 11, color: Gx.textLabel),
              textAlign: TextAlign.center),
          const SizedBox(height: 8),

          // Botón Replay.
          GestureDetector(
            onTap: _replay,
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 3),
              decoration: BoxDecoration(
                color: Gx.surfaceFill,
                border: Border.all(color: Gx.borderPanel),
                borderRadius: BorderRadius.circular(Gx.rChip),
              ),
              child: Text('Replay',
                  style: Gx.uiSans(fontSize: 10, color: Gx.textSecondary)),
            ),
          ),
        ],
      ),
    );
  }
}

// Painter del arco radial del gauge. Recibe [progress] (0.0–1.0) de la
// animación y traza el arco hasta (progress × targetFraction × ángulo total).
// El número central también se interpola desde 0 hasta [targetFraction].
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
  void paint(Canvas canvas, Size size) {
    final center = Offset(size.width / 2, size.height / 2);
    final radius = size.shortestSide / 2 - 8;

    // Ángulo total del arco: de -200° a +20° (240° de barrido).
    const startAngle = -200.0 * pi / 180;
    const totalSweep = 240.0 * pi / 180;

    // Riel del gauge (fondo del arco).
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
      canvas.drawArc(
        Rect.fromCircle(center: center, radius: radius),
        startAngle,
        currentSweep,
        false,
        Paint()
          ..style = PaintingStyle.stroke
          ..strokeWidth = 8
          ..strokeCap = StrokeCap.round
          // Glow del arco con color semántico.
          ..color = color.withAlpha(70)
          ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 6),
      );
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
    final pct = (progress * targetFraction * 100).toStringAsFixed(0);
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
      Offset(center.dx - tp.width / 2, center.dy - tp.height / 2),
    );
  }

  @override
  bool shouldRepaint(_GaugePainter old) =>
      old.progress != progress || old.targetFraction != targetFraction;
}

// ---------------------------------------------------------------------------
// Tarea 5 — Equity-curve con path drawing
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

  void _replay() => _ctrl.forward(from: 0.0);

  @override
  Widget build(BuildContext context) {
    // Panel sólido que contiene el lienzo y el botón Replay.
    return Container(
      width: 360,
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        gradient: LinearGradient(
          begin: Alignment.topCenter,
          end: Alignment.bottomCenter,
          colors: [Gx.surfacePanel, Gx.surfaceCard],
        ),
        border: Border.all(color: Gx.borderPanel),
        borderRadius: BorderRadius.circular(Gx.rPanel),
        boxShadow: Gx.glow(Gx.optimaCyan, blur: 20, opacity: 0.08),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          // Cabecera del panel.
          Row(children: [
            Icon(Icons.show_chart, size: 14, color: Gx.textSecondary),
            const SizedBox(width: 6),
            Text('Equity Curve', style: Gx.panelTitle),
          ]),
          const SizedBox(height: 8),

          // Lienzo del gráfico con path drawing + efecto eléctrico.
          AnimatedBuilder(
            animation: _ctrl,
            builder: (_, __) {
              const crossFraction = 0.8;
              final p = _ctrl.value;
              // scanProgress: fracción del cruce del scan.
              final scanProgress =
                  p <= crossFraction ? p / crossFraction : 1.0;
              // scanOpacity: 1.0 durante el cruce, luego fade.
              final scanOpacity = p <= crossFraction
                  ? 1.0
                  : 1.0 - ((p - crossFraction) / (1.0 - crossFraction));
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
          const SizedBox(height: 8),

          // Botón Replay para repetir el trazado y el efecto eléctrico.
          GestureDetector(
            onTap: _replay,
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
              decoration: BoxDecoration(
                color: Gx.surfaceFill,
                border: Border.all(color: Gx.borderPanel),
                borderRadius: BorderRadius.circular(Gx.rChip),
              ),
              child: Text('Replay',
                  style: Gx.uiSans(fontSize: 11, color: Gx.textSecondary)),
            ),
          ),
        ],
      ),
    );
  }
}

// Painter de la equity curve con efecto eléctrico integrado.
// Recibe [scanProgress] (0.0–1.0): el scan avanza y revela la línea mientras
// la ilumina con ignición eléctrica (intensidad que decae exponencialmente).
// Tras todas las líneas: comet tail + scan line con [scanOpacity].
class _EquityCurveElectricPainter extends CustomPainter {
  final double scanProgress; // fracción 0→1 del cruce del scan
  final double scanOpacity;  // 1.0 mientras cruza, fade a 0 en los últimos 200ms

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
            colors: [Gx.optimaCyan.withAlpha(22), Colors.transparent],
          ).createShader(Offset.zero & size),
      );
    }

    // Dibuja cada segmento de la línea con ignición eléctrica.
    for (var i = 0; i < n - 1; i++) {
      final x0 = i * dx;
      if (x0 >= scanX) break;

      // Calcula la intensidad de ignición eléctrica para este segmento.
      final intensity = electricIntensity(x0, scanX, size.width);
      final effectiveOpacity = 0.7 + intensity * 0.3;
      final extraStroke = intensity * 2.0;

      final y0 = toY(_pts[i]);
      final y1 = toY(_pts[i + 1]);

      // Glow difuso eléctrico.
      if (intensity > 0.05) {
        canvas.drawLine(
          Offset(x0, y0), Offset(x0 + dx, y1),
          Paint()
            ..color = Gx.optimaCyan.withOpacity(intensity * 0.45)
            ..strokeWidth = 1.5 + extraStroke + 4
            ..maskFilter = MaskFilter.blur(BlurStyle.normal, 5 + intensity * 14),
        );
      }

      // Línea nítida con degradado gradOptima (usando interpolación de color manual).
      canvas.drawLine(
        Offset(x0, y0), Offset(x0 + dx, y1),
        Paint()
          ..color = Gx.optimaCyan.withOpacity(effectiveOpacity)
          ..strokeWidth = 1.5 + extraStroke,
      );
    }

    // Comet tail y scan line con efecto eléctrico.
    paintCometTail(canvas, scanX, size, Gx.optimaCyan);
    paintScanLine(canvas, scanX, size.height, Gx.optimaCyan, scanOpacity);
  }

  @override
  bool shouldRepaint(_EquityCurveElectricPainter old) =>
      old.scanProgress != scanProgress || old.scanOpacity != scanOpacity;
}
