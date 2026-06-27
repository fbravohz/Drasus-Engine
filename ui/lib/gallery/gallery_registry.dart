// Registro central del catálogo de la galería de componentes.
//
// Este archivo es la fuente única de verdad para:
//   1. El modelo de datos (GalleryEntry, GalleryCategory).
//   2. Todos los builders de las 21 secciones reales del catálogo.
//   3. Los helpers privados que comparten los builders.
//
// La función pública clave es buildGalleryCatalog(context), que retorna la
// lista completa de categorías. gallery_tab.dart la llama una sola vez por
// build; los builders individuales se invocan SOLO cuando el usuario navega
// a ese componente (construcción bajo demanda).

import 'package:flutter/material.dart';
import 'gallery_tokens.dart';
import 'gallery_fx.dart';
import 'gallery_painters.dart';
import '../drasus_theme.dart';
import 'sections/section_nav.dart';
import 'sections/section_inputs_extended.dart';
import 'sections/section_buttons_extended.dart';
import 'sections/section_data_display_extended.dart';
import 'sections/section_feedback_extended.dart';
import 'sections/section_dataviz_extended.dart';
import 'sections/section_drasus_core_extended.dart';
import 'sections/section_std_missing.dart';
import 'sections/section_dataviz_quant.dart';
import 'sections/section_dataviz_new.dart';
import 'sections/section_dag_nodes.dart';
import 'sections/section_animations.dart';
import 'sections/section_trade_tape.dart';

// ---------------------------------------------------------------------------
// Modelo de datos
// ---------------------------------------------------------------------------

/// Entrada de catálogo — representa un componente individual.
/// El [builder] se invoca SOLO cuando el usuario selecciona este componente,
/// garantizando construcción bajo demanda (los pesados no se crean al inicio).
class GalleryEntry {
  final String title;
  final WidgetBuilder builder; // construye el widget bajo demanda
  final bool fullWidth;        // true = Column en vez de Wrap en vista de categoría
  const GalleryEntry(this.title, this.builder, {this.fullWidth = false});
}

/// Categoría del catálogo — agrupa entradas relacionadas.
class GalleryCategory {
  final String title;
  final List<GalleryEntry> entries;
  const GalleryCategory(this.title, this.entries);
}

// ---------------------------------------------------------------------------
// Punto de entrada público — construye las 21 categorías en orden canónico.
// El orden replica EXACTAMENTE las líneas 46-66 de gallery_tab.dart original.
// ---------------------------------------------------------------------------

/// Construye el catálogo completo de las 21 categorías.
/// El orden coincide con el orden de renderizado original de gallery_tab.dart.
List<GalleryCategory> buildGalleryCatalog(BuildContext context) => [
      GalleryCategory('Fundamentos', _foundations()),
      GalleryCategory('Layout y estructura', _layout()),
      GalleryCategory('Navegación', _navigation()),
      GalleryCategory('Inputs y formularios', _inputs()),
      GalleryCategory('Inputs extendidos', _inputsExtended()),
      GalleryCategory('Botones y acciones', _buttons()),
      GalleryCategory('Botones extendidos', _buttonsExtended()),
      GalleryCategory('Data display', _dataDisplay()),
      GalleryCategory('Data display extendido', _dataDisplayExtended()),
      GalleryCategory('Feedback y overlays', _feedback()),
      GalleryCategory('Feedback extendido', _feedbackExtended()),
      GalleryCategory('Data-viz (dominio Drasus)', _dataviz()),
      GalleryCategory('Data-viz extendida', _datavizExtended()),
      GalleryCategory('Data-viz cuantitativa', _datavizQuant()),
      GalleryCategory('Monte Carlo + Cluster 3D', _datavizNew()),
      GalleryCategory('Nodos y Conexiones DAG', _dagNodes()),
      GalleryCategory('Trade Tape + Ticker', _tradeTape()),
      GalleryCategory('Núcleo Drasus', _drasusCore()),
      GalleryCategory('Núcleo Drasus extendido', _drasusCoreExtended()),
      GalleryCategory('Animaciones de Vitalidad', _vitalityAnimations()),
      GalleryCategory('Odómetro + Gauge + Path Drawing', _animationsNew()),
    ];

// ---------------------------------------------------------------------------
// Helper público — marco con etiqueta usada tanto en el registry como en
// gallery_tab.dart (panel de detalle de entrada individual).
// ---------------------------------------------------------------------------

/// Marco visual con etiqueta superior y ancho configurable.
/// Equivalente a la función _frame original de gallery_tab.dart.
Widget galleryFrame(String label, Widget child, {double width = 280}) {
  return SizedBox(
    width: width,
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(label, style: Gx.microLabel),
        const SizedBox(height: 6),
        child,
      ],
    ),
  );
}

// ---------------------------------------------------------------------------
// Helpers privados compartidos por los builders
// ---------------------------------------------------------------------------

/// Panel de superficie que respeta el modo global (glass / tint / solid).
Widget _panelSolid(
    {required Widget child,
    EdgeInsets? padding,
    Color glowColor = Gx.transitionIndigo}) {
  final mode = DrasusThemeState.globalSurfaceMode;
  final pad = padding ?? const EdgeInsets.all(12);

  if (mode == DrasusSurfaceMode.solid) {
    return Container(
      padding: pad,
      decoration: BoxDecoration(
        gradient: Gx.linear([Gx.surfacePanel, Gx.surfaceCard],
            begin: Alignment.topCenter, end: Alignment.bottomCenter),
        border: Border.all(color: Gx.borderBase),
        borderRadius: BorderRadius.circular(Gx.rPanel),
        boxShadow: Gx.glow(glowColor, blur: 20, opacity: 0.10),
      ),
      child: child,
    );
  }

  return panelSurface(
    padding: pad,
    radius: Gx.rPanel,
    glow: Gx.glow(glowColor, blur: 20, opacity: 0.10),
    child: child,
  );
}

/// Chip de estado con glow en borde y texto (neón encendido).
Widget _chip(String text, Color fg, Color bg, Color border,
    {bool pill = false}) {
  return Container(
    padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
    decoration: BoxDecoration(
      color: bg,
      border: Border.all(color: border),
      borderRadius: BorderRadius.circular(pill ? 999 : Gx.rChip),
      boxShadow: Gx.glow(fg, blur: 12, opacity: 0.30),
    ),
    child: Text(text,
        style: Gx
            .uiSans(fontSize: 12, color: fg, height: 1.2)
            .copyWith(shadows: Gx.textGlow(fg))),
  );
}

/// Encabezado de panel con icono y título truncado.
Widget _panelHeader(IconData icon, String title) {
  return Row(children: [
    Icon(icon, size: 14, color: Gx.textBaseSecondary),
    const SizedBox(width: 6),
    Flexible(
        child: Text(title,
            style: Gx.panelTitle, overflow: TextOverflow.ellipsis)),
  ]);
}

