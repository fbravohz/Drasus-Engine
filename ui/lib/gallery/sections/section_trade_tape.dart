// §10 Data-viz — trade-tape y trade-ticker-bar.
// Cinta de órdenes en vivo (vertical) y ticker horizontal.
// Todos los datos son sintéticos — sin Rust ni FFI.
// Usa Timer.periodic + AnimatedList para el scroll de nuevas entradas.
// El ticker usa AnimationController infinito para el desplazamiento lateral.
// Tokens: superficies via panelSurface()/Gx.surfacePanel, texto via Gx.textBase*,
//   bordes via Gx.borderBase, espaciado via Gx.space*.

import 'dart:async';
import 'dart:math';
import 'package:flutter/material.dart';
import '../gallery_tokens.dart';
import '../gallery_fx.dart';

// ---------------------------------------------------------------------------
// _TradeEntry — modelo de una entrada de trade sintética
// Todos los campos son datos de visualización; sin lógica de negocio.
// ---------------------------------------------------------------------------

// Modelo de datos de un trade sintético para la vitrina.
class _TradeEntry {
  final String symbol;
  final bool isBuy;
  final double price;
  final double lots;
  final double pnl;

  const _TradeEntry({
    required this.symbol,
    required this.isBuy,
    required this.price,
    required this.lots,
    required this.pnl,
  });
}

// Símbolos ciclando para la generación sintética.
const _symbols = ['EURUSD', 'GBPUSD', 'XAUUSD', 'USDJPY', 'USDCAD'];

// ---------------------------------------------------------------------------
// _randomEntry() — generador de entradas sintéticas con semilla de tiempo
// Produce un _TradeEntry aleatorio usando el cursor de símbolo para el par.
// ---------------------------------------------------------------------------

// Genera una entrada sintética aleatoria con precio base por símbolo.
_TradeEntry _randomEntry(Random rnd, int symbolIdx) {
  final symbol = _symbols[symbolIdx % _symbols.length];
  final isBuy = rnd.nextBool();
  final basePrice = switch (symbol) {
    'EURUSD' => 1.08432,
    'GBPUSD' => 1.26891,
    'XAUUSD' => 2341.20,
    'USDJPY' => 149.82,
    _ => 1.35124,
  };
  final price = basePrice + (rnd.nextDouble() - 0.5) * 0.002;
  final lots = (rnd.nextDouble() * 4 + 0.1).roundToDouble();
  // PnL ligeramente sesgado al positivo para la demostración.
  final pnl = (rnd.nextDouble() - 0.4) * 300;
  return _TradeEntry(
    symbol: symbol,
    isBuy: isBuy,
    price: price,
    lots: lots,
    pnl: pnl,
  );
}

// ===========================================================================
// TradeTapeSection — sección completa de la galería
// Incluye: header de sección + TradeTapeWidget + TradeTicker
// Parámetros: ninguno (estado en subwidgets).
// Tokens de chrome: ver subwidgets.
// ===========================================================================

// Sección exportable para gallery_tab.dart.
// Muestra el header con chip LIVE parpadeante, la cinta vertical y el ticker.
class TradeTapeSection extends StatelessWidget {
  const TradeTapeSection({super.key});

  @override
  // Renderiza header, cinta vertical y ticker horizontal.
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [
        // Header de sección con chip LIVE parpadeante.
        _TapeSectionHeader(),
        SizedBox(height: Gx.space16),
        // Las dos variantes lado a lado en un Wrap.
        Wrap(
          spacing: Gx.space16,
          runSpacing: Gx.space16,
          children: const [
            // Cinta vertical (300px × 350px).
            TradeTapeWidget(),
            // Información sobre la variante ticker.
            _TickerDemo(),
          ],
        ),
        SizedBox(height: Gx.space12),
        // Ticker horizontal en ancho completo.
        const TradeTickerBar(),
      ],
    );
  }
}

// ---------------------------------------------------------------------------
// _TapeSectionHeader — header con ícono + título + chip "LIVE" parpadeante
// Tokens de chrome: Gx.textBase (título panelTitle dinámico).
// Colores de dato: Gx.reactorGreen (señal LIVE — estado de actividad, se conserva).
// ---------------------------------------------------------------------------

// Header con ícono de relámpago, título y chip "LIVE" parpadeante en reactorGreen.
class _TapeSectionHeader extends StatefulWidget {
  @override
  State<_TapeSectionHeader> createState() => _TapeSectionHeaderState();
}

