// Drawer de configuración deslizable desde la derecha.
// Muestra: cuenta del usuario, selector de color de énfasis, selector de paleta
// de fondo, e información del sistema. No contiene lógica de negocio: solo
// llama a ThemeState.setAccent / setPalette para cambiar el tema visual.

import 'dart:ui' as ui;
import 'package:flutter/material.dart';
import '../theme/theme_scope.dart';
import '../gallery/gallery_tokens.dart';
// El alias del namespace de componentes es `uic` (no `ui`) porque este archivo
// ya importa `dart:ui as ui`. Ver caveat en ADR-0138 / ui/COMPONENTS.md.
import '../components/components.dart' as uic;

// ---------------------------------------------------------------------------
// Swatches curados para el selector de color de énfasis (12 opciones).
// Todos respetan el espectro de vitalidad del sistema de tokens.
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

// ---------------------------------------------------------------------------
// Swatches curados para el selector de color de texto base (9 opciones).
// Incluye variantes claras (para fondos oscuros) y oscuras (para fondos claros).
// ---------------------------------------------------------------------------
const List<Color> _kTextPresets = [
  Color(0xFFE6ECF8), // textPrimary — claro standard
  Color(0xFFFFFFFF), // blanco puro
  Color(0xFFAEBBD6), // textSecondary — azul-grisáceo claro
  Color(0xFF8492B0), // textLabel — gris-azul suave
  Color(0xFF54E8D0), // optimaCyan (texto tintado)
  Color(0xFF9A8CFF), // transitionIndigo (texto tintado)
  Color(0xFF1A1E2E), // oscuro estándar (para paletas claras)
  Color(0xFF080A18), // deepSpace (casi-negro)
  Color(0xFF2A2E40), // gris-azul oscuro
];

// ---------------------------------------------------------------------------
// Swatches curados para el selector de color de fondo de componentes (12 opciones).
// Tonos oscuros y sofisticados apropiados para componentes financieros.
// ---------------------------------------------------------------------------
const List<Color> _kComponentBgPresets = [
  Color(0xFF1A1A2E), // Midnight blue (default)
  Color(0xFF16213E), // Deep navy
  Color(0xFF0F3460), // Ocean blue
  Color(0xFF1B1B2F), // Dark indigo
  Color(0xFF1A1A1A), // Near black
  Color(0xFF2D2D3F), // Dark slate purple
  Color(0xFF1E272E), // Dark teal gray
  Color(0xFF2C3E50), // Wet asphalt
  Color(0xFF1C1C2E), // Dark violet gray
  Color(0xFF0D1B2A), // Deep ocean
  Color(0xFF2B2D42), // Gunmetal
  Color(0xFF1F1F3A), // Deep purple gray
];

// Nombre corto de cada paleta para mostrarlo bajo el chip.
const Map<BackgroundPalette, String> _kPaletteNames = {
  BackgroundPalette.bunker: 'bunker',
  BackgroundPalette.ash: 'ash',
  BackgroundPalette.crimson: 'crimson',
  BackgroundPalette.forest: 'forest',
  BackgroundPalette.navy: 'navy',
  BackgroundPalette.void_: 'void',
  BackgroundPalette.slate: 'slate',
  BackgroundPalette.paper: 'paper',
};

// ---------------------------------------------------------------------------
// SettingsDrawer — panel lateral de 320px de ancho con vidrio Apple.
// ---------------------------------------------------------------------------

// Se monta como endDrawer del Scaffold en OperationalPanel.
// Al abrirlo, Flutter anima la entrada desde la derecha de forma nativa.
class SettingsDrawer extends StatelessWidget {
  const SettingsDrawer({super.key});