/// Barra de gradiente horizontal con glow.
Widget _gradBar(List<Color> colors) => Container(
      height: 14,
      decoration: BoxDecoration(
          gradient: Gx.linear(colors),
          borderRadius: BorderRadius.circular(7),
          boxShadow: Gx.glow(colors.first, blur: 12, opacity: 0.4)),
    );

/// Icono con glow de color.
Widget _glowIcon(IconData icon, Color c) =>
    Icon(icon, size: 20, color: c, shadows: Gx.textGlow(c, 10));

/// Cuadrícula de muestras de color con etiqueta.
Widget _swatches(List<List<Object>> entries, {bool glow = false}) {
  return _panelSolid(
    child: Wrap(
      spacing: 8,
      runSpacing: 8,
      children: entries.map((e) {
        final name = e[0] as String;
        final color = e[1] as Color;
        return Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
          Container(
              width: 64,
              height: 28,
              decoration: BoxDecoration(
                  color: color,
                  borderRadius: BorderRadius.circular(6),
                  border: Border.all(color: Gx.borderBase),
                  boxShadow:
                      glow ? Gx.glow(color, blur: 14, opacity: 0.6) : null)),
          const SizedBox(height: 3),
          SizedBox(width: 64, child: Text(name, style: Gx.microLabel)),
        ]);
      }).toList(),
    ),
  );
}

/// Tabs simuladas (macro/meso/micro) con indicador de gradiente activo.
Widget _tabsMock() {
  Widget tab(String t, bool active) => Padding(
        padding: const EdgeInsets.only(right: 16),
        child: Column(children: [
          Text(t,
              style: TextStyle(
                  fontSize: 13,
                  color: active ? Gx.textBase : Gx.textBaseLabel)),
          const SizedBox(height: 6),
          Container(
              height: 2,
              width: 28,
              decoration: BoxDecoration(
                  gradient: active ? Gx.linear(Gx.gradTransition) : null,
                  boxShadow: active
                      ? Gx.glow(Gx.transitionIndigo, blur: 8, opacity: 0.8)
                      : null)),
        ]),
      );
  return _panelSolid(
      child:
          Row(children: [tab('MACRO', true), tab('MESO', false), tab('MICRO', false)]));
}

/// Pipeline de 5 pasos con puntos de color por estado.
Widget _pipelineMock() {
  final steps = ['Ingest', 'Genera', 'Valida', 'Incuba', 'Ejecuta'];
  final colors = [
    Gx.optimaCyan,
    Gx.optimaCyan,
    Gx.transitionIndigo,
    Gx.textBaseMuted,
    Gx.textBaseMuted
  ];
  return _panelSolid(
    child: Row(
      children: List.generate(steps.length, (i) {
        return Expanded(
          child: Column(children: [
            Container(
                width: 10,
                height: 10,
                decoration: BoxDecoration(
                    shape: BoxShape.circle,
                    color: colors[i],
                    boxShadow: Gx.glow(colors[i], blur: 8, opacity: 0.7))),
            const SizedBox(height: 4),
            Text(steps[i], style: TextStyle(fontSize: 10, color: colors[i])),
          ]),
        );
      }),
    ),
  );
}

/// Checkbox visual con estado on/off.
Widget _checkbox(bool on) => Container(
      width: 18,
      height: 18,
      decoration: BoxDecoration(
          color: on ? Gx.optimaCyan : Colors.transparent,
          borderRadius: BorderRadius.circular(4),
          border: Border.all(color: on ? Gx.optimaCyan : Gx.textBaseMuted),
          boxShadow: on ? Gx.glow(Gx.optimaCyan, blur: 10, opacity: 0.6) : null),
      child: on ? Icon(Gx.iconCheck, size: 14, color: Gx.canvasBase) : null,
    );

/// Radio button visual con estado on/off.
Widget _radio(bool on) => Container(
      width: 18,
      height: 18,
      decoration: BoxDecoration(
          shape: BoxShape.circle,
          border: Border.all(color: on ? Gx.optimaCyan : Gx.textBaseMuted),
          boxShadow: on ? Gx.glow(Gx.optimaCyan, blur: 10, opacity: 0.5) : null),
      child: on
          ? Center(
              child: Container(
                  width: 8,
                  height: 8,
                  decoration: const BoxDecoration(
                      shape: BoxShape.circle, color: Gx.optimaCyan)))
          : null,
    );

/// Botón icono con borde y fondo de tarjeta.
Widget _iconBtn(IconData icon) => Container(
      padding: const EdgeInsets.all(8),
      decoration: BoxDecoration(
          color: Gx.surfaceCard,
          borderRadius: BorderRadius.circular(Gx.rButton),
          border: Border.all(color: Gx.borderBase)),
      child: Icon(icon, size: 18, color: Gx.textBase),
    );

/// Fila clave-valor con borde inferior y color de valor.
Widget _kv(String k, String v, Color vc) => Container(
      padding: const EdgeInsets.symmetric(vertical: 6),
      decoration:
          BoxDecoration(border: Border(bottom: BorderSide(color: Gx.borderBase))),
      child: Row(mainAxisAlignment: MainAxisAlignment.spaceBetween, children: [
        Flexible(
            child: Text(k,
                overflow: TextOverflow.ellipsis,
                style: TextStyle(fontSize: 13, color: Gx.textBaseLabel))),
        Text(v,
            style: TextStyle(
                fontFamily: Gx.fontMono,
                fontSize: 13,
                color: vc,
                shadows: Gx.textGlow(vc, 6))),
      ]),
    );

/// Tabla simulada con tres filas de datos y cabecera.
Widget _tableMock() {
  Widget cell(String t,
          {bool num = false, Color? c, bool header = false}) =>
      Expanded(
        child: Text(t,
            textAlign: num ? TextAlign.right : TextAlign.left,
            style: header
                ? Gx.microLabel
                : TextStyle(
                    fontFamily: num ? Gx.fontMono : null,
                    fontSize: 13,
                    color: c ?? Gx.textBase,
                    shadows: c != null ? Gx.textGlow(c, 6) : null)),
      );
  Widget row(List<Widget> cells,
          {bool header = false, bool hover = false}) =>
      Container(
        padding: const EdgeInsets.symmetric(vertical: 7, horizontal: 8),
        decoration: BoxDecoration(
            color: hover ? Gx.surfaceRaisedDynamic : Colors.transparent,
            border:
                Border(bottom: BorderSide(color: Gx.borderBase))),
        child: Row(children: cells),
      );
  return _panelSolid(
    padding: EdgeInsets.zero,
    child: Column(children: [
      row([
        cell('ID', header: true),
        cell('RÉGIMEN', header: true),
        cell('SHARPE', num: true, header: true)
      ], header: true),
      row([
        cell('node-07'),
        cell('Tendencia', c: Gx.optimaCyan),
        cell('1.84', num: true, c: Gx.optimaCyan)
      ]),
      row([
        cell('node-12'),
        cell('Volátil', c: Gx.alertAmber),
        cell('0.42', num: true, c: Gx.alertAmber)
      ], hover: true),
      row([
        cell('node-19'),
        cell('Fallo', c: Gx.criticalCrimson),
        cell('-0.9', num: true, c: Gx.criticalCrimson)
      ]),
    ]),
  );
}