class _TapeSectionHeaderState extends State<_TapeSectionHeader>
    with SingleTickerProviderStateMixin {
  late AnimationController _blinkCtrl;

  @override
  void initState() {
    super.initState();
    // Parpadeo del chip LIVE: ciclo de 1200ms, va y vuelve en opacidad.
    _blinkCtrl = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 1200),
    )..repeat(reverse: true);
  }

  @override
  void dispose() {
    _blinkCtrl.dispose();
    super.dispose();
  }

  @override
  // Renderiza el ícono, el título y el chip LIVE animado con token de texto dinámico.
  Widget build(BuildContext context) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        // reactorGreen es el color del estado "activo" — señalización interna.
        Icon(Icons.flash_on,
            size: 14,
            color: Gx.reactorGreen,
            shadows: Gx.textGlow(Gx.reactorGreen)),
        SizedBox(width: Gx.space4 + Gx.space4 / 2),
        // Título con token dinámico de panel.
        Text('Cinta de Órdenes',
            style: Gx.panelTitle.copyWith(color: Gx.textBaseSecondary)),
        SizedBox(width: Gx.space8 + Gx.space4),
        // Chip LIVE que parpadea en reactorGreen (señal de actividad).
        AnimatedBuilder(
          animation: _blinkCtrl,
          builder: (_, __) => Container(
            padding: const EdgeInsets.symmetric(
                horizontal: Gx.space8, vertical: 3),
            decoration: BoxDecoration(
              color: Gx.reactorGreen
                  .withOpacity(0.12 + _blinkCtrl.value * 0.10),
              border: Border.all(
                color: Gx.reactorGreen
                    .withOpacity(0.50 + _blinkCtrl.value * 0.30),
              ),
              borderRadius: BorderRadius.circular(999),
              boxShadow: Gx.glow(Gx.reactorGreen,
                  blur: 8,
                  opacity: 0.20 + _blinkCtrl.value * 0.25),
            ),
            child: Text('LIVE',
                style: Gx.dataMono(
                    fontSize: 10,
                    color: Gx.reactorGreen
                        .withOpacity(0.70 + _blinkCtrl.value * 0.30))),
          ),
        ),
      ],
    );
  }
}

// ===========================================================================
// TradeTapeWidget — cinta vertical de órdenes
// Tokens de chrome: Gx.surfacePanel (fondo del contenedor), Gx.borderBase (borde).
// Nota ShaderMask: Colors.black y Colors.transparent son la MÁSCARA de opacidad
//   del BlendMode.dstIn — Colors.black opaco = pixel visible, Colors.transparent =
//   pixel borrado. NO son colores de chrome; este uso es correcto e inamovible.
// ===========================================================================

// Cinta vertical de órdenes en vivo. Timer.periodic cada 800ms inserta una
// nueva entrada al inicio con AnimatedList (slide desde arriba).
// ShaderMask aplica fade de 32px en el borde superior e inferior.
class TradeTapeWidget extends StatefulWidget {
  const TradeTapeWidget({super.key});

  @override
  State<TradeTapeWidget> createState() => _TradeTapeWidgetState();
}

class _TradeTapeWidgetState extends State<TradeTapeWidget> {
  final GlobalKey<AnimatedListState> _listKey = GlobalKey<AnimatedListState>();
  final List<_TradeEntry> _entries = [];
  late Timer _timer;
  final Random _rnd = Random();
  int _symbolCursor = 0;

  @override
  void initState() {
    super.initState();
    // Rellena la lista con 10 entradas iniciales sin animación.
    for (var i = 0; i < 10; i++) {
      _entries.add(_randomEntry(_rnd, _symbolCursor++));
    }
    // Inserta una nueva entrada cada 800ms con animación de slide.
    _timer = Timer.periodic(const Duration(milliseconds: 800), (_) {
      final entry = _randomEntry(_rnd, _symbolCursor++);
      _entries.insert(0, entry);
      _listKey.currentState
          ?.insertItem(0, duration: const Duration(milliseconds: 350));
      // Limita la lista a 40 entradas para no acumular memoria.
      if (_entries.length > 40) {
        _entries.removeAt(_entries.length - 1);
      }
    });
  }

  @override
  void dispose() {
    _timer.cancel();
    super.dispose();
  }

