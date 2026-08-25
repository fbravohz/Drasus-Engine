// Tipos de datos estáticos del sistema de tema.
// Todo aquí es const puro: sin estado, sin SharedPreferences, sin ChangeNotifier.
// Consumido por theme_scope.dart (estado) y tokens.dart (ThemeExtension).

import 'package:flutter/material.dart';

// ---------------------------------------------------------------------------
// Enums de paleta y modo de superficie.
// ---------------------------------------------------------------------------

// Cada valor representa un esquema de color completo para las 5 capas
// de superficie (deepSpace → surfaceRaised). El nombre "void_" lleva
// guión bajo para evitar colisión con la palabra reservada "void" de Dart.
enum BackgroundPalette {
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
enum SurfaceMode {
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
// Añadir un 5º/6º modo = una entrada aquí + su lógica en frosted()/FrostedSurface.
// Cero cambios en componentes.
const Map<SurfaceMode, SurfaceModeRecipe> kSurfaceModeRegistry = {
  SurfaceMode.glass: SurfaceModeRecipe(
    label: 'Vidrio Apple',
    description: 'BackdropFilter blur 36 + rim-light',
  ),
  SurfaceMode.tint: SurfaceModeRecipe(
    label: 'Translúcido',
    description: 'Solo glassFill sin blur ni rim',
  ),
  SurfaceMode.solid: SurfaceModeRecipe(
    label: 'Sólido oscuro',
    description: 'panelSolid sin translucidez, para datos densos',
  ),
  SurfaceMode.enhancedGlass: SurfaceModeRecipe(
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
class SurfacePalette {
  final Color deepSpace;
  final Color navRail;
  final Color panelSolid;
  final Color cardInner;
  final Color surfaceRaised;

  const SurfacePalette({
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
const Map<BackgroundPalette, SurfacePalette> kPalettes = {
  BackgroundPalette.bunker: SurfacePalette(
    deepSpace: Color(0xFF04050E),
    navRail: Color(0xFF060819),
    panelSolid: Color(0xFF090D1F),
    cardInner: Color(0xFF0C1228),
    surfaceRaised: Color(0xFF111833),
  ),
  BackgroundPalette.ash: SurfacePalette(
    deepSpace: Color(0xFF070707),
    navRail: Color(0xFF0A0A0A),
    panelSolid: Color(0xFF0D0D0D),
    cardInner: Color(0xFF111111),
    surfaceRaised: Color(0xFF161616),
  ),
  BackgroundPalette.crimson: SurfacePalette(
    deepSpace: Color(0xFF0E0406),
    navRail: Color(0xFF160608),
    panelSolid: Color(0xFF1A080B),
    cardInner: Color(0xFF1E0B0F),
    surfaceRaised: Color(0xFF231215),
  ),
  BackgroundPalette.forest: SurfacePalette(
    deepSpace: Color(0xFF040E06),
    navRail: Color(0xFF061508),
    panelSolid: Color(0xFF091A0B),
    cardInner: Color(0xFF0C1E0E),
    surfaceRaised: Color(0xFF112414),
  ),
  BackgroundPalette.navy: SurfacePalette(
    deepSpace: Color(0xFF04080E),
    navRail: Color(0xFF060C18),
    panelSolid: Color(0xFF090F1F),
    cardInner: Color(0xFF0C1428),
    surfaceRaised: Color(0xFF111B33),
  ),
  BackgroundPalette.void_: SurfacePalette(
    deepSpace: Color(0xFF07040E),
    navRail: Color(0xFF0A0619),
    panelSolid: Color(0xFF0D091F),
    cardInner: Color(0xFF110D28),
    surfaceRaised: Color(0xFF161233),
  ),
  BackgroundPalette.slate: SurfacePalette(
    deepSpace: Color(0xFFD8DCE8),
    navRail: Color(0xFFCDD2DF),
    panelSolid: Color(0xFFC2C8D6),
    cardInner: Color(0xFFB7BECD),
    surfaceRaised: Color(0xFFACB4C4),
  ),
  BackgroundPalette.paper: SurfacePalette(
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
const Map<BackgroundPalette, Color> kTextDefaults = {
  BackgroundPalette.bunker:  Color(0xFFE6ECF8),
  BackgroundPalette.ash:     Color(0xFFE6ECF8),
  BackgroundPalette.crimson: Color(0xFFE6ECF8),
  BackgroundPalette.forest:  Color(0xFFE6ECF8),
  BackgroundPalette.navy:    Color(0xFFE6ECF8),
  BackgroundPalette.void_:   Color(0xFFE6ECF8),
  BackgroundPalette.slate:   Color(0xFF1A1E2E), // oscuro sobre fondo claro
  BackgroundPalette.paper:   Color(0xFF1A1E2E), // oscuro sobre fondo claro
};

// Color de fondo de componentes automático por paleta.
const Map<BackgroundPalette, Color> kAutoComponentBgDefaults = {
  BackgroundPalette.bunker:  Color(0xFF090D1F),
  BackgroundPalette.ash:     Color(0xFF0D0D0D),
  BackgroundPalette.crimson: Color(0xFF1A080B),
  BackgroundPalette.forest:  Color(0xFF091A0B),
  BackgroundPalette.navy:    Color(0xFF090F1F),
  BackgroundPalette.void_:   Color(0xFF0D091F),
  BackgroundPalette.slate:   Color(0xFFC2C8D6), // claro para fondo claro
  BackgroundPalette.paper:   Color(0xFFDADEE8), // claro para fondo claro
};

// Color de énfasis automático por paleta.
// Oscuros → mantiene el default transitionIndigo. Claros → variante más
// oscura para mantener contraste sobre fondo blanco.
const Map<BackgroundPalette, Color> kAutoAccentDefaults = {
  BackgroundPalette.bunker:  Color(0xFF9A8CFF), // transitionIndigo
  BackgroundPalette.ash:     Color(0xFF9A8CFF),
  BackgroundPalette.crimson: Color(0xFFCC2B2B), // criticalCrimson oscuro
  BackgroundPalette.forest:  Color(0xFF54E8D0), // optimaCyan
  BackgroundPalette.navy:    Color(0xFF56A8FF), // transitionBlue
  BackgroundPalette.void_:   Color(0xFF9A8CFF),
  BackgroundPalette.slate:   Color(0xFF6C5CE7), // indigo más oscuro para fondo claro
  BackgroundPalette.paper:   Color(0xFF6C5CE7), // indigo más oscuro para fondo claro
};
