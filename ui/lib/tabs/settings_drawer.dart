// Drawer de configuración deslizable desde la derecha.
// Muestra: cuenta del usuario, selector de color de énfasis, selector de paleta
// de fondo, e información del sistema. No contiene lógica de negocio: solo
// llama a DrasusThemeState.setAccent / setPalette para cambiar el tema visual.

import 'dart:ui' as ui;
import 'package:flutter/material.dart';
import '../drasus_theme.dart';
import '../gallery/gallery_tokens.dart';

// ---------------------------------------------------------------------------
// Colores de énfasis predefinidos — los 12 swatches del selector.
// ---------------------------------------------------------------------------
const List<Color> _kAccentPresets = [
  Color(0xFF54E8D0), // optimaCyan
  Color(0xFF7CF06A), // reactorGreen
  Color(0xFF9A8CFF), // transitionIndigo (default)
  Color(0xFF56A8FF), // transitionBlue
  Color(0xFFFFC94D), // alertAmber
  Color(0xFFCC2B2B), // accentPrimaryA / criticalCrimson oscuro
  Color(0xFFB4BFCE), // neutro frío
  Color(0xFFFF6B6B), // coral
  Color(0xFFAAFF00), // lima eléctrico
  Color(0xFFFF4FD8), // magenta neón
  Color(0xFFFFD700), // dorado
  Color(0xFF00FFFF), // cian
];

// Nombre corto de cada paleta para mostrarlo bajo el chip.
const Map<DrasusBackgroundPalette, String> _kPaletteNames = {
  DrasusBackgroundPalette.bunker: 'bunker',
  DrasusBackgroundPalette.ash: 'ash',
  DrasusBackgroundPalette.crimson: 'crimson',
  DrasusBackgroundPalette.forest: 'forest',
  DrasusBackgroundPalette.navy: 'navy',
  DrasusBackgroundPalette.void_: 'void',
  DrasusBackgroundPalette.slate: 'slate',
  DrasusBackgroundPalette.paper: 'paper',
};

// ---------------------------------------------------------------------------
// SettingsDrawer — panel lateral de 320px de ancho con vidrio Apple.
// ---------------------------------------------------------------------------

// Se monta como endDrawer del Scaffold en PanelOperativo.
// Al abrirlo, Flutter anima la entrada desde la derecha de forma nativa.
class SettingsDrawer extends StatelessWidget {
  const SettingsDrawer({super.key});

