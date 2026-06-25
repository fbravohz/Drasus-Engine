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
    return frosted(
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
                  color: isActive ? Gx.textPrimary : Gx.textLabel,
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
            color: isLast ? Gx.textPrimary : Gx.textLabel,
            weight: isLast ? FontWeight.w500 : FontWeight.w400,
          ),
        ),
        if (!isLast)
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 6),
            child: Text('/', style: Gx.uiSans(fontSize: 13, color: Gx.textMuted)),
          ),
      ];
    }).toList(),
  );
}

// ---------------------------------------------------------------------------
// Pagination — paginado con página activa resaltada
// ---------------------------------------------------------------------------

// Muestra controles de paginación; la página activa lleva glow y borde neón.
class GlowPagination extends StatefulWidget {
  const GlowPagination({super.key});
  @override
  State<GlowPagination> createState() => _GlowPaginationState();
}

class _GlowPaginationState extends State<GlowPagination> {
  // Página activa (1-indexada, de un total de 5 páginas).
  int _page = 2;

  @override
  Widget build(BuildContext context) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        // Botón previo: flecha izquierda deshabilitada en página 1.
        _navBtn(Gx.iconChevronDown, onTap: _page > 1
            ? () => setState(() => _page--)
            : null, rotate: true),
        const SizedBox(width: 6),
        ...List.generate(5, (i) {
          final p = i + 1;
          final isActive = p == _page;
          return GestureDetector(
            onTap: () => setState(() => _page = p),
            child: AnimatedContainer(
              duration: const Duration(milliseconds: 180),
              width: 28,
              height: 28,
              margin: const EdgeInsets.symmetric(horizontal: 3),
              alignment: Alignment.center,
              decoration: BoxDecoration(
                shape: BoxShape.circle,
                color: isActive ? Gx.transitionIndigo.withAlpha(40) : Colors.transparent,
                border: isActive
                    ? Border.all(color: Gx.transitionIndigo)
                    : Border.all(color: Colors.transparent),
                boxShadow: isActive
                    ? Gx.glow(Gx.transitionIndigo, blur: 10, opacity: 0.5)
                    : null,
              ),
              child: Text(
                '$p',
                style: Gx.dataMono(
                  fontSize: 12,
                  color: isActive ? Gx.transitionIndigo : Gx.textLabel,
                ),
              ),
            ),
          );
        }),
        const SizedBox(width: 6),
        _navBtn(Gx.iconChevronDown, onTap: _page < 5
            ? () => setState(() => _page++)
            : null),
      ],
    );
  }

  // Botón de flecha con glow al hover, opcionalmente rotado 180°.
  Widget _navBtn(IconData icon, {VoidCallback? onTap, bool rotate = false}) {
    return GestureDetector(
      onTap: onTap,
      child: Container(
        width: 28,
        height: 28,
        alignment: Alignment.center,
        decoration: BoxDecoration(
          shape: BoxShape.circle,
          color: Gx.surfaceFill,
          border: Border.all(color: Gx.borderPanel),
        ),
        child: Transform.rotate(
          angle: rotate ? 3.14159 : 0,
          child: Icon(icon, size: 14,
              color: onTap != null ? Gx.textSecondary : Gx.textMuted),
        ),
      ),
    );
  }
}

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
    return frosted(
      padding: EdgeInsets.zero,
      glow: Gx.glow(Gx.transitionIndigo, blur: 20, opacity: 0.3),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          // Campo de búsqueda.
          Padding(
            padding: const EdgeInsets.all(10),
            child: Row(children: [
              Icon(Gx.iconChart, size: 14, color: Gx.textLabel),
              const SizedBox(width: 8),
              Expanded(
                child: TextField(
                  controller: _ctrl,
                  style: Gx.dataMono(fontSize: 13),
                  decoration: InputDecoration.collapsed(
                    hintText: 'Buscar o ejecutar…',
                    hintStyle: Gx.uiSans(fontSize: 13, color: Gx.textMuted),
                  ),
                  cursorColor: Gx.transitionIndigo,
                ),
              ),
              // Atajo de teclado decorativo.
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 3),
                decoration: BoxDecoration(
                  color: Gx.surfaceCard,
                  borderRadius: BorderRadius.circular(4),
                  border: Border.all(color: Gx.borderPanel),
                ),
                child: Text('⌘K', style: Gx.dataMono(fontSize: 11, color: Gx.textMuted)),
              ),
            ]),
          ),
          // Separador.
          const Divider(color: Gx.divider, height: 1),
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
                      color: isSelected ? Gx.transitionIndigo : Gx.textMuted),
                  const SizedBox(width: 10),
                  Expanded(
                    child: Text(
                      e.value,
                      style: Gx.uiSans(
                        fontSize: 13,
                        color: isSelected ? Gx.textPrimary : Gx.textSecondary,
                      ),
                    ),
                  ),
                  if (isSelected)
                    Text('↵', style: Gx.dataMono(fontSize: 11, color: Gx.textMuted)),
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
class GlowTreeView extends StatefulWidget {
  const GlowTreeView({super.key});
  @override
  State<GlowTreeView> createState() => _GlowTreeViewState();
}

class _GlowTreeViewState extends State<GlowTreeView> {
  // Mapa de índice a estado abierto/cerrado.
  final _expanded = {0: true, 1: false};
  // Índice del nodo seleccionado.
  int _selected = 2;

