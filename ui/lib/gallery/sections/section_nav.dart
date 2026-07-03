// Sección §5 Navegación — vitrina de componentes de navegación de Drasus Engine.
// Render-only: datos hardcodeados, sin lógica de negocio ni FFI.
// Glow + gradientes + vidrio Apple en todos los componentes (DESIGN.md).

import 'package:flutter/material.dart';
import '../gallery_tokens.dart';
import '../gallery_fx.dart';

// ---------------------------------------------------------------------------
// ZUI Nav Pill — pill flotante con tabs de nivel MACRO/MESO/MICRO.
// ---------------------------------------------------------------------------

// Muestra el pill de cristal flotante del ZUI con el tab activo resaltado
// por un filo neón de 2px. Toca cada opción para cambiar el nivel activo.
// Parámetros: ninguno (estado local _active).
// Tokens: frosted() · Gx.glow() · Gx.space4 · Gx.space8 · Gx.space12 ·
//   Gx.transitionIndigo · Gx.textBase · Gx.textBaseLabel · Gx.rButton.
class ZuiNavPill extends StatefulWidget {
  const ZuiNavPill({super.key});
  @override
  State<ZuiNavPill> createState() => _ZuiNavPillState();
}

class _ZuiNavPillState extends State<ZuiNavPill> {
  // Índice del nivel ZUI activo: 0=MACRO, 1=MESO, 2=MICRO.
  int _active = 0;

  @override
  Widget build(BuildContext context) {
    // Pill de cristal flotante: vidrio Apple con rim-light y glow del tab activo.
    return panelSurface(
      radius: 999,
      padding: const EdgeInsets.all(4),
      glow: Gx.glow(Gx.transitionIndigo, blur: 14, opacity: 0.25),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: ['MACRO', 'MESO', 'MICRO'].asMap().entries.map((e) {
          final isActive = e.key == _active;
          return GestureDetector(
            onTap: () => setState(() => _active = e.key),
            child: AnimatedContainer(
              duration: const Duration(milliseconds: 200),
              padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 8),
              decoration: BoxDecoration(
                borderRadius: BorderRadius.circular(999),
                // Tab activo: filo neón de 2px en índigo.
                border: isActive
                    ? Border.all(color: Gx.transitionIndigo, width: 2)
                    : null,
                boxShadow: isActive
                    ? Gx.glow(Gx.transitionIndigo, blur: 10, opacity: 0.5)
                    : null,
              ),
              child: Text(
                e.value,
                style: Gx.uiSans(
                  fontSize: 12,
                  color: isActive ? Gx.textBase : Gx.textBaseLabel,
                  weight:
                      isActive ? FontWeight.w500 : FontWeight.w400,
                ),
              ),
            ),
          );
        }).toList(),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Breadcrumbs — ruta jerárquica separada por /
// ---------------------------------------------------------------------------

// Muestra la ruta de navegación jerárquica; el último ítem es el nivel actual.
Widget breadcrumbs() {
  final items = ['Flota', 'alpha-01', 'Análisis'];
  return Wrap(
    crossAxisAlignment: WrapCrossAlignment.center,
    children: items.asMap().entries.expand((e) {
      final isLast = e.key == items.length - 1;
      return [
        Text(
          e.value,
          style: Gx.uiSans(
            fontSize: 13,
            color: isLast ? Gx.textBase : Gx.textBaseLabel,
            weight: isLast ? FontWeight.w500 : FontWeight.w400,
          ),
        ),
        if (!isLast)
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 6),
            child: Text('/', style: Gx.uiSans(fontSize: 13, color: Gx.textBaseMuted)),
          ),
      ];
    }).toList(),
  );
}

// ---------------------------------------------------------------------------
// Pagination — paginado con página activa resaltada
// ---------------------------------------------------------------------------

// Muestra controles de paginación; la página activa lleva glow y borde neón.
// ---------------------------------------------------------------------------
// Command Palette — buscador flotante (Cmd+K) con lista de sugerencias
// ---------------------------------------------------------------------------

// Muestra la paleta de comandos: input en vidrio + lista de acciones sugeridas.
class CommandPalette extends StatefulWidget {
  const CommandPalette({super.key});
  @override
  State<CommandPalette> createState() => _CommandPaletteState();
}

class _CommandPaletteState extends State<CommandPalette> {
  final _ctrl = TextEditingController(text: 'exec');
  // Índice del ítem resaltado en la lista de sugerencias.
  int _sel = 0;

  static const _suggestions = [
    'Ejecutar backtest SPX',
    'Incubar estrategia #12',
    'Retirar node-19',
    'Ver autopsia node-07',
  ];

  @override
  void dispose() {
    _ctrl.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return panelSurface(
      padding: const EdgeInsets.all(0),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          // Campo de búsqueda.
          Padding(
            padding: const EdgeInsets.all(10),
            child: Row(children: [
              Icon(Gx.iconChart, size: 14, color: Gx.textBaseLabel),
              const SizedBox(width: 8),
              Expanded(
                child: TextField(
                  controller: _ctrl,
                  style: Gx.dataMono(fontSize: 13, color: Gx.textBase),
                  decoration: InputDecoration.collapsed(
                    hintText: 'Buscar o ejecutar…',
                    hintStyle: Gx.uiSans(fontSize: 13, color: Gx.textBaseMuted),
                  ),
                  cursorColor: Gx.transitionIndigo,
                ),
              ),
              // Atajo de teclado decorativo.
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 3),
                decoration: BoxDecoration(
                  color: Gx.surfaceCard,
                  borderRadius: BorderRadius.circular(Gx.rChip),
                  border: Border.all(color: Gx.borderBase),
                ),
                child: Text('⌘K', style: Gx.dataMono(fontSize: 11, color: Gx.textBaseMuted)),
              ),
            ]),
          ),
          // Separador.
          Divider(color: Gx.borderBase, height: 1),
          // Lista de sugerencias.
          ..._suggestions.asMap().entries.map((e) {
            final isSelected = e.key == _sel;
            return GestureDetector(
              onTap: () => setState(() => _sel = e.key),
              child: AnimatedContainer(
                duration: const Duration(milliseconds: 140),
                padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 9),
                color: isSelected
                    ? Gx.transitionIndigo.withAlpha(25)
                    : Colors.transparent,
                child: Row(children: [
                  Icon(Gx.iconBolt, size: 13,
                      color: isSelected ? Gx.transitionIndigo : Gx.textBaseMuted),
                  const SizedBox(width: 10),
                  Expanded(
                    child: Text(
                      e.value,
                      style: Gx.uiSans(
                        fontSize: 13,
                        color: isSelected ? Gx.textBase : Gx.textBaseSecondary,
                      ),
                    ),
                  ),
                  if (isSelected)
                    Text('↵', style: Gx.dataMono(fontSize: 11, color: Gx.textBaseMuted)),
                ]),
              ),
            );
          }),
          const SizedBox(height: 4),
        ],
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Tree View — árbol de navegación con nodos expandibles
// ---------------------------------------------------------------------------

// Muestra un árbol de estrategias/nodos con expansión y selección interactiva.
// ---------------------------------------------------------------------------
// Anchor / Scrollspy — índice lateral de secciones con sección activa.
// ---------------------------------------------------------------------------

// Lista de anclas decorativa: muestra los títulos de sección del documento
// con la sección activa resaltada mediante un filo neón de 2px y glow.
// La sección "activa" cambia al tocar cada ítem (estado UI local, sin scroll
// real — la galería es una vitrina de diseño, no un documento navegable).