  // build() retorna un Drawer con superficie de vidrio: BackdropFilter +
  // glassFill semitransparente + borde izquierdo tintado de índigo.
  @override
  Widget build(BuildContext context) {
    // Leemos el estado del tema para reaccionar a cambios en tiempo real.
    final theme = DrasusTheme.of(context);
    final accent = theme?.accentColor ?? Gx.transitionIndigo;
    final palette = theme?.backgroundPalette ?? DrasusBackgroundPalette.bunker;

    return Drawer(
      width: 320,
      // Eliminamos el fondo predeterminado de Material para que nuestro
      // BackdropFilter sea el único fondo visible.
      backgroundColor: Colors.transparent,
      child: ClipRect(
        child: BackdropFilter(
          // Desenfoque del contenido detrás del drawer: efecto vidrio Apple.
          filter: ui.ImageFilter.blur(sigmaX: 36, sigmaY: 36),
          child: Container(
            decoration: BoxDecoration(
              color: Gx.glassFill,
              border: const Border(
                // Borde izquierdo tintado de índigo: separa visualmente el drawer.
                left: BorderSide(color: Color(0x380E2AFF), width: 1),
              ),
            ),
            child: SafeArea(
              child: ListView(
                padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 24),
                children: [
                  // ---- Sección CUENTA ----
                  _SectionCuenta(accent: accent),
                  const SizedBox(height: 28),
                  _Divider(),
                  const SizedBox(height: 24),

                  // ---- Sección APARIENCIA ----
                  _SectionApariencia(
                    accent: accent,
                    palette: palette,
                    theme: theme,
                  ),
                  const SizedBox(height: 28),
                  _Divider(),
                  const SizedBox(height: 24),

                  // ---- Sección SUPERFICIE ----
                  _SectionSuperficie(
                    surfaceMode: theme?.surfaceMode ?? DrasusSurfaceMode.glass,
                    theme: theme,
                  ),
                  const SizedBox(height: 28),
                  _Divider(),
                  const SizedBox(height: 24),

                  // ---- Sección SISTEMA ----
                  const _SectionSistema(),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Separador visual entre secciones.
// ---------------------------------------------------------------------------
class _Divider extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Container(height: 1, color: Gx.borderPanel);
  }
}

// ---------------------------------------------------------------------------
// Sección CUENTA — avatar + nombre + email + versión.
// ---------------------------------------------------------------------------
class _SectionCuenta extends StatelessWidget {
  final Color accent;
  const _SectionCuenta({required this.accent});

  @override
  Widget build(BuildContext context) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.center,
      children: [
        // Avatar circular con iniciales del usuario sobre fondo de énfasis al 30%.
        Container(
          width: 40,
          height: 40,
          decoration: BoxDecoration(
            shape: BoxShape.circle,
            color: accent.withOpacity(0.30),
          ),
          alignment: Alignment.center,
          child: Text(
            'FB',
            style: Gx.displayGrotesque(fontSize: 14, color: Colors.white, weight: FontWeight.w600),
          ),
        ),
        const SizedBox(width: 12),
        // Columna con nombre, email y versión en distintas escalas.
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text('Felipe Bravo', style: Gx.displayGrotesque(fontSize: 16, color: Gx.textPrimary)),
              const SizedBox(height: 2),
              Text('fbravo.hz@gmail.com', style: Gx.dataMono(fontSize: 12, color: Gx.textSecondary)),
              const SizedBox(height: 1),
              Text('Drasus Engine v0.1.0-α', style: Gx.dataMono(fontSize: 11, color: Gx.textMuted)),
            ],
          ),
        ),
      ],
    );
  }
}

// ---------------------------------------------------------------------------
// Sección APARIENCIA — swatches de acento + strip de preview + paletas de fondo.
// ---------------------------------------------------------------------------
class _SectionApariencia extends StatelessWidget {
  final Color accent;
  final DrasusBackgroundPalette palette;
  final DrasusThemeState? theme;

  const _SectionApariencia({
    required this.accent,
    required this.palette,
    required this.theme,
  });

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // Etiqueta de subsección: letras espaciadas en mayúsculas.
        Text(
          'COLOR DE ÉNFASIS',
          style: Gx.uiSans(fontSize: 11, color: Gx.textLabel)
              .copyWith(letterSpacing: 1.5),
        ),
        const SizedBox(height: 12),

        // Grid 6 × 2 de swatches: 12 colores predefinidos en cuadros 24×24.
        Wrap(
          spacing: 8,
          runSpacing: 8,
          children: _kAccentPresets.map((color) {
            // Comparamos el entero ARGB32 para verificar si este swatch es el activo.
            // toARGB32() reemplaza .value (no-deprecated en Flutter 3.44+).
            final isSelected = color.toARGB32() == accent.toARGB32();
            return _AccentSwatch(
              color: color,
              isSelected: isSelected,
              onTap: () => theme?.setAccent(color),
            );
          }).toList(),
        ),
        const SizedBox(height: 12),

        // Strip de preview del acento actual: 100% ancho, 4px alto.
        Container(
          height: 4,
          width: double.infinity,
          decoration: BoxDecoration(
            color: accent,
            borderRadius: BorderRadius.circular(2),
          ),
        ),
        const SizedBox(height: 24),

        // Etiqueta de subsección de paletas.
        Text(
          'PALETA DE FONDO',
          style: Gx.uiSans(fontSize: 11, color: Gx.textLabel)
              .copyWith(letterSpacing: 1.5),
        ),
        const SizedBox(height: 12),

        // Grid 4 × 2 de chips de paleta: muestra el deepSpace de cada una.
        Wrap(
          spacing: 8,
          runSpacing: 10,
          children: DrasusBackgroundPalette.values.map((p) {
            final surfaces = _kPalettesPublic[p]!;
            final isSelected = p == palette;
            return _PaletteChip(
              palette: p,
              deepSpaceColor: surfaces.deepSpace,
              label: _kPaletteNames[p] ?? '',
              isSelected: isSelected,
              accentColor: accent,
              onTap: () => theme?.setPalette(p),
            );
          }).toList(),
        ),
      ],
    );
  }
}

