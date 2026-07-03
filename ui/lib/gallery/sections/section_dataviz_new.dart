// §10 Data-viz nuevos: monte-carlo-lines y strategy-cluster-3d.
// Ambos son widgets de galería con datos sintéticos (random walk / puntos
// aleatorios) — sin Rust ni FFI. La lógica de generación de datos vive aquí
// porque es exclusivamente visual (seed fija, no financiera).
//
// Cumple DESIGN.md §10 y §Motion Philosophy:
//   • ElectricScanMixin integrado al painter principal (una sola vez al montar)
//   • scanInitLine con comet tail + ignición eléctrica por segmento
//   • Rotación orbital continua con AnimationController infinito (cluster 3D)
//   • Hover detiene la rotación + tooltip vidrio Apple (cluster 3D)

import 'dart:math';
import 'dart:ui' as ui;
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import '../gallery_fx.dart';
import '../gallery_tokens.dart';
import '../../widgets/electric_primitives.dart';
import '../../widgets/frosted_surface.dart';

// ===========================================================================
// PRIMITIVOS ELÉCTRICOS — migrados a lib/widgets/electric_primitives.dart.
// Las funciones top-level (electricIntensity, paintCometTail, paintScanLine)
// se reexportan aquí por compatibilidad con los painters locales que las
// usan directamente sin tocar el import. No es nueva fuente de verdad: el
// código vive en el primitivo.
// ===========================================================================

// ===========================================================================
// TAREA 6 — MonteCarloLinesWidget (refactorizado con efecto eléctrico completo)
// Optimización de rendimiento: ui.Picture cache + capa dinámica limitada a 80
// trayectorias representativas → de ~600K ops/frame a ~4.800 ops/frame.
// ===========================================================================

// ---------------------------------------------------------------------------
// _Trajectory — modelo interno de una trayectoria pre-procesada.
// Agrupa los puntos normalizados (valores de random walk), el color semántico,
// la opacidad base y el grosor de línea. Se construye una sola vez por
// regeneración de datos y se reutiliza en ambas capas del painter.
// ---------------------------------------------------------------------------
class _Trajectory {
  final List<double> values; // valores sin normalizar (raw del random walk)
  final Color color;
  final double baseOpacity;
  final double baseStroke;

  const _Trajectory({
    required this.values,
    required this.color,
    required this.baseOpacity,
    required this.baseStroke,
  });
}

// Selecciona hasta 80 trayectorias representativas del conjunto completo.
// Criterio:
//   • 25 ganadoras (equity final más alto)
//   • 25 perdedoras (equity final más bajo)
//   • 20 del rango medio (percentiles 40–60)
//   • La mediana exacta (p50), las fronteras del cono (p5 y p95)
// Si el total es menor o igual a 80, devuelve todas sin filtrar.
List<_Trajectory> _selectTop80(List<_Trajectory> all) {
  if (all.length <= 80) return all;

  // Ordena índices por equity final (último valor).
  final indexed = List.generate(all.length, (i) => i);
  indexed.sort((a, b) => all[a].values.last.compareTo(all[b].values.last));

  final selected = <int>{};

  // 25 peores (primeras posiciones del sorted).
  for (var i = 0; i < 25 && i < indexed.length; i++) {
    selected.add(indexed[i]);
  }
  // 25 mejores (últimas posiciones).
  for (var i = indexed.length - 1; i >= indexed.length - 25 && i >= 0; i--) {
    selected.add(indexed[i]);
  }
  // p5, p50, p95.
  selected.add(indexed[((0.05 * (indexed.length - 1)).round()).clamp(0, indexed.length - 1)]);
  selected.add(indexed[((0.50 * (indexed.length - 1)).round()).clamp(0, indexed.length - 1)]);
  selected.add(indexed[((0.95 * (indexed.length - 1)).round()).clamp(0, indexed.length - 1)]);

  // 20 del rango medio (p40–p60).
  final p40 = (0.40 * (indexed.length - 1)).round();
  final p60 = (0.60 * (indexed.length - 1)).round();
  final midStep = max(1, (p60 - p40) ~/ 20);
  for (var i = p40; i <= p60 && selected.length < 80; i += midStep) {
    selected.add(indexed[i.clamp(0, indexed.length - 1)]);
  }

  return selected.map((i) => all[i]).toList();
}

// Widget principal del gráfico de Monte Carlo con múltiples trayectorias.
// SegmentedControl permite elegir 300/1.000/5.000/10.000 líneas.
// Al montarse ejecuta el scan eléctrico completo (1200ms).
// • 0.0–0.8 del controller: scanX avanza de 0→width, lines se revelan
// • 0.8–1.0 del controller: scanOpacity fade 1→0
// Botón "Replay scanInit" para repetir el efecto.
class MonteCarloLinesWidget extends StatefulWidget {
  const MonteCarloLinesWidget({super.key});

  @override
  State<MonteCarloLinesWidget> createState() => _MonteCarloLinesWidgetState();
}