/// Barra de medidor (gauge) con etiqueta, porcentaje y glow.
Widget _gauge(String label, double v, List<Color> grad, Color glow) =>
    Row(children: [
      SizedBox(width: 48, child: Text(label, style: Gx.microLabel)),
      Expanded(
        child: Container(
          height: 6,
          decoration: BoxDecoration(
              color: Gx.gaugeTrack,
              borderRadius: BorderRadius.circular(3)),
          child: FractionallySizedBox(
            alignment: Alignment.centerLeft,
            widthFactor: v,
            child: Container(
                decoration: BoxDecoration(
                    gradient: Gx.linear(grad),
                    borderRadius: BorderRadius.circular(3),
                    boxShadow: Gx.glow(glow, blur: 8, opacity: 0.6))),
          ),
        ),
      ),
      const SizedBox(width: 8),
      Text(v.toStringAsFixed(2),
          style: TextStyle(
              fontFamily: Gx.fontMono, fontSize: 12, color: glow)),
    ]);

/// Barra de progreso horizontal con gradiente y glow.
Widget _progress(double v, List<Color> grad, Color glow) => Container(
      height: 6,
      decoration: BoxDecoration(
          color: Gx.gaugeTrack, borderRadius: BorderRadius.circular(3)),
      child: FractionallySizedBox(
        alignment: Alignment.centerLeft,
        widthFactor: v,
        child: Container(
            decoration: BoxDecoration(
                gradient: Gx.linear(grad),
                borderRadius: BorderRadius.circular(3),
                boxShadow: Gx.glow(glow, blur: 8, opacity: 0.6))),
      ),
    );

/// Línea skeleton de carga con ancho proporcional.
Widget _skeletonLine(double w) => FractionallySizedBox(
      alignment: Alignment.centerLeft,
      widthFactor: w,
      child: Container(
          height: 10,
          decoration: BoxDecoration(
              color: Gx.surfaceRaisedDynamic,
              borderRadius: BorderRadius.circular(4))),
    );

/// Alerta con icono, mensaje y borde de color izquierdo.
Widget _alert(IconData icon, String msg, Color c, Color bg) => frosted(
      radius: Gx.rPanel,
      padding: const EdgeInsets.all(10),
      glow: Gx.glow(c, blur: 14, opacity: 0.2),
      child: Container(
        decoration: BoxDecoration(
          border: Border(left: BorderSide(color: c, width: 3)),
        ),
        child: Row(children: [
          Icon(icon, size: 16, color: c, shadows: Gx.textGlow(c)),
          const SizedBox(width: 8),
          Expanded(child: Text(msg, style: Gx.bodySecondary)),
        ]),
      ),
    );

/// Modal de confirmación simulado con botón de retiro.
Widget _modalMock() => Container(
      decoration: BoxDecoration(
          color: Gx.canvasBase.withOpacity(0.6),
          borderRadius: BorderRadius.circular(Gx.rChrome)),
      padding: const EdgeInsets.all(12),
      child: panelSurface(
        glow: Gx.glow(Gx.criticalCrimson, blur: 22, opacity: 0.2),
        child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
          Text('Confirmar retiro', style: Gx.subheading),
          const SizedBox(height: 6),
          Text('La célula node-19 será archivada.',
              style: Gx.bodySecondary),
          const SizedBox(height: 12),
          Row(mainAxisAlignment: MainAxisAlignment.end, children: const [
            GlowButton(
                label: 'RETIRAR',
                gradient: Gx.gradCritical,
                glowColor: Gx.criticalCrimson,
                textColor: Gx.pureWhite),
          ]),
        ]),
      ),
    );

/// Célula organismo con gauge de salud y chip de estado.
Widget _organismCard() => Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
          gradient: Gx.linear([Gx.surfaceCard, Gx.surfacePanel]),
          borderRadius: BorderRadius.circular(Gx.rPanel),
          border: Border.all(color: Gx.alertAmber.withOpacity(0.5)),
          boxShadow: Gx.glow(Gx.alertAmber, blur: 16, opacity: 0.18)),
      child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
        Row(mainAxisAlignment: MainAxisAlignment.spaceBetween, children: [
          Text('node-12', style: Gx.dataSmall),
          _chip('VOLÁTIL', Gx.alertAmber, Gx.alertChipBg,
              Gx.alertChipBorder),
        ]),
        const SizedBox(height: 10),
        _gauge('Salud', 0.46, Gx.gradAlert, Gx.alertAmber),
        const SizedBox(height: 6),
        _gauge('Sharpe', 0.42, Gx.gradAlert, Gx.alertAmber),
      ]),
    );

/// Orbe de cristal: gradiente radial + glow potente.
Widget _crystalOrb() => _panelSolid(
      child: Center(
        child: Container(
          width: 60,
          height: 60,
          decoration: BoxDecoration(
            shape: BoxShape.circle,
            gradient: const RadialGradient(
              colors: [
                Gx.optimaCyan,
                Gx.transitionIndigo,
                Gx.transitionPurple
              ],
              stops: [0.0, 0.6, 1.0],
            ),
            boxShadow: Gx.glowStrong(Gx.transitionIndigo, 1.4),
          ),
        ),
      ),
    );

/// Ítem de leyenda con punto de color y texto.
Widget _legend(String t, Color c) => Padding(
      padding: const EdgeInsets.symmetric(vertical: 3),
      child: Row(children: [
        Container(
            width: 10,
            height: 10,
            decoration: BoxDecoration(
                shape: BoxShape.circle,
                color: c,
                boxShadow: Gx.glow(c, blur: 8, opacity: 0.8))),
        const SizedBox(width: 8),
        Flexible(
            child: Text(t,
                style: Gx.bodySecondary,
                overflow: TextOverflow.ellipsis)),
      ]),
    );

