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
// Para añadir un nuevo modo: (1) añade el valor aquí y (2) añade su entrada
// en kSurfaceModeRegistry. Los componentes NO necesitan actualizarse.
// ---------------------------------------------------------------------------
enum DrasusSurfaceMode {
  glass,         // BackdropFilter + blur 36 + rim-light (vidrio Apple completo)
  tint,          // Solo glassFill sin blur (panel translúcido ligero)
  solid,         // panelSolid/cardInner (sólido oscuro, sin translucidez)
  enhancedGlass, // Gradiente profundo + borde de énfasis dinámico + glow amplio
}

// ---------------------------------------------------------------------------
// Receta de metadatos por modo de superficie.
// label:       etiqueta que el panel de configuración muestra al usuario.
// description: descripción breve del efecto visual (tooltip / subtítulo).
// ---------------------------------------------------------------------------
class SurfaceModeRecipe {
  final String label;
  final String description;
  const SurfaceModeRecipe({required this.label, required this.description});
}

// Registro completo de modos de superficie.
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
// Mapa canónico de paletas: DrasusBackgroundPalette → DrasusSurfacePalette.
// Hexadecimales spec de DESIGN.md (2026-06-24).
// ÚNICA fuente de verdad de los colores de paleta. Público para que el
// SettingsDrawer y cualquier otro consumidor lo lea sin duplicarlo (ADR-0139).
// ---------------------------------------------------------------------------
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
// DrasusThemeState — el estado mutable del tema, persiste en SharedPreferences.
// ---------------------------------------------------------------------------

// Clave de persistencia para el color de énfasis (entero ARGB serializado).
const _kKeyAccent = 'accent_color';
// Clave de persistencia para la paleta de fondo (índice del enum).
const _kKeyPalette = 'background_palette';
// Clave de persistencia para el modo de superficie (índice del enum).
const _kKeySurfaceMode = 'surface_mode';
// Clave de persistencia para el override de color de texto (ARGB32; -1 = auto).
const _kKeyTextColor = 'text_color_override';
// Valor centinela que codifica "modo automático" en la clave de texto.
const _kNoTextOverride = -1;
// Clave de persistencia para el color de fondo de componentes (ARGB32).
const _kKeyComponentBg = 'component_bg_color';
// Clave de persistencia para el modo automático de paleta (1 = auto, 0 = manual).
const _kKeyAutoPalette = 'auto_palette';
// Claves de persistencia para familias tipográficas.
const _kKeyFontDisplay = 'font_display';
const _kKeyFontSans = 'font_sans';
const _kKeyFontMono = 'font_mono';

// Color de énfasis por defecto: transitionIndigo.
const _kDefaultAccent = Color(0xFF9A8CFF);
// Paleta por defecto: bunker nocturno.
const _kDefaultPalette = DrasusBackgroundPalette.bunker;
// Modo de superficie por defecto: vidrio Apple completo.
const _kDefaultSurfaceMode = DrasusSurfaceMode.glass;
// Color de fondo de componentes por defecto: midnight blue sutil.
// Base neutra oscura que funciona como tinte en glass y fondo en solid.
const _kDefaultComponentBg = Color(0xFF1A1A2E);
// Familias tipográficas por defecto.
const _kDefaultFontDisplay = 'SpaceGrotesk';
const _kDefaultFontSans = 'Inter';
const _kDefaultFontMono = 'JetBrainsMono';

// ---------------------------------------------------------------------------
// Color de texto base por paleta.
// Claro (0xFFE6ECF8) sobre fondos oscuros; oscuro (0xFF1A1E2E) sobre fondos
// claros (slate y paper). Ningún componente hardcodea el color del texto base.
// ---------------------------------------------------------------------------
const Map<DrasusBackgroundPalette, Color> kTextDefaults = {
  DrasusBackgroundPalette.bunker: Color(0xFFE6ECF8),
  DrasusBackgroundPalette.ash:    Color(0xFFE6ECF8),
  DrasusBackgroundPalette.crimson: Color(0xFFE6ECF8),
  DrasusBackgroundPalette.forest: Color(0xFFE6ECF8),
  DrasusBackgroundPalette.navy:   Color(0xFFE6ECF8),
  DrasusBackgroundPalette.void_:  Color(0xFFE6ECF8),
  DrasusBackgroundPalette.slate:  Color(0xFF1A1E2E), // oscuro sobre fondo claro
  DrasusBackgroundPalette.paper:  Color(0xFF1A1E2E), // oscuro sobre fondo claro
};

