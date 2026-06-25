// Galería de Componentes de Drasus Engine — pestaña "Components".
// Vitrina del sistema de diseño (docs/DESIGN.md + docs/DESIGN-COMPONENTS-GALLERY.md).
// Los componentes son cáscaras visuales con datos hardcodeados; lo único que
// manejan es estado de UI local (hover, foco, valor de slider) para las
// micro-animaciones. Sin lógica de negocio ni FFI. Glow, gradientes y vidrio
// Apple a lo largo de TODOS los componentes (inspiración Reflect/galaxia/cristal).

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

// Widget raíz de la pestaña. Telón cósmico estático + catálogo en scroll.
class GalleryTab extends StatelessWidget {
  const GalleryTab({super.key});

  @override
  Widget build(BuildContext context) {
    final theme = DrasusTheme.of(context);
    final surfaces = theme?.surfaces;
    final ds = surfaces?.deepSpace ?? Gx.deepSpace;

    return Container(
      color: ds,
      child: SingleChildScrollView(
          padding: const EdgeInsets.all(24),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              _hero(),
              const SizedBox(height: 32),
              _section(context, 'Fundamentos', _foundations()),
              _section(context, 'Layout y estructura', _layout()),
              _section(context, 'Navegación', _navigation()),
              _section(context, 'Inputs y formularios', _inputs()),
              _section(context, 'Inputs extendidos', _inputsExtended()),
              _section(context, 'Botones y acciones', _buttons()),
              _section(context, 'Botones extendidos', _buttonsExtended()),
              _section(context, 'Data display', _dataDisplay()),
              _section(context, 'Data display extendido', _dataDisplayExtended()),
              _section(context, 'Feedback y overlays', _feedback()),
              _section(context, 'Feedback extendido', _feedbackExtended()),
              _section(context, 'Data-viz (dominio Drasus)', _dataviz()),
              _section(context, 'Data-viz extendida', _datavizExtended()),
              _section(context, 'Data-viz cuantitativa', _datavizQuant()),
              _sectionFull(context, 'Monte Carlo + Cluster 3D', _datavizNew()),
              _sectionFull(context, 'Nodos y Conexiones DAG', _dagNodes()),
              _sectionFull(context, 'Trade Tape + Ticker', _tradeTape()),
              _section(context, 'Núcleo Drasus', _drasusCore()),
              _section(context, 'Núcleo Drasus extendido', _drasusCoreExtended()),
              _section(context, 'Animaciones de Vitalidad', _vitalityAnimations()),
              _sectionFull(context, 'Odómetro + Gauge + Path Drawing', _animationsNew()),
              const SizedBox(height: 48),
            ],
          ),
        ),
      );
    }

  // ---------------------------------------------------------------------------
  // Estructura
  // ---------------------------------------------------------------------------

  Widget _hero() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        ShaderMask(
          shaderCallback: (rect) =>
              const LinearGradient(colors: Gx.gradCosmic).createShader(rect),
          child: Text('Drasus Design System',
              style: Gx.zuiTitle.copyWith(color: Colors.white)),
        ),
        const SizedBox(height: 8),
        // Texto con propagación de luz: tócalo para ver la "explosión".
        const LightBurstText(
            'Terminal futurista en GPU — toca este texto, pasa el mouse, arrastra.'),
      ],
    );
  }

  Widget _section(BuildContext context, String title, List<Widget> children) {
    return Padding(
      padding: const EdgeInsets.only(top: 28),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(children: [
            // Barra de acento con gradiente y glow.
            Container(
                width: 3,
                height: 20,
                margin: const EdgeInsets.only(right: 10),
                decoration: BoxDecoration(
                    gradient: Gx.linear(Gx.gradAurora,
                        begin: Alignment.topCenter, end: Alignment.bottomCenter),
                    boxShadow: Gx.glow(Gx.transitionIndigo, blur: 10, opacity: 0.7))),
            Text(title, style: Gx.sectionHeading),
          ]),
          const SizedBox(height: 16),
          Wrap(spacing: 16, runSpacing: 16, children: children),
        ],
      ),
    );
  }

  // Variante de _section que usa Column en vez de Wrap — para widgets
  // de ancho completo como Monte Carlo y Cluster 3D.
  Widget _sectionFull(BuildContext context, String title, List<Widget> children) {
    return Padding(
      padding: const EdgeInsets.only(top: 28),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(children: [
            Container(
                width: 3,
                height: 20,
                margin: const EdgeInsets.only(right: 10),
                decoration: BoxDecoration(
                    gradient: Gx.linear(Gx.gradAurora,
                        begin: Alignment.topCenter, end: Alignment.bottomCenter),
                    boxShadow: Gx.glow(Gx.transitionIndigo, blur: 10, opacity: 0.7))),
            Text(title, style: Gx.sectionHeading),
          ]),
          const SizedBox(height: 16),
          ...children,
        ],
      ),
    );
  }

  Widget _frame(String label, Widget child, {double width = 280}) {
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
  // Helpers
  // ---------------------------------------------------------------------------

  // Panel de Datos sólido: degradado vertical sutil + hairline + glow tenue.
  Widget _panelSolid(
      {required Widget child,
      EdgeInsets? padding,
      Color glowColor = Gx.transitionIndigo}) {
    return Container(
      padding: padding ?? const EdgeInsets.all(12),
      decoration: BoxDecoration(
        gradient: Gx.linear([Gx.surfacePanel, Gx.surfaceCard],
            begin: Alignment.topCenter, end: Alignment.bottomCenter),
        border: Border.all(color: Gx.borderPanel),
        borderRadius: BorderRadius.circular(Gx.rPanel),
        boxShadow: Gx.glow(glowColor, blur: 20, opacity: 0.10),
      ),
      child: child,
    );
  }

  // Chip de estado con glow en el borde y en el texto (neón encendido).
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
          style: Gx.uiSans(fontSize: 12, color: fg, height: 1.2).copyWith(shadows: Gx.textGlow(fg))),
    );
  }

  Widget _panelHeader(IconData icon, String title) {
    return Row(children: [
      Icon(icon, size: 14, color: Gx.textSecondary),
      const SizedBox(width: 6),
      Flexible(
          child: Text(title,
              style: Gx.panelTitle, overflow: TextOverflow.ellipsis)),
    ]);
  }

  // ---------------------------------------------------------------------------
  // §3 Fundamentos
  // ---------------------------------------------------------------------------

  List<Widget> _foundations() => [
        _frame('Paleta — superficies', _swatches([
          ['deepSpace', Gx.deepSpace],
          ['navRail', Gx.navRail],
          ['panelSolid', Gx.surfacePanel],
          ['cardInner', Gx.surfaceCard],
          ['surfaceRaised', Gx.surfaceRaised],
        ])),
        _frame('Paleta — vitalidad (con glow)', _swatches(const [
          ['optimaCyan', Gx.optimaCyan],
          ['reactorGreen', Gx.reactorGreen],
          ['transitionIndigo', Gx.transitionIndigo],
          ['transitionBlue', Gx.transitionBlue],
          ['alertAmber', Gx.alertAmber],
          ['criticalCrimson', Gx.criticalCrimson],
        ], glow: true)),
        _frame('Gradientes', _panelSolid(
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
        _frame('Tipografía — escala', _panelSolid(
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
        _frame('Iconografía (con glow)', _panelSolid(
          child: Wrap(spacing: 14, runSpacing: 12, children: [
            _glowIcon(Gx.iconHub, Gx.transitionIndigo),
            _glowIcon(Gx.iconBolt, Gx.optimaCyan),
            _glowIcon(Gx.iconWarning, Gx.alertAmber),
            _glowIcon(Gx.iconDanger, Gx.criticalCrimson),
            _glowIcon(Gx.iconScience, Gx.transitionBlue),
            _glowIcon(Gx.iconChart, Gx.optimaTeal),
          ]),
        )),
        _frame('Superficie — vidrio Apple', frosted(
          glow: Gx.glow(Gx.transitionIndigo, blur: 18, opacity: 0.25),
          child: _panelHeader(Gx.iconBlurOn, 'Frosted translúcido'),
        )),
        _frame('Acento dinámico', _panelSolid(
          padding: const EdgeInsets.all(8),
          child: const AccentAbSection(),
        ), width: 380),
      ];

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
                    border: Border.all(color: Gx.borderPanel),
                    boxShadow:
                        glow ? Gx.glow(color, blur: 14, opacity: 0.6) : null)),
            const SizedBox(height: 3),
            SizedBox(width: 64, child: Text(name, style: Gx.microLabel)),
          ]);
        }).toList(),
      ),
    );
  }

  Widget _gradBar(List<Color> colors) => Container(
        height: 14,
        decoration: BoxDecoration(
            gradient: Gx.linear(colors),
            borderRadius: BorderRadius.circular(7),
            boxShadow: Gx.glow(colors.first, blur: 12, opacity: 0.4)),
      );

  Widget _glowIcon(IconData icon, Color c) =>
      Icon(icon, size: 20, color: c, shadows: Gx.textGlow(c, 10));

  // ---------------------------------------------------------------------------
  // §4 Layout
  // ---------------------------------------------------------------------------

  List<Widget> _layout() => [
        _frame('Panel de datos (hover)', HoverGlow(
          color: Gx.transitionIndigo,
          child: _panelSolid(
            child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
              _panelHeader(Gx.iconDashboard, 'Comando de Flota'),
              const SizedBox(height: 8),
              Text('Pasa el mouse: la tarjeta se enciende.',
                  style: Gx.bodySecondary),
            ]),
          ),
        )),
        _frame('Stat / KPI', _panelSolid(
          child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
            Text('SHARPE', style: Gx.microLabel),
            const SizedBox(height: 4),
            // Número con gradiente + glow.
            ShaderMask(
              shaderCallback: (r) =>
                  const LinearGradient(colors: Gx.gradOptima).createShader(r),
              child: const Text('1.84',
                  style: TextStyle(
                      fontFamily: Gx.fontMono,
                      fontSize: 28,
                      height: 1.1,
                      color: Colors.white)),
            ),
            Text('óptimo',
                style: TextStyle(
                    fontSize: 12,
                    color: Gx.optimaCyan,
                    shadows: Gx.textGlow(Gx.optimaCyan))),
          ]),
        )),
        _frame('Tabs', _tabsMock()),
        _frame('Pipeline de 8 pasos', _pipelineMock()),
        _frame('Divider', _panelSolid(
          child: Column(children: [
            Text('Arriba', style: Gx.body),
            const Divider(color: Gx.divider, height: 16),
            Text('Abajo', style: Gx.body),
          ]),
        )),
      ];

  Widget _tabsMock() {
    Widget tab(String t, bool active) => Padding(
          padding: const EdgeInsets.only(right: 16),
          child: Column(children: [
            Text(t,
                style: TextStyle(
                    fontSize: 13,
                    color: active ? Gx.textPrimary : Gx.textLabel)),
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
        child: Row(children: [tab('MACRO', true), tab('MESO', false), tab('MICRO', false)]));
  }

  Widget _pipelineMock() {
    final steps = ['Ingest', 'Genera', 'Valida', 'Incuba', 'Ejecuta'];
    final colors = [
      Gx.optimaCyan,
      Gx.optimaCyan,
      Gx.transitionIndigo,
      Gx.textMuted,
      Gx.textMuted
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

  // ---------------------------------------------------------------------------
  // §6 Inputs (funcionales)
  // ---------------------------------------------------------------------------

  List<Widget> _inputs() => [
        _frame('Text field (foco con glow)',
            const GlowInput(hint: 'Símbolo…', initial: 'SPX')),
        _frame('Search', const GlowInput(hint: 'Buscar estrategia…', color: Gx.optimaCyan)),
        _frame('Dropdown (abre)', const GlowDropdown(
            label: 'Régimen…', options: ['Tendencia', 'Rango', 'Volátil', 'Calmo'])),
        _frame('Switch (toca)', Row(children: const [
          GlowSwitch(initial: true),
          SizedBox(width: 12),
          GlowSwitch(initial: false, color: Gx.transitionIndigo),
        ])),
        _frame('Slider (arrastra)', const GlowSlider()),
        _frame('Checkbox / Radio', Row(children: [
          _checkbox(true),
          const SizedBox(width: 10),
          _checkbox(false),
          const SizedBox(width: 16),
          _radio(true),
          const SizedBox(width: 10),
          _radio(false),
        ])), // checkbox y radio usan iconos Phosphor internamente vía _checkbox/_radio
        _frame('Tags', _panelSolid(
          child: Wrap(spacing: 6, runSpacing: 6, children: [
            _chip('SPX', Gx.transitionIndigo, Gx.transitionChipBg, Gx.transitionChipBorder),
            _chip('G10', Gx.transitionIndigo, Gx.transitionChipBg, Gx.transitionChipBorder),
            Icon(Gx.iconAdd, size: 16, color: Gx.textMuted),
          ]),
        )),
      ];

  Widget _checkbox(bool on) => Container(
        width: 18,
        height: 18,
        decoration: BoxDecoration(
            color: on ? Gx.optimaCyan : Colors.transparent,
            borderRadius: BorderRadius.circular(4),
            border: Border.all(color: on ? Gx.optimaCyan : Gx.textMuted),
            boxShadow: on ? Gx.glow(Gx.optimaCyan, blur: 10, opacity: 0.6) : null),
        child: on ? Icon(Gx.iconCheck, size: 14, color: Gx.deepSpace) : null,
      );

  Widget _radio(bool on) => Container(
        width: 18,
        height: 18,
        decoration: BoxDecoration(
            shape: BoxShape.circle,
            border: Border.all(color: on ? Gx.optimaCyan : Gx.textMuted),
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

  // ---------------------------------------------------------------------------
  // §7 Botones (funcionales: hover + propagación de luz al clic)
  // ---------------------------------------------------------------------------

  List<Widget> _buttons() => [
        _frame('Acción viva (clic)', const GlowButton(
            label: 'EJECUTAR', gradient: Gx.gradReactor, glowColor: Gx.reactorGreen)),
        _frame('Primario — cian', const GlowButton(
            label: 'CONFIRMAR', gradient: Gx.gradOptima, glowColor: Gx.optimaCyan)),
        _frame('Transición', const GlowButton(
            label: 'INCUBAR',
            gradient: Gx.gradTransition,
            glowColor: Gx.transitionIndigo,
            textColor: Gx.pureWhite)),
        _frame('Peligro', const GlowButton(
            label: 'RETIRAR',
            gradient: Gx.gradCritical,
            glowColor: Gx.criticalCrimson,
            textColor: Gx.pureWhite)),
        _frame('Cristal (secundario)', frosted(
          glow: Gx.glow(Gx.transitionIndigo, blur: 12, opacity: 0.2),
          child: Text('Detalles', style: Gx.body),
        )),
        _frame('Icon buttons (hover)', Row(children: [
          HoverGlow(color: Gx.optimaCyan, radius: Gx.rButton, child: _iconBtn(Gx.iconPlay)),
          const SizedBox(width: 10),
          HoverGlow(color: Gx.transitionIndigo, radius: Gx.rButton, child: _iconBtn(Gx.iconPause)),
          const SizedBox(width: 10),
          HoverGlow(color: Gx.transitionBlue, radius: Gx.rButton, child: _iconBtn(Gx.iconRefresh)),
        ])),
      ];

  Widget _iconBtn(IconData icon) => Container(
        padding: const EdgeInsets.all(8),
        decoration: BoxDecoration(
            color: Gx.surfaceCard,
            borderRadius: BorderRadius.circular(Gx.rButton),
            border: Border.all(color: Gx.borderPanel)),
        child: Icon(icon, size: 18, color: Gx.textPrimary),
      );

  // ---------------------------------------------------------------------------
  // §8 Data display
  // ---------------------------------------------------------------------------

  List<Widget> _dataDisplay() => [
        _frame('Chips de estado', _panelSolid(
          child: Wrap(spacing: 6, runSpacing: 6, children: [
            _chip('ÓPTIMO', Gx.optimaCyan, Gx.optimaChipBg, Gx.optimaChipBorder, pill: true),
            _chip('INCUBA', Gx.transitionIndigo, Gx.transitionChipBg, Gx.transitionChipBorder, pill: true),
            _chip('VOLÁTIL', Gx.alertAmber, Gx.alertChipBg, Gx.alertChipBorder, pill: true),
            _chip('FALLO', Gx.criticalCrimson, Gx.criticalChipBg, Gx.criticalChipBorder, pill: true),
          ]),
        )),
        _frame('Key-value rows', _panelSolid(
          child: Column(children: [
            _kv('Drawdown', '-4.2%', Gx.alertAmber),
            _kv('Sharpe', '1.84', Gx.optimaCyan),
            _kv('Slippage', '0.03%', Gx.textPrimary),
          ]),
        )),
        _frame('Tabla densa', _tableMock(), width: 360),
        _frame('Micro-gauge', _panelSolid(
          child: Column(children: [
            _gauge('Salud', 0.82, Gx.gradOptima, Gx.optimaCyan),
            const SizedBox(height: 8),
            _gauge('Riesgo', 0.41, Gx.gradAlert, Gx.alertAmber),
          ]),
        )),
        _frame('Progress', _panelSolid(
          child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
            Text('Escaneo 68%', style: Gx.microLabel),
            const SizedBox(height: 6),
            _progress(0.68, Gx.gradTransition, Gx.transitionIndigo),
          ]),
        )),
        _frame('Calendario (toca un día)', const GlowCalendar()),
        _frame('Tooltip', frosted(
          glow: Gx.glow(Gx.transitionIndigo, blur: 12, opacity: 0.25),
          radius: Gx.rTooltip,
          child: Text('Sharpe ajustado por régimen', style: Gx.dataSmall),
        )),
        _frame('Skeleton', _panelSolid(
          child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
            _skeletonLine(0.8),
            const SizedBox(height: 6),
            _skeletonLine(0.55),
            const SizedBox(height: 6),
            _skeletonLine(0.65),
          ]),
        )),
      ];

  Widget _kv(String k, String v, Color vc) => Container(
        padding: const EdgeInsets.symmetric(vertical: 6),
        decoration: const BoxDecoration(
            border: Border(bottom: BorderSide(color: Gx.divider))),
        child: Row(mainAxisAlignment: MainAxisAlignment.spaceBetween, children: [
          Flexible(
              child: Text(k,
                  overflow: TextOverflow.ellipsis,
                  style: const TextStyle(fontSize: 13, color: Gx.textLabel))),
          Text(v,
              style: TextStyle(
                  fontFamily: Gx.fontMono,
                  fontSize: 13,
                  color: vc,
                  shadows: Gx.textGlow(vc, 6))),
        ]),
      );

  Widget _tableMock() {
    Widget cell(String t, {bool num = false, Color? c, bool header = false}) =>
        Expanded(
          child: Text(t,
              textAlign: num ? TextAlign.right : TextAlign.left,
              style: header
                  ? Gx.microLabel
                  : TextStyle(
                      fontFamily: num ? Gx.fontMono : null,
                      fontSize: 13,
                      color: c ?? Gx.textPrimary,
                      shadows: c != null ? Gx.textGlow(c, 6) : null)),
        );
    Widget row(List<Widget> cells, {bool header = false, bool hover = false}) =>
        Container(
          padding: const EdgeInsets.symmetric(vertical: 7, horizontal: 8),
          decoration: BoxDecoration(
              color: hover ? Gx.surfaceRaised : Colors.transparent,
              border: const Border(bottom: BorderSide(color: Gx.divider))),
          child: Row(children: cells),
        );
    return _panelSolid(
      padding: EdgeInsets.zero,
      child: Column(children: [
        row([cell('ID', header: true), cell('RÉGIMEN', header: true), cell('SHARPE', num: true, header: true)], header: true),
        row([cell('node-07'), cell('Tendencia', c: Gx.optimaCyan), cell('1.84', num: true, c: Gx.optimaCyan)]),
        row([cell('node-12'), cell('Volátil', c: Gx.alertAmber), cell('0.42', num: true, c: Gx.alertAmber)], hover: true),
        row([cell('node-19'), cell('Fallo', c: Gx.criticalCrimson), cell('-0.9', num: true, c: Gx.criticalCrimson)]),
      ]),
    );
  }

  Widget _gauge(String label, double v, List<Color> grad, Color glow) =>
      Row(children: [
        SizedBox(width: 48, child: Text(label, style: Gx.microLabel)),
        Expanded(
          child: Container(
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
          ),
        ),
        const SizedBox(width: 8),
        Text(v.toStringAsFixed(2),
            style: TextStyle(
                fontFamily: Gx.fontMono, fontSize: 12, color: glow)),
      ]);

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

  Widget _skeletonLine(double w) => FractionallySizedBox(
        alignment: Alignment.centerLeft,
        widthFactor: w,
        child: Container(
            height: 10,
            decoration: BoxDecoration(
                color: Gx.surfaceRaised,
                borderRadius: BorderRadius.circular(4))),
      );

  // ---------------------------------------------------------------------------
  // §9 Feedback
  // ---------------------------------------------------------------------------

  List<Widget> _feedback() => [
        _frame('Alert — óptimo', _alert(Gx.iconCheck, 'Estrategia dentro del sobre.', Gx.optimaCyan, Gx.optimaChipBg)),
        _frame('Alert — alerta', _alert(Gx.iconWarning, 'SPX pasó a Volátil.', Gx.alertAmber, Gx.alertChipBg)),
        _frame('Alert — crítico', _alert(Gx.iconDanger, 'Slippage letal: retiro.', Gx.criticalCrimson, Gx.criticalChipBg)),
        _frame('Toast', frosted(
          glow: Gx.glow(Gx.optimaCyan, blur: 14, opacity: 0.3),
          child: Row(mainAxisSize: MainAxisSize.min, children: [
            Icon(Gx.iconBolt, size: 16, color: Gx.optimaCyan, shadows: Gx.textGlow(Gx.optimaCyan)),
            const SizedBox(width: 8),
            Text('Job encolado', style: Gx.body),
          ]),
        )),
        _frame('Modal / dialog', _modalMock(), width: 320),
        _frame('Spinner', _panelSolid(
          child: Row(children: [
            const SizedBox(
                width: 18,
                height: 18,
                child: CircularProgressIndicator(
                    strokeWidth: 2, color: Gx.transitionIndigo)),
            const SizedBox(width: 10),
            Flexible(child: Text('Incubando…', overflow: TextOverflow.ellipsis, style: Gx.bodySecondary)),
          ]),
        )),
      ];

  Widget _alert(IconData icon, String msg, Color c, Color bg) => Container(
        padding: const EdgeInsets.all(10),
        decoration: BoxDecoration(
            gradient: Gx.linear([bg, Gx.surfacePanel]),
            borderRadius: BorderRadius.circular(Gx.rPanel),
            border: Border(left: BorderSide(color: c, width: 3)),
            boxShadow: Gx.glow(c, blur: 14, opacity: 0.2)),
        child: Row(children: [
          Icon(icon, size: 16, color: c, shadows: Gx.textGlow(c)),
          const SizedBox(width: 8),
          Expanded(child: Text(msg, style: Gx.bodySecondary)),
        ]),
      );

  Widget _modalMock() => Container(
        decoration: BoxDecoration(
            color: Gx.deepSpace.withOpacity(0.6),
            borderRadius: BorderRadius.circular(Gx.rChrome)),
        padding: const EdgeInsets.all(12),
        child: frosted(
          glow: Gx.glow(Gx.criticalCrimson, blur: 22, opacity: 0.2),
          child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
            Text('Confirmar retiro', style: Gx.subheading),
            const SizedBox(height: 6),
            Text('La célula node-19 será archivada.', style: Gx.bodySecondary),
            const SizedBox(height: 12),
            Row(mainAxisAlignment: MainAxisAlignment.end, children: const [
              GlowButton(label: 'RETIRAR', gradient: Gx.gradCritical, glowColor: Gx.criticalCrimson, textColor: Gx.pureWhite),
            ]),
          ]),
        ),
      );

  // ---------------------------------------------------------------------------
  // §10 Data-viz
  // ---------------------------------------------------------------------------

  List<Widget> _dataviz() => [
        _frame('DAG (hover en nodos)', _panelSolid(child: const InteractiveDag()), width: 360),
        _frame('Cono de Monte Carlo (hover)', _panelSolid(
          padding: EdgeInsets.zero,
          child: HoverableChart(builder: (h) => MonteCarloPainter(hover: h), height: 120),
        ), width: 360),
        _frame('Sparkline (hover)', _panelSolid(
          padding: EdgeInsets.zero,
          child: HoverableChart(builder: (h) => MonteCarloPainter(hover: h), height: 42),
        )),
      ];

  // ---------------------------------------------------------------------------
  // §11 Núcleo Drasus
  // ---------------------------------------------------------------------------

  List<Widget> _drasusCore() => [
        _frame('Célula / organismo (hover)', HoverGlow(
          color: Gx.alertAmber,
          child: _organismCard(),
        )),
        _frame('Orbe de cristal', _crystalOrb()),
        _frame('Leyenda de vitalidad', _panelSolid(
          child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
            _legend('Óptimo / tendencia', Gx.optimaCyan),
            _legend('Transición / incubación', Gx.transitionIndigo),
            _legend('Alerta / volátil', Gx.alertAmber),
            _legend('Crítico / muerte', Gx.criticalCrimson),
          ]),
        )),
        _frame('Portada de autopsia', _autopsyHeader(), width: 300),
      ];

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
            _chip('VOLÁTIL', Gx.alertAmber, Gx.alertChipBg, Gx.alertChipBorder),
          ]),
          const SizedBox(height: 10),
          _gauge('Salud', 0.46, Gx.gradAlert, Gx.alertAmber),
          const SizedBox(height: 6),
          _gauge('Sharpe', 0.42, Gx.gradAlert, Gx.alertAmber),
        ]),
      );

  // Orbe de cristal: gradiente radial + glow potente (sustituye la aberración
  // cromática que quedaba mal). Limpio, estilo reactor/Apple.
  Widget _crystalOrb() => _panelSolid(
        child: Center(
          child: Container(
            width: 60,
            height: 60,
            decoration: BoxDecoration(
              shape: BoxShape.circle,
              gradient: const RadialGradient(
                colors: [Gx.optimaCyan, Gx.transitionIndigo, Gx.transitionPurple],
                stops: [0.0, 0.6, 1.0],
              ),
              boxShadow: Gx.glowStrong(Gx.transitionIndigo, 1.4),
            ),
          ),
        ),
      );

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
          Flexible(child: Text(t, style: Gx.bodySecondary, overflow: TextOverflow.ellipsis)),
        ]),
      );

  // ---------------------------------------------------------------------------
  // Secciones extendidas (delegan a archivos en sections/)
  // ---------------------------------------------------------------------------

  // §5 Navegación: ZUI pill, breadcrumbs, pagination, command palette, tree.
  List<Widget> _navigation() => [
        _frame('ZUI Nav Pill', const ZuiNavPill()),
        _frame('Breadcrumbs', _panelSolid(child: breadcrumbs())),
        _frame('Pagination', _panelSolid(child: const GlowPagination())),
        _frame('Command Palette', const CommandPalette(), width: 320),
        _frame('Tree View', _panelSolid(child: const GlowTreeView()), width: 240),
        // back-to-top: botón flotante vidrio Apple — §5 STD.
        _frame('Back to Top', _panelSolid(
          child: SizedBox(height: 60, child: const GlowBackToTop()),
        )),
        _frame('Anchor / Scrollspy', const GlowScrollspy(), width: 200),
      ];

  // §6 Inputs extendidos: combobox, multiselect, number, textarea, OTP, rating, rich-text, form-field.
  List<Widget> _inputsExtended() => [
        _frame('Combobox / Autocomplete', const GlowCombobox()),
        _frame('Multiselect', const GlowMultiSelect()),
        _frame('Number Input', _panelSolid(child: const GlowNumberInput())),
        _frame('Textarea', const GlowTextarea()),
        _frame('OTP / PIN Input', _panelSolid(child: const GlowOtpInput())),
        _frame('Rating', _panelSolid(child: const GlowRating())),
        _frame('Rich Text (placeholder)', richTextEditorPlaceholder()),
        _frame('Form Field (normal)', const GlowFormField()),
        _frame('Form Field (error)', const GlowFormField(error: true)),
        // Piezas STD §6 faltantes — cascader, transfer, date-range,
        // time-picker, color-picker, dropzone, mention.
        _frame('Cascader', const GlowCascader(), width: 300),
        _frame('Transfer / Dual-list', const GlowTransferList(), width: 380),
        _frame('Date-range Picker', const GlowDateRangePicker(), width: 340),
        _frame('Time Picker', const GlowTimePicker(), width: 200),
        _frame('Color Picker', const GlowColorPicker(), width: 260),
        _frame('File Upload / Dropzone', const GlowDropzone(), width: 280),
        _frame('Mention Input', const GlowMentionInput(), width: 280),
      ];

  // §7 Botones extendidos: toggle, loading, group, FAB, segmented.
  List<Widget> _buttonsExtended() => [
        _frame('Toggle Button', _panelSolid(
            child: Row(children: const [
          GlowToggleButton(label: 'AUTO', labelOff: 'MANUAL', initial: true),
          SizedBox(width: 10),
          GlowToggleButton(label: 'AUTO', labelOff: 'MANUAL', initial: false),
        ]))),
        _frame('Loading Button', _panelSolid(child: const GlowLoadingButton())),
        _frame('Button Group', _panelSolid(child: const GlowButtonGroup())),
        _frame('FAB', _panelSolid(child: const GlowFab())),
        _frame('Segmented Control', const GlowSegmented()),
        // Split-button — botón con acción principal + dropdown de variantes. §7 STD.
        _frame('Split Button', const GlowSplitButton(), width: 200),
      ];

  // §8 Data display extendido: avatar, timeline, code-block, kbd, desc-list,
  // empty-state, image, progress-circular, popover, tree-table, carousel.
  List<Widget> _dataDisplayExtended() => [
        _frame('Avatar Group', _panelSolid(child: avatarGroup())),
        _frame('Timeline', _panelSolid(child: timeline())),
        _frame('Code Block', codeBlock(), width: 300),
        _frame('Kbd', _panelSolid(child: kbdRow())),
        _frame('Description List', _panelSolid(child: descriptionList())),
        _frame('Empty State', _panelSolid(child: emptyState())),
        _frame('Image / Thumbnail', imageThumbnail()),
        _frame('Progress Circular', _panelSolid(
          child: Row(children: const [
            GlowProgressCircular(value: 0.68, color: Gx.transitionIndigo),
            SizedBox(width: 12),
            GlowProgressCircular(value: 0.82, color: Gx.optimaCyan),
            SizedBox(width: 12),
            GlowProgressCircular(value: 0.35, color: Gx.alertAmber),
          ]),
        )),
        _frame('Popover', popoverExample()),
        _frame('Tree Table', _panelSolid(child: const GlowTreeTable()), width: 340),
        _frame('Carousel', _panelSolid(child: const GlowCarousel()), width: 280),
      ];

  // §9 Feedback extendido: notification, popconfirm, snackbars, result, backdrop, stepper, accordion.
  List<Widget> _feedbackExtended() => [
        _frame('Notification Card', const GlowNotificationCard()),
        _frame('Popconfirm', const GlowPopconfirm()),
        _frame('Snackbar variantes', snackbarVariants(), width: 320),
        _frame('Result — éxito', resultPage(success: true), width: 260),
        _frame('Result — error', resultPage(success: false), width: 260),
        _frame('Backdrop / Scrim', backdropExample(), width: 280),
        _frame('Stepper / Wizard', _panelSolid(child: const GlowStepper()), width: 340),
        _frame('Accordion / Collapse', const GlowAccordion(), width: 300),
      ];

  // §10 Data-viz extendida: heatmap, scatter, regime-map, parallel-coords,
  // correlation matrix, drawdown curve.
  List<Widget> _datavizExtended() => [
        _frame('Heatmap', _panelSolid(
          child: SizedBox(height: 120, child: CustomPaint(painter: HeatmapPainter(), size: Size.infinite)),
        )),
        _frame('Scatter UMAP/PCA', _panelSolid(
          child: SizedBox(height: 120, child: CustomPaint(painter: ScatterPainter(), size: Size.infinite)),
        )),
        _frame('Regime Map', _panelSolid(
          child: SizedBox(height: 20, child: CustomPaint(painter: RegimeMapPainter(), size: Size.infinite)),
        ), width: 360),
        _frame('Parallel Coordinates', _panelSolid(
          child: SizedBox(height: 120, child: CustomPaint(painter: ParallelCoordPainter(), size: Size.infinite)),
        ), width: 360),
        _frame('Correlation Matrix', _panelSolid(
          child: SizedBox(height: 140, child: CustomPaint(painter: CorrelationMatrixPainter(), size: Size.infinite)),
        ), width: 200),
        _frame('Drawdown Curve (hover)', _panelSolid(
          padding: EdgeInsets.zero,
          child: HoverableChart(builder: (h) => DrawdownCurvePainter(hover: h), height: 82),
        ), width: 360),
      ];

  // §11 Núcleo Drasus extendido: fleet-command-panel, zui-zoom-frame,
  // expectation-badge, pipeline-8-steps completo.
  List<Widget> _drasusCoreExtended() => [
        _frame('Fleet Command Panel', const FleetCommandPanel(), width: 480),
        _frame('ZUI Zoom Frame', const ZuiZoomFrame()),
        _frame('Expectation Envelope', const ExpectationEnvelopeBadge()),
        _frame('Pipeline 8 pasos', _panelSolid(child: const Pipeline8Steps()), width: 400),
      ];

  // ---------------------------------------------------------------------------
  // §10 Data-viz cuantitativa (13 gráficos financieros nuevos)
  // ---------------------------------------------------------------------------

  // Gráficos típicos de plataformas quant con hover interactivo (HoverableChart).
  // En hover: línea más gruesa, glow intensificado, cursor vertical, punto de datos.
  List<Widget> _datavizQuant() => [
        _frame('Equity Curve (hover)', _panelSolid(
          padding: EdgeInsets.zero,
          child: HoverableChart(builder: (h) => EquityCurvePainter(hover: h), height: 100),
        ), width: 360),
        _frame('Multi-Equity (áreas apiladas)', _panelSolid(
          padding: EdgeInsets.zero,
          child: HoverableChart(builder: (h) => MultiEquityOverlayPainter(hover: h), height: 110),
        ), width: 360),
        _frame('Walk Forward Analysis', _panelSolid(
          padding: EdgeInsets.zero,
          child: HoverableChart(builder: (h) => WfaChartPainter(hover: h), height: 72),
        ), width: 380),
        _frame('Trade Timeline (hover marcas)', _panelSolid(
          padding: EdgeInsets.zero,
          child: HoverableChart(builder: (h) => TradeTimelinePainter(hover: h), height: 58),
        ), width: 360),
        _frame('Returns Calendar', _panelSolid(
          padding: EdgeInsets.zero,
          child: HoverableChart(builder: (h) => ReturnsCalendarPainter(hover: h), height: 84),
        ), width: 260),
        _frame('Fitness Evolution (AG)', _panelSolid(
          padding: EdgeInsets.zero,
          child: HoverableChart(builder: (h) => FitnessEvolutionPainter(hover: h), height: 96),
        ), width: 320),
        _frame('Rolling Metrics', _panelSolid(
          padding: EdgeInsets.zero,
          child: HoverableChart(builder: (h) => RollingMetricPainter(hover: h), height: 96),
        ), width: 360),
        _frame('Underwater Plot', _panelSolid(
          padding: EdgeInsets.zero,
          child: HoverableChart(builder: (h) => UnderwaterPlotPainter(hover: h), height: 82),
        ), width: 360),
        _frame('Risk-Return Scatter', _panelSolid(
          padding: EdgeInsets.zero,
          child: HoverableChart(builder: (h) => RiskReturnScatterPainter(hover: h), height: 124),
        ), width: 260),
        _frame('Trade Distribution', _panelSolid(
          padding: EdgeInsets.zero,
          child: HoverableChart(builder: (h) => TradeDistributionPainter(hover: h), height: 94),
        ), width: 320),
        _frame('Parameter Sensitivity', _panelSolid(
          padding: EdgeInsets.zero,
          child: HoverableChart(builder: (h) => ParameterSensitivityPainter(hover: h), height: 84),
        ), width: 280),
        _frame('Regime Timeline', _panelSolid(
          padding: EdgeInsets.zero,
          child: HoverableChart(builder: (h) => RegimeTimelinePainter(hover: h), height: 44),
        ), width: 380),
        _frame('Optimization Contour (crosshair)', _panelSolid(
          padding: EdgeInsets.zero,
          child: HoverableChart(builder: (h) => OptimizationContourPainter(hover: h), height: 114),
        ), width: 220),
      ];

  // §10 Data-viz nuevos: Monte Carlo scanInit eléctrico + Cluster 3D nebulosa.
  List<Widget> _datavizNew() => [
        const MonteCarloLinesWidget(),
        const SizedBox(height: 16),
        SingleChildScrollView(
          scrollDirection: Axis.horizontal,
           child: const StrategyCluster3dWidget(),
        ),
      ];

  // §10 Data-viz — nodos y conexiones DAG.
  List<Widget> _dagNodes() => [
        const DagNodesSection(),
      ];

  // §10 Data-viz — trade tape + ticker bar.
  List<Widget> _tradeTape() => [
        const TradeTapeSection(),
      ];

  // Animaciones universales: odómetro, gauge radial, path drawing eléctrico.
  List<Widget> _animationsNew() => [
        const OdometerSection(),
        const SizedBox(height: 16),
        const GaugeSection(),
        const SizedBox(height: 16),
        SizedBox(
          width: double.infinity,
          child: _panelSolid(
            padding: EdgeInsets.zero,
            child: const EquityCurveAnimated(),
          ),
        ),
      ];

  // ---------------------------------------------------------------------------
  // Animaciones de Vitalidad — sonarPulse y scanRing
  // ---------------------------------------------------------------------------

  // Muestra los dos primitivos de animación de vida definidos en Motion Philosophy
  // de DESIGN.md: sonarPulse (evento discreto) y scanRing (monitoreo sostenido).
  List<Widget> _vitalityAnimations() => [
        _frame('Sonar Pulse (toca)', _panelSolid(
          child: SizedBox(
            height: 100,
            child: Center(
              child: SonarPulseWidget(
                color: Gx.optimaCyan,
                maxRadius: 44,
                // El orbe de cristal es el subject que emite el pulso al activarse.
                child: Container(
                  width: 48,
                  height: 48,
                  decoration: BoxDecoration(
                    shape: BoxShape.circle,
                    gradient: const RadialGradient(
                      colors: [Gx.optimaCyan, Gx.transitionIndigo, Gx.transitionPurple],
                      stops: [0.0, 0.6, 1.0],
                    ),
                    boxShadow: Gx.glowStrong(Gx.transitionIndigo),
                  ),
                ),
              ),
            ),
          ),
        )),
        _frame('Scan Ring — activo (2.8s)', _panelSolid(
          child: SizedBox(
            height: 100,
            child: Center(
              child: ScanRingWidget(
                color: Gx.optimaCyan,
                maxRadius: 44,
                // La célula organismo emite scan rings mientras está en live.
                child: Container(
                  padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
                  decoration: BoxDecoration(
                    color: Gx.surfaceCard,
                    borderRadius: BorderRadius.circular(Gx.rChip),
                    border: Border.all(color: Gx.optimaCyan.withOpacity(0.5)),
                    boxShadow: Gx.glow(Gx.optimaCyan, blur: 12, opacity: 0.2),
                  ),
                  child: Text('LIVE', style: TextStyle(
                      fontFamily: Gx.fontMono,
                      fontSize: 11,
                      color: Gx.optimaCyan,
                      shadows: Gx.textGlow(Gx.optimaCyan))),
                ),
              ),
            ),
          ),
        )),
        _frame('Scan Ring — alerta (1.4s)', _panelSolid(
          child: SizedBox(
            height: 100,
            child: Center(
              child: ScanRingWidget(
                color: Gx.alertAmber,
                maxRadius: 44,
                // Período más rápido expresa urgencia del estado de alerta.
                period: const Duration(milliseconds: 1400),
                child: Container(
                  padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
                  decoration: BoxDecoration(
                    color: Gx.alertChipBg,
                    borderRadius: BorderRadius.circular(Gx.rChip),
                    border: Border.all(color: Gx.alertAmber.withOpacity(0.5)),
                    boxShadow: Gx.glow(Gx.alertAmber, blur: 12, opacity: 0.25),
                  ),
                  child: Text('ALERTA', style: TextStyle(
                      fontFamily: Gx.fontMono,
                      fontSize: 11,
                      color: Gx.alertAmber,
                      shadows: Gx.textGlow(Gx.alertAmber))),
                ),
              ),
            ),
          ),
        )),
        _frame('Scan Ring — incubando (5s)', _panelSolid(
          child: SizedBox(
            height: 100,
            child: Center(
              child: ScanRingWidget(
                color: Gx.transitionIndigo,
                maxRadius: 44,
                // Período lento expresa calma de la incubación.
                period: const Duration(milliseconds: 5000),
                child: Container(
                  padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
                  decoration: BoxDecoration(
                    color: Gx.transitionChipBg,
                    borderRadius: BorderRadius.circular(Gx.rChip),
                    border: Border.all(color: Gx.transitionIndigo.withOpacity(0.5)),
                    boxShadow: Gx.glow(Gx.transitionIndigo, blur: 12, opacity: 0.2),
                  ),
                  child: Text('INCUBA', style: TextStyle(
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

  Widget _autopsyHeader() => Container(
        padding: const EdgeInsets.all(14),
        decoration: BoxDecoration(
            gradient: Gx.linear([Gx.surfacePanel, Gx.deepSpace]),
            borderRadius: BorderRadius.circular(Gx.rChrome),
            border: Border.all(color: Gx.criticalChipBorder),
            boxShadow: Gx.glow(Gx.criticalCrimson, blur: 20, opacity: 0.2)),
        child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
          Text('REPORTE FUNERARIO', style: Gx.microLabel),
          const SizedBox(height: 4),
          ShaderMask(
            shaderCallback: (rect) =>
                const LinearGradient(colors: Gx.gradCosmic).createShader(rect),
            child: const Text('Autopsia',
                style: TextStyle(
                    fontSize: 36,
                    fontWeight: FontWeight.w500,
                    letterSpacing: -0.6,
                    color: Colors.white)),
          ),
          const SizedBox(height: 4),
          Text('node-19 · slippage letal',
              style: Gx.dataSmall.copyWith(
                  color: Gx.criticalRed, shadows: Gx.textGlow(Gx.criticalRed, 6))),
        ]),
      );
}