// Mapa público de paletas para consultar desde dentro del archivo.
// Duplica la referencia que en drasus_theme.dart es _kPalettes (privado).
const Map<DrasusBackgroundPalette, DrasusSurfacePalette> _kPalettesPublic = {
  DrasusBackgroundPalette.bunker: DrasusSurfacePalette(
    deepSpace: Color(0xFF04050E),
    navRail: Color(0xFF060819),
    panelSolid: Color(0xFF090D1F),
    cardInner: Color(0xFF0C1228),
    surfaceRaised: Color(0xFF111833),
  ),
  DrasusBackgroundPalette.ash: DrasusSurfacePalette(
    deepSpace: Color(0xFF070707),
    navRail: Color(0xFF0A0A0A),
    panelSolid: Color(0xFF0D0D0D),
    cardInner: Color(0xFF111111),
    surfaceRaised: Color(0xFF161616),
  ),
  DrasusBackgroundPalette.crimson: DrasusSurfacePalette(
    deepSpace: Color(0xFF0E0406),
    navRail: Color(0xFF160608),
    panelSolid: Color(0xFF1A080B),
    cardInner: Color(0xFF1E0B0F),
    surfaceRaised: Color(0xFF231215),
  ),
  DrasusBackgroundPalette.forest: DrasusSurfacePalette(
    deepSpace: Color(0xFF040E06),
    navRail: Color(0xFF061508),
    panelSolid: Color(0xFF091A0B),
    cardInner: Color(0xFF0C1E0E),
    surfaceRaised: Color(0xFF112414),
  ),
  DrasusBackgroundPalette.navy: DrasusSurfacePalette(
    deepSpace: Color(0xFF04080E),
    navRail: Color(0xFF060C18),
    panelSolid: Color(0xFF090F1F),
    cardInner: Color(0xFF0C1428),
    surfaceRaised: Color(0xFF111B33),
  ),
  DrasusBackgroundPalette.void_: DrasusSurfacePalette(
    deepSpace: Color(0xFF07040E),
    navRail: Color(0xFF0A0619),
    panelSolid: Color(0xFF0D091F),
    cardInner: Color(0xFF110D28),
    surfaceRaised: Color(0xFF161233),
  ),
  DrasusBackgroundPalette.slate: DrasusSurfacePalette(
    deepSpace: Color(0xFFD8DCE8),
    navRail: Color(0xFFCDD2DF),
    panelSolid: Color(0xFFC2C8D6),
    cardInner: Color(0xFFB7BECD),
    surfaceRaised: Color(0xFFACB4C4),
  ),
  DrasusBackgroundPalette.paper: DrasusSurfacePalette(
    deepSpace: Color(0xFFF0F2F8),
    navRail: Color(0xFFE5E8F0),
    panelSolid: Color(0xFFDADEE8),
    cardInner: Color(0xFFCFD4E0),
    surfaceRaised: Color(0xFFC4CAD8),
  ),
};

// ---------------------------------------------------------------------------
// _AccentSwatch — chip cuadrado 24×24 de un color de énfasis predefinido.
// ---------------------------------------------------------------------------

// Muestra borde blanco 2px + glow cuando está seleccionado.
class _AccentSwatch extends StatelessWidget {
  final Color color;
  final bool isSelected;
  final VoidCallback onTap;