class _MonteCarloLinesWidgetState extends State<MonteCarloLinesWidget>
    with TickerProviderStateMixin {
  // Opciones de número de trayectorias disponibles en el SegmentedControl.
  static const _countOptions = [300, 1000, 5000, 10000];
  int _selectedIdx = 0; // índice en _countOptions actualmente seleccionado

  // Único AnimationController de 1200ms para el scan eléctrico completo.
  // 0.0–0.8: scan cruza el canvas (scanProgress 0→1).
  // 0.8–1.0: el scan se desvanece (scanOpacity 1→0).
  late AnimationController _scanCtrl;

  // Trayectorias clasificadas para el fondo (todas) y el efecto eléctrico (~80).
  // Se regeneran al cambiar el recuento; no cambian frame a frame.
  List<_Trajectory> _bgTrajectories = [];    // todas → van al ui.Picture
  List<_Trajectory> _top80Trajectories = []; // subconjunto → efecto eléctrico

  // Rango global precalculado en _regenerate() para no recalcularlo en build().
  double _globalMinV = 0.0;
  double _globalMaxV = 1.0;

  // Líneas de percentiles (p5, p50, p95) — trayectorias representativas
  // calculadas con un solo sort (O(n log n), no O(180 * n log n)).
  List<double> _medianLine = [];
  List<double> _p5Line = [];
  List<double> _p95Line = [];

  // Calcula las líneas de percentiles (mediana, p5, p95) dibujadas con glow eléctrico.
  // Optimización: ordena los índices por equity final UNA SOLA VEZ y lee los
  // valores de paso de las trayectorias en los percentiles. Esto evita 180 sorts
  // de 10K elementos cada uno y reduce la complejidad de O(180 * n log n) a O(n log n).
  void _computePercentiles() {
    final sortedIdx = List.generate(_rawTrajectories.length, (i) => i);
    sortedIdx.sort((a, b) => _rawTrajectories[a].last.compareTo(_rawTrajectories[b].last));

    _medianLine = _rawTrajectories[sortedIdx[(0.50 * (sortedIdx.length - 1)).round()]];
    _p5Line = _rawTrajectories[sortedIdx[(0.05 * (sortedIdx.length - 1)).round()]];
    _p95Line = _rawTrajectories[sortedIdx[(0.95 * (sortedIdx.length - 1)).round()]];
  }

  // Caché del ui.Picture que contiene el fondo estático de todas las trayectorias.
  // Se invalida (null) cuando cambian los datos base o el tamaño del canvas.
  // Vive aquí (en el State) porque CustomPainter es const y no puede mutarse.
  ui.Picture? _cachedBgPicture;
  Size _cachedSize = Size.zero;

  // Datos raw del random walk (necesarios para calcular percentiles).
  List<List<double>> _rawTrajectories = [];

  static const int _steps = 60; // puntos de tiempo por trayectoria

  @override
  void initState() {
    super.initState();
    // 1200ms: 960ms cruce + 240ms fade (proporcional a crossFraction=0.8).
    _scanCtrl = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 1200),
    );
    _regenerate();
    _startAnimations();
  }

  // Genera datos sintéticos con random walk. Semilla fija para reproducibilidad.
  // Clasifica cada trayectoria en ganadora/perdedora/neutra y construye los
  // modelos _Trajectory con color, opacidad y grosor ya resueltos.
  void _regenerate() {
    final count = _countOptions[_selectedIdx];
    final rnd = Random(42); // semilla fija para galería reproducible

    // Genera los valores crudos del random walk.
    _rawTrajectories = List.generate(count, (_) {
      double v = 0.0;
      return List.generate(_steps, (_) {
        v += (rnd.nextDouble() - 0.495) * 0.08;
        return v;
      });
    });

    // Calcula las líneas de percentiles (1 sort en lugar de 180).
    _computePercentiles();

    // Calcula umbrales p40/p60 para clasificar trayectorias neutras.
    final sortedFinals = _rawTrajectories.map((t) => t.last).toList()..sort();
    final p40Val = sortedFinals[((0.40 * (sortedFinals.length - 1)).round())];
    final p60Val = sortedFinals[((0.60 * (sortedFinals.length - 1)).round())];
    final medianFinal = _medianLine.last;

    // Construye modelos _Trajectory con semántica de color ya asignada.
    _bgTrajectories = List.generate(count, (ti) {
      final t = _rawTrajectories[ti];
      final finalVal = t.last;
      Color color;
      double opacity;
      if (finalVal >= p40Val && finalVal <= p60Val) {
        // Neutras: ciclan entre 4 colores para espectro cinematográfico.
        color = _neutralColors[ti % _neutralColors.length];
        opacity = 0.20;
      } else if (finalVal > medianFinal) {
        color = Gx.optimaCyan;
        opacity = 0.25;
      } else {
        color = Gx.criticalRed;
        opacity = 0.15;
      }
      return _Trajectory(
        values: t,
        color: color,
        baseOpacity: opacity,
        baseStroke: 0.5,
      );
    });

    // Selecciona las ~80 representativas para el efecto eléctrico.
    _top80Trajectories = _selectTop80(_bgTrajectories);

    // Calcula el rango global una sola vez — se usa en el painter y en el Picture.
    double minV = double.infinity, maxV = double.negativeInfinity;
    for (final t in _rawTrajectories) {
      for (final v in t) {
        if (v < minV) minV = v;
        if (v > maxV) maxV = v;
      }
    }
    _globalMinV = minV;
    _globalMaxV = maxV;

    // Invalida la caché del Picture porque los datos cambiaron.
    _cachedBgPicture = null;
  }

  // Lanza la animación del scan desde cero.
  void _startAnimations() {
    _scanCtrl.forward(from: 0.0);
  }

  // Obtiene o genera el ui.Picture del fondo estático dado el tamaño del canvas.
  // El Picture se genera una sola vez por combinación de datos+tamaño.
  // Usa _globalMinV/_globalMaxV precalculados en _regenerate() — cero trabajo extra.
  ui.Picture _getOrBuildBgPicture(Size size) {
    if (_cachedBgPicture != null && _cachedSize == size) {
      return _cachedBgPicture!;
    }

    final minV = _globalMinV;
    final range = _globalMaxV - minV;
    final n = _steps;
    final dx = size.width / (n - 1);

    // Graba todas las trayectorias en un Picture sin efectos dinámicos.
    final recorder = ui.PictureRecorder();
    final c = Canvas(recorder);
    final paint = Paint()..style = PaintingStyle.stroke;

    for (final traj in _bgTrajectories) {
      final path = Path();
      bool started = false;
      for (var i = 0; i < n; i++) {
        final x = i * dx;
        final y = _toY(traj.values[i], minV, range, size.height);
        if (!started) {
          path.moveTo(x, y);
          started = true;
        } else {
          path.lineTo(x, y);
        }
      }
      paint
        ..color = traj.color.withOpacity(traj.baseOpacity)
        ..strokeWidth = traj.baseStroke;
      c.drawPath(path, paint);
    }

    _cachedBgPicture = recorder.endRecording();
    _cachedSize = size;
    return _cachedBgPicture!;
  }

  @override
  void dispose() {
    _cachedBgPicture?.dispose();
    _scanCtrl.dispose();
    super.dispose();
  }

  // Replay: reinicia la animación del scan desde cero.
  void _replay() {
    _scanCtrl.forward(from: 0.0);
  }

  @override
  // Monte Carlo principal: envuelto en panelSurface() para reaccionar a los N modos
  // (glass/tint/solid/enhancedGlass). Icono de cabecera usa textBaseSecondary (dinámico).
  Widget build(BuildContext context) {
    return panelSurface(
      padding: const EdgeInsets.all(Gx.space16 - 2), // 14px
      child: SizedBox(
      width: double.infinity,
      // minWidth de 500px para que el gráfico de Monte Carlo tenga espacio mínimo.
      child: ConstrainedBox(
        constraints: const BoxConstraints(minWidth: 500),
        child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          // Cabecera con título y selector de número de líneas.
          Row(
            children: [
              Icon(Icons.multiline_chart, size: 14, color: Gx.textBaseSecondary),
              const SizedBox(width: 6),
              // Título de panel con énfasis dinámico — reacciona al color de acento activo.
              Text('Monte Carlo — Trayectorias',
                  style: Gx.panelTitle.copyWith(color: Gx.accentDynamic)),
              const Spacer(),
              // SegmentedControl de recuento de líneas.
              _LineCountSelector(
                options: _countOptions,
                selectedIdx: _selectedIdx,
                onChanged: (i) => setState(() {
                  _selectedIdx = i;
                  _regenerate();
                  _startAnimations();
                }),
              ),
            ],
          ),
          const SizedBox(height: 10),

          // Lienzo principal — painter en 3 capas: Picture estático + glow eléctrico
          // en ~80 trayectorias + comet tail / scan line.
          // RepaintBoundary: aisla los repaints del scan del resto de la UI.
          SizedBox(
            height: 420, // mínimo 420px según spec DESIGN.md §10
            child: RepaintBoundary(
              child: ClipRRect(
                borderRadius: BorderRadius.circular(Gx.rChip),
                child: Container(
                  decoration: BoxDecoration(
                    borderRadius: BorderRadius.circular(Gx.rChip),
                  ),
                  child: AnimatedBuilder(
                    animation: _scanCtrl,
                    builder: (context, __) {
                      const crossFraction = 0.8;
                      final p = _scanCtrl.value;
                      // scanProgress: fracción 0→1 del cruce.
                      final scanProgress =
                          p <= crossFraction ? p / crossFraction : 1.0;
                      // scanOpacity: 1.0 durante el cruce, luego fade 1→0.
                      final scanOpacity = p <= crossFraction
                          ? 1.0
                          : 1.0 - ((p - crossFraction) / (1.0 - crossFraction));
                      return LayoutBuilder(
                        builder: (ctx, constraints) {
                          final size = Size(
                            constraints.maxWidth,
                            constraints.maxHeight,
                          );
                          // Garantiza que el Picture esté listo antes de dibujar.
                          final bgPicture = size.isEmpty
                              ? null
                              : _getOrBuildBgPicture(size);
                          return CustomPaint(
                            painter: _MCPainter(
                              bgPicture: bgPicture,
                              top80Traj: _top80Trajectories,
                              medianLine: _medianLine,
                              p5Line: _p5Line,
                              p95Line: _p95Line,
                              minV: _globalMinV,
                              maxV: _globalMaxV,
                              steps: _steps,
                              scanProgress: scanProgress,
                              scanOpacity: scanOpacity.clamp(0.0, 1.0),
                            ),
                            size: Size.infinite,
                          );
                        },
                      );
                    },
                  ),
                ),
              ),
            ),
          ),
          const SizedBox(height: 10),

          // Leyenda + botón Replay.
          Row(
            children: [
              _LegendDot(color: Gx.optimaCyan, label: 'Ganadoras'),
              const SizedBox(width: 12),
              _LegendDot(color: Gx.criticalCrimson, label: 'Perdedoras'),
              const SizedBox(width: 12),
              _LegendDot(color: Gx.transitionIndigo, label: 'Neutras'),
              const SizedBox(width: 12),
              _LegendDot(color: Gx.optimaCyan, label: 'Mediana (p50)', thick: true),
              const Spacer(),
              // Botón para repetir la animación eléctrica.
              GestureDetector(
                onTap: _replay,
                // Chip "Replay scanInit": superficie dinámica con borde estructural global.
                child: Container(
                  padding: const EdgeInsets.symmetric(horizontal: Gx.space8 + 2, vertical: Gx.space4),
                  decoration: BoxDecoration(
                    color: Gx.surfaceFill,
                    border: Border.all(color: Gx.borderBase),
                    borderRadius: BorderRadius.circular(Gx.rChip),
                  ),
                  // Texto del botón Replay con token dinámico secundario.
                  child: Text('Replay scanInit',
                      style: Gx.uiSans(fontSize: 11, color: Gx.textBaseSecondary)),
                ),
              ),
            ],
          ),
        ],
      ),
      ),   // ConstrainedBox
      ),   // SizedBox
    );     // panelSurface
  }
}

