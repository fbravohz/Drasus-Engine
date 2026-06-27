// Sistema de temas dinámico de Drasus Engine.
// Gestiona el estado mutable del tema (acento, paleta, modo de superficie,
// fuentes) y lo expone al árbol de widgets vía InheritedNotifier.
//
// Patrón: InheritedWidget + ChangeNotifier + SharedPreferences.
// InheritedWidget → lectura O(1) desde cualquier widget del árbol.
// ChangeNotifier  → rebuilds reactivos cuando cambia el tema.
// SharedPreferences → el tema elegido sobrevive reinicios de la app.
//
// Tipos de datos puros (enums, paletas, defaults) → theme/drasus_palettes.dart
// ThemeExtension (vidrio, movimiento, superficies) → theme/drasus_tokens.dart

import 'package:flutter/material.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'theme/drasus_palettes.dart';
import 'theme/drasus_tokens.dart';
// Re-exporta los tipos públicos de paletas para que los consumidores existentes
// no necesiten cambiar su import de 'drasus_theme.dart'.
export 'theme/drasus_palettes.dart';

// ---------------------------------------------------------------------------
// Claves y defaults de SharedPreferences.
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
// Familias tipográficas por defecto — todas Rajdhani.
const _kDefaultFontDisplay = 'Rajdhani';
const _kDefaultFontSans = 'Rajdhani';
const _kDefaultFontMono = 'Rajdhani';

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

// Espejos estáticos de la escala Gx, sincronizados en _syncStyleScale().
// Permiten que los helpers nombrados Gx deleguen al theme provider en vez de
// llevar fontSize/color hardcodeados (patrón de bypass — prohibido).
TextStyle _globalMicroLabel = const TextStyle(fontSize: 13);
TextStyle _globalLabel = const TextStyle(fontSize: 14);
TextStyle _globalBody = const TextStyle(fontSize: 14);
TextStyle _globalBodySecondary = const TextStyle(fontSize: 14);
TextStyle _globalSubheading = const TextStyle(fontSize: 16);
TextStyle _globalPanelTitle = const TextStyle(fontSize: 16);
TextStyle _globalSectionHeading = const TextStyle(fontSize: 22);
TextStyle _globalZuiTitle = const TextStyle(fontSize: 40);
TextStyle _globalDataSmall = const TextStyle(fontSize: 14);
TextStyle _globalDataHero = const TextStyle(fontSize: 28);

