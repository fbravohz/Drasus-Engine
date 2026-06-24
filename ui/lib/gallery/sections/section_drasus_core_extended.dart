// Sección §11 Núcleo Drasus extendido — fleet-command-panel, zui-zoom-frame,
// expectation-envelope-badge, pipeline-8-steps completo.
// Render-only con datos hardcodeados. Sin lógica de negocio ni FFI.

import 'package:flutter/material.dart';
import '../gallery_tokens.dart';
import '../gallery_fx.dart';

// ---------------------------------------------------------------------------
// Fleet Command Panel — panel del Comando de Flota (MACRO)
// ---------------------------------------------------------------------------

// Panel de datos denso que simula la vista MACRO: tabla de nodos con chips
// de régimen, métricas en 5 columnas y un micro-gauge por fila.
class FleetCommandPanel extends StatelessWidget {
  const FleetCommandPanel({super.key});

  // Datos de la flota hardcodeados para la vitrina.
  static const _nodes = [
    ('alpha-01', 'ÓPTIMO', Gx.optimaCyan, Gx.optimaChipBg, Gx.optimaChipBorder, 1.84, 0.82),
    ('beta-02', 'VOLÁTIL', Gx.alertAmber, Gx.alertChipBg, Gx.alertChipBorder, 0.42, 0.46),
    ('gamma-03', 'FALLO', Gx.criticalCrimson, Gx.criticalChipBg, Gx.criticalChipBorder, -0.9, 0.1),
  ];

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: BoxDecoration(
        color: Gx.panelSolid,
        borderRadius: BorderRadius.circular(Gx.rPanel),
        border: Border.all(color: Gx.borderPanel),
        boxShadow: Gx.glow(Gx.transitionIndigo, blur: 18, opacity: 0.08),
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          // Cabecera del panel.
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
            child: Row(children: [
              Icon(Gx.iconDashboard, size: 13, color: Gx.textSecondary),
              const SizedBox(width: 6),
              Text('Comando de Flota', style: Gx.panelTitle),
              const Spacer(),
              Text('3 nodos',
                  style: Gx.uiSans(fontSize: 11, color: Gx.textMuted)),
            ]),
          ),
          const Divider(color: Gx.divider, height: 1),
          // Fila de cabecera de tabla.
          _headerRow(),
          // Filas de datos.
          ..._nodes.map((n) => _dataRow(n)),
        ],
      ),
    );
  }

  Widget _headerRow() => Padding(
        padding:
            const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
        child: Row(children: [
          Expanded(
              flex: 2,
              child: Text('ID',
                  style: Gx.uiSans(fontSize: 11, color: Gx.textLabel))),
          Expanded(
              flex: 2,
              child: Text('RÉGIMEN',
                  style: Gx.uiSans(fontSize: 11, color: Gx.textLabel))),
          Expanded(
              flex: 2,
              child: Text('SHARPE',
                  textAlign: TextAlign.right,
                  style: Gx.uiSans(fontSize: 11, color: Gx.textLabel))),
          const Expanded(flex: 3, child: SizedBox()),
        ]),
      );

  Widget _dataRow((String, String, Color, Color, Color, double, double) n) =>
      Container(
        padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 7),
        decoration: const BoxDecoration(
            border: Border(top: BorderSide(color: Gx.divider))),
        child: Row(children: [
          // ID del nodo.
          Expanded(
              flex: 2,
              child: Text(n.$1,
                  style: Gx.dataMono(fontSize: 12, color: Gx.textPrimary))),
          // Chip de régimen.
          Expanded(
              flex: 2,
              child: _chip(n.$2, n.$3, n.$4, n.$5)),
          // Sharpe alineado a la derecha.
          Expanded(
              flex: 2,
              child: Text(n.$6.toStringAsFixed(2),
                  textAlign: TextAlign.right,
                  style: Gx.dataMono(
                      fontSize: 12, color: n.$3))),
          // Micro-gauge de salud.
          Expanded(
            flex: 3,
            child: Padding(
              padding: const EdgeInsets.only(left: 8),
              child: _gauge(n.$7, n.$3),
            ),
          ),
        ]),
      );

  // Chip de estado compacto para la tabla.
  Widget _chip(String label, Color fg, Color bg, Color border) => Container(
        padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 3),
        decoration: BoxDecoration(
          color: bg,
          borderRadius: BorderRadius.circular(Gx.rChip),
          border: Border.all(color: border),
          boxShadow: Gx.glow(fg, blur: 8, opacity: 0.25),
        ),
        child: Text(label,
            style: Gx.uiSans(fontSize: 10, color: fg)),
      );

  // Mini-barra de salud para la tabla.
  Widget _gauge(double v, Color c) => Container(
        height: 4,
        decoration: BoxDecoration(
            color: Gx.gaugeTrack,
            borderRadius: BorderRadius.circular(2)),
        child: FractionallySizedBox(
          alignment: Alignment.centerLeft,
          widthFactor: v,
          child: Container(
            decoration: BoxDecoration(
              color: c,
              borderRadius: BorderRadius.circular(2),
              boxShadow: Gx.glow(c, blur: 6, opacity: 0.6),
            ),
          ),
        ),
      );
}