  @override
  // Contenedor de 300×350px con borde estructural global y lista animada con fade.
  Widget build(BuildContext context) {
    return SizedBox(
      width: 300,
      height: 350,
      child: panelSurface(
        padding: EdgeInsets.zero,
        radius: Gx.rPanel,
        child: Column(
            children: [
              // Header interno con conteo de trades.
              _TapeHeader(count: _entries.length),
              // Lista con fade en los bordes (ShaderMask con BlendMode.dstIn).
              Expanded(
                child: ShaderMask(
                  blendMode: BlendMode.dstIn,
                  shaderCallback: (bounds) => LinearGradient(
                    begin: Alignment.topCenter,
                    end: Alignment.bottomCenter,
                    colors: const [
                      // NOTA: Colors.transparent y Colors.black son la MÁSCARA de opacidad
                      // del BlendMode.dstIn — NO son colores de chrome de la UI.
                      // Colors.black opaco = preserva el pixel (visible).
                      // Colors.transparent = borra el pixel (fade a nada).
                      // Este patrón es estándar e invariante de Flutter para ShaderMask.
                      Colors.transparent, // fade superior 32px
                      Colors.black,
                      Colors.black,
                      Colors.transparent, // fade inferior 32px
                    ],
                    stops: [
                      0.0,
                      32 / 318, // aproximación de bounds.height (350 - header ~32)
                      1.0 - 32 / 318,
                      1.0,
                    ],
                  ).createShader(bounds),
                  child: AnimatedList(
                    key: _listKey,
                    initialItemCount: _entries.length,
                    itemBuilder: (ctx, index, animation) {
                      if (index >= _entries.length)
                        return const SizedBox.shrink();
                      return _TradeEntryRow(
                        entry: _entries[index],
                        animation: animation,
                      );
                    },
                  ),
                ),
              ),
            ],
          ),
        ),
    );
  }
}

// ---------------------------------------------------------------------------
// _TapeHeader — header interno de la cinta con "LIVE TRADE TAPE" + conteo
// Tokens de chrome: Gx.borderBase (borde inferior), Gx.textBaseSecondary (título),
//   Gx.textBaseMuted (contador).
// ---------------------------------------------------------------------------

// Header interno: etiqueta "LIVE TRADE TAPE" y contador de trades.
class _TapeHeader extends StatelessWidget {
  final int count;
  const _TapeHeader({required this.count});

