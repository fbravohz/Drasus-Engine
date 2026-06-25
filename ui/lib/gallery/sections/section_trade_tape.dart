// §10 Data-viz — trade-tape y trade-ticker-bar.
// Cinta de órdenes en vivo (vertical) y ticker horizontal.
// Todos los datos son sintéticos — sin Rust ni FFI.
// Usa Timer.periodic + AnimatedList para el scroll de nuevas entradas.
// El ticker usa AnimationController infinito para el desplazamiento lateral.

import 'dart:async';
import 'dart:math';
import 'package:flutter/material.dart';
import '../gallery_tokens.dart';

// Modelo de una entrada de trade sintética.
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

// Genera una entrada sintética aleatoria con semilla de tiempo.
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
  final pnl = (rnd.nextDouble() - 0.4) * 300; // ligeramente positivo
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
// ===========================================================================

// Widget de sección exportable para gallery_tab.dart.
// Muestra el header con chip LIVE parpadeante, el widget de cinta vertical
// y el ticker horizontal.
class TradeTapeSection extends StatelessWidget {
  const TradeTapeSection({super.key});

  @override
  Widget build(BuildContext context) {
    // Columna con header y las dos variantes del trade tape.
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [
        // Header de sección con chip LIVE parpadeante.
        _TapeSectionHeader(),
        const SizedBox(height: 16),
        // Las dos variantes lado a lado en un Wrap.
        Wrap(
          spacing: 16,
          runSpacing: 16,
          children: const [
            // Cinta vertical (300px × 350px).
            TradeTapeWidget(),
            // Información sobre la variante ticker.
            _TickerDemo(),
          ],
        ),
        const SizedBox(height: 12),
        // Ticker horizontal en ancho completo.
        const TradeTickerBar(),
      ],
    );
  }
}