/// Cabecera de reporte de autopsia con título degradado y metadatos.
Widget _autopsyHeader() => Container(
      padding: const EdgeInsets.all(14),
      decoration: BoxDecoration(
          gradient: Gx.linear([Gx.surfacePanel, Gx.canvasBase]),
          borderRadius: BorderRadius.circular(Gx.rChrome),
          border: Border.all(color: Gx.criticalChipBorder),
          boxShadow: Gx.glow(Gx.criticalCrimson, blur: 20, opacity: 0.2)),
      child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
        Text('REPORTE FUNERARIO', style: Gx.microLabel),
        const SizedBox(height: 4),
        ShaderMask(
          shaderCallback: (rect) =>
              const LinearGradient(colors: Gx.gradCosmic).createShader(rect),
          child: Text('Autopsia',
              style: TextStyle(
                  fontSize: 36,
                  fontWeight: FontWeight.w500,
                  letterSpacing: -0.6,
                  color: Gx.pureWhite)),
        ),
        const SizedBox(height: 4),
        Text('node-19 · slippage letal',
            style: Gx.dataSmall.copyWith(
                color: Gx.criticalRed,
                shadows: Gx.textGlow(Gx.criticalRed, 6))),
      ]),
    );

// ---------------------------------------------------------------------------
// Builders de sección — retornan List<GalleryEntry>.
// Patrón: cada _frame('X', widget) del original → GalleryEntry('X', (ctx) => widget).
// ---------------------------------------------------------------------------

/// §3 Fundamentos: paleta, gradientes, tipografía, iconografía, superficies.
List<GalleryEntry> _foundations() => [
      GalleryEntry('Paleta — superficies',
          (ctx) => _swatches([
                ['deepSpace', Gx.deepSpace],
                ['navRail', Gx.navRail],
                ['panelSolid', Gx.surfacePanel],
                ['cardInner', Gx.surfaceCard],
                ['surfaceRaised', Gx.surfaceRaised],
              ])),
      GalleryEntry('Paleta — vitalidad (con glow)',
          (ctx) => _swatches(const [
                ['optimaCyan', Gx.optimaCyan],
                ['reactorGreen', Gx.reactorGreen],
                ['transitionIndigo', Gx.transitionIndigo],
                ['transitionBlue', Gx.transitionBlue],
                ['alertAmber', Gx.alertAmber],
                ['criticalCrimson', Gx.criticalCrimson],
              ], glow: true)),
      GalleryEntry(
          'Gradientes',
          (ctx) => _panelSolid(
                child: Column(children: [
                  _gradBar(Gx.gradReactor),
                  const SizedBox(height: 6),
                  _gradBar(Gx.gradAurora),
                  const SizedBox(height: 6),
                  _gradBar(Gx.gradAlert),
                  const SizedBox(height: 6),
                  _gradBar(Gx.gradCritical),
                  const SizedBox(height: 6),
                  _gradBar(Gx.gradCosmic),
                ]),
              )),
      GalleryEntry(
          'Tipografía — escala',
          (ctx) => _panelSolid(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text('ZUI title 40', style: Gx.zuiTitle),
                    const SizedBox(height: 4),
                    Text('Section 22', style: Gx.sectionHeading),
                    const SizedBox(height: 4),
                    Text('Body 14 — texto de interfaz', style: Gx.body),
                    const SizedBox(height: 4),
                    Text('1.847  SPX  node-07', style: Gx.dataSmall),
                  ],
                ),
              )),
      GalleryEntry(
          'Iconografía (con glow)',
          (ctx) => _panelSolid(
                child: Wrap(spacing: 14, runSpacing: 12, children: [
                  _glowIcon(Gx.iconHub, Gx.transitionIndigo),
                  _glowIcon(Gx.iconBolt, Gx.optimaCyan),
                  _glowIcon(Gx.iconWarning, Gx.alertAmber),
                  _glowIcon(Gx.iconDanger, Gx.criticalCrimson),
                  _glowIcon(Gx.iconScience, Gx.transitionBlue),
                  _glowIcon(Gx.iconChart, Gx.optimaTeal),
                ]),
              )),
      GalleryEntry(
          'Superficie — vidrio Apple',
          (ctx) => frosted(
                glow: Gx.glow(Gx.transitionIndigo, blur: 18, opacity: 0.25),
                child: _panelHeader(Gx.iconBlurOn, 'Frosted translúcido'),
              )),
      GalleryEntry(
          'Acento dinámico',
          (ctx) => _panelSolid(
                padding: const EdgeInsets.all(8),
                child: AccentAbSection(),
              )),
    ];

/// §4 Layout: panel de datos, KPI, tabs, pipeline, divider.
List<GalleryEntry> _layout() => [
      GalleryEntry(
          'Panel de datos (hover)',
          (ctx) => HoverGlow(
                color: Gx.transitionIndigo,
                child: _panelSolid(
                  child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        _panelHeader(Gx.iconDashboard, 'Comando de Flota'),
                        const SizedBox(height: 8),
                        Text('Pasa el mouse: la tarjeta se enciende.',
                            style: Gx.bodySecondary),
                      ]),
                ),
              )),
      GalleryEntry(
          'Stat / KPI',
          (ctx) => _panelSolid(
                child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text('SHARPE', style: Gx.microLabel),
                      const SizedBox(height: 4),
                      // Número con gradiente + glow.
                      ShaderMask(
                        shaderCallback: (r) =>
                            const LinearGradient(colors: Gx.gradOptima)
                                .createShader(r),
                        child: Text('1.84',
                            style: TextStyle(
                                fontFamily: Gx.fontMono,
                                fontSize: 28,
                                height: 1.1,
                                color: Gx.pureWhite)),
                      ),
                      Text('óptimo',
                          style: TextStyle(
                              fontSize: 12,
                              color: Gx.optimaCyan,
                              shadows: Gx.textGlow(Gx.optimaCyan))),
                    ]),
              )),
      GalleryEntry('Tabs', (ctx) => _tabsMock()),
      GalleryEntry('Pipeline de 8 pasos', (ctx) => _pipelineMock()),
      GalleryEntry(
          'Divider',
          (ctx) => _panelSolid(
                child: Column(children: [
                  Text('Arriba', style: Gx.body),
                  Divider(color: Gx.borderBase, height: 16),
                  Text('Abajo', style: Gx.body),
                ]),
              )),
    ];

/// §5 Navegación: ZUI pill, breadcrumbs, pagination, command palette, tree.
List<GalleryEntry> _navigation() => [
      GalleryEntry('ZUI Nav Pill', (ctx) => ZuiNavPill()),
      GalleryEntry('Breadcrumbs',
          (ctx) => _panelSolid(child: breadcrumbs())),
      GalleryEntry('Pagination',
          (ctx) => _panelSolid(child: GlowPagination())),
      GalleryEntry('Command Palette', (ctx) => const CommandPalette()),
      GalleryEntry('Tree View',
          (ctx) => _panelSolid(child: GlowTreeView())),
      GalleryEntry(
          'Back to Top',
          (ctx) => _panelSolid(
                child: SizedBox(height: 60, child: GlowBackToTop()),
              )),
      GalleryEntry('Anchor / Scrollspy', (ctx) => GlowScrollspy()),
    ];

