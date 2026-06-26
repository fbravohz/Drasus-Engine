// Sección §11 Núcleo Drasus extendido — fleet-command-panel, zui-zoom-frame,
// expectation-envelope-badge, pipeline-8-steps completo.
// Render-only con datos hardcodeados. Sin lógica de negocio ni FFI.
// Tokens: superficies via wrappers frosted()/panelSurface(), texto via Gx.textBase*,
// bordes via Gx.borderBase/accentDynamic, espaciado via Gx.space*.

import 'package:flutter/material.dart';
import '../gallery_tokens.dart';
import '../gallery_fx.dart';

// ---------------------------------------------------------------------------
// FleetCommandPanel — panel del Comando de Flota (vista MACRO)
// Parámetros: ninguno (datos sintéticos internos).
// Tokens de chrome: panelSurface() (superficie), Gx.borderBase (borde),
//   Gx.textBase*/textBaseLabel (texto), Gx.gaugeTrack (riel de mini-gauge).
// Colores de estado: optimaCyan/alertAmber/criticalCrimson y sus chipBg/Border
//   (señalizan régimen de cada nodo — se conservan).
// ---------------------------------------------------------------------------

// Panel de datos denso que simula la vista MACRO: tabla de nodos con chips
// de régimen, métricas en columnas y un micro-gauge por fila.
class FleetCommandPanel extends StatelessWidget {
  const FleetCommandPanel({super.key});

  // Datos de la flota hardcodeados para la vitrina.
  static const _nodes = [
    ('alpha-01', 'ÓPTIMO', Gx.optimaCyan, Gx.optimaChipBg,
        Gx.optimaChipBorder, 1.84, 0.82),
    ('beta-02', 'VOLÁTIL', Gx.alertAmber, Gx.alertChipBg,
        Gx.alertChipBorder, 0.42, 0.46),
    ('gamma-03', 'FALLO', Gx.criticalCrimson, Gx.criticalChipBg,
        Gx.criticalChipBorder, -0.9, 0.1),
  ];

  @override
  // Renderiza la tabla con cabecera, filas de nodos y micro-gauges.
  Widget build(BuildContext context) {
    return panelSurface(
      padding: EdgeInsets.zero,
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          // Cabecera del panel con ícono y contador.
          Padding(
            padding: const EdgeInsets.symmetric(
                horizontal: Gx.space8 + Gx.space4,
                vertical: Gx.space8),
            child: Row(children: [
              Icon(Gx.iconDashboard, size: 13, color: Gx.textBaseSecondary),
              SizedBox(width: Gx.space4 + Gx.space4 / 2),
              // Título de panel con token dinámico de panel.
              Text('Comando de Flota',
                  style: Gx.panelTitle
                      .copyWith(color: Gx.textBaseSecondary)),
              const Spacer(),
              // Texto muted dinámico para el contador.
              Text('3 nodos',
                  style: Gx.uiSans(
                      fontSize: 11, color: Gx.textBaseMuted)),
            ]),
          ),
          Divider(color: Gx.divider, height: 1),
          // Fila de cabecera de tabla.
          _headerRow(),
          // Filas de datos por cada nodo.
          ..._nodes.map((n) => _dataRow(n)),
        ],
      ),
    );
  }

  // Cabecera de la tabla con etiquetas de columna en textBaseLabel.
  Widget _headerRow() => Padding(
        padding: const EdgeInsets.symmetric(
            horizontal: Gx.space8 + Gx.space4, vertical: Gx.space4 + Gx.space4 / 2),
        child: Row(children: [
          Expanded(
              flex: 2,
              child: Text('ID',
                  style: Gx.uiSans(
                      fontSize: 11, color: Gx.textBaseLabel))),
          Expanded(
              flex: 2,
              child: Text('RÉGIMEN',
                  style: Gx.uiSans(
                      fontSize: 11, color: Gx.textBaseLabel))),
          Expanded(
              flex: 2,
              child: Text('SHARPE',
                  textAlign: TextAlign.right,
                  style: Gx.uiSans(
                      fontSize: 11, color: Gx.textBaseLabel))),
          const Expanded(flex: 3, child: SizedBox()),
        ]),
      );

  // Fila de datos de un nodo con chip de régimen y micro-gauge.
  Widget _dataRow(
          (String, String, Color, Color, Color, double, double) n) =>
      Container(
        padding: const EdgeInsets.symmetric(
            horizontal: Gx.space8 + Gx.space4, vertical: 7),
        decoration: BoxDecoration(
            border: Border(top: BorderSide(color: Gx.divider))),
        child: Row(children: [
          // ID del nodo con token base dinámico.
          Expanded(
              flex: 2,
              child: Text(n.$1,
                  style:
                      Gx.dataMono(fontSize: 12, color: Gx.textBase))),
          // Chip de régimen con colores semánticos del estado.
          Expanded(flex: 2, child: _chip(n.$2, n.$3, n.$4, n.$5)),
          // Sharpe alineado a la derecha; color semántico del nodo.
          Expanded(
              flex: 2,
              child: Text(n.$6.toStringAsFixed(2),
                  textAlign: TextAlign.right,
                  style: Gx.dataMono(fontSize: 12, color: n.$3))),
          // Micro-gauge de salud del nodo.
          Expanded(
            flex: 3,
            child: Padding(
              padding: EdgeInsets.only(left: Gx.space8),
              child: _gauge(n.$7, n.$3),
            ),
          ),
        ]),
      );

  // Chip de estado compacto: color semántico del régimen (señalización interna).
  Widget _chip(String label, Color fg, Color bg, Color border) =>
      Container(
        padding: const EdgeInsets.symmetric(
            horizontal: Gx.space4 + Gx.space4 / 2, vertical: 3),
        decoration: BoxDecoration(
          color: bg,
          borderRadius: BorderRadius.circular(Gx.rChip),
          border: Border.all(color: border),
          boxShadow: Gx.glow(fg, blur: 8, opacity: 0.25),
        ),
        child: Text(label, style: Gx.uiSans(fontSize: 10, color: fg)),
      );

  // Mini-barra de salud: riel gaugeTrack + relleno de color del nodo.
  Widget _gauge(double v, Color c) => Container(
        height: 4,
        decoration: BoxDecoration(
            color: Gx.gaugeTrack,
            // barra del mini-gauge (4px alto): radio decorativo
            borderRadius: BorderRadius.circular(2)),
        child: FractionallySizedBox(
          alignment: Alignment.centerLeft,
          widthFactor: v,
          child: Container(
            decoration: BoxDecoration(
              color: c,
              // barra del mini-gauge (4px alto): radio decorativo
              borderRadius: BorderRadius.circular(2),
              boxShadow: Gx.glow(c, blur: 6, opacity: 0.6),
            ),
          ),
        ),
      );
}