// Header con ícono de relámpago + título + chip "LIVE" parpadeante.
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
  Widget build(BuildContext context) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Icon(Icons.flash_on, size: 14, color: Gx.reactorGreen, shadows: Gx.textGlow(Gx.reactorGreen)),
        const SizedBox(width: 6),
        Text('Cinta de Órdenes', style: Gx.panelTitle),
        const SizedBox(width: 10),
        // Chip LIVE que parpadea en reactorGreen.
        AnimatedBuilder(
          animation: _blinkCtrl,
          builder: (_, __) => Container(
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
            decoration: BoxDecoration(
              color: Gx.reactorGreen.withOpacity(0.12 + _blinkCtrl.value * 0.10),
              border: Border.all(
                color: Gx.reactorGreen.withOpacity(0.50 + _blinkCtrl.value * 0.30),
              ),
              borderRadius: BorderRadius.circular(999),
              boxShadow: Gx.glow(Gx.reactorGreen,
                  blur: 8, opacity: 0.20 + _blinkCtrl.value * 0.25),
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
      _listKey.currentState?.insertItem(0,
          duration: const Duration(milliseconds: 350));
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
  Widget build(BuildContext context) {
    // Contenedor con dimensiones fijas (300×350px) y fondo panelSolid.
    return SizedBox(
      width: 300,
      height: 350,
      child: Container(
        decoration: BoxDecoration(
          color: Gx.surfacePanel,
          border: Border.all(color: Gx.borderPanel),
          borderRadius: BorderRadius.circular(Gx.rPanel),
        ),
        child: ClipRRect(
          borderRadius: BorderRadius.circular(Gx.rPanel),
          child: Column(
            children: [
              // Header interno con conteo de trades.
              _TapeHeader(count: _entries.length),
              // Lista con fade en los bordes (ShaderMask).
              Expanded(
                child: ShaderMask(
                  blendMode: BlendMode.dstIn,
                  shaderCallback: (bounds) => LinearGradient(
                    begin: Alignment.topCenter,
                    end: Alignment.bottomCenter,
                    colors: [
                      Colors.transparent,        // fade superior 32px
                      Colors.black,
                      Colors.black,
                      Colors.transparent,        // fade inferior 32px
                    ],
                    stops: [
                      0.0,
                      32 / bounds.height,
                      1.0 - 32 / bounds.height,
                      1.0,
                    ],
                  ).createShader(bounds),
                  child: AnimatedList(
                    key: _listKey,
                    initialItemCount: _entries.length,
                    itemBuilder: (ctx, index, animation) {
                      if (index >= _entries.length) return const SizedBox.shrink();
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
      ),
    );
  }
}

// Header interno de la cinta con "LIVE TRADE TAPE" + conteo.
class _TapeHeader extends StatelessWidget {
  final int count;
  const _TapeHeader({required this.count});

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
      decoration: const BoxDecoration(
        border: Border(bottom: BorderSide(color: Gx.borderPanel)),
      ),
      child: Row(
        children: [
          Text('LIVE TRADE TAPE',
              style: Gx.dataMono(fontSize: 11, color: Gx.textSecondary)),
          const Spacer(),
          Text('${count > 40 ? "5.737" : count} trades',
              style: Gx.dataMono(fontSize: 10, color: Gx.textMuted)),
        ],
      ),
    );
  }
}

// Fila individual de una entrada de trade con animación de slide entrada.
class _TradeEntryRow extends StatelessWidget {
  final _TradeEntry entry;
  final Animation<double> animation;

  const _TradeEntryRow({
    required this.entry,
    required this.animation,
  });

  @override
  Widget build(BuildContext context) {
    // Slide desde arriba con SizeTransition + FadeTransition.
    return SizeTransition(
      sizeFactor: CurvedAnimation(parent: animation, curve: Curves.easeOut),
      child: FadeTransition(
        opacity: animation,
        child: _TradeRowContent(entry: entry),
      ),
    );
  }
}

// Contenido de la fila: símbolo, dirección, precio, lotes, PnL.
class _TradeRowContent extends StatelessWidget {
  final _TradeEntry entry;
  const _TradeRowContent({required this.entry});

  @override
  Widget build(BuildContext context) {
    final dirColor = entry.isBuy ? Gx.optimaCyan : Gx.criticalRed;
    final pnlColor = entry.pnl >= 0 ? Gx.optimaCyan : Gx.criticalRed;
    final pnlSign = entry.pnl >= 0 ? '+' : '';
    final dirLabel = entry.isBuy ? 'BUY' : 'SELL';

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
      decoration: const BoxDecoration(
        border: Border(bottom: BorderSide(color: Gx.divider, width: 0.5)),
      ),
      child: Row(
        children: [
          // Símbolo del par.
          SizedBox(
            width: 60,
            child: Text(entry.symbol,
                style: Gx.dataMono(fontSize: 12, color: Gx.textPrimary)),
          ),
          // Dirección BUY/SELL.
          SizedBox(
            width: 36,
            child: Text(dirLabel,
                style: Gx.dataMono(fontSize: 11, color: dirColor)),
          ),
          // Precio.
          Expanded(
            child: Text(entry.price.toStringAsFixed(5),
                style: Gx.dataMono(fontSize: 12, color: Gx.textSecondary)),
          ),
          // Lotes.
          SizedBox(
            width: 30,
            child: Text(entry.lots.toStringAsFixed(1),
                textAlign: TextAlign.right,
                style: Gx.dataMono(fontSize: 11, color: Gx.textMuted)),
          ),
          const SizedBox(width: 6),
          // PnL.
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

// Demo panel que explica la variante ticker horizontal.
class _TickerDemo extends StatelessWidget {
  const _TickerDemo();

  @override
  Widget build(BuildContext context) {
    return Container(
      width: 260,
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: Gx.surfacePanel,
        border: Border.all(color: Gx.borderPanel),
        borderRadius: BorderRadius.circular(Gx.rPanel),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          Text('trade-ticker-bar', style: Gx.panelTitle),
          const SizedBox(height: 8),
          Text(
            'Variante horizontal: texto de trades scrolleando de derecha '
            'a izquierda en una línea de 28px. Se ubica en el footer '
            'o status bar. Muestra símbolo, dirección y precio en loop infinito.',
            style: Gx.uiSans(fontSize: 12, color: Gx.textSecondary),
          ),
          const SizedBox(height: 8),
          Text('Ver ticker abajo ↓',
              style: Gx.dataMono(fontSize: 11, color: Gx.textMuted)),
        ],
      ),
    );
  }
}

// ===========================================================================
// TradeTickerBar — variante horizontal scrolleante
// ===========================================================================

// Línea única de 28px que scrollea de derecha a izquierda indefinidamente.
// Texto: trades BUY/SELL alternando optimaCyan y criticalRed.
// Fondo navRail con borde superior borderPanel.
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
  Widget build(BuildContext context) {
    return Container(
      height: 28,
      decoration: const BoxDecoration(
        color: Gx.navRail,
        border: Border(top: BorderSide(color: Gx.borderPanel, width: 1)),
      ),
      child: ClipRect(
        child: AnimatedBuilder(
          animation: _scrollCtrl,
          builder: (ctx, _) {
            return LayoutBuilder(builder: (ctx, constraints) {
              final w = constraints.maxWidth;
              // El texto se desplaza desde x=w hasta x=-textWidth en cada ciclo.
              // Usamos Transform.translate para el desplazamiento.
              return OverflowBox(
                alignment: Alignment.centerLeft,
                maxWidth: double.infinity,
                child: Transform.translate(
                  offset: Offset(w - _scrollCtrl.value * (w + 900), 0),
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

// Texto del ticker con colores alternados por segmento BUY/SELL.
// Divide el texto por '·' y colorea cada segmento según su dirección.
class _TickerTextRow extends StatelessWidget {
  final String text;
  const _TickerTextRow({required this.text});

  @override
  Widget build(BuildContext context) {
    // Separa los segmentos por el separador '·'.
    final segments = text.split('·').where((s) => s.trim().isNotEmpty).toList();

    return Row(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.center,
      children: segments.map((seg) {
        final isBuy = seg.contains('BUY');
        final isSell = seg.contains('SELL');
        final color = isBuy
            ? Gx.optimaCyan
            : isSell
                ? Gx.criticalRed
                : Gx.textMuted;
        return Row(mainAxisSize: MainAxisSize.min, children: [
          Text(
            seg.trim(),
            style: Gx.dataMono(fontSize: 12, color: color),
          ),
          // Separador entre segmentos.
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 8),
            child: Text('·',
                style: Gx.dataMono(fontSize: 12, color: Gx.textMuted)),
          ),
        ]);
      }).toList(),
    );
  }
}