// Selector segmentado de número de líneas (300 / 1.000 / 5.000 / 10.000).
class _LineCountSelector extends StatelessWidget {
  final List<int> options;
  final int selectedIdx;
  final ValueChanged<int> onChanged;

  const _LineCountSelector({
    required this.options,
    required this.selectedIdx,
    required this.onChanged,
  });

  @override
  // Selector segmentado: fondo con frosted(), borde estructural global, texto dinámico.
  Widget build(BuildContext context) {
    return panelSurface(
      padding: EdgeInsets.zero,
      radius: Gx.rChip,
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: List.generate(options.length, (i) {
          final active = i == selectedIdx;
          final label = options[i] >= 1000
              ? '${options[i] ~/ 1000}K'
              : '${options[i]}';
          return GestureDetector(
            onTap: () => onChanged(i),
            child: AnimatedContainer(
              duration: const Duration(milliseconds: 150),
              padding: const EdgeInsets.symmetric(horizontal: Gx.space8 + 2, vertical: Gx.space4 + 1),
              decoration: BoxDecoration(
                // Activo: fondo semántico tenue (estado de selección).
                color: active ? Gx.transitionIndigo.withOpacity(0.20) : Colors.transparent,
                borderRadius: BorderRadius.circular(Gx.rChip - 1),
                border: Border.all(
                  // Activo: borde semántico del estado de selección. Inactivo: invisible.
                  color: active ? Gx.transitionIndigo : Colors.transparent,
                  width: active ? Gx.borderHairline : 0,
                ),
              ),
              // Texto activo: semántico (estado seleccionado). Inactivo: dinámico muted.
              child: Text(label,
                  style: Gx.dataMono(
                      fontSize: 11,
                      color: active ? Gx.transitionIndigo : Gx.textBaseMuted)),
            ),
          );
        }),
      ),
    );
  }
}