// ---------------------------------------------------------------------------
// ZuiZoomFrame — marco de transición de zoom MACRO↔MESO↔MICRO
// Parámetros: ninguno (estado local _level).
// Tokens de chrome: frosted() (superficie), Gx.borderBase (bordes inactivos),
//   Gx.surfaceCard (fondo de marcos), Gx.textBaseMuted (etiqueta inactiva).
// Colores de estado: transitionBlue/transitionIndigo/optimaCyan por nivel
//   (señalizan el nivel activo — se conservan).
// pureWhite: necesario para que ShaderMask aplique el gradiente correctamente.
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
  // Colores de estado por nivel: señalizan qué nivel está activo.
  static const _colors = [
    Gx.transitionBlue,
    Gx.transitionIndigo,
    Gx.optimaCyan,
  ];

  @override
  // Renderiza el contenedor frosted con la etiqueta del nivel y los marcos anidados.
  Widget build(BuildContext context) {
    return panelSurface(
      padding: const EdgeInsets.all(Gx.space8 + Gx.space4),
      glow: Gx.glow(_colors[_level], blur: 18, opacity: 0.2),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          // Etiqueta del nivel activo con ShaderMask del color de estado.
          // pureWhite: necesario para que ShaderMask coloree el texto.
          ShaderMask(
            shaderCallback: (r) => LinearGradient(
                    colors: [_colors[_level], Gx.textBaseSecondary])
                .createShader(r),
            child: Text('NIVEL: ${_labels[_level]}',
                style: Gx.displayGrotesque(
                    fontSize: 12,
                    color: Gx.pureWhite,
                    weight: FontWeight.w500)),
          ),
          SizedBox(height: Gx.space8 + Gx.space4),
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
                    // marco activo usa token; inactivo reduce a 6px decorativo
                    borderRadius:
                        BorderRadius.circular(isActive ? Gx.rButton : 6),
                    border: Border.all(
                      // Borde activo: color semántico del nivel. Inactivo: borderBase.
                      color: isActive ? _colors[i] : Gx.borderBase,
                      width: isActive ? 2 : Gx.borderHairline,
                    ),
                    boxShadow: isActive
                        ? Gx.glowStrong(_colors[i], 0.8)
                        : null,
                    color: Gx.surfaceCard.withAlpha(isActive ? 60 : 30),
                  ),
                  child: Center(
                    child: Text(_labels[i],
                        style: Gx.uiSans(
                            fontSize: 8,
                            // Etiqueta: semántica en activo, muted en inactivo.
                            color: isActive
                                ? _colors[i]
                                : Gx.textBaseMuted)),
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
// ExpectationEnvelopeBadge — indicador "dentro / fuera del sobre"
// Parámetros: ninguno (estado local _inside).
// Tokens de chrome: chipBg/chipBorder estáticos (fondos y bordes de chip de estado).
// Colores de dato: optimaCyan/criticalCrimson + sus chipBg/Border
//   (señalizan si la estrategia está dentro o fuera del cono de Monte Carlo).
// ---------------------------------------------------------------------------

// Muestra el estado de una estrategia respecto al "sobre de expectativa"
// del cono de Monte Carlo. dentro=óptimo (cian), fuera=crítico (carmesí).
// Al tocar alterna entre ambos estados (demo de interacción).
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
  // Renderiza el badge con color semántico según el estado actual.
  Widget build(BuildContext context) {
    final c = _inside ? Gx.optimaCyan : Gx.criticalCrimson;
    // chipBg y chipBorder son tokens estáticos de fondos/bordes de chip por estado.
    final bg = _inside ? Gx.optimaChipBg : Gx.criticalChipBg;
    final border = _inside ? Gx.optimaChipBorder : Gx.criticalChipBorder;
    final label = _inside ? '✓ Dentro del sobre' : '✗ Fuera del sobre';

    return GestureDetector(
      onTap: () => setState(() => _inside = !_inside),
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 300),
        padding: const EdgeInsets.symmetric(
            horizontal: Gx.space12 + Gx.space4, vertical: Gx.space8),
        decoration: BoxDecoration(
          color: bg,
          borderRadius: BorderRadius.circular(999),
          border: Border.all(color: border),
          boxShadow: Gx.glow(c, blur: 14, opacity: 0.45),
        ),
        child: Text(
          label,
          style: Gx.uiSans(
              fontSize: 13, color: c, weight: FontWeight.w500),
        ),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Pipeline8Steps — conducto Ingestión→Retiro (8 pasos)
// Parámetros: ninguno (estado local _active).
// Tokens de chrome: Gx.gaugeTrack (riel de la línea de progreso),
//   Gx.textBaseSecondary (etiqueta activa abajo), Gx.textBaseMuted (paso pendiente).
// Colores de estado: optimaCyan (completado), transitionIndigo (en curso),
//   criticalCrimson (retiro), textBaseMuted (pendiente) — se conservan.
// ---------------------------------------------------------------------------

// Los 8 pasos del conducto de Drasus con su estado semántico codificado
// por color. Al tocar un nodo se activa ese paso (demo de interacción).
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
  // Renderiza la fila de nodos, la línea de progreso y la etiqueta del paso activo.
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        // Fila de nodos del pipeline.
        Row(
          children: List.generate(_steps.length, (i) {
            // Color del nodo según su posición relativa al activo (señalización interna).
            final Color c;
            if (i < _active) {
              c = Gx.optimaCyan; // completado
            } else if (i == _active) {
              c = Gx.transitionIndigo; // en curso
            } else if (i == _steps.length - 1) {
              c = Gx.criticalCrimson; // retiro — siempre en rojo como señal de fin de ciclo
            } else {
              c = Gx.textBaseMuted; // pendiente
            }
            return Expanded(
              child: GestureDetector(
                onTap: () => setState(() => _active = i),
                child: Column(children: [
                  // Nodo circular con estado animado.
                  AnimatedContainer(
                    duration: const Duration(milliseconds: 200),
                    width: 12,
                    height: 12,
                    decoration: BoxDecoration(
                      shape: BoxShape.circle,
                      // Activo: relleno semántico; inactivo: transparent.
                      color: i == _active ? c : Colors.transparent,
                      border:
                          Border.all(color: c, width: i == _active ? 2 : 1),
                      boxShadow: i <= _active
                          ? Gx.glow(c, blur: 8, opacity: 0.7)
                          : null,
                    ),
                  ),
                  SizedBox(height: 3),
                  // Etiqueta abreviada del paso con color de estado.
                  Text(
                    _steps[i].substring(0, 3),
                    style: Gx.uiSans(fontSize: 8, color: c),
                  ),
                ]),
              ),
            );
          }),
        ),
        SizedBox(height: Gx.space8),
        // Línea de progreso entre nodos.
        Container(
          height: 2,
          decoration: BoxDecoration(
            color: Gx.gaugeTrack,
            // línea de progreso (2px alto): radio decorativo
            borderRadius: BorderRadius.circular(1),
          ),
          child: FractionallySizedBox(
            alignment: Alignment.centerLeft,
            widthFactor: _active / (_steps.length - 1),
            child: Container(
              decoration: BoxDecoration(
                gradient: Gx.linear(Gx.gradOptima),
                // línea de progreso (2px alto): radio decorativo
                borderRadius: BorderRadius.circular(1),
                boxShadow: Gx.glow(Gx.optimaCyan, blur: 6, opacity: 0.5),
              ),
            ),
          ),
        ),
        SizedBox(height: Gx.space4 + Gx.space4 / 2),
        // Etiqueta del paso activo con token dinámico secundario.
        Text(
          'Etapa activa: ${_steps[_active]}',
          style: Gx.uiSans(fontSize: 11, color: Gx.textBaseSecondary),
        ),
      ],
    );
  }
}