/// §6 Inputs: text field, search, dropdown, switch, slider, checkbox, tags.
List<GalleryEntry> _inputs() => [
      GalleryEntry('Text field (foco con glow)',
          (ctx) => GlowInput(hint: 'Símbolo…', initial: 'SPX')),
      GalleryEntry('Search',
          (ctx) => GlowInput(hint: 'Buscar estrategia…', color: Gx.optimaCyan)),
      GalleryEntry(
          'Dropdown (abre)',
          (ctx) => GlowDropdown(
              label: 'Régimen…',
              options: ['Tendencia', 'Rango', 'Volátil', 'Calmo'])),
      GalleryEntry(
          'Switch (toca)',
          (ctx) => Row(children: const [
                GlowSwitch(initial: true),
                SizedBox(width: 12),
                GlowSwitch(initial: false, color: Gx.transitionIndigo),
              ])),
      GalleryEntry('Slider (arrastra)', (ctx) => GlowSlider()),
      GalleryEntry(
          'Checkbox / Radio',
          (ctx) => Row(children: [
                _checkbox(true),
                const SizedBox(width: 10),
                _checkbox(false),
                const SizedBox(width: 16),
                _radio(true),
                const SizedBox(width: 10),
                _radio(false),
              ])),
      GalleryEntry(
          'Tags',
          (ctx) => _panelSolid(
                child: Wrap(spacing: 6, runSpacing: 6, children: [
                  _chip('SPX', Gx.transitionIndigo, Gx.transitionChipBg,
                      Gx.transitionChipBorder),
                  _chip('G10', Gx.transitionIndigo, Gx.transitionChipBg,
                      Gx.transitionChipBorder),
                  Icon(Gx.iconAdd, size: 16, color: Gx.textBaseMuted),
                ]),
              )),
    ];

/// §6 Inputs extendidos: combobox, multiselect, number, textarea, OTP, rating, rich-text, etc.
List<GalleryEntry> _inputsExtended() => [
      GalleryEntry('Combobox / Autocomplete', (ctx) => GlowCombobox()),
      GalleryEntry('Multiselect', (ctx) => GlowMultiSelect()),
      GalleryEntry('Number Input',
          (ctx) => _panelSolid(child: GlowNumberInput())),
      GalleryEntry('Textarea', (ctx) => GlowTextarea()),
      GalleryEntry('OTP / PIN Input',
          (ctx) => _panelSolid(child: GlowOtpInput())),
      GalleryEntry('Rating', (ctx) => _panelSolid(child: GlowRating())),
      GalleryEntry('Rich Text (placeholder)',
          (ctx) => richTextEditorPlaceholder()),
      GalleryEntry('Form Field (normal)', (ctx) => GlowFormField()),
      GalleryEntry('Form Field (error)', (ctx) => GlowFormField(error: true)),
      GalleryEntry('Cascader', (ctx) => GlowCascader()),
      GalleryEntry('Transfer / Dual-list', (ctx) => GlowTransferList()),
      GalleryEntry('Date-range Picker', (ctx) => GlowDateRangePicker()),
      GalleryEntry('Time Picker', (ctx) => GlowTimePicker()),
      GalleryEntry('Color Picker', (ctx) => GlowColorPicker()),
      GalleryEntry('File Upload / Dropzone', (ctx) => GlowDropzone()),
      GalleryEntry('Mention Input', (ctx) => GlowMentionInput()),
    ];

/// §7 Botones: acción viva, primario, transición, peligro, cristal, icon buttons.
List<GalleryEntry> _buttons() => [
      GalleryEntry(
          'Acción viva (clic)',
          (ctx) => GlowButton(
              label: 'EJECUTAR',
              gradient: Gx.gradReactor,
              glowColor: Gx.reactorGreen)),
      GalleryEntry(
          'Primario — cian',
          (ctx) => GlowButton(
              label: 'CONFIRMAR',
              gradient: Gx.gradOptima,
              glowColor: Gx.optimaCyan)),
      GalleryEntry(
          'Transición',
          (ctx) => GlowButton(
              label: 'INCUBAR',
              gradient: Gx.gradTransition,
              glowColor: Gx.transitionIndigo,
              textColor: Gx.pureWhite)),
      GalleryEntry(
          'Peligro',
          (ctx) => GlowButton(
              label: 'RETIRAR',
              gradient: Gx.gradCritical,
              glowColor: Gx.criticalCrimson,
              textColor: Gx.pureWhite)),
      GalleryEntry(
          'Cristal (secundario)',
          (ctx) => frosted(
                glow: Gx.glow(Gx.transitionIndigo, blur: 12, opacity: 0.2),
                child: Text('Detalles', style: Gx.body),
              )),
      GalleryEntry(
          'Icon buttons (hover)',
          (ctx) => Row(children: [
                HoverGlow(
                    color: Gx.optimaCyan,
                    radius: Gx.rButton,
                    child: _iconBtn(Gx.iconPlay)),
                const SizedBox(width: 10),
                HoverGlow(
                    color: Gx.transitionIndigo,
                    radius: Gx.rButton,
                    child: _iconBtn(Gx.iconPause)),
                const SizedBox(width: 10),
                HoverGlow(
                    color: Gx.transitionBlue,
                    radius: Gx.rButton,
                    child: _iconBtn(Gx.iconRefresh)),
              ])),
    ];

/// §7 Botones extendidos: toggle, loading, group, FAB, segmented, split.
List<GalleryEntry> _buttonsExtended() => [
      GalleryEntry(
          'Toggle Button',
          (ctx) => _panelSolid(
                child: Row(children: const [
                  GlowToggleButton(
                      label: 'AUTO', labelOff: 'MANUAL', initial: true),
                  SizedBox(width: 10),
                  GlowToggleButton(
                      label: 'AUTO', labelOff: 'MANUAL', initial: false),
                ]),
              )),
      GalleryEntry('Loading Button',
          (ctx) => _panelSolid(child: GlowLoadingButton())),
      GalleryEntry(
          'Button Group', (ctx) => _panelSolid(child: GlowButtonGroup())),
      GalleryEntry('FAB', (ctx) => _panelSolid(child: GlowFab())),
      GalleryEntry('Segmented Control', (ctx) => GlowSegmented()),
      GalleryEntry('Split Button', (ctx) => GlowSplitButton()),
    ];

