// Galería de Componentes de Drasus Engine — pestaña "Components".
//
// Cáscara maestro-detalle navegable: panel lateral con búsqueda + lista de
// categorías/entradas, panel de detalle que muestra el componente seleccionado
// bajo demanda. Todo el contenido (builders y helpers) vive en gallery_registry.dart.
//
// Restricción: la clase GalleryTab mantiene su nombre y constructor const
// GalleryTab({super.key}) para no romper operational_panel.dart.

import 'package:flutter/material.dart';
import 'gallery_tokens.dart';
import '../theme/theme_scope.dart';
import 'gallery_registry.dart';

// Widget raíz de la pestaña. Ahora es StatefulWidget para manejar la selección
// de categoría, entrada y el texto del buscador.
class GalleryTab extends StatefulWidget {
  const GalleryTab({super.key});

  @override
  State<GalleryTab> createState() => _GalleryTabState();
}

class _GalleryTabState extends State<GalleryTab> {
  // Índice de la categoría seleccionada (0 = primera categoría al abrir).
  int _selectedCategoryIndex = 0;
  // Índice de la entrada seleccionada dentro de la categoría (-1 = vista panorámica).
  int _selectedEntryIndex = -1;
  // Texto del buscador; vacío = sin filtro.
  String _searchText = '';

  // ---------------------------------------------------------------------------
  // Build raíz
  // ---------------------------------------------------------------------------

  @override
  Widget build(BuildContext context) {
    final theme = ThemeScope.of(context);
    final surfaces = theme?.surfaces;
    final ds = surfaces?.deepSpace ?? Gx.deepSpace;

    // Construye el catálogo completo una vez por build; buildGalleryCatalog
    // solo crea las listas de metadatos — los widgets concretos NO se instancian
    // hasta que el usuario navega a ellos (builder bajo demanda).
    final catalog = buildGalleryCatalog(context);

    return Container(
      color: ds,
      child: Row(
        children: [
          // Panel lateral fijo de 260 px.
          SizedBox(
            width: 260,
            child: _buildSidebar(context, catalog),
          ),
          // Divisor vertical de 1 px.
          VerticalDivider(width: 1, thickness: 1, color: Gx.borderBase),
          // Panel de detalle ocupa el espacio restante.
          Expanded(child: _buildDetail(context, catalog)),
        ],
      ),
    );
  }

  // ---------------------------------------------------------------------------
  // Panel lateral — búsqueda + lista navegable
  // ---------------------------------------------------------------------------

  Widget _buildSidebar(
      BuildContext context, List<GalleryCategory> catalog) {
    // Calcula la lista filtrada según el texto de búsqueda.
    final filtered = _filterCatalog(catalog);

    return Container(
      color: Gx.navRail,
      child: Column(
        children: [
          // Hero compacto del sistema de diseño.
          _buildSidebarHero(),
          // Campo de búsqueda.
          _buildSearchField(),
          const SizedBox(height: 4),
          // Lista de categorías y entradas filtradas.
          Expanded(
            child: ListView.builder(
              padding: const EdgeInsets.only(bottom: 24),
              itemCount: filtered.length,
              itemBuilder: (ctx, i) => _buildSidebarCategory(
                  ctx, filtered[i], catalog),
            ),
          ),
        ],
      ),
    );
  }