// Leyenda con punto de color y etiqueta.
class _LegendDot extends StatelessWidget {
  final Color color;
  final String label;
  final bool thick;

  const _LegendDot({
    required this.color,
    required this.label,
    this.thick = false,
  });

  @override
  Widget build(BuildContext context) {
    return Row(mainAxisSize: MainAxisSize.min, children: [
      Container(
        width: thick ? 16 : 8,
        height: thick ? 2.0 : 8,
        decoration: BoxDecoration(
          color: color,
          // punto de leyenda decorativo, radio menor sin token canónico
          borderRadius: BorderRadius.circular(4),
        ),
      ),
      const SizedBox(width: 5),
      // Etiqueta de leyenda con token dinámico muted — legible en paper y bunker.
      Text(label, style: Gx.uiSans(fontSize: 10, color: Gx.textBaseMuted)),
    ]);
  }
}

// ---------------------------------------------------------------------------
// Helpers de geometría — compartidos por _MCPainter y _getOrBuildBgPicture.
// ---------------------------------------------------------------------------

// Convierte un valor a coordenada Y en el lienzo.
// minV y range provienen del rango global precalculado en el State.
double _toY(double v, double minV, double range, double height) {
  if (range == 0) return height / 2;
  return height * 0.9 * (1 - (v - minV) / range) + height * 0.05;
}

// Colores para trayectorias neutras (p40–p60), ciclando entre 4 opciones.
// Crea un espectro más cinematográfico que usar solo transitionIndigo.
const _neutralColors = [
  Gx.transitionIndigo,
  Gx.transitionBlue,
  Gx.transitionPurple,
  Gx.alertAmber,
];

// ---------------------------------------------------------------------------
// _MCPainter — painter en 3 capas para 60 fps.
//
// Capa 1 (estática): recibe un [bgPicture] ya grabado en el State con TODAS
//   las trayectorias en su color/opacidad final. Se dibuja en una sola
//   llamada drawPicture() — costo fijo independiente del número de líneas.
//
// Capa 2 (dinámica): efecto eléctrico + glow solo en [top80Traj] (~80
//   trayectorias). Máximo ~4.800 operaciones/frame vs. 600K del painter viejo.
//
// Capa 3 (dinámica): mediana/p5/p95 con glow, comet tail y scan line.
//
// El painter es const (inmutable); la caché del Picture vive en el State
// porque Dart no permite fields mutables en clases const.
// ---------------------------------------------------------------------------
class _MCPainter extends CustomPainter {
  final ui.Picture? bgPicture;          // Picture pregrabado con el fondo estático
  final List<_Trajectory> top80Traj;   // ~80 trayectorias para efecto eléctrico
  final List<double> medianLine;
  final List<double> p5Line;
  final List<double> p95Line;
  final double minV;                    // mínimo global del rango de valores
  final double maxV;                    // máximo global del rango de valores
  final int steps;                      // puntos por trayectoria
  final double scanProgress;            // 0.0–1.0: qué fracción del canvas ha cruzado el scan
  final double scanOpacity;             // 1.0 durante el cruce, fade a 0 en los últimos 200ms

  const _MCPainter({
    required this.bgPicture,
    required this.top80Traj,
    required this.medianLine,
    required this.p5Line,
    required this.p95Line,
    required this.minV,
    required this.maxV,
    required this.steps,
    required this.scanProgress,
    required this.scanOpacity,
  });

  @override
  void paint(Canvas canvas, Size size) {
    if (size.isEmpty) return;
    final range = maxV - minV;
    final dx = size.width / (steps - 1);
    // scanX para el cálculo de intensidad eléctrica (posición real del scan).
    final scanX = scanProgress * size.width;
    // paintScanX desplazado +dx/2 para que la línea visual se alinee con el
    // centro del segmento más brillante (la intensidad se calcula en x0).
    final paintScanX = scanX + dx * 0.5;

    // --- Capa 1: Picture estático — costo de una sola llamada GPU ---
    // Dibuja el fondo con TODAS las trayectorias sin efectos dinámicos.
    if (bgPicture != null) {
      canvas.drawPicture(bgPicture!);
    }

    // --- Capa 2: efecto eléctrico solo en las ~80 representativas ---
    // Cada trayectoria recibe glow proporcional a la intensidad del scan.
    for (final traj in top80Traj) {
      _paintElectricTraj(canvas, traj, size, range, dx, scanX);
    }

    // --- Capa 3: líneas de percentiles con glow, comet tail y scan line ---
    _drawElectricLine(canvas, size, medianLine, range, dx, scanX,
        color: Gx.optimaCyan, strokeWidth: 1.5, baseOpacity: 0.90);
    _drawElectricLine(canvas, size, p5Line, range, dx, scanX,
        color: Gx.transitionIndigo, strokeWidth: 1.0, baseOpacity: 0.60);
    _drawElectricLine(canvas, size, p95Line, range, dx, scanX,
        color: Gx.transitionIndigo, strokeWidth: 1.0, baseOpacity: 0.60);

    // Cola de cometa + scan line en paintScanX (desplazado para alinear).
    paintCometTail(canvas, paintScanX, size, Gx.optimaCyan);
    paintScanLine(canvas, paintScanX, size.height, Gx.optimaCyan, scanOpacity);
  }