// ---------------------------------------------------------------------------
// ZUI Zoom Frame — marco de transición de zoom MACRO↔MESO↔MICRO
// ---------------------------------------------------------------------------

// Muestra tres "niveles" de zoom anidados con el marco del nivel activo
// resaltado por un borde neón de 2px. Toca para cambiar el nivel activo.
class ZuiZoomFrame extends StatefulWidget {
  const ZuiZoomFrame({super.key});
  @override
  State<ZuiZoomFrame> createState() => _ZuiZoomFrameState();
}

class _ZuiZoomFrameState extends State<ZuiZoomFrame> {
  // Nivel activo: 0=MACRO, 1=MESO, 2=MICRO.
  int _level = 1;

  static const _labels = ['MACRO', 'MESO', 'MICRO'];
  static const _colors = [
    Gx.transitionBlue,
    Gx.transitionIndigo,
    Gx.optimaCyan,
  ];

  @override
  Widget build(BuildContext context) {
    return frosted(
      padding: const EdgeInsets.all(10),
      glow: Gx.glow(_colors[_level], blur: 18, opacity: 0.2),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          // Etiqueta del nivel activo.
          ShaderMask(
            shaderCallback: (r) => LinearGradient(
                    colors: [_colors[_level], Gx.textSecondary])
                .createShader(r),
            child: Text('NIVEL: ${_labels[_level]}',
                style: Gx.displayGrotesque(
                    fontSize: 12,
                    color: Colors.white,
                    weight: FontWeight.w500)),
          ),
          const SizedBox(height: 10),
          // Marcos anidados que representan los 3 niveles.
          Stack(
            alignment: Alignment.center,
            children: List.generate(3, (i) {
              final isActive = i == _level;
              final size = 80.0 - i * 22.0;
              return GestureDetector(
                onTap: () => setState(() => _level = i),
                child: AnimatedContainer(
                  duration: const Duration(milliseconds: 260),
                  width: size,
                  height: size,
                  decoration: BoxDecoration(
                    borderRadius: BorderRadius.circular(isActive ? 10 : 6),
                    border: Border.all(
                      color: isActive ? _colors[i] : Gx.borderPanel,
                      width: isActive ? 2 : 1,
                    ),
                    boxShadow: isActive
                        ? Gx.glowStrong(_colors[i], 0.8)
                        : null,
                    color: Gx.cardInner.withAlpha(isActive ? 60 : 30),
                  ),
                  child: Center(
                    child: Text(_labels[i],
                        style: Gx.uiSans(
                            fontSize: 8,
                            color: isActive ? _colors[i] : Gx.textMuted)),
                  ),
                ),
              );
            }).reversed.toList(),
          ),
        ],
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Expectation Envelope Badge — indicador "dentro / fuera del sobre"
// ---------------------------------------------------------------------------

// Muestra el estado de una estrategia respecto al "sobre de expectativa"
// del cono de Monte Carlo. dentro=óptimo (cian), fuera=crítico (carmesí).
class ExpectationEnvelopeBadge extends StatefulWidget {
  const ExpectationEnvelopeBadge({super.key});
  @override
  State<ExpectationEnvelopeBadge> createState() =>
      _ExpectationEnvelopeBadgeState();
}

class _ExpectationEnvelopeBadgeState
    extends State<ExpectationEnvelopeBadge> {
  // Estado: true = dentro del sobre, false = fuera.
  bool _inside = true;

  @override
  Widget build(BuildContext context) {
    final c = _inside ? Gx.optimaCyan : Gx.criticalCrimson;
    final bg = _inside ? Gx.optimaChipBg : Gx.criticalChipBg;
    final border = _inside ? Gx.optimaChipBorder : Gx.criticalChipBorder;
    final label = _inside ? '✓ Dentro del sobre' : '✗ Fuera del sobre';

    return GestureDetector(
      onTap: () => setState(() => _inside = !_inside),
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 300),
        padding:
            const EdgeInsets.symmetric(horizontal: 14, vertical: 8),
        decoration: BoxDecoration(
          color: bg,
          borderRadius: BorderRadius.circular(999),
          border: Border.all(color: border),
          boxShadow: Gx.glow(c, blur: 14, opacity: 0.45),
        ),
        child: Text(
          label,
          style: Gx.uiSans(
              fontSize: 13,
              color: c,
              weight: FontWeight.w500),
        ),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Pipeline 8 steps — conducto Ingestión→Retiro completo
// ---------------------------------------------------------------------------

// Los 8 pasos del conducto de Drasus con su estado semántico codificado
// por color (óptimo, transición, pendiente). Render-only.
class Pipeline8Steps extends StatefulWidget {
  const Pipeline8Steps({super.key});
  @override
  State<Pipeline8Steps> createState() => _Pipeline8StepsState();
}

class _Pipeline8StepsState extends State<Pipeline8Steps> {
  // Índice del paso activo (toca para avanzar en la vitrina).
  int _active = 2;

  static const _steps = [
    'Ingest',
    'Genera',
    'Valida',
    'Incuba',
    'Gestiona',
    'Ejecuta',
    'Feedback',
    'Retiro',
  ];

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        // Fila de nodos del pipeline.
        Row(
          children: List.generate(_steps.length, (i) {
            // Determina el color del nodo según su posición relativa al activo.
            final Color c;
            if (i < _active) {
              c = Gx.optimaCyan; // completado
            } else if (i == _active) {
              c = Gx.transitionIndigo; // en curso
            } else if (i == _steps.length - 1) {
              c = Gx.criticalCrimson; // retiro siempre en rojo
            } else {
              c = Gx.textMuted; // pendiente
            }
            return Expanded(
              child: GestureDetector(
                onTap: () => setState(() => _active = i),
                child: Column(children: [
                  // Nodo.
                  AnimatedContainer(
                    duration: const Duration(milliseconds: 200),
                    width: 12,
                    height: 12,
                    decoration: BoxDecoration(
                      shape: BoxShape.circle,
                      color: i == _active ? c : Colors.transparent,
                      border: Border.all(color: c, width: i == _active ? 2 : 1),
                      boxShadow: i <= _active
                          ? Gx.glow(c, blur: 8, opacity: 0.7)
                          : null,
                    ),
                  ),
                  const SizedBox(height: 3),
                  // Etiqueta reducida para que quepa en 8 columnas.
                  Text(
                    _steps[i].substring(0, 3),
                    style: Gx.uiSans(fontSize: 8, color: c),
                  ),
                ]),
              ),
            );
          }),
        ),
        const SizedBox(height: 8),
        // Línea de progreso entre nodos.
        Container(
          height: 2,
          decoration: BoxDecoration(
            color: Gx.gaugeTrack,
            borderRadius: BorderRadius.circular(1),
          ),
          child: FractionallySizedBox(
            alignment: Alignment.centerLeft,
            widthFactor: _active / (_steps.length - 1),
            child: Container(
              decoration: BoxDecoration(
                gradient: Gx.linear(Gx.gradOptima),
                borderRadius: BorderRadius.circular(1),
                boxShadow: Gx.glow(Gx.optimaCyan, blur: 6, opacity: 0.5),
              ),
            ),
          ),
        ),
        const SizedBox(height: 6),
        // Etiqueta del paso activo.
        Text(
          'Etapa activa: ${_steps[_active]}',
          style: Gx.uiSans(fontSize: 11, color: Gx.textSecondary),
        ),
      ],
    );
  }
}