  const _AccentSwatch({
    required this.color,
    required this.isSelected,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: onTap,
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 180),
        width: 24,
        height: 24,
        decoration: BoxDecoration(
          color: color,
          borderRadius: BorderRadius.circular(6),
          border: isSelected
              ? Border.all(color: Colors.white, width: 2)
              : Border.all(color: Colors.transparent, width: 2),
          boxShadow: isSelected ? Gx.glow(color, blur: 12, opacity: 0.7) : null,
        ),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// _PaletteChip — chip rectangular 40×24 que muestra el deepSpace de la paleta.
// ---------------------------------------------------------------------------

// El borde activo usa el color de énfasis; la etiqueta es el nombre de la paleta.
class _PaletteChip extends StatelessWidget {
  final DrasusBackgroundPalette palette;
  final Color deepSpaceColor;
  final String label;
  final bool isSelected;
  final Color accentColor;
  final VoidCallback onTap;

  const _PaletteChip({
    required this.palette,
    required this.deepSpaceColor,
    required this.label,
    required this.isSelected,
    required this.accentColor,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: onTap,
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          AnimatedContainer(
            duration: const Duration(milliseconds: 180),
            width: 40,
            height: 24,
            decoration: BoxDecoration(
              color: deepSpaceColor,
              borderRadius: BorderRadius.circular(4),
              border: isSelected
                  ? Border.all(color: accentColor, width: 1.5)
                  : Border.all(color: Gx.borderPanel, width: 1),
            ),
          ),
          const SizedBox(height: 4),
          Text(
            label,
            style: Gx.dataMono(fontSize: 9, color: Gx.textMuted),
          ),
        ],
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Sección SUPERFICIE — selector de modo global: glass / tint / solid.
// ---------------------------------------------------------------------------
class _SectionSuperficie extends StatelessWidget {
  final DrasusSurfaceMode surfaceMode;
  final DrasusThemeState? theme;

  const _SectionSuperficie({
    required this.surfaceMode,
    required this.theme,
  });

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          'MODO DE SUPERFICIE',
          style: Gx.uiSans(fontSize: 11, color: Gx.textLabel)
              .copyWith(letterSpacing: 1.5),
        ),
        const SizedBox(height: 12),
        ...DrasusSurfaceMode.values.map((mode) {
          final isSelected = mode == surfaceMode;
          return _SurfaceModeOption(
            mode: mode,
            isSelected: isSelected,
            onTap: () => theme?.setSurfaceMode(mode),
          );
        }),
      ],
    );
  }
}

class _SurfaceModeOption extends StatelessWidget {
  final DrasusSurfaceMode mode;
  final bool isSelected;
  final VoidCallback onTap;

  const _SurfaceModeOption({
    required this.mode,
    required this.isSelected,
    required this.onTap,
  });

  String get _label {
    switch (mode) {
      case DrasusSurfaceMode.glass:
        return 'Vidrio Apple (blur + rim)';
      case DrasusSurfaceMode.tint:
        return 'Translúcido (solo color)';
      case DrasusSurfaceMode.solid:
        return 'Sólido oscuro (datos)';
    }
  }

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: onTap,
      child: Container(
        margin: const EdgeInsets.only(bottom: 8),
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
        decoration: BoxDecoration(
          color: isSelected
              ? Gx.transitionIndigo.withAlpha(30)
              : Colors.transparent,
          borderRadius: BorderRadius.circular(8),
          border: Border.all(
            color: isSelected
                ? Gx.transitionIndigo.withAlpha(80)
                : Gx.borderPanel,
          ),
        ),
        child: Row(
          children: [
            Icon(
              isSelected ? Icons.radio_button_checked : Icons.radio_button_off,
              size: 16,
              color: isSelected ? Gx.transitionIndigo : Gx.textMuted,
            ),
            const SizedBox(width: 10),
            Expanded(
              child: Text(
                _label,
                style: Gx.uiSans(
                  fontSize: 13,
                  color: isSelected ? Gx.textPrimary : Gx.textSecondary,
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
// Sección SISTEMA — estado de conexión con el core Rust + versión del build.
// ---------------------------------------------------------------------------
class _SectionSistema extends StatelessWidget {
  const _SectionSistema();

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          'SISTEMA',
          style: Gx.uiSans(fontSize: 11, color: Gx.textLabel)
              .copyWith(letterSpacing: 1.5),
        ),
        const SizedBox(height: 12),
        Row(
          children: [
            Text('Rust core', style: Gx.uiSans(fontSize: 13, color: Gx.textSecondary)),
            const SizedBox(width: 8),
            // Chip de estado: siempre "conectado" por ahora (el bridge está activo).
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
              decoration: BoxDecoration(
                color: Gx.optimaChipBg,
                borderRadius: BorderRadius.circular(Gx.rChip),
                border: Border.all(color: Gx.optimaChipBorder),
              ),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Container(
                    width: 6,
                    height: 6,
                    decoration: const BoxDecoration(
                      shape: BoxShape.circle,
                      color: Gx.reactorGreen,
                    ),
                  ),
                  const SizedBox(width: 5),
                  Text(
                    'conectado',
                    style: Gx.dataMono(fontSize: 11, color: Gx.reactorGreen),
                  ),
                ],
              ),
            ),
          ],
        ),
        const SizedBox(height: 8),
        Text(
          'Build 2026-06-24',
          style: Gx.dataMono(fontSize: 11, color: Gx.textMuted),
        ),
      ],
    );
  }
}