/// §8 Data display: chips, key-value, tabla, gauge, progress, calendario, tooltip, skeleton.
List<GalleryEntry> _dataDisplay() => [
      GalleryEntry(
          'Chips de estado',
          (ctx) => _panelSolid(
                child: Wrap(spacing: 6, runSpacing: 6, children: [
                  _chip('ÓPTIMO', Gx.optimaCyan, Gx.optimaChipBg,
                      Gx.optimaChipBorder,
                      pill: true),
                  _chip('INCUBA', Gx.transitionIndigo, Gx.transitionChipBg,
                      Gx.transitionChipBorder,
                      pill: true),
                  _chip('VOLÁTIL', Gx.alertAmber, Gx.alertChipBg,
                      Gx.alertChipBorder,
                      pill: true),
                  _chip('FALLO', Gx.criticalCrimson, Gx.criticalChipBg,
                      Gx.criticalChipBorder,
                      pill: true),
                ]),
              )),
      GalleryEntry(
          'Key-value rows',
          (ctx) => _panelSolid(
                child: Column(children: [
                  _kv('Drawdown', '-4.2%', Gx.alertAmber),
                  _kv('Sharpe', '1.84', Gx.optimaCyan),
                  _kv('Slippage', '0.03%', Gx.textBase),
                ]),
              )),
      GalleryEntry('Tabla densa', (ctx) => _tableMock()),
      GalleryEntry(
          'Micro-gauge',
          (ctx) => _panelSolid(
                child: Column(children: [
                  _gauge('Salud', 0.82, Gx.gradOptima, Gx.optimaCyan),
                  const SizedBox(height: 8),
                  _gauge('Riesgo', 0.41, Gx.gradAlert, Gx.alertAmber),
                ]),
              )),
      GalleryEntry(
          'Progress',
          (ctx) => _panelSolid(
                child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text('Escaneo 68%', style: Gx.microLabel),
                      const SizedBox(height: 6),
                      _progress(0.68, Gx.gradTransition, Gx.transitionIndigo),
                    ]),
              )),
      GalleryEntry('Calendario (toca un día)', (ctx) => GlowCalendar()),
      GalleryEntry(
          'Tooltip',
          (ctx) => frosted(
                glow:
                    Gx.glow(Gx.transitionIndigo, blur: 12, opacity: 0.25),
                radius: Gx.rTooltip,
                child:
                    Text('Sharpe ajustado por régimen', style: Gx.dataSmall),
              )),
      GalleryEntry(
          'Skeleton',
          (ctx) => _panelSolid(
                child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      _skeletonLine(0.8),
                      const SizedBox(height: 6),
                      _skeletonLine(0.55),
                      const SizedBox(height: 6),
                      _skeletonLine(0.65),
                    ]),
              )),
    ];

/// §8 Data display extendido: avatar, timeline, code-block, kbd, desc-list, etc.
List<GalleryEntry> _dataDisplayExtended() => [
      GalleryEntry('Avatar Group',
          (ctx) => _panelSolid(child: avatarGroup())),
      GalleryEntry('Timeline', (ctx) => _panelSolid(child: timeline())),
      GalleryEntry('Code Block', (ctx) => codeBlock()),
      GalleryEntry('Kbd', (ctx) => _panelSolid(child: kbdRow())),
      GalleryEntry('Description List',
          (ctx) => _panelSolid(child: descriptionList())),
      GalleryEntry('Empty State',
          (ctx) => _panelSolid(child: emptyState())),
      GalleryEntry('Image / Thumbnail', (ctx) => imageThumbnail()),
      GalleryEntry(
          'Progress Circular',
          (ctx) => _panelSolid(
                child: Row(children: const [
                  GlowProgressCircular(
                      value: 0.68, color: Gx.transitionIndigo),
                  SizedBox(width: 12),
                  GlowProgressCircular(value: 0.82, color: Gx.optimaCyan),
                  SizedBox(width: 12),
                  GlowProgressCircular(value: 0.35, color: Gx.alertAmber),
                ]),
              )),
      GalleryEntry('Popover', (ctx) => popoverExample()),
      GalleryEntry('Tree Table',
          (ctx) => _panelSolid(child: GlowTreeTable())),
      GalleryEntry('Carousel', (ctx) => _panelSolid(child: GlowCarousel())),
    ];

/// §9 Feedback: alertas, toast, modal, spinner.
List<GalleryEntry> _feedback() => [
      GalleryEntry('Alert — óptimo',
          (ctx) => _alert(Gx.iconCheck, 'Estrategia dentro del sobre.', Gx.optimaCyan, Gx.optimaChipBg)),
      GalleryEntry('Alert — alerta',
          (ctx) => _alert(Gx.iconWarning, 'SPX pasó a Volátil.', Gx.alertAmber, Gx.alertChipBg)),
      GalleryEntry('Alert — crítico',
          (ctx) => _alert(Gx.iconDanger, 'Slippage letal: retiro.', Gx.criticalCrimson, Gx.criticalChipBg)),
      GalleryEntry(
          'Toast',
          (ctx) => frosted(
                glow: Gx.glow(Gx.optimaCyan, blur: 14, opacity: 0.3),
                child: Row(mainAxisSize: MainAxisSize.min, children: [
                  Icon(Gx.iconBolt,
                      size: 16,
                      color: Gx.optimaCyan,
                      shadows: Gx.textGlow(Gx.optimaCyan)),
                  const SizedBox(width: 8),
                  Text('Job encolado', style: Gx.body),
                ]),
              )),
      GalleryEntry('Modal / dialog', (ctx) => _modalMock()),
      GalleryEntry(
          'Spinner',
          (ctx) => _panelSolid(
                child: Row(children: [
                  const SizedBox(
                      width: 18,
                      height: 18,
                      child: CircularProgressIndicator(
                          strokeWidth: 2, color: Gx.transitionIndigo)),
                  const SizedBox(width: 10),
                  Flexible(
                      child: Text('Incubando…',
                          overflow: TextOverflow.ellipsis,
                          style: Gx.bodySecondary)),
                ]),
              )),
    ];

/// §9 Feedback extendido: notification, popconfirm, snackbars, result, backdrop, stepper, accordion.
List<GalleryEntry> _feedbackExtended() => [
      GalleryEntry('Notification Card', (ctx) => GlowNotificationCard()),
      GalleryEntry('Popconfirm', (ctx) => GlowPopconfirm()),
      GalleryEntry('Snackbar variantes', (ctx) => snackbarVariants()),
      GalleryEntry('Result — éxito', (ctx) => resultPage(success: true)),
      GalleryEntry('Result — error', (ctx) => resultPage(success: false)),
      GalleryEntry('Backdrop / Scrim', (ctx) => backdropExample()),
      GalleryEntry('Stepper / Wizard',
          (ctx) => _panelSolid(child: GlowStepper())),
      GalleryEntry('Accordion / Collapse', (ctx) => GlowAccordion()),
    ];