// ---------------------------------------------------------------------------
// Color de fondo de componentes automático por paleta.
// ---------------------------------------------------------------------------
const Map<DrasusBackgroundPalette, Color> kAutoComponentBgDefaults = {
  DrasusBackgroundPalette.bunker: Color(0xFF090D1F),
  DrasusBackgroundPalette.ash:    Color(0xFF0D0D0D),
  DrasusBackgroundPalette.crimson: Color(0xFF1A080B),
  DrasusBackgroundPalette.forest: Color(0xFF091A0B),
  DrasusBackgroundPalette.navy:   Color(0xFF090F1F),
  DrasusBackgroundPalette.void_:  Color(0xFF0D091F),
  DrasusBackgroundPalette.slate:  Color(0xFFC2C8D6), // claro para fondo claro
  DrasusBackgroundPalette.paper:  Color(0xFFDADEE8), // claro para fondo claro
};

// ---------------------------------------------------------------------------
// Color de énfasis automático por paleta.
// Oscuros → mantiene el default transitionIndigo. Claros → variante más
// oscura para mantener contraste sobre fondo blanco.
// ---------------------------------------------------------------------------
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

// ---------------------------------------------------------------------------
// Espejos estáticos globales — leídos por Gx helpers sin BuildContext.
// Se sincronizan en load() y en cada mutador (setAccent, setPalette, etc.).
// ---------------------------------------------------------------------------

// Espejo del modo de superficie activo.
DrasusSurfaceMode _globalSurfaceMode = _kDefaultSurfaceMode;
// Espejo del color de énfasis activo.
Color _globalAccent = _kDefaultAccent;
// Espejo del color de texto base efectivo (override manual o auto por paleta).
Color _globalTextColor = kTextDefaults[_kDefaultPalette]!;
// Espejo del color de fondo de componentes (tinte de glass / fondo de solid).
Color _globalComponentBgColor = _kDefaultComponentBg;
// Espejos de familias tipográficas.
String _globalFontDisplay = _kDefaultFontDisplay;
String _globalFontSans = _kDefaultFontSans;
String _globalFontMono = _kDefaultFontMono;
// Espejo del color de lienzo base (deepSpace de la paleta activa).
Color _globalCanvasBase = kPalettes[_kDefaultPalette]!.deepSpace;
// Espejo del color raised/hover (surfaceRaised de la paleta activa).
Color _globalSurfaceRaised = kPalettes[_kDefaultPalette]!.surfaceRaised;

// DrasusThemeState notifica a todos los widgets suscritos cuando el tema cambia.
class DrasusThemeState extends ChangeNotifier {
  Color _accentColor = _kDefaultAccent;
  DrasusBackgroundPalette _palette = _kDefaultPalette;
  DrasusSurfaceMode _surfaceMode = _kDefaultSurfaceMode;
  // Override manual de color de texto; null = modo automático por paleta.
  Color? _textOverride;
  // Color de fondo de componentes: base para solid, tinte para glass/tint.
  Color _componentBgColor = _kDefaultComponentBg;
  // Modo automático de paleta: true = acento/componente/texto se auto-seleccionan.
  bool _autoPalette = true;
  // Familias tipográficas activas.
  String _fontDisplay = _kDefaultFontDisplay;
  String _fontSans = _kDefaultFontSans;
  String _fontMono = _kDefaultFontMono;

  // Devuelve el color de énfasis activo.
  Color get accentColor => _accentColor;

  // Devuelve la paleta de fondo activa.
  DrasusBackgroundPalette get backgroundPalette => _palette;