  // Aplica el glow eléctrico a una sola trayectoria del top-80.
  // Sin MaskFilter.blur por segmento — el glow se logra con línea más gruesa
  // a baja opacidad, similar al rendimiento del Picture estático.
  void _paintElectricTraj(
    Canvas canvas,
    _Trajectory traj,
    Size size,
    double range,
    double dx,
    double scanX,
  ) {
    for (var i = 0; i < traj.values.length - 1; i++) {
      final x0 = i * dx;
      if (x0 >= scanX) break;

      final intensity = electricIntensity(x0, scanX, size.width);
      if (intensity < 0.01) continue;

      final x1 = x0 + dx;
      final y0 = _toY(traj.values[i], minV, range, size.height);
      final y1 = _toY(traj.values[i + 1], minV, range, size.height);
      final extraStroke = intensity * 1.5;

      // Glow sin blur: línea gruesa a baja opacidad.
      canvas.drawLine(
        Offset(x0, y0), Offset(x1, y1),
        Paint()
          ..color = traj.color.withOpacity(intensity * 0.15)
          ..strokeWidth = traj.baseStroke + extraStroke + 6
          ..strokeCap = StrokeCap.round,
      );
      // Línea nítida encima.
      canvas.drawLine(
        Offset(x0, y0), Offset(x1, y1),
        Paint()
          ..color = traj.color.withOpacity(
              traj.baseOpacity + intensity * (1.0 - traj.baseOpacity))
          ..strokeWidth = traj.baseStroke + extraStroke,
      );
    }
  }

  // Dibuja una línea de percentil con glow sin MaskFilter.blur.
  void _drawElectricLine(
    Canvas canvas,
    Size size,
    List<double> pts,
    double range,
    double dx,
    double scanX, {
    required Color color,
    required double strokeWidth,
    required double baseOpacity,
  }) {
    if (pts.length < 2) return;
    for (var i = 0; i < pts.length - 1; i++) {
      final x0 = i * dx;
      if (x0 >= scanX) break;

      final intensity = electricIntensity(x0, scanX, size.width);
      final effectiveOpacity = baseOpacity + intensity * (1.0 - baseOpacity);
      final y0 = _toY(pts[i], minV, range, size.height);
      final y1 = _toY(pts[i + 1], minV, range, size.height);

      // Glow sin blur: línea más gruesa y transparente.
      if (intensity > 0.05) {
        canvas.drawLine(Offset(x0, y0), Offset(x0 + dx, y1), Paint()
          ..color = color.withOpacity(intensity * 0.15)
          ..strokeWidth = strokeWidth + 6
          ..strokeCap = StrokeCap.round);
      }
      canvas.drawLine(Offset(x0, y0), Offset(x0 + dx, y1), Paint()
        ..color = color.withOpacity(effectiveOpacity)
        ..strokeWidth = strokeWidth);
    }
  }

  @override
  // Solo repinta si cambió el progreso del scan. El Picture estático ya fue
  // resuelto en el State antes de construir el painter.
  bool shouldRepaint(_MCPainter old) =>
      old.scanProgress != scanProgress ||
      old.scanOpacity != scanOpacity ||
      old.bgPicture != bgPicture;
}

// ScanInitLinePainter sigue disponible para uso en el cluster 3D y otras secciones.
// Cruza de x=0 a x=width en 800ms (progress 0.0→0.8),
// luego fade opacity 1→0 en progress 0.8→1.0.
class ScanInitLinePainter extends CustomPainter {
  final double progress; // 0.0–1.0 del AnimationController total

  const ScanInitLinePainter({required this.progress});

  @override
  void paint(Canvas canvas, Size size) {
    if (progress <= 0.0 || progress >= 1.0) return;

    const crossFraction = 0.8;
    final x = progress <= crossFraction
        ? size.width * (progress / crossFraction)
        : size.width;
    final opacity = progress <= crossFraction
        ? 1.0
        : 1.0 - ((progress - crossFraction) / (1.0 - crossFraction));

    if (opacity <= 0) return;

    // Comet tail completo.
    paintCometTail(canvas, x, size, Gx.optimaCyan);
    // Scan line con glow.
    paintScanLine(canvas, x, size.height, Gx.optimaCyan, opacity);
  }

  @override
  bool shouldRepaint(ScanInitLinePainter old) => old.progress != progress;
}

// ===========================================================================
// TAREA 7 — StrategyCluster3dWidget (upgrade a 5.000 puntos + nebulosa)
// ===========================================================================

// "Galaxia de estrategias" — scatter plot 3D con proyección perspectiva manual
// y rotación orbital continua. Spec DESIGN.md §10 strategy-cluster-3d.
//
// 5.000 puntos pre-calculados en compute() Isolate para no bloquear la UI.
// Efecto nebulosa: saveLayer con ImageFilter.blur(8,8) → puntos borrosos,
// luego puntos nítidos de 1.5px encima.
// Panel lateral de leyenda (180px) con hover que resalta el cluster.
// Tamaño mínimo: 840×600px.
class StrategyCluster3dWidget extends StatefulWidget {
  const StrategyCluster3dWidget({super.key});

  @override
  State<StrategyCluster3dWidget> createState() =>
      _StrategyCluster3dWidgetState();
}

// Modelo de un punto de estrategia en el espacio 3D.
class _Strategy3d {
  final double x, y, z;  // coordenadas unitarias (-1 a 1)
  final int group;        // 0=óptimas, 1=neutras, 2=perdedoras, 3=sin clasificar
  final Color color;      // color semántico del cluster
  final String clusterName;

  const _Strategy3d({
    required this.x,
    required this.y,
    required this.z,
    required this.group,
    required this.color,
    required this.clusterName,
  });
}

// Datos de cada cluster: (centroX, centroY, centroZ, grupo, color, nombre, conteo).
typedef _ClusterSpec = ({
  double cx,
  double cy,
  double cz,
  int group,
  Color color,
  String name,
  int count,
});