/// §10 Data-viz (dominio Drasus): DAG, Monte Carlo, sparkline.
List<GalleryEntry> _dataviz() => [
      GalleryEntry('DAG (hover en nodos)',
          (ctx) => _panelSolid(child: InteractiveDag())),
      GalleryEntry(
          'Cono de Monte Carlo (hover)',
          (ctx) => _panelSolid(
                padding: EdgeInsets.zero,
                child: HoverableChart(
                    builder: (h) => MonteCarloPainter(hover: h), height: 120),
              )),
      GalleryEntry(
          'Sparkline (hover)',
          (ctx) => _panelSolid(
                padding: EdgeInsets.zero,
                child: HoverableChart(
                    builder: (h) => MonteCarloPainter(hover: h), height: 42),
              )),
    ];

/// §10 Data-viz extendida: heatmap, scatter, regime map, parallel coords, correlation, drawdown.
List<GalleryEntry> _datavizExtended() => [
      GalleryEntry(
          'Heatmap',
          (ctx) => _panelSolid(
                child: SizedBox(
                    height: 120,
                    child: CustomPaint(
                        painter: HeatmapPainter(), size: Size.infinite)),
              )),
      GalleryEntry(
          'Scatter UMAP/PCA',
          (ctx) => _panelSolid(
                child: SizedBox(
                    height: 120,
                    child: CustomPaint(
                        painter: ScatterPainter(), size: Size.infinite)),
              )),
      GalleryEntry(
          'Regime Map',
          (ctx) => _panelSolid(
                child: SizedBox(
                    height: 20,
                    child: CustomPaint(
                        painter: RegimeMapPainter(), size: Size.infinite)),
              )),
      GalleryEntry(
          'Parallel Coordinates',
          (ctx) => _panelSolid(
                child: SizedBox(
                    height: 120,
                    child: CustomPaint(
                        painter: ParallelCoordPainter(), size: Size.infinite)),
              )),
      GalleryEntry(
          'Correlation Matrix',
          (ctx) => _panelSolid(
                child: SizedBox(
                    height: 140,
                    child: CustomPaint(
                        painter: CorrelationMatrixPainter(),
                        size: Size.infinite)),
              )),
      GalleryEntry(
          'Drawdown Curve (hover)',
          (ctx) => _panelSolid(
                padding: EdgeInsets.zero,
                child: HoverableChart(
                    builder: (h) => DrawdownCurvePainter(hover: h),
                    height: 82),
              )),
    ];

/// §10 Data-viz cuantitativa: 13 gráficos financieros con hover interactivo.
List<GalleryEntry> _datavizQuant() => [
      GalleryEntry(
          'Equity Curve (hover)',
          (ctx) => _panelSolid(
                padding: EdgeInsets.zero,
                child: HoverableChart(
                    builder: (h) => EquityCurvePainter(hover: h), height: 100),
              )),
      GalleryEntry(
          'Multi-Equity (áreas apiladas)',
          (ctx) => _panelSolid(
                padding: EdgeInsets.zero,
                child: HoverableChart(
                    builder: (h) => MultiEquityOverlayPainter(hover: h),
                    height: 110),
              )),
      GalleryEntry(
          'Walk Forward Analysis',
          (ctx) => _panelSolid(
                padding: EdgeInsets.zero,
                child: HoverableChart(
                    builder: (h) => WfaChartPainter(hover: h), height: 72),
              )),
      GalleryEntry(
          'Trade Timeline (hover marcas)',
          (ctx) => _panelSolid(
                padding: EdgeInsets.zero,
                child: HoverableChart(
                    builder: (h) => TradeTimelinePainter(hover: h), height: 58),
              )),
      GalleryEntry(
          'Returns Calendar',
          (ctx) => _panelSolid(
                padding: EdgeInsets.zero,
                child: HoverableChart(
                    builder: (h) => ReturnsCalendarPainter(hover: h),
                    height: 84),
              )),
      GalleryEntry(
          'Fitness Evolution (AG)',
          (ctx) => _panelSolid(
                padding: EdgeInsets.zero,
                child: HoverableChart(
                    builder: (h) => FitnessEvolutionPainter(hover: h),
                    height: 96),
              )),
      GalleryEntry(
          'Rolling Metrics',
          (ctx) => _panelSolid(
                padding: EdgeInsets.zero,
                child: HoverableChart(
                    builder: (h) => RollingMetricPainter(hover: h), height: 96),
              )),
      GalleryEntry(
          'Underwater Plot',
          (ctx) => _panelSolid(
                padding: EdgeInsets.zero,
                child: HoverableChart(
                    builder: (h) => UnderwaterPlotPainter(hover: h), height: 82),
              )),
      GalleryEntry(
          'Risk-Return Scatter',
          (ctx) => _panelSolid(
                padding: EdgeInsets.zero,
                child: HoverableChart(
                    builder: (h) => RiskReturnScatterPainter(hover: h),
                    height: 124),
              )),
      GalleryEntry(
          'Trade Distribution',
          (ctx) => _panelSolid(
                padding: EdgeInsets.zero,
                child: HoverableChart(
                    builder: (h) => TradeDistributionPainter(hover: h),
                    height: 94),
              )),
      GalleryEntry(
          'Parameter Sensitivity',
          (ctx) => _panelSolid(
                padding: EdgeInsets.zero,
                child: HoverableChart(
                    builder: (h) => ParameterSensitivityPainter(hover: h),
                    height: 84),
              )),
      GalleryEntry(
          'Regime Timeline',
          (ctx) => _panelSolid(
                padding: EdgeInsets.zero,
                child: HoverableChart(
                    builder: (h) => RegimeTimelinePainter(hover: h), height: 44),
              )),
      GalleryEntry(
          'Optimization Contour (crosshair)',
          (ctx) => _panelSolid(
                padding: EdgeInsets.zero,
                child: HoverableChart(
                    builder: (h) => OptimizationContourPainter(hover: h),
                    height: 114),
              )),
    ];

/// §15 Monte Carlo eléctrico + Cluster 3D nebulosa — ancho completo.
List<GalleryEntry> _datavizNew() => [
      GalleryEntry('Monte Carlo eléctrico',
          (ctx) => MonteCarloLinesWidget(), fullWidth: true),
      GalleryEntry(
          'Cluster 3D nebulosa',
          (ctx) => SingleChildScrollView(
                scrollDirection: Axis.horizontal,
                child: StrategyCluster3dWidget(),
              ),
          fullWidth: true),
    ];

/// §16 Nodos y Conexiones DAG — ancho completo.
List<GalleryEntry> _dagNodes() => [
      GalleryEntry('DAG interactivo', (ctx) => DagNodesSection(),
          fullWidth: true),
    ];