  // Devuelve true si el modo automático de paleta está activo.
  bool get isAutoPalette => _autoPalette;

  // Devuelve los 5 colores de superficie para la paleta activa.
  DrasusSurfacePalette get surfaces => kPalettes[_palette]!;

  // Devuelve el modo de superficie activo.
  DrasusSurfaceMode get surfaceMode => _surfaceMode;

  // Devuelve el color de texto base efectivo: override manual o automático por paleta.
  Color get effectiveTextColor => _textOverride ?? kTextDefaults[_palette]!;

  // Indica si el color de texto está en modo automático (sin override manual).
  bool get isTextColorAuto => _textOverride == null;

  // Devuelve el color de fondo de componentes activo.
  Color get componentBgColor => _componentBgColor;

  // Devuelve la familia tipográfica display grotesca activa.
  String get fontDisplay => _fontDisplay;

  // Devuelve la familia tipográfica sans (UI) activa.
  String get fontSans => _fontSans;

  // Devuelve la familia tipográfica mono (datos) activa.
  String get fontMono => _fontMono;

  // Acceso estático global: frosted() y GlassSurface lo leen sin BuildContext.
  static DrasusSurfaceMode get globalSurfaceMode => _globalSurfaceMode;

  // Acceso estático global: Gx.accentDynamic lo lee sin BuildContext.
  static Color get globalAccent => _globalAccent;

  // Acceso estático global: Gx.textBase lo lee sin BuildContext.
  static Color get globalTextColor => _globalTextColor;

  // Acceso estático global: Gx.surfaceFill/surfacePanel/surfaceCard lo leen sin BuildContext.
  static Color get globalComponentBgColor => _globalComponentBgColor;

  // Acceso estático global: Gx.fontDisplay/fontSans/fontMono lo leen sin BuildContext.
  static String get globalFontDisplay => _globalFontDisplay;
  static String get globalFontSans => _globalFontSans;
  static String get globalFontMono => _globalFontMono;

  // Acceso estático global: Gx.canvasBase y Gx.surfaceRaisedDynamic lo leen sin BuildContext.
  static Color get globalCanvasBase => _globalCanvasBase;
  static Color get globalSurfaceRaised => _globalSurfaceRaised;