  // build() retorna un Drawer con superficie de vidrio: BackdropFilter +
  // glassFill semitransparente + borde izquierdo tintado de índigo.
  @override
  Widget build(BuildContext context) {
    // Leemos el estado del tema para reaccionar a cambios en tiempo real.
    final theme = ThemeScope.of(context);
    final accent = theme?.accentColor ?? Gx.transitionIndigo;
    final palette = theme?.backgroundPalette ?? BackgroundPalette.bunker;
    // Color de texto base efectivo (override manual o auto por paleta).
    final textColor = theme?.effectiveTextColor ?? ThemeState.globalTextColor;
    // Color de fondo de componentes activo (tinte de glass / fondo de solid).
    final componentBgColor = theme?.componentBgColor ?? ThemeState.globalComponentBgColor;
    // Modo automático de paleta (true = acento/texto/componente se auto-seleccionan).
    final isAutoPalette = theme?.isAutoPalette ?? true;

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
              color: Gx.surfaceFill,
              // Borde izquierdo tintado con el énfasis dinámico: separa el drawer
              // y reacciona al cambio de color de énfasis. No es const para
              // reconstruirse cuando el énfasis cambia.
              border: Border(
                left: BorderSide(color: Gx.borderBase, width: Gx.borderHairline),
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
                    textColor: textColor,
                    componentBgColor: componentBgColor,
                    theme: theme,
                    isAutoPalette: isAutoPalette,
                  ),
                  const SizedBox(height: 28),
                  _Divider(),
                  const SizedBox(height: 24),

                  // ---- Sección SUPERFICIE ----
                  _SectionSuperficie(
                    surfaceMode: theme?.surfaceMode ?? SurfaceMode.glass,
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
              Text('Felipe Bravo', style: Gx.displayGrotesque(fontSize: 16, color: Gx.textBase)),
              const SizedBox(height: 2),
              Text('fbravo.hz@gmail.com', style: Gx.dataMono(fontSize: 12, color: Gx.textBaseSecondary)),
              const SizedBox(height: 1),
              Text('Drasus Engine v0.1.0-α', style: Gx.dataMono(fontSize: 11, color: Gx.textBaseMuted)),
            ],
          ),
        ),
      ],
    );
  }
}

// ---------------------------------------------------------------------------
// Sección APARIENCIA — selector de énfasis (híbrido), color de fuente,
// y strip de paletas de fondo.
// ---------------------------------------------------------------------------
class _SectionApariencia extends StatelessWidget {
  final Color accent;
  final BackgroundPalette palette;
  final Color textColor;
  final Color componentBgColor;
  final ThemeState? theme;
  final bool isAutoPalette;

  const _SectionApariencia({
    required this.accent,
    required this.palette,
    required this.textColor,
    required this.componentBgColor,
    required this.theme,
    required this.isAutoPalette,
  });