  /// Hero compacto: título con ShaderMask y subtítulo en texto pequeño.
  Widget _buildSidebarHero() {
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 20, 16, 12),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          ShaderMask(
            shaderCallback: (rect) =>
                const LinearGradient(colors: Gx.gradCosmic).createShader(rect),
            child: Text('Drasus',
                style: TextStyle(
                    fontFamily: Gx.fontDisplay,
                    fontSize: 22,
                    fontWeight: FontWeight.w500,
                    letterSpacing: -0.4,
                    color: Gx.pureWhite)),
          ),
          Text('Design System',
              style: Gx.microLabel.copyWith(color: Gx.textBaseMuted)),
        ],
      ),
    );
  }

  /// Campo de búsqueda con decoración Gx coherente con el panel lateral.
  Widget _buildSearchField() {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
      child: Container(
        height: 34,
        decoration: BoxDecoration(
          color: Gx.surfaceCard,
          borderRadius: BorderRadius.circular(Gx.rChip),
          border: Border.all(color: Gx.borderBase),
        ),
        child: Row(
          children: [
            const SizedBox(width: 10),
            // Icons.search es nativo de Material — Gx no define un token de búsqueda.
            Icon(Icons.search, size: 14, color: Gx.textBaseMuted),
            const SizedBox(width: 6),
            Expanded(
              child: TextField(
                style: TextStyle(
                    fontFamily: Gx.fontSans, fontSize: 13, color: Gx.textBase),
                decoration: InputDecoration(
                  hintText: 'Buscar componente…',
                  hintStyle: TextStyle(
                      fontFamily: Gx.fontSans,
                      fontSize: 13,
                      color: Gx.textBaseMuted),
                  border: InputBorder.none,
                  isDense: true,
                  contentPadding: EdgeInsets.zero,
                ),
                // Actualiza el estado del buscador y reinicia la selección al filtrar.
                onChanged: (v) => setState(() {
                  _searchText = v;
                  _selectedCategoryIndex = 0;
                  _selectedEntryIndex = -1;
                }),
              ),
            ),
          ],
        ),
      ),
    );
  }

  /// Elemento de la lista lateral para una categoría y sus entradas filtradas.
  Widget _buildSidebarCategory(BuildContext context,
      _FilteredCategory filtered, List<GalleryCategory> fullCatalog) {
    // Índice real de la categoría en el catálogo completo (para la selección).
    final catIndex = fullCatalog.indexOf(filtered.category);

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // Encabezado de categoría — clic → modo panorámica de esa categoría.
        InkWell(
          onTap: () => setState(() {
            _selectedCategoryIndex = catIndex;
            _selectedEntryIndex = -1;
          }),
          child: Container(
            padding:
                const EdgeInsets.symmetric(horizontal: 16, vertical: 7),
            color: _selectedCategoryIndex == catIndex &&
                    _selectedEntryIndex == -1
                ? Gx.surfaceRaisedDynamic
                : Colors.transparent,
            child: Row(children: [
              // Barra de acento cuando la categoría está seleccionada.
              if (_selectedCategoryIndex == catIndex &&
                  _selectedEntryIndex == -1)
                Container(
                    width: 2,
                    height: 14,
                    margin: const EdgeInsets.only(right: 8),
                    decoration: BoxDecoration(
                        gradient: Gx.linear(Gx.gradAurora,
                            begin: Alignment.topCenter,
                            end: Alignment.bottomCenter),
                        boxShadow:
                            Gx.glow(Gx.transitionIndigo, blur: 6, opacity: 0.7))),
              Expanded(
                child: Text(filtered.category.title,
                    overflow: TextOverflow.ellipsis,
                    style: TextStyle(
                        fontFamily: Gx.fontSans,
                        fontSize: 12,
                        fontWeight: FontWeight.w600,
                        letterSpacing: 0.4,
                        color: _selectedCategoryIndex == catIndex
                            ? Gx.textBase
                            : Gx.textBaseLabel)),
              ),
            ]),
          ),
        ),
        // Entradas filtradas de la categoría — indentadas.
        ...filtered.entries.map((entry) {
          final entryIndex =
              filtered.category.entries.indexOf(entry);
          final isSelected = _selectedCategoryIndex == catIndex &&
              _selectedEntryIndex == entryIndex;
          return InkWell(
            onTap: () => setState(() {
              _selectedCategoryIndex = catIndex;
              _selectedEntryIndex = entryIndex;
            }),
            child: Container(
              padding: const EdgeInsets.fromLTRB(28, 5, 16, 5),
              color: isSelected
                  ? Gx.surfaceRaisedDynamic
                  : Colors.transparent,
              child: Row(children: [
                // Punto de acento cuando la entrada está seleccionada.
                if (isSelected)
                  Container(
                      width: 5,
                      height: 5,
                      margin: const EdgeInsets.only(right: 8),
                      decoration: BoxDecoration(
                          shape: BoxShape.circle,
                          color: Gx.optimaCyan,
                          boxShadow: Gx.glow(Gx.optimaCyan,
                              blur: 6, opacity: 0.8)))
                else
                  const SizedBox(width: 13),
                Expanded(
                  child: Text(entry.title,
                      overflow: TextOverflow.ellipsis,
                      style: TextStyle(
                          fontFamily: Gx.fontSans,
                          fontSize: 12,
                          color: isSelected
                              ? Gx.optimaCyan
                              : Gx.textBaseMuted)),
                ),
              ]),
            ),
          );
        }),
      ],
    );
  }

  // ---------------------------------------------------------------------------
  // Panel de detalle — componente individual o panorámica de categoría
  // ---------------------------------------------------------------------------

  Widget _buildDetail(BuildContext context, List<GalleryCategory> catalog) {
    // Guarda de índice fuera de rango (puede ocurrir al filtrar).
    if (_selectedCategoryIndex >= catalog.length) {
      return const SizedBox.shrink();
    }
    final category = catalog[_selectedCategoryIndex];

    // Modo entrada individual: el usuario seleccionó un componente concreto.
    if (_selectedEntryIndex >= 0 &&
        _selectedEntryIndex < category.entries.length) {
      final entry = category.entries[_selectedEntryIndex];
      return SingleChildScrollView(
        padding: const EdgeInsets.all(24),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // Encabezado de sección con barra de acento.
            _sectionHeader(category.title),
            const SizedBox(height: 16),
            // El builder se invoca aquí: construcción bajo demanda.
            entry.fullWidth
                ? entry.builder(context)
                : galleryFrame(entry.title, entry.builder(context)),
            const SizedBox(height: 48),
          ],
        ),
      );
    }

    // Modo panorámica de categoría: muestra todos los componentes en Wrap / Column.
    final hasFullWidth = category.entries.any((e) => e.fullWidth);

    return SingleChildScrollView(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _sectionHeader(category.title),
          const SizedBox(height: 16),
          // fullWidth → Column; el resto → Wrap.
          hasFullWidth
              ? Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: category.entries
                      .map((e) => Padding(
                            padding: const EdgeInsets.only(bottom: 16),
                            child: e.builder(context),
                          ))
                      .toList(),
                )
              : Wrap(
                  spacing: 16,
                  runSpacing: 16,
                  children: category.entries
                      .map((e) =>
                          galleryFrame(e.title, e.builder(context)))
                      .toList(),
                ),
          const SizedBox(height: 48),
        ],
      ),
    );
  }

  // ---------------------------------------------------------------------------
  // Encabezado de sección — barra de acento + título (idéntico al original)
  // ---------------------------------------------------------------------------

  /// Encabezado de sección con barra de acento y gradiente aurora.
  Widget _sectionHeader(String title) {
    return Row(children: [
      Container(
          width: 3,
          height: 20,
          margin: const EdgeInsets.only(right: 10),
          decoration: BoxDecoration(
              gradient: Gx.linear(Gx.gradAurora,
                  begin: Alignment.topCenter, end: Alignment.bottomCenter),
              boxShadow:
                  Gx.glow(Gx.transitionIndigo, blur: 10, opacity: 0.7))),
      Text(title, style: Gx.sectionHeading),
    ]);
  }

  // ---------------------------------------------------------------------------
  // Filtrado del catálogo — case-insensitive por título de entrada
  // ---------------------------------------------------------------------------

  /// Filtra el catálogo por [_searchText] (case-insensitive).
  /// Sin texto → devuelve todas las categorías con todas sus entradas.
  /// Con texto → devuelve solo las categorías que tengan al menos una entrada
  /// cuyo título contenga el texto, mostrando únicamente las que coinciden.
  List<_FilteredCategory> _filterCatalog(List<GalleryCategory> catalog) {
    if (_searchText.isEmpty) {
      // Sin filtro: todas las categorías con todas sus entradas.
      return catalog
          .map((c) => _FilteredCategory(c, c.entries))
          .toList();
    }

    final query = _searchText.toLowerCase();
    final result = <_FilteredCategory>[];
    for (final category in catalog) {
      final matching = category.entries
          .where((e) => e.title.toLowerCase().contains(query))
          .toList();
      if (matching.isNotEmpty) {
        result.add(_FilteredCategory(category, matching));
      }
    }
    return result;
  }
}

// ---------------------------------------------------------------------------
// Modelo interno de filtrado — solo visible en este archivo
// ---------------------------------------------------------------------------

/// Representa una categoría con las entradas que pasaron el filtro de búsqueda.
class _FilteredCategory {
  final GalleryCategory category;
  final List<GalleryEntry> entries;
  const _FilteredCategory(this.category, this.entries);
}