  // Construye el ThemeData unificado con las cuatro ThemeExtension de
  // ADR-0138. La textTheme se mapea explícitamente para que cualquier widget
  // Text() que use Theme.of(context).textTheme.* obtenga nuestras familias
  // tipográficas sin depender de los helpers Gx en cada callsite.
  ThemeData buildThemeData() {
    final pal = kPalettes[_palette]!;
    final tColor = effectiveTextColor;
    final baseTextStyle = TextStyle(
      fontFamily: _fontSans,
      color: tColor,
    );
    return ThemeData.dark(useMaterial3: true).copyWith(
      textTheme: TextTheme(
        displayLarge: baseTextStyle.copyWith(fontSize: 40, fontWeight: FontWeight.w500, letterSpacing: -0.8, fontFamily: _fontDisplay),
        displayMedium: baseTextStyle.copyWith(fontSize: 32, fontWeight: FontWeight.w500, fontFamily: _fontDisplay),
        displaySmall: baseTextStyle.copyWith(fontSize: 22, fontWeight: FontWeight.w500, fontFamily: _fontDisplay, letterSpacing: -0.4),
        headlineLarge: baseTextStyle.copyWith(fontSize: 20, fontWeight: FontWeight.w500),
        headlineMedium: baseTextStyle.copyWith(fontSize: 18, fontWeight: FontWeight.w500),
        headlineSmall: baseTextStyle.copyWith(fontSize: 16, fontWeight: FontWeight.w500),
        titleLarge: baseTextStyle.copyWith(fontSize: 16, fontWeight: FontWeight.w500),
        titleMedium: baseTextStyle.copyWith(fontSize: 14, fontWeight: FontWeight.w500),
        titleSmall: baseTextStyle.copyWith(fontSize: 13, fontWeight: FontWeight.w500),
        bodyLarge: baseTextStyle.copyWith(fontSize: 14, fontWeight: FontWeight.w400, height: 1.5),
        bodyMedium: baseTextStyle.copyWith(fontSize: 13, fontWeight: FontWeight.w400, height: 1.5),
        bodySmall: baseTextStyle.copyWith(fontSize: 12, fontWeight: FontWeight.w400, height: 1.4),
        labelLarge: baseTextStyle.copyWith(fontSize: 13, fontWeight: FontWeight.w500, fontFamily: _fontMono),
        labelMedium: baseTextStyle.copyWith(fontSize: 12, fontWeight: FontWeight.w400, fontFamily: _fontMono),
        labelSmall: baseTextStyle.copyWith(fontSize: 11, fontWeight: FontWeight.w400, fontFamily: _fontMono),
      ),
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
  // Al finalizar, los espejos estáticos (_globalSurfaceMode, _globalAccent,
  // _globalTextColor) reflejan los valores persistidos o sus defaults.
  Future<void> load() async {
    final prefs = await SharedPreferences.getInstance();
    final accentInt = prefs.getInt(_kKeyAccent);
    final paletteIdx = prefs.getInt(_kKeyPalette);

    if (accentInt != null) {
      // Color() acepta el entero ARGB32 guardado con toARGB32().
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

    // Leer override de color de texto (_kNoTextOverride = -1 indica modo auto).
    final textInt = prefs.getInt(_kKeyTextColor);
    if (textInt != null && textInt != _kNoTextOverride) {
      _textOverride = Color(textInt);
    } else {
      _textOverride = null;
    }

    // Leer color de fondo de componentes (ARGB32 serializado).
    final compBgInt = prefs.getInt(_kKeyComponentBg);
    if (compBgInt != null) {
      _componentBgColor = Color(compBgInt);
    }

    // Leer modo automático de paleta (default true).
    final autoPaletteVal = prefs.getInt(_kKeyAutoPalette);
    _autoPalette = autoPaletteVal == null || autoPaletteVal == 1;

    // Leer familias tipográficas (string).
    final fontDisplayStr = prefs.getString(_kKeyFontDisplay);
    if (fontDisplayStr != null) _fontDisplay = fontDisplayStr;
    final fontSansStr = prefs.getString(_kKeyFontSans);
    if (fontSansStr != null) _fontSans = fontSansStr;
    final fontMonoStr = prefs.getString(_kKeyFontMono);
    if (fontMonoStr != null) _fontMono = fontMonoStr;

    // Inicializar los espejos estáticos con los valores cargados.
    _globalAccent = _accentColor;
    _globalTextColor = effectiveTextColor;
    _globalComponentBgColor = _componentBgColor;
    _globalFontDisplay = _fontDisplay;
    _globalFontSans = _fontSans;
    _globalFontMono = _fontMono;
    // Sincronizar espejos de paleta.
    final palSurface = kPalettes[_palette]!;
    _globalCanvasBase = palSurface.deepSpace;
    _globalSurfaceRaised = palSurface.surfaceRaised;

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

  // Cambia el color de énfasis, actualiza el espejo estático y lo persiste.
  // Notifica a todos los widgets suscritos.
  Future<void> setAccent(Color color) async {
    _accentColor = color;
    _globalAccent = color; // sincroniza el espejo estático para Gx.accentDynamic
    notifyListeners();
    final prefs = await SharedPreferences.getInstance();
    // toARGB32() reemplaza .value (no-deprecated en Flutter 3.44+).
    await prefs.setInt(_kKeyAccent, color.toARGB32());
  }

  // Cambia la paleta de fondo y la persiste. Notifica a los widgets.
  // En modo automático de paleta, recalcula texto, acento y fondo de componentes.
  Future<void> setPalette(DrasusBackgroundPalette palette) async {
    _palette = palette;
    final palSurface = kPalettes[_palette]!;

    if (_autoPalette) {
      // Auto: texto, acento y fondo de componentes según la paleta.
      _textOverride = null;
      _globalTextColor = kTextDefaults[_palette]!;
      _accentColor = kAutoAccentDefaults[_palette]!;
      _globalAccent = _accentColor;
      _componentBgColor = kAutoComponentBgDefaults[_palette]!;
      _globalComponentBgColor = _componentBgColor;
    } else {
      // Manual: solo recalcula texto si no hay override.
      if (_textOverride == null) {
        _globalTextColor = kTextDefaults[_palette]!;
      }
    }
    // Sincronizar espejos de paleta.
    _globalCanvasBase = palSurface.deepSpace;
    _globalSurfaceRaised = palSurface.surfaceRaised;
    notifyListeners();
    final prefs = await SharedPreferences.getInstance();
    await prefs.setInt(_kKeyPalette, DrasusBackgroundPalette.values.indexOf(palette));
  }

  // Establece un override manual de color de texto base y lo persiste.
  // Todos los componentes que lean Gx.textBase verán este color inmediatamente.
  Future<void> setTextColor(Color color) async {
    _textOverride = color;
    _globalTextColor = color;
    notifyListeners();
    final prefs = await SharedPreferences.getInstance();
    await prefs.setInt(_kKeyTextColor, color.toARGB32());
  }

  // Vuelve al modo automático: el color de texto lo determina la paleta activa.
  // Persiste el valor centinela _kNoTextOverride para indicar "auto".
  Future<void> setTextColorAuto() async {
    _textOverride = null;
    _globalTextColor = kTextDefaults[_palette]!;
    notifyListeners();
    final prefs = await SharedPreferences.getInstance();
    await prefs.setInt(_kKeyTextColor, _kNoTextOverride);
  }

  // Cambia el color de fondo de componentes, actualiza el espejo estático y lo persiste.
  // Notifica a todos los widgets suscritos. Este color controla el tinte/fondo
  // que usan los componentes como base en los 4 modos de superficie.
  Future<void> setComponentBgColor(Color color) async {
    _componentBgColor = color;
    _globalComponentBgColor = color; // sincroniza el espejo estático para Gx
    notifyListeners();
    final prefs = await SharedPreferences.getInstance();
    await prefs.setInt(_kKeyComponentBg, color.toARGB32());
  }

  // Activa/desactiva el modo automático de paleta.
  // ON → al cambiar de paleta se auto-ajustan acento, texto y fondo de componentes.
  // OFF → los controles manuales se muestran; los valores actuales se congelan.
  Future<void> setAutoPalette(bool value) async {
    _autoPalette = value;
    if (value) {
      // Auto: aplicar defaults, liberar override de texto.
      _textOverride = null;
      _globalTextColor = kTextDefaults[_palette]!;
      _accentColor = kAutoAccentDefaults[_palette]!;
      _globalAccent = _accentColor;
      _componentBgColor = kAutoComponentBgDefaults[_palette]!;
      _globalComponentBgColor = _componentBgColor;
    } else {
      // Manual: congelar valores actuales como overrides.
      _textOverride = _globalTextColor;
    }
    notifyListeners();
    final prefs = await SharedPreferences.getInstance();
    await prefs.setInt(_kKeyAutoPalette, value ? 1 : 0);
  }

  // ---------------------------------------------------------------------------
  // Mutadores de fuente — cambian la familia tipográfica globalmente.
  // ---------------------------------------------------------------------------

  // Cambia la fuente display grotesca y la persiste. Notifica a los widgets.
  Future<void> setFontDisplay(String family) async {
    _fontDisplay = family;
    _globalFontDisplay = family;
    notifyListeners();
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(_kKeyFontDisplay, family);
  }

  // Cambia la fuente sans (UI) y la persiste. Notifica a los widgets.
  Future<void> setFontSans(String family) async {
    _fontSans = family;
    _globalFontSans = family;
    notifyListeners();
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(_kKeyFontSans, family);
  }

  // Cambia la fuente mono (datos) y la persiste. Notifica a los widgets.
  Future<void> setFontMono(String family) async {
    _fontMono = family;
    _globalFontMono = family;
    notifyListeners();
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(_kKeyFontMono, family);
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