  static const _tree = [
    {'label': 'Flota activa', 'children': ['node-07', 'node-12']},
    {'label': 'Archivados', 'children': ['node-19', 'node-21']},
  ];

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: _tree.asMap().entries.map((root) {
        final isOpen = _expanded[root.key] ?? false;
        final children = (root.value['children'] as List).cast<String>();
        return Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // Nodo raíz: click abre/cierra.
            GestureDetector(
              onTap: () =>
                  setState(() => _expanded[root.key] = !isOpen),
              child: Padding(
                padding: const EdgeInsets.symmetric(vertical: 5, horizontal: 8),
                child: Row(children: [
                  AnimatedRotation(
                    turns: isOpen ? 0.25 : 0,
                    duration: const Duration(milliseconds: 180),
                    child: Icon(Gx.iconChevronDown,
                        size: 12, color: Gx.textSecondary),
                  ),
                  const SizedBox(width: 6),
                  Text(root.value['label'] as String,
                      style: Gx.uiSans(fontSize: 13, color: Gx.textSecondary,
                          weight: FontWeight.w500)),
                ]),
              ),
            ),
            // Hijos: visibles solo si el nodo raíz está abierto.
            AnimatedSize(
              duration: const Duration(milliseconds: 200),
              curve: Curves.easeOut,
              child: isOpen
                  ? Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: children.asMap().entries.map((child) {
                        // Índice global único para selección.
                        final idx = root.key * 10 + child.key;
                        final isSel = idx == _selected;
                        return GestureDetector(
                          onTap: () => setState(() => _selected = idx),
                          child: AnimatedContainer(
                            duration: const Duration(milliseconds: 140),
                            padding: const EdgeInsets.symmetric(
                                vertical: 5, horizontal: 8),
                            margin: const EdgeInsets.only(left: 20),
                            decoration: BoxDecoration(
                              color: isSel
                                  ? Gx.transitionIndigo.withAlpha(20)
                                  : Colors.transparent,
                              borderRadius: BorderRadius.circular(6),
                              border: isSel
                                  ? Border(
                                      left: BorderSide(
                                          color: Gx.transitionIndigo,
                                          width: 2))
                                  : null,
                            ),
                            child: Text(
                              child.value,
                              style: Gx.dataMono(
                                fontSize: 12,
                                color: isSel
                                    ? Gx.transitionIndigo
                                    : Gx.textLabel,
                              ),
                            ),
                          ),
                        );
                      }).toList(),
                    )
                  : const SizedBox.shrink(),
            ),
          ],
        );
      }).toList(),
    );
  }
}

// ---------------------------------------------------------------------------
// Anchor / Scrollspy — índice lateral de secciones con sección activa.
// ---------------------------------------------------------------------------

// Lista de anclas decorativa: muestra los títulos de sección del documento
// con la sección activa resaltada mediante un filo neón de 2px y glow.
// La sección "activa" cambia al tocar cada ítem (estado UI local, sin scroll
// real — la galería es una vitrina de diseño, no un documento navegable).
class GlowScrollspy extends StatefulWidget {
  const GlowScrollspy({super.key});

  @override
  State<GlowScrollspy> createState() => _GlowScrollspyState();
}

class _GlowScrollspyState extends State<GlowScrollspy> {
  // Índice de la sección activa simulada (0-indexado).
  int _active = 1;

  // Secciones del documento — datos hardcodeados, render-only.
  // Los nombres reflejan las secciones de un informe de estrategia Drasus,
  // distintos de los encabezados de la galería para no colisionar con
  // los find.text del smoke test.
  static const _sections = [
    'Introducción',
    'Parámetros',
    'Régimen',
    'Backtest',
    'Riesgo',
    'Retiro',
    'Anexos',
  ];

  @override
  Widget build(BuildContext context) {
    // Panel de cristal que contiene la lista de anclas.
    return Container(
      padding: const EdgeInsets.symmetric(vertical: 8, horizontal: 0),
      decoration: BoxDecoration(
        color: Gx.surfaceFill,
        borderRadius: BorderRadius.circular(Gx.rPanel),
        border: Border.all(color: Gx.borderPanel),
        boxShadow: Gx.glow(Gx.transitionIndigo, blur: 14, opacity: 0.15),
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: _sections.asMap().entries.map((e) {
          final isActive = e.key == _active;
          return GestureDetector(
            onTap: () => setState(() => _active = e.key),
            child: AnimatedContainer(
              duration: const Duration(milliseconds: 180),
              curve: Curves.easeOut,
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 9),
              decoration: BoxDecoration(
                // Fondo tenue en la sección activa.
                color: isActive
                    ? Gx.transitionIndigo.withAlpha(20)
                    : Colors.transparent,
                // Filo neón de 2px en el lado izquierdo (indicador de posición).
                border: Border(
                  left: BorderSide(
                    color: isActive ? Gx.transitionIndigo : Colors.transparent,
                    width: 2,
                  ),
                ),
              ),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  // Punto de estado: lleno en la sección activa, hueco en las demás.
                  AnimatedContainer(
                    duration: const Duration(milliseconds: 200),
                    width: 5,
                    height: 5,
                    margin: const EdgeInsets.only(right: 8),
                    decoration: BoxDecoration(
                      shape: BoxShape.circle,
                      color: isActive ? Gx.transitionIndigo : Gx.textMuted,
                      boxShadow: isActive
                          ? Gx.glow(Gx.transitionIndigo, blur: 6, opacity: 0.8)
                          : null,
                    ),
                  ),
                  // Etiqueta de la sección.
                  Text(
                    e.value,
                    style: Gx.uiSans(
                      fontSize: 12,
                      color: isActive ? Gx.textPrimary : Gx.textLabel,
                      weight: isActive ? FontWeight.w500 : FontWeight.w400,
                    ),
                  ),
                ],
              ),
            ),
          );
        }).toList(),
      ),
    );
  }
}