// ---------------------------------------------------------------------------
// DrasusThemeState — el estado mutable del tema, persiste en SharedPreferences.
// ---------------------------------------------------------------------------

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

  // Getters estáticos para la escala Gx — espejos sincronizados en _syncStyleScale().
  static TextStyle get globalMicroLabel => _globalMicroLabel;
  static TextStyle get globalLabel => _globalLabel;
  static TextStyle get globalBody => _globalBody;
  static TextStyle get globalBodySecondary => _globalBodySecondary;
  static TextStyle get globalSubheading => _globalSubheading;
  static TextStyle get globalPanelTitle => _globalPanelTitle;
  static TextStyle get globalSectionHeading => _globalSectionHeading;
  static TextStyle get globalZuiTitle => _globalZuiTitle;
  static TextStyle get globalDataSmall => _globalDataSmall;
  static TextStyle get globalDataHero => _globalDataHero;

  // Construye el TextTheme temático desde el estado actual de la instancia.
  // También lo usa _syncStyleScale() para mantener los espejos Gx sincronizados.
  TextTheme _buildTextTheme() {
    final tColor = effectiveTextColor;
    final base = TextStyle(fontFamily: _fontSans, color: tColor);
    return TextTheme(
      displayLarge: base.copyWith(fontSize: 40, fontWeight: FontWeight.w500, letterSpacing: -0.8, fontFamily: _fontDisplay),
      displayMedium: base.copyWith(fontSize: 32, fontWeight: FontWeight.w500, fontFamily: _fontDisplay),
      displaySmall: base.copyWith(fontSize: 22, fontWeight: FontWeight.w500, fontFamily: _fontDisplay, letterSpacing: -0.4),
      headlineLarge: base.copyWith(fontSize: 20, fontWeight: FontWeight.w500),
      headlineMedium: base.copyWith(fontSize: 18, fontWeight: FontWeight.w500),
      headlineSmall: base.copyWith(fontSize: 16, fontWeight: FontWeight.w500),
      titleLarge: base.copyWith(fontSize: 16, fontWeight: FontWeight.w500),
      titleMedium: base.copyWith(fontSize: 14, fontWeight: FontWeight.w500),
      titleSmall: base.copyWith(fontSize: 13, fontWeight: FontWeight.w500),
      bodyLarge: base.copyWith(fontSize: 14, fontWeight: FontWeight.w400, height: 1.5),
      bodyMedium: base.copyWith(fontSize: 13, fontWeight: FontWeight.w400, height: 1.5),
      bodySmall: base.copyWith(fontSize: 12, fontWeight: FontWeight.w400, height: 1.4),
      labelLarge: base.copyWith(fontSize: 13, fontWeight: FontWeight.w500, fontFamily: _fontMono),
      labelMedium: base.copyWith(fontSize: 12, fontWeight: FontWeight.w400, fontFamily: _fontMono),
      labelSmall: base.copyWith(fontSize: 11, fontWeight: FontWeight.w400, fontFamily: _fontMono),
    );
  }

  // Construye el ThemeData unificado con las cuatro ThemeExtension de
  // ADR-0138. La textTheme se mapea explícitamente para que cualquier widget
  // Text() que use Theme.of(context).textTheme.* obtenga nuestras familias
  // tipográficas sin depender de los helpers Gx en cada callsite.
  ThemeData buildThemeData() {
    final pal = kPalettes[_palette]!;
    return ThemeData.dark(useMaterial3: true).copyWith(
      textTheme: _buildTextTheme(),
      extensions: [
        DrasusGlass.defaults,
        DrasusMotion.defaults,
        DrasusSurfaces.fromPalette(pal),
        DrasusPalette(accentColor: _accentColor, backgroundPalette: _palette),
      ],
    );
  }

  /// Reconstruye los espejos estáticos de la escala Gx desde el estado actual.
  /// Se llama al final de load() y en cada mutador que afecte texto
  /// (color, familia tipográfica). Mantiene _globalMicroLabel, _globalBody,
  /// etc. sincronizados para que los helpers Gx deleguen al theme provider.
  void _syncStyleScale() {
    final tt = _buildTextTheme();
    final tColor = effectiveTextColor;
    _globalMicroLabel = tt.titleSmall?.copyWith(color: tColor.withOpacity(0.55)) ?? _globalMicroLabel;
    _globalLabel = tt.titleMedium?.copyWith(color: tColor.withOpacity(0.55)) ?? _globalLabel;
    _globalBody = tt.bodyLarge ?? _globalBody;
    _globalBodySecondary = tt.bodyLarge?.copyWith(color: tColor.withOpacity(0.75)) ?? _globalBodySecondary;
    _globalSubheading = tt.headlineSmall?.copyWith(height: 1.5) ?? _globalSubheading;
    _globalPanelTitle = tt.titleLarge?.copyWith(fontFamily: _fontDisplay, color: tColor.withOpacity(0.75)) ?? _globalPanelTitle;
    _globalSectionHeading = tt.displaySmall ?? _globalSectionHeading;
    _globalZuiTitle = tt.displayLarge ?? _globalZuiTitle;
    _globalDataSmall = tt.bodyLarge?.copyWith(fontFamily: _fontMono, height: 1.4) ?? _globalDataSmall;
    _globalDataHero = tt.displayMedium?.copyWith(fontFamily: _fontMono, height: 1.1) ?? _globalDataHero;
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
    // Sincronizar la escala Gx con el TextTheme.
    _syncStyleScale();

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
    _syncStyleScale();
    notifyListeners();
    final prefs = await SharedPreferences.getInstance();
    await prefs.setInt(_kKeyPalette, DrasusBackgroundPalette.values.indexOf(palette));
  }

  // Establece un override manual de color de texto base y lo persiste.
  // Todos los componentes que lean Gx.textBase verán este color inmediatamente.
  Future<void> setTextColor(Color color) async {
    _textOverride = color;
    _globalTextColor = color;
    _syncStyleScale();
    notifyListeners();
    final prefs = await SharedPreferences.getInstance();
    await prefs.setInt(_kKeyTextColor, color.toARGB32());
  }

  // Vuelve al modo automático: el color de texto lo determina la paleta activa.
  // Persiste el valor centinela _kNoTextOverride para indicar "auto".
  Future<void> setTextColorAuto() async {
    _textOverride = null;
    _globalTextColor = kTextDefaults[_palette]!;
    _syncStyleScale();
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
    _syncStyleScale();
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
    _syncStyleScale();
    notifyListeners();
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(_kKeyFontDisplay, family);
  }

  // Cambia la fuente sans (UI) y la persiste. Notifica a los widgets.
  Future<void> setFontSans(String family) async {
    _fontSans = family;
    _globalFontSans = family;
    _syncStyleScale();
    notifyListeners();
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(_kKeyFontSans, family);
  }

  // Cambia la fuente mono (datos) y la persiste. Notifica a los widgets.
  Future<void> setFontMono(String family) async {
    _fontMono = family;
    _globalFontMono = family;
    _syncStyleScale();
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