  @override
  // Muestra toggle único de paleta automática + selectores manuales condicionales.
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // ---- Toggle ÚNICO de paleta automática ----
        Row(
          children: [
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    'Paleta automática',
                    style: Gx.uiSans(fontSize: 13, color: Gx.textBase),
                  ),
                  const SizedBox(height: 2),
                  Text(
                    isAutoPalette
                        ? 'Acento / texto / fondo según paleta'
                        : 'Controles manuales activos',
                    style: Gx.uiSans(fontSize: 11, color: Gx.textBaseMuted),
                  ),
                ],
              ),
            ),
            _SimpleToggle(
              value: isAutoPalette,
              onToggle: () => theme?.setAutoPalette(!isAutoPalette),
            ),
          ],
        ),
        const SizedBox(height: 24),

        // ---- Selectores manuales (ocultos cuando auto-paleta está activa) ----
        if (!isAutoPalette) ...[
          // Aceler: selector de énfasis.
          Text(
            'COLOR DE ÉNFASIS',
            style: Gx.uiSans(fontSize: 11, color: Gx.textBaseLabel)
                .copyWith(letterSpacing: 1.5),
          ),
          const SizedBox(height: 12),
          uic.ColorPicker(
            swatches: _kAccentPresets,
            value: accent,
            onChanged: (color) => theme?.setAccent(color),
          ),
          const SizedBox(height: 24),

          // Color de fuente: selector directo (sin toggle interno).
          Text(
            'COLOR DE FUENTE',
            style: Gx.uiSans(fontSize: 11, color: Gx.textBaseLabel)
                .copyWith(letterSpacing: 1.5),
          ),
          const SizedBox(height: 12),
          uic.ColorPicker(
            swatches: _kTextPresets,
            value: textColor,
            onChanged: (color) => theme?.setTextColor(color),
          ),
          const SizedBox(height: 24),

          // Fondo de componentes.
          Text(
            'FONDO DE COMPONENTES',
            style: Gx.uiSans(fontSize: 11, color: Gx.textBaseLabel)
                .copyWith(letterSpacing: 1.5),
          ),
          const SizedBox(height: 12),
          uic.ColorPicker(
            swatches: _kComponentBgPresets,
            value: componentBgColor,
            onChanged: (color) => theme?.setComponentBgColor(color),
          ),
          const SizedBox(height: 24),
        ],

        // ---- Paletas de fondo (siempre visibles) ----
        Text(
          'PALETA DE FONDO',
          style: Gx.uiSans(fontSize: 11, color: Gx.textBaseLabel)
              .copyWith(letterSpacing: 1.5),
        ),
        const SizedBox(height: 12),
        Wrap(
          spacing: 8,
          runSpacing: 10,
          children: BackgroundPalette.values.map((p) {
            final surfaces = kPalettes[p]!;
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

// ---------------------------------------------------------------------------
// _SimpleToggle — switch visual controlado desde fuera.
// ---------------------------------------------------------------------------

// Toggle de palanca reactivo (no mantiene estado propio).
// Llama a onToggle cuando el usuario lo toca; el padre decide el estado.
// Usa reactorGreen como color activo (alineado con el sistema de tokens).
class _SimpleToggle extends StatelessWidget {
  final bool value;
  final VoidCallback onToggle;

  const _SimpleToggle({
    required this.value,
    required this.onToggle,
  });

  @override
  // Switch de palanca sin estado propio; animado con AnimatedContainer.
  // El color activo es Gx.reactorGreen (token canónico de "encendido").
  Widget build(BuildContext context) {
    const activeColor = Gx.reactorGreen;
    return GestureDetector(
      onTap: onToggle,
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 200),
        width: 44,
        height: 24,
        padding: const EdgeInsets.all(3),
        decoration: BoxDecoration(
          gradient: value
              ? LinearGradient(colors: [
                  activeColor.withOpacity(0.4),
                  activeColor.withOpacity(0.15),
                ])
              : null,
          color: value ? null : Gx.gaugeTrack,
          borderRadius: BorderRadius.circular(999),
          border: Border.all(
            color: value ? activeColor : Gx.borderPanel,
          ),
          boxShadow: value ? Gx.glow(activeColor, blur: 10, opacity: 0.4) : null,
        ),
        child: AnimatedAlign(
          duration: const Duration(milliseconds: 200),
          alignment: value ? Alignment.centerRight : Alignment.centerLeft,
          child: Container(
            width: 16,
            height: 16,
            decoration: BoxDecoration(
              shape: BoxShape.circle,
              color: value ? activeColor : Gx.textBaseMuted,
            ),
          ),
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
  final BackgroundPalette palette;
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
            style: Gx.dataMono(fontSize: 9, color: Gx.textBaseMuted),
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
  final SurfaceMode surfaceMode;
  final ThemeState? theme;

  const _SectionSuperficie({
    required this.surfaceMode,
    required this.theme,
  });

  @override
  // Itera kSurfaceModeRegistry (no SurfaceMode.values directamente)
  // para que los modos nuevos aparezcan solos al añadirse al registro.
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          'MODO DE SUPERFICIE',
          style: Gx.uiSans(fontSize: 11, color: Gx.textBaseLabel)
              .copyWith(letterSpacing: 1.5),
        ),
        const SizedBox(height: 12),
        // La iteración es sobre el registro: kSurfaceModeRegistry.entries.
        // Al añadir un 5º modo al registro, aparece aquí automáticamente.
        ...kSurfaceModeRegistry.entries.map((entry) {
          final mode = entry.key;
          final recipe = entry.value;
          final isSelected = mode == surfaceMode;
          return _SurfaceModeOption(
            mode: mode,
            recipe: recipe,
            isSelected: isSelected,
            onTap: () => theme?.setSurfaceMode(mode),
          );
        }),
      ],
    );
  }
}

// Opción de modo de superficie.
// Recibe la recipe del registro kSurfaceModeRegistry para mostrar
// etiqueta y descripción sin switch hardcodeado: N-extensible por diseño.
class _SurfaceModeOption extends StatelessWidget {
  final SurfaceMode mode;
  final SurfaceModeRecipe recipe;
  final bool isSelected;
  final VoidCallback onTap;

  const _SurfaceModeOption({
    required this.mode,
    required this.recipe,
    required this.isSelected,
    required this.onTap,
  });

  @override
  // Fila con radio + label + descripción breve; borde de énfasis cuando está seleccionado.
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
              color: isSelected ? Gx.transitionIndigo : Gx.textBaseMuted,
            ),
            const SizedBox(width: 10),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  // Label principal del modo (viene del registro, no hardcodeado).
                  Text(
                    recipe.label,
                    style: Gx.uiSans(
                      fontSize: 13,
                      color: isSelected ? Gx.textBase : Gx.textBaseSecondary,
                    ),
                  ),
                  // Descripción breve del efecto visual.
                  Text(
                    recipe.description,
                    style: Gx.uiSans(fontSize: 11, color: Gx.textBaseMuted),
                  ),
                ],
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
          style: Gx.uiSans(fontSize: 11, color: Gx.textBaseLabel)
              .copyWith(letterSpacing: 1.5),
        ),
        const SizedBox(height: 12),
        Row(
          children: [
            Text('Rust core', style: Gx.uiSans(fontSize: 13, color: Gx.textBaseSecondary)),
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
          style: Gx.dataMono(fontSize: 11, color: Gx.textBaseMuted),
        ),
      ],
    );
  }
}
