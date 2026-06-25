// Sistema de temas dinámico de Drasus Engine.
// Provee el color de énfasis (accentColor) y la paleta de fondo activa
// a todo el árbol de widgets sin pasar parámetros a mano.
//
// Patrón: InheritedWidget + ChangeNotifier + SharedPreferences.
// InheritedWidget → lectura O(1) desde cualquier widget del árbol.
// ChangeNotifier  → rebuilds reactivos cuando cambia el tema.
// SharedPreferences → el tema elegido sobrevive reinicios de la app.

import 'package:flutter/material.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'theme/drasus_tokens.dart';

// ---------------------------------------------------------------------------
// Paleta de fondo — los 8 modos de ambientación disponibles.
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

// ---------------------------------------------------------------------------
// Modo global de superficie — controla qué receta visual usan TODOS los
// componentes que dibujan superficies (frosted, GlassSurface, inputs, paneles).
// Cambiar aquí = cambia en toda la app sin tocar componente por componente.
// ---------------------------------------------------------------------------
enum DrasusSurfaceMode {
  glass, // BackdropFilter + blur 36 + rim-light (vidrio Apple completo)
  tint,  // Solo glassFill sin blur (panel translúcido ligero)
  solid, // panelSolid/cardInner (sólido oscuro, sin translucidez)
}

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
// Mapa canónico de paletas: DrasusBackgroundPalette → DrasusSurfacePalette.
// Hexadecimales spec de DESIGN.md (2026-06-24).
// ---------------------------------------------------------------------------
const Map<DrasusBackgroundPalette, DrasusSurfacePalette> _kPalettes = {
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
// DrasusThemeState — el estado mutable del tema, persiste en SharedPreferences.
// ---------------------------------------------------------------------------

// Clave de persistencia para el color de énfasis (entero ARGB serializado).
const _kKeyAccent = 'accent_color';
// Clave de persistencia para la paleta de fondo (índice del enum).
const _kKeyPalette = 'background_palette';
// Color de énfasis por defecto: transitionIndigo.
const _kDefaultAccent = Color(0xFF9A8CFF);
// Paleta por defecto: bunker nocturno.
const _kDefaultPalette = DrasusBackgroundPalette.bunker;
// Modo de superficie por defecto: vidrio Apple completo.
const _kDefaultSurfaceMode = DrasusSurfaceMode.glass;
const _kKeySurfaceMode = 'surface_mode';

// Variable estática global: frosted() la lee sin BuildContext.
DrasusSurfaceMode _globalSurfaceMode = _kDefaultSurfaceMode;

// DrasusThemeState notifica a todos los widgets suscritos cuando el tema cambia.
class DrasusThemeState extends ChangeNotifier {
  Color _accentColor = _kDefaultAccent;
  DrasusBackgroundPalette _palette = _kDefaultPalette;
  DrasusSurfaceMode _surfaceMode = _kDefaultSurfaceMode;

  // Devuelve el color de énfasis activo.
  Color get accentColor => _accentColor;

  // Devuelve la paleta de fondo activa.
  DrasusBackgroundPalette get backgroundPalette => _palette;

  // Devuelve los 5 colores de superficie para la paleta activa.
  DrasusSurfacePalette get surfaces => _kPalettes[_palette]!;

  // Devuelve el modo de superficie activo (glass/tint/solid).
  DrasusSurfaceMode get surfaceMode => _surfaceMode;

  // Acceso estático global: frosted() y GlassSurface lo leen sin BuildContext.
  static DrasusSurfaceMode get globalSurfaceMode => _globalSurfaceMode;

  // Construye el ThemeData unificado con las cuatro ThemeExtension de
  // ADR-0138. Es la única fuente de verdad del tema: el MaterialApp la
  // consume y se reconstruye cuando este notifier dispara.
  ThemeData buildThemeData() {
    final pal = _kPalettes[_palette]!;
    return ThemeData.dark(useMaterial3: true).copyWith(
      extensions: [
        DrasusGlass.defaults,
        DrasusMotion.defaults,
        DrasusSurfaces.fromPalette(pal),
        DrasusPalette(accentColor: _accentColor, backgroundPalette: _palette),
      ],
    );
  }

  // Carga el tema guardado desde SharedPreferences. Debe llamarse una vez
  // durante la inicialización de la app, antes de montar el árbol.
  Future<void> load() async {
    final prefs = await SharedPreferences.getInstance();
    final accentInt = prefs.getInt(_kKeyAccent);
    final paletteIdx = prefs.getInt(_kKeyPalette);

    if (accentInt != null) {
      // Color.fromARGB32 / Color() aceptan el entero ARGB32 guardado con toARGB32().
      _accentColor = Color(accentInt);
    }
    if (paletteIdx != null &&
        paletteIdx >= 0 &&
        paletteIdx < DrasusBackgroundPalette.values.length) {
      _palette = DrasusBackgroundPalette.values[paletteIdx];
    }
    final modeIdx = prefs.getInt(_kKeySurfaceMode);
    if (modeIdx != null &&
        modeIdx >= 0 &&
        modeIdx < DrasusSurfaceMode.values.length) {
      _surfaceMode = DrasusSurfaceMode.values[modeIdx];
      _globalSurfaceMode = _surfaceMode;
    }
    // No se notifica aquí: load() se llama antes de que haya oyentes.
  }

  // Cambia el modo de superficie global (glass/tint/solid).
  Future<void> setSurfaceMode(DrasusSurfaceMode mode) async {
    _surfaceMode = mode;
    _globalSurfaceMode = mode;
    notifyListeners();
    final prefs = await SharedPreferences.getInstance();
    await prefs.setInt(_kKeySurfaceMode, DrasusSurfaceMode.values.indexOf(mode));
  }

  // Cambia el color de énfasis y lo persiste. Notifica a los widgets.
  Future<void> setAccent(Color color) async {
    _accentColor = color;
    notifyListeners();
    final prefs = await SharedPreferences.getInstance();
    // toARGB32() es el reemplazo de .value (no-deprecated en Flutter 3.44+).
    await prefs.setInt(_kKeyAccent, color.toARGB32());
  }

  // Cambia la paleta de fondo y la persiste. Notifica a los widgets.
  Future<void> setPalette(DrasusBackgroundPalette palette) async {
    _palette = palette;
    notifyListeners();
    final prefs = await SharedPreferences.getInstance();
    await prefs.setInt(_kKeyPalette, DrasusBackgroundPalette.values.indexOf(palette));
  }
}

// ---------------------------------------------------------------------------
// DrasusTheme — InheritedNotifier que expone DrasusThemeState al árbol.
// ---------------------------------------------------------------------------

// InheritedNotifier es una versión especializada de InheritedWidget que
// se suscribe automáticamente a un ChangeNotifier y reconstruye los widgets
// dependientes cada vez que el notifier dispara. No requiere AnimatedBuilder
// ni ListenableBuilder extra.
class DrasusTheme extends InheritedNotifier<DrasusThemeState> {
  const DrasusTheme({
    super.key,
    required DrasusThemeState state,
    required super.child,
  }) : super(notifier: state);

  // Acceso estático al estado del tema desde cualquier descendiente.
  // Retorna null si DrasusTheme no está montado sobre el widget que llama.
  static DrasusThemeState? of(BuildContext context) {
    return context
        .dependOnInheritedWidgetOfExactType<DrasusTheme>()
        ?.notifier;
  }

  // Retorna los 5 colores de superficie de la paleta activa.
  // Atajo para DrasusTheme.of(context)?.surfaces.
  static DrasusSurfacePalette? surfaceFor(BuildContext context) {
    return of(context)?.surfaces;
  }
}