/// §17 Trade Tape + Ticker — ancho completo.
List<GalleryEntry> _tradeTape() => [
      GalleryEntry('Trade Tape + Ticker', (ctx) => TradeTapeSection(),
          fullWidth: true),
    ];

/// §11 Núcleo Drasus: célula organismo, orbe de cristal, leyenda, autopsia.
List<GalleryEntry> _drasusCore() => [
      GalleryEntry(
          'Célula / organismo (hover)',
          (ctx) => HoverGlow(
                color: Gx.alertAmber,
                child: _organismCard(),
              )),
      GalleryEntry('Orbe de cristal', (ctx) => _crystalOrb()),
      GalleryEntry(
          'Leyenda de vitalidad',
          (ctx) => _panelSolid(
                child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      _legend('Óptimo / tendencia', Gx.optimaCyan),
                      _legend('Transición / incubación', Gx.transitionIndigo),
                      _legend('Alerta / volátil', Gx.alertAmber),
                      _legend('Crítico / muerte', Gx.criticalCrimson),
                    ]),
              )),
      GalleryEntry('Portada de autopsia', (ctx) => _autopsyHeader()),
    ];

/// §11 Núcleo Drasus extendido: fleet-command, zui-zoom, expectation, pipeline.
List<GalleryEntry> _drasusCoreExtended() => [
      GalleryEntry('Fleet Command Panel', (ctx) => FleetCommandPanel()),
      GalleryEntry('ZUI Zoom Frame', (ctx) => ZuiZoomFrame()),
      GalleryEntry('Expectation Envelope', (ctx) => ExpectationEnvelopeBadge()),
      GalleryEntry('Pipeline 8 pasos',
          (ctx) => _panelSolid(child: Pipeline8Steps())),
    ];

/// Animaciones de vitalidad: sonar pulse y scan ring en tres estados.
List<GalleryEntry> _vitalityAnimations() => [
      GalleryEntry(
          'Sonar Pulse (toca)',
          (ctx) => _panelSolid(
                child: SizedBox(
                  height: 100,
                  child: Center(
                    child: SonarPulseWidget(
                      color: Gx.optimaCyan,
                      maxRadius: 44,
                      // El orbe de cristal emite el pulso al activarse.
                      child: Container(
                        width: 48,
                        height: 48,
                        decoration: BoxDecoration(
                          shape: BoxShape.circle,
                          gradient: const RadialGradient(
                            colors: [
                              Gx.optimaCyan,
                              Gx.transitionIndigo,
                              Gx.transitionPurple
                            ],
                            stops: [0.0, 0.6, 1.0],
                          ),
                          boxShadow: Gx.glowStrong(Gx.transitionIndigo),
                        ),
                      ),
                    ),
                  ),
                ),
              )),
      GalleryEntry(
          'Scan Ring — activo (2.8s)',
          (ctx) => _panelSolid(
                child: SizedBox(
                  height: 100,
                  child: Center(
                    child: ScanRingWidget(
                      color: Gx.optimaCyan,
                      maxRadius: 44,
                      // La célula organismo emite scan rings mientras está en live.
                      child: Container(
                        padding: const EdgeInsets.symmetric(
                            horizontal: 10, vertical: 6),
                        decoration: BoxDecoration(
                          color: Gx.surfaceCard,
                          borderRadius: BorderRadius.circular(Gx.rChip),
                          border: Border.all(
                              color: Gx.optimaCyan.withOpacity(0.5)),
                          boxShadow:
                              Gx.glow(Gx.optimaCyan, blur: 12, opacity: 0.2),
                        ),
                        child: Text('LIVE',
                            style: TextStyle(
                                fontFamily: Gx.fontMono,
                                fontSize: 11,
                                color: Gx.optimaCyan,
                                shadows: Gx.textGlow(Gx.optimaCyan))),
                      ),
                    ),
                  ),
                ),
              )),
      GalleryEntry(
          'Scan Ring — alerta (1.4s)',
          (ctx) => _panelSolid(
                child: SizedBox(
                  height: 100,
                  child: Center(
                    child: ScanRingWidget(
                      color: Gx.alertAmber,
                      maxRadius: 44,
                      // Período más rápido expresa urgencia del estado de alerta.
                      period: const Duration(milliseconds: 1400),
                      child: Container(
                        padding: const EdgeInsets.symmetric(
                            horizontal: 10, vertical: 6),
                        decoration: BoxDecoration(
                          color: Gx.alertChipBg,
                          borderRadius: BorderRadius.circular(Gx.rChip),
                          border: Border.all(
                              color: Gx.alertAmber.withOpacity(0.5)),
                          boxShadow:
                              Gx.glow(Gx.alertAmber, blur: 12, opacity: 0.25),
                        ),
                        child: Text('ALERTA',
                            style: TextStyle(
                                fontFamily: Gx.fontMono,
                                fontSize: 11,
                                color: Gx.alertAmber,
                                shadows: Gx.textGlow(Gx.alertAmber))),
                      ),
                    ),
                  ),
                ),
              )),
      GalleryEntry(
          'Scan Ring — incubando (5s)',
          (ctx) => _panelSolid(
                child: SizedBox(
                  height: 100,
                  child: Center(
                    child: ScanRingWidget(
                      color: Gx.transitionIndigo,
                      maxRadius: 44,
                      // Período lento expresa calma de la incubación.
                      period: const Duration(milliseconds: 5000),
                      child: Container(
                        padding: const EdgeInsets.symmetric(
                            horizontal: 10, vertical: 6),
                        decoration: BoxDecoration(
                          color: Gx.transitionChipBg,
                          borderRadius: BorderRadius.circular(Gx.rChip),
                          border: Border.all(
                              color: Gx.transitionIndigo.withOpacity(0.5)),
                          boxShadow: Gx.glow(Gx.transitionIndigo,
                              blur: 12, opacity: 0.2),
                        ),
                        child: Text('INCUBA',
                            style: TextStyle(
                                fontFamily: Gx.fontMono,
                                fontSize: 11,
                                color: Gx.transitionIndigo,
                                shadows: Gx.textGlow(Gx.transitionIndigo))),
                      ),
                    ),
                  ),
                ),
              )),
    ];

/// §21 Odómetro + Gauge + Path Drawing — ancho completo.
List<GalleryEntry> _animationsNew() => [
      GalleryEntry('Odómetro', (ctx) => OdometerSection(), fullWidth: true),
      GalleryEntry('Gauge radial', (ctx) => GaugeSection(), fullWidth: true),
      GalleryEntry(
          'Equity Curve animada',
          (ctx) => SizedBox(
                width: double.infinity,
                child: _panelSolid(
                  padding: EdgeInsets.zero,
                  child: EquityCurveAnimated(),
                ),
              ),
          fullWidth: true),
    ];