// Función top-level para usar con compute() (Isolate). Recibe solo datos
// primitivos porque los Isolates no comparten memoria con Flutter.
// Retorna listas de [x, y, z, group] (color y nombre se recomputan en el widget).
List<List<double>> _generatePointsIsolate(int seed) {
  final rnd = Random(seed);
  final result = <List<double>>[];

  // Clusters con sus centros y tamaños (total 5.000 puntos).
  final specs = [
    // Óptimas: 1.500 puntos alrededor de (0.4, 0.1, 0.3)
    (0.4, 0.1, 0.3, 0, 1500),
    // Neutras: 1.400 puntos alrededor de (-0.3, 0.0, -0.2)
    (-0.3, 0.0, -0.2, 1, 1400),
    // Perdedoras: 1.200 puntos alrededor de (-0.1, -0.3, 0.1)
    (-0.1, -0.3, 0.1, 2, 1200),
    // Sin clasificar: 900 puntos dispersos
    (0.0, 0.0, 0.0, 3, 900),
  ];

  for (final (cx, cy, cz, group, count) in specs) {
    final spread = group == 3 ? 1.0 : 0.45; // sin clasificar más disperso
    for (var j = 0; j < count; j++) {
      final x = (cx + (rnd.nextDouble() - 0.5) * spread).clamp(-1.0, 1.0);
      final y = (cy + (rnd.nextDouble() - 0.5) * spread).clamp(-1.0, 1.0);
      final z = (cz + (rnd.nextDouble() - 0.5) * spread).clamp(-1.0, 1.0);
      result.add([x, y, z, group.toDouble()]);
    }
  }
  return result;
}

