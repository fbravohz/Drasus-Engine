// Tipos de datos estáticos del sistema de tema Drasus.
// Todo aquí es const puro: sin estado, sin SharedPreferences, sin ChangeNotifier.
// Consumido por drasus_theme.dart (estado) y drasus_tokens.dart (ThemeExtension).

import 'package:flutter/material.dart';

// ---------------------------------------------------------------------------
// Enums de paleta y modo de superficie.
// ---------------------------------------------------------------------------

// Cada valor representa un esquema de color completo para las 5 capas
// de superficie (deepSpace → surfaceRaised). El nombre "void_" lleva
// guión bajo para evitar colisión con la palabra reservada "void" de Dart.
enum DrasusBackgroundPalette {
  bunker,
  ash,
  crimson,
  forest,
  navy,
  void_,
  slate,
  paper,
}

// Controla qué receta visual usan TODOS los componentes que dibujan superficies.
// Para añadir un nuevo modo: (1) añade el valor aquí y (2) añade su entrada
// en kSurfaceModeRegistry. Los componentes NO necesitan actualizarse.
enum DrasusSurfaceMode {
  glass,         // BackdropFilter + blur 36 + rim-light (vidrio Apple completo)
  tint,          // Solo glassFill sin blur (panel translúcido ligero)
  solid,         // panelSolid/cardInner (sólido oscuro, sin translucidez)
  enhancedGlass, // Gradiente profundo + borde de énfasis dinámico + glow amplio
}

// ---------------------------------------------------------------------------
// Registro de modos de superficie.
// ---------------------------------------------------------------------------

// label:       etiqueta que el panel de configuración muestra al usuario.
// description: descripción breve del efecto visual (tooltip / subtítulo).
class SurfaceModeRecipe {
  final String label;
  final String description;
  const SurfaceModeRecipe({required this.label, required this.description});
}

// EL PANEL ITERA ESTE MAPA — nunca una lista hardcodeada.
// Añadir un 5º/6º modo = una entrada aquí + su lógica en frosted()/GlassSurface.
// Cero cambios en componentes.
const Map<DrasusSurfaceMode, SurfaceModeRecipe> kSurfaceModeRegistry = {
  DrasusSurfaceMode.glass: SurfaceModeRecipe(
    label: 'Vidrio Apple',
    description: 'BackdropFilter blur 36 + rim-light',
  ),
  DrasusSurfaceMode.tint: SurfaceModeRecipe(
    label: 'Translúcido',
    description: 'Solo glassFill sin blur ni rim',
  ),
  DrasusSurfaceMode.solid: SurfaceModeRecipe(
    label: 'Sólido oscuro',
    description: 'panelSolid sin translucidez, para datos densos',
  ),
  DrasusSurfaceMode.enhancedGlass: SurfaceModeRecipe(
    label: 'Vidrio Premium',
    description: 'Gradiente profundo + borde de énfasis + glow amplio',
  ),
};

// ---------------------------------------------------------------------------
// Paleta de superficie — los 5 colores que componen una paleta de fondo.
// ---------------------------------------------------------------------------

// deepSpace: el lienzo base más oscuro (fondo del canvas/ZUI).
// navRail: superficie del riel de navegación lateral.
// panelSolid: panel de datos con borde.
// cardInner: tarjeta interna dentro del panel.
// surfaceRaised: hover de fila, celda activa, superficie elevada.
class DrasusSurfacePalette {
  final Color deepSpace;
  final Color navRail;
  final Color panelSolid;
  final Color cardInner;
  final Color surfaceRaised;

  const DrasusSurfacePalette({
    required this.deepSpace,
    required this.navRail,
    required this.panelSolid,
    required this.cardInner,
    required this.surfaceRaised,
  });
}

// ---------------------------------------------------------------------------
// Mapa canónico de paletas.
// ---------------------------------------------------------------------------

// Hexadecimales spec de DESIGN.md (2026-06-24).
// ÚNICA fuente de verdad de los colores de paleta. Público para que el
// SettingsDrawer y cualquier otro consumidor lo lea sin duplicarlo (ADR-0139).
const Map<DrasusBackgroundPalette, DrasusSurfacePalette> kPalettes = {
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
// Defaults automáticos por paleta.
// ---------------------------------------------------------------------------

// Claro (0xFFE6ECF8) sobre fondos oscuros; oscuro (0xFF1A1E2E) sobre fondos
// claros (slate y paper). Ningún componente hardcodea el color del texto base.
const Map<DrasusBackgroundPalette, Color> kTextDefaults = {
  DrasusBackgroundPalette.bunker:  Color(0xFFE6ECF8),
  DrasusBackgroundPalette.ash:     Color(0xFFE6ECF8),
  DrasusBackgroundPalette.crimson: Color(0xFFE6ECF8),
  DrasusBackgroundPalette.forest:  Color(0xFFE6ECF8),
  DrasusBackgroundPalette.navy:    Color(0xFFE6ECF8),
  DrasusBackgroundPalette.void_:   Color(0xFFE6ECF8),
  DrasusBackgroundPalette.slate:   Color(0xFF1A1E2E), // oscuro sobre fondo claro
  DrasusBackgroundPalette.paper:   Color(0xFF1A1E2E), // oscuro sobre fondo claro
};

// Color de fondo de componentes automático por paleta.
const Map<DrasusBackgroundPalette, Color> kAutoComponentBgDefaults = {
  DrasusBackgroundPalette.bunker:  Color(0xFF090D1F),
  DrasusBackgroundPalette.ash:     Color(0xFF0D0D0D),
  DrasusBackgroundPalette.crimson: Color(0xFF1A080B),
  DrasusBackgroundPalette.forest:  Color(0xFF091A0B),
  DrasusBackgroundPalette.navy:    Color(0xFF090F1F),
  DrasusBackgroundPalette.void_:   Color(0xFF0D091F),
  DrasusBackgroundPalette.slate:   Color(0xFFC2C8D6), // claro para fondo claro
  DrasusBackgroundPalette.paper:   Color(0xFFDADEE8), // claro para fondo claro
};

// Color de énfasis automático por paleta.
// Oscuros → mantiene el default transitionIndigo. Claros → variante más
// oscura para mantener contraste sobre fondo blanco.
const Map<DrasusBackgroundPalette, Color> kAutoAccentDefaults = {
  DrasusBackgroundPalette.bunker:  Color(0xFF9A8CFF), // transitionIndigo
  DrasusBackgroundPalette.ash:     Color(0xFF9A8CFF),
  DrasusBackgroundPalette.crimson: Color(0xFFCC2B2B), // criticalCrimson oscuro
  DrasusBackgroundPalette.forest:  Color(0xFF54E8D0), // optimaCyan
  DrasusBackgroundPalette.navy:    Color(0xFF56A8FF), // transitionBlue
  DrasusBackgroundPalette.void_:   Color(0xFF9A8CFF),
  DrasusBackgroundPalette.slate:   Color(0xFF6C5CE7), // indigo más oscuro para fondo claro
  DrasusBackgroundPalette.paper:   Color(0xFF6C5CE7), // indigo más oscuro para fondo claro
};