  @override
  // Renderiza la barra de cabecera con borde inferior estructural global.
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(
          horizontal: Gx.space8 + Gx.space4, vertical: Gx.space8),
      decoration: BoxDecoration(
        // Borde inferior estructural global dinámico — reacciona al énfasis activo.
        border: Border(
            bottom: BorderSide(color: Gx.borderBase, width: Gx.borderHairline)),
      ),
      child: Row(
        children: [
          Text('LIVE TRADE TAPE',
              style:
                  Gx.dataMono(fontSize: 11, color: Gx.textBaseSecondary)),
          const Spacer(),
          Text('${count > 40 ? "5.737" : count} trades',
              style: Gx.dataMono(fontSize: 10, color: Gx.textBaseMuted)),
        ],
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// _TradeEntryRow — fila individual con animación de slide entrada
// Envuelve _TradeRowContent en SizeTransition + FadeTransition al insertarse.
// ---------------------------------------------------------------------------

// Fila con animación de deslizamiento desde arriba al insertarse en la lista.
class _TradeEntryRow extends StatelessWidget {
  final _TradeEntry entry;
  final Animation<double> animation;

  const _TradeEntryRow({
    required this.entry,
    required this.animation,
  });

  @override
  // Aplica SizeTransition y FadeTransition al contenido de la fila.
  Widget build(BuildContext context) {
    return SizeTransition(
      sizeFactor:
          CurvedAnimation(parent: animation, curve: Curves.easeOut),
      child: FadeTransition(
        opacity: animation,
        child: _TradeRowContent(entry: entry),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// _TradeRowContent — contenido de la fila: símbolo, dirección, precio, lotes, PnL
// Tokens de chrome: Gx.divider (borde inferior de fila), Gx.textBase (símbolo),
//   Gx.textBaseSecondary (precio), Gx.textBaseMuted (lotes).
// Colores de dato: optimaCyan (BUY/PnL positivo), criticalRed (SELL/PnL negativo)
//   — señalizan la dirección y el resultado del trade, se conservan.
// ---------------------------------------------------------------------------

// Contenido visual de una entrada de trade: símbolo, dirección, precio, lotes y PnL.
class _TradeRowContent extends StatelessWidget {
  final _TradeEntry entry;
  const _TradeRowContent({required this.entry});

  @override
  // Fila con columnas fijas para símbolo, dirección, precio, lotes y PnL.
  Widget build(BuildContext context) {
    // Colores de dato: señalizan la dirección y el resultado del trade.
    final dirColor = entry.isBuy ? Gx.optimaCyan : Gx.criticalRed;
    final pnlColor = entry.pnl >= 0 ? Gx.optimaCyan : Gx.criticalRed;
    final pnlSign = entry.pnl >= 0 ? '+' : '';
    final dirLabel = entry.isBuy ? 'BUY' : 'SELL';

    return Container(
      padding: const EdgeInsets.symmetric(
          horizontal: Gx.space8 + Gx.space4, vertical: Gx.space4),
      decoration: BoxDecoration(
        border: Border(
            bottom:
                BorderSide(color: Gx.divider, width: Gx.borderHairline / 2)),
      ),
      child: Row(
        children: [
          // Símbolo del par con token base dinámico.
          SizedBox(
            width: 60,
            child: Text(entry.symbol,
                style: Gx.dataMono(fontSize: 12, color: Gx.textBase)),
          ),
          // Dirección BUY/SELL: color semántico de dato (señalización interna).
          SizedBox(
            width: 36,
            child: Text(dirLabel,
                style: Gx.dataMono(fontSize: 11, color: dirColor)),
          ),
          // Precio con token secundario dinámico.
          Expanded(
            child: Text(entry.price.toStringAsFixed(5),
                style: Gx.dataMono(
                    fontSize: 12, color: Gx.textBaseSecondary)),
          ),
          // Lotes con token muted dinámico.
          SizedBox(
            width: 30,
            child: Text(entry.lots.toStringAsFixed(1),
                textAlign: TextAlign.right,
                style:
                    Gx.dataMono(fontSize: 11, color: Gx.textBaseMuted)),
          ),
          SizedBox(width: Gx.space4 + Gx.space4 / 2),
          // PnL: color semántico de dato (positivo/negativo).
          SizedBox(
            width: 48,
            child: Text(
              '$pnlSign\$${entry.pnl.abs().toStringAsFixed(0)}',
              textAlign: TextAlign.right,
              style: Gx.dataMono(fontSize: 12, color: pnlColor),
            ),
          ),
        ],
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// _TickerDemo — panel de información sobre la variante ticker horizontal
// Tokens de chrome: panelSurface() (superficie), Gx.textBaseSecondary (descripción),
//   Gx.textBaseMuted (nota).
// ---------------------------------------------------------------------------

// Panel explicativo de la variante ticker horizontal con superficie dinámica.
class _TickerDemo extends StatelessWidget {
  const _TickerDemo();

  @override
  // Muestra el nombre del componente, descripción y referencia visual al ticker.
  Widget build(BuildContext context) {
    return SizedBox(
      width: 260,
      child: panelSurface(
        padding: const EdgeInsets.all(Gx.space12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisSize: MainAxisSize.min,
          children: [
            Text('trade-ticker-bar',
                style: Gx.panelTitle
                    .copyWith(color: Gx.textBaseSecondary)),
            SizedBox(height: Gx.space8),
            Text(
              'Variante horizontal: texto de trades scrolleando de derecha '
              'a izquierda en una línea de 28px. Se ubica en el footer '
              'o status bar. Muestra símbolo, dirección y precio en loop infinito.',
              style: Gx.uiSans(
                  fontSize: 12, color: Gx.textBaseSecondary),
            ),
            SizedBox(height: Gx.space8),
            Text('Ver ticker abajo ↓',
                style: Gx.dataMono(fontSize: 11, color: Gx.textBaseMuted)),
          ],
        ),
      ),
    );
  }
}

// ===========================================================================
// TradeTickerBar — variante horizontal scrolleante
// Tokens de chrome: Gx.navRail (fondo del ticker — token de estructura del riel),
//   Gx.borderBase (borde superior).
// Colores de dato: optimaCyan (BUY), criticalRed (SELL), textBaseMuted (separador)
//   — señalizan la dirección del trade en el ticker.
// NOTA sobre Colors.black / Colors.transparent: NO aparecen en este widget.
//   El ShaderMask con esas constantes está en TradeTapeWidget (explicado allá).
// ===========================================================================

// Línea única de 28px que scrollea de derecha a izquierda indefinidamente.
// Fondo navRail con borde superior borderBase.
class TradeTickerBar extends StatefulWidget {
  const TradeTickerBar({super.key});

  @override
  State<TradeTickerBar> createState() => _TradeTickerBarState();
}

class _TradeTickerBarState extends State<TradeTickerBar>
    with SingleTickerProviderStateMixin {
  // AnimationController infinito — desplaza el texto de derecha a izquierda.
  late AnimationController _scrollCtrl;

  // Texto del ticker: lista fija de trades ciclando.
  static const _tickerText =
      'EURUSD  BUY  1.08432  ▲+124 · '
      'GBPUSD  SELL  1.26891  ▼−78 · '
      'XAUUSD  BUY  2341.20  ▲+441 · '
      'USDJPY  SELL  149.82  ▼−32 · '
      'USDCAD  BUY  1.35124  ▲+88 · '
      'EURUSD  SELL  1.08310  ▼−56 · '
      'XAUUSD  SELL  2339.40  ▼−203 · ';

  @override
  void initState() {
    super.initState();
    // 18s para un ciclo completo — velocidad cómoda de lectura.
    _scrollCtrl = AnimationController(
      vsync: this,
      duration: const Duration(seconds: 18),
    )..repeat();
  }

  @override
  void dispose() {
    _scrollCtrl.dispose();
    super.dispose();
  }

  @override
  // Contenedor de 28px de alto con fondo navRail y borde superior estructural.
  // Desplaza el texto con Transform.translate animado.
  Widget build(BuildContext context) {
    return Container(
      height: 28,
      decoration: BoxDecoration(
        // navRail: token de estructura del riel de navegación — fondo correcto para el ticker.
        color: Gx.navRail,
        // Borde superior estructural global dinámico.
        border: Border(
            top: BorderSide(
                color: Gx.borderBase, width: Gx.borderHairline)),
      ),
      child: ClipRect(
        child: AnimatedBuilder(
          animation: _scrollCtrl,
          builder: (ctx, _) {
            return LayoutBuilder(builder: (ctx, constraints) {
              final w = constraints.maxWidth;
              // El texto se desplaza desde x=w hasta x=-textWidth en cada ciclo.
              return OverflowBox(
                alignment: Alignment.centerLeft,
                maxWidth: double.infinity,
                child: Transform.translate(
                  offset: Offset(
                      w - _scrollCtrl.value * (w + 900), 0),
                  child: _TickerTextRow(text: _tickerText),
                ),
              );
            });
          },
        ),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// _TickerTextRow — texto del ticker con colores alternados por segmento BUY/SELL
// Tokens de chrome: Gx.textBaseMuted (separador y texto neutro).
// Colores de dato: optimaCyan (BUY), criticalRed (SELL) — señalizan dirección.
// ---------------------------------------------------------------------------

// Fila de texto con segmentos coloreados por dirección BUY/SELL.
// Divide el texto por '·' y colorea cada segmento según su dirección.
class _TickerTextRow extends StatelessWidget {
  final String text;
  const _TickerTextRow({required this.text});

  @override
  // Renderiza los segmentos del ticker con su color de dato y separadores neutros.
  Widget build(BuildContext context) {
    // Separa los segmentos por el separador '·'.
    final segments =
        text.split('·').where((s) => s.trim().isNotEmpty).toList();

    return Row(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.center,
      children: segments.map((seg) {
        final isBuy = seg.contains('BUY');
        final isSell = seg.contains('SELL');
        // Color de dato: señaliza la dirección del trade.
        final color = isBuy
            ? Gx.optimaCyan
            : isSell
                ? Gx.criticalRed
                : Gx.textBaseMuted;
        return Row(mainAxisSize: MainAxisSize.min, children: [
          Text(
            seg.trim(),
            style: Gx.dataMono(fontSize: 12, color: color),
          ),
          // Separador con token muted dinámico.
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: Gx.space8),
            child: Text('·',
                style:
                    Gx.dataMono(fontSize: 12, color: Gx.textBaseMuted)),
          ),
        ]);
      }).toList(),
    );
  }
}