class _StrategyCluster3dWidgetState extends State<StrategyCluster3dWidget>
    with TickerProviderStateMixin {
  // Controlador de rotación orbital infinita — 10s por ciclo completo.
  late AnimationController _rotCtrl;
  // Controlador de scanInitLine (1000ms, una sola vez).
  late AnimationController _scanCtrl;

  // Puntos generados en el Isolate (null mientras carga).
  List<_Strategy3d>? _points;

  // Caché de proyección 3D → 2D: evita recalcular 5000 proyecciones por frame.
  // Solo se invalida cuando el ángulo cambia. El painter recibe esta lista ya
  // ordenada por profundidad (painter's algorithm) y usa solo drawCircle().
  List<({int idx, double px, double py, double scale})>? _cachedProjected;
  double _cachedAngle = double.nan;

  // Cluster resaltado desde el panel de leyenda (-1 = ninguno).
  int _highlightedGroup = -1;

  // Posición del cursor para hit-test en el painter.
  Offset? _hoverPos;
  int _hoveredIdx = -1;

  // Colores y nombres de cada cluster (orden debe coincidir con _generatePointsIsolate).
  static const _clusterColors = [
    Gx.optimaCyan,
    Gx.transitionIndigo,
    Gx.criticalRed,
    Gx.textMuted,
  ];
  static const _clusterNames = ['Óptimas', 'Neutras', 'Perdedoras', 'Sin clasificar'];
  static const _clusterCounts = [1500, 1400, 1200, 900];

  @override
  void initState() {
    super.initState();

    // Rotación infinita: linear para velocidad constante.
    _rotCtrl = AnimationController(
      vsync: this,
      duration: const Duration(seconds: 10),
    )..repeat();

    // scanInitLine una sola vez al montar.
    _scanCtrl = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 1000),
    )..forward();

    // Pre-calcula los 5.000 puntos en Isolate para no bloquear la UI.
    compute(_generatePointsIsolate, 13).then((rawPoints) {
      if (!mounted) return;
      // Convierte la lista de primitivos a _Strategy3d con color y nombre.
      final points = rawPoints.map((p) {
        final group = p[3].toInt().clamp(0, 3);
        return _Strategy3d(
          x: p[0], y: p[1], z: p[2],
          group: group,
          color: _clusterColors[group],
          clusterName: _clusterNames[group],
        );
      }).toList();
      setState(() => _points = points);
    });
  }

  @override
  void dispose() {
    _rotCtrl.dispose();
    _scanCtrl.dispose();
    super.dispose();
  }

  // Pausa la rotación cuando hay hover sobre el canvas.
  void _onHoverCanvas(PointerEvent event) {
    setState(() => _hoverPos = event.localPosition);
    if (_rotCtrl.isAnimating) _rotCtrl.stop();
  }

  void _onExitCanvas(PointerEvent event) {
    setState(() {
      _hoverPos = null;
      _hoveredIdx = -1;
    });
    if (!_rotCtrl.isAnimating) _rotCtrl.repeat();
  }

  void _onHitResult(int idx) {
    if (idx != _hoveredIdx) setState(() => _hoveredIdx = idx);
  }

  // Proyección + ordenamiento cacheado. Solo se recalcula si el ángulo cambió.
  // Retorna los 5000 puntos ya proyectados y ordenados por profundidad.
  // Se invoca desde AnimatedBuilder (1 vez por frame), no desde paint().
  List<({int idx, double px, double py, double scale})> _getProjected(
      Size canvasSize) {
    final angle = _rotCtrl.value * 2 * pi;
    if (_cachedProjected != null && _cachedAngle == angle) {
      return _cachedProjected!;
    }
    final pts = _points!;
    final cosA = cos(angle);
    final sinA = sin(angle);
    const focal = 2.5;
    final halfW = canvasSize.width / 2;
    final halfH = canvasSize.height / 2;
    final rw = canvasSize.width * 0.40;
    final rh = canvasSize.height * 0.40;

    final projected = <({int idx, double px, double py, double scale})>[];
    for (var i = 0; i < pts.length; i++) {
      final p = pts[i];
      // Rotación Y
      final rx = p.x * cosA + p.z * sinA;
      final ry = p.y;
      final rz = -p.x * sinA + p.z * cosA;
      // Proyección perspectiva
      final s = focal / (focal + rz);
      final px = halfW + rx * s * rw;
      final py = halfH - ry * s * rh;
      projected.add((idx: i, px: px, py: py, scale: s));
    }
    projected.sort((a, b) => a.scale.compareTo(b.scale)); // painter's algorithm

    _cachedProjected = projected;
    _cachedAngle = angle;
    return projected;
  }

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: 840,
      height: 600,
      child: Row(
        children: [
          // Panel lateral de leyenda (180px).
          SizedBox(
            width: 180,
            child: _ClusterLegendPanel(
              clusterColors: _clusterColors,
              clusterNames: _clusterNames,
              clusterCounts: _clusterCounts,
              highlightedGroup: _highlightedGroup,
              totalPoints: 5000,
              onHoverGroup: (g) => setState(() => _highlightedGroup = g),
              onExitGroup: () => setState(() => _highlightedGroup = -1),
            ),
          ),

          // Canvas 3D: fondo transparente para que herede la superficie dinámica del contenedor
          // panelSurface() que envuelve esta sección. La separación visual vertical se mantiene
          // con el borde izquierdo.
          Expanded(
            child: Container(
              decoration: BoxDecoration(
                border: Border(left: BorderSide(color: Gx.borderBase)),
                borderRadius: const BorderRadius.only(
                  topRight: Radius.circular(Gx.rPanel),
                  bottomRight: Radius.circular(Gx.rPanel),
                ),
                boxShadow: Gx.glow(Gx.transitionIndigo, blur: 24, opacity: 0.08),
              ),
              child: ClipRRect(
                borderRadius: const BorderRadius.only(
                  topRight: Radius.circular(Gx.rPanel),
                  bottomRight: Radius.circular(Gx.rPanel),
                ),
                child: Stack(
                  children: [
                    // Indicador de carga mientras el Isolate calcula.
                    if (_points == null)
                      const Center(
                        child: CircularProgressIndicator(
                          strokeWidth: 2,
                          color: Gx.transitionIndigo,
                        ),
                      ),

                    // Painter principal de la galaxia — recibe proyección pre-calculada.
                    if (_points != null)
                      Positioned.fill(
                        child: LayoutBuilder(builder: (ctx, box) {
                          final canvasSize = Size(box.maxWidth, box.maxHeight);
                          return MouseRegion(
                            onHover: _onHoverCanvas,
                            onExit: _onExitCanvas,
                            child: AnimatedBuilder(
                              animation: _rotCtrl,
                              builder: (_, __) {
                                final projected =
                                    _getProjected(canvasSize);
                                return RepaintBoundary(
                                  child: CustomPaint(
                                    painter: _Cluster3dPainter(
                                      points: _points!,
                                      projected: projected,
                                      hoverPos: _hoverPos,
                                      hoveredIdx: _hoveredIdx,
                                      highlightedGroup: _highlightedGroup,
                                      onHitResult: _onHitResult,
                                    ),
                                    size: Size.infinite,
                                  ),
                                );
                              },
                            ),
                          );
                        }),
                      ),

                    // scanInitLine encima — una sola vez al montar.
                    Positioned.fill(
                      child: AnimatedBuilder(
                        animation: _scanCtrl,
                        builder: (_, __) => CustomPaint(
                          painter: ScanInitLinePainter(progress: _scanCtrl.value),
                          size: Size.infinite,
                        ),
                      ),
                    ),

                    // Tooltip vidrio Apple al hover sobre un punto.
                    if (_hoveredIdx >= 0 && _hoverPos != null && _points != null)
                      _TooltipOverlay(
                        point: _points![_hoveredIdx],
                        position: _hoverPos!,
                      ),
                  ],
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}

// Panel lateral de leyenda con un item por cluster.
// MouseRegion en hover → notifica qué cluster resaltar en el canvas.
class _ClusterLegendPanel extends StatelessWidget {
  final List<Color> clusterColors;
  final List<String> clusterNames;
  final List<int> clusterCounts;
  final int highlightedGroup;
  final int totalPoints;
  final ValueChanged<int> onHoverGroup;
  final VoidCallback onExitGroup;

  const _ClusterLegendPanel({
    required this.clusterColors,
    required this.clusterNames,
    required this.clusterCounts,
    required this.highlightedGroup,
    required this.totalPoints,
    required this.onHoverGroup,
    required this.onExitGroup,
  });

  @override
  // Panel de leyenda izquierdo: envuelto en PanelFromDecoration para respetar el radio
  // asimétrico (solo esquinas izquierdas) y reaccionar a los N modos de superficie.
  Widget build(BuildContext context) {
    return PanelFromDecoration(
      decoration: BoxDecoration(
        color: Gx.surfacePanel,
        border: Border.all(color: Gx.borderBase),
        borderRadius: const BorderRadius.only(
          topLeft: Radius.circular(Gx.rPanel),
          bottomLeft: Radius.circular(Gx.rPanel),
        ),
      ),
      padding: const EdgeInsets.symmetric(horizontal: Gx.space12, vertical: Gx.space16 - 2),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Título del panel con énfasis dinámico.
          Text('Clusters', style: Gx.panelTitle.copyWith(color: Gx.accentDynamic)),
          const SizedBox(height: Gx.space4),
          // Subtítulo con token dinámico muted.
          Text('${totalPoints.toString()} estrategias',
              style: Gx.dataMono(fontSize: 11, color: Gx.textBaseMuted)),
          const SizedBox(height: Gx.space16),
          // Un item por cluster con hover interactivo.
          ...List.generate(clusterColors.length, (i) {
            final pct = (clusterCounts[i] / totalPoints * 100).toStringAsFixed(0);
            return _ClusterLegendItem(
              color: clusterColors[i],
              name: clusterNames[i],
              count: clusterCounts[i],
              pct: pct,
              isHighlighted: highlightedGroup == i,
              isOtherHighlighted: highlightedGroup >= 0 && highlightedGroup != i,
              onHover: () => onHoverGroup(i),
              onExit: onExitGroup,
            );
          }),
        ],
      ),
    );
  }
}

// Item individual del panel de leyenda con MouseRegion interactivo.
class _ClusterLegendItem extends StatelessWidget {
  final Color color;
  final String name;
  final int count;
  final String pct;
  final bool isHighlighted;
  final bool isOtherHighlighted;
  final VoidCallback onHover;
  final VoidCallback onExit;

  const _ClusterLegendItem({
    required this.color,
    required this.name,
    required this.count,
    required this.pct,
    required this.isHighlighted,
    required this.isOtherHighlighted,
    required this.onHover,
    required this.onExit,
  });

  @override
  Widget build(BuildContext context) {
    // Cuando otro cluster está resaltado, este se atenúa al 30%.
    final opacity = isOtherHighlighted ? 0.3 : 1.0;
    return MouseRegion(
      onEnter: (_) => onHover(),
      onExit: (_) => onExit(),
      child: AnimatedOpacity(
        duration: const Duration(milliseconds: 150),
        opacity: opacity,
        child: Container(
          margin: const EdgeInsets.only(bottom: 10),
          padding: const EdgeInsets.all(8),
          decoration: BoxDecoration(
            color: isHighlighted ? color.withOpacity(0.08) : Colors.transparent,
            borderRadius: BorderRadius.circular(Gx.rChip),
            border: Border.all(
              color: isHighlighted ? color.withOpacity(0.4) : Colors.transparent,
            ),
          ),
          child: Row(
            children: [
              // Círculo de color del cluster.
              Container(
                width: 10,
                height: 10,
                decoration: BoxDecoration(
                  shape: BoxShape.circle,
                  color: color,
                  boxShadow: isHighlighted
                      ? Gx.glow(color, blur: 8, opacity: 0.6)
                      : null,
                ),
              ),
              const SizedBox(width: 8),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    // Nombre de cluster: token dinámico base — legible en paper y bunker.
                    Text(name, style: Gx.dataMono(fontSize: 11, color: Gx.textBase)),
                    // Conteo y porcentaje: token dinámico muted.
                    Text('$count · $pct%',
                        style: Gx.dataMono(fontSize: 10, color: Gx.textBaseMuted)),
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

// Painter principal: proyecta los 5.000 puntos 3D → 2D con perspectiva y rotación Y.
// Efecto nebulosa: saveLayer con ImageFilter.blur(8,8) dibuja puntos borrosos grandes,
// luego canvas.restore(), y encima los puntos nítidos de 1.5px.
// Cuando [highlightedGroup] >= 0, los otros clusters se atenúan a alpha 0.3.
class _Cluster3dPainter extends CustomPainter {
  final List<_Strategy3d> points;
  final List<({int idx, double px, double py, double scale})> projected;
  final Offset? hoverPos;
  final int hoveredIdx;
  final int highlightedGroup;
  final ValueChanged<int> onHitResult;

  const _Cluster3dPainter({
    required this.points,
    required this.projected,
    required this.hoverPos,
    required this.hoveredIdx,
    required this.highlightedGroup,
    required this.onHitResult,
  });

  static const double _pointRadius = 1.5;
  static const double _hoverRadius = 6.0;

  @override
  void paint(Canvas canvas, Size size) {
    // Proyección ya hecha en _getProjected(). Sin saveLayer — el blur de
    // nebulosa costaba ~100ms/frame en GPU. Puntos grandes a baja opacidad
    // producen el mismo efecto de nube de polvo estelar.

    int newHovered = -1;
    if (hoverPos != null) {
      double bestDist = 20.0;
      for (final p in projected) {
        final d = sqrt(pow(p.px - hoverPos!.dx, 2) + pow(p.py - hoverPos!.dy, 2));
        if (d < bestDist) { bestDist = d; newHovered = p.idx; }
      }
    }
    if (newHovered != hoveredIdx) {
      WidgetsBinding.instance.addPostFrameCallback((_) => onHitResult(newHovered));
    }

    // ── Capa única: halo grande semitransparente + punto nítido ─────────────
    final paint = Paint();
    for (final p in projected) {
      final pt = points[p.idx];
      final isHov = p.idx == hoveredIdx;
      final dimmed = highlightedGroup >= 0 && pt.group != highlightedGroup;
      final alpha = dimmed ? 0.3 : 0.55;
      final r = isHov ? _hoverRadius : _pointRadius * 3.0 * p.scale.clamp(0.5, 1.5);

      // Halo nebulosa: círculo grande a muy baja opacidad.
      paint.color = pt.color.withOpacity(alpha * 0.12 * p.scale.clamp(0.4, 1.0));
      canvas.drawCircle(Offset(p.px, p.py), r, paint);

      if (isHov) {
        canvas.drawCircle(Offset(p.px, p.py), _hoverRadius + 8, Paint()
          ..color = pt.color.withOpacity(0.28));
        canvas.drawCircle(Offset(p.px, p.py), _hoverRadius + 4, Paint()
          ..color = pt.color.withOpacity(0.55));
        paint.color = pt.color;
        canvas.drawCircle(Offset(p.px, p.py), _hoverRadius, paint);
      } else {
        paint.color = pt.color.withOpacity(alpha * 0.6 * p.scale.clamp(0.4, 1.0));
        canvas.drawCircle(Offset(p.px, p.py), _pointRadius * p.scale.clamp(0.5, 1.5), paint);
      }
    }
  }

  @override
  bool shouldRepaint(_Cluster3dPainter old) =>
      old.hoverPos != hoverPos ||
      old.hoveredIdx != hoveredIdx ||
      old.highlightedGroup != highlightedGroup ||
      old.points != points ||
      old.projected != projected;
}

// Tooltip vidrio Apple para el punto hovereado en el cluster 3D.
// Migrado a FrostedSurface (ADR-0138) — el primitivo encapsula el BackdropFilter
// + glassFill + rim-light; este widget solo aporta el contenido del tooltip.
class _TooltipOverlay extends StatelessWidget {
  final _Strategy3d point;
  final Offset position;

  const _TooltipOverlay({
    required this.point,
    required this.position,
  });

  @override
  Widget build(BuildContext context) {
    const offsetX = 12.0;
    const offsetY = -40.0;
    return Positioned(
      left: position.dx + offsetX,
      top: (position.dy + offsetY).clamp(4.0, double.infinity),
      child: FrostedSurface(
        borderRadius: Gx.rTooltip,
        padding:
            const EdgeInsets.symmetric(horizontal: 10, vertical: 7),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(mainAxisSize: MainAxisSize.min, children: [
              Container(
                width: 6, height: 6,
                decoration: BoxDecoration(
                  shape: BoxShape.circle,
                  color: point.color,
                ),
              ),
              const SizedBox(width: 5),
              // Nombre del cluster: token dinámico base — legible sobre el tooltip de vidrio.
              Text(point.clusterName,
                  style: Gx.uiSans(
                      fontSize: 12,
                      color: Gx.textBase,
                      weight: FontWeight.w500)),
            ]),
            const SizedBox(height: Gx.space4),
            // Coordenadas del punto: token dinámico muted.
            Text(
              'x=${point.x.toStringAsFixed(2)}  y=${point.y.toStringAsFixed(2)}',
              style: Gx.dataMono(fontSize: 10, color: Gx.textBaseMuted),
            ),
          ],
        ),
      ),
    );
  }
}
