// Tokens de diseño de la Galería de Componentes.
// ESTE es el ÚNICO archivo donde los hex de docs/DESIGN.md se vuelven Color(0xFF…).
// Todos los componentes de la galería leen de aquí: cambiar un token en
// docs/DESIGN.md se refleja editando solo este archivo (spec-driven).
//
// Tipografía: los helpers displayGrotesque / uiSans / dataMono usan los .ttf
// embebidos en assets/fonts/ (declarados en pubspec.yaml). Google Fonts ya NO
// se usa para servir las familias en runtime: las fuentes son 100% offline.
// google_fonts se eliminó de pubspec.yaml; toda la tipografía es offline.

import 'package:flutter/material.dart';
import 'package:iconsax_plus/iconsax_plus.dart';
import '../drasus_theme.dart';

// Gx = "Gallery tokens". Espejo 1:1 de la sección "Tokens — Colors" de DESIGN.md.
class Gx {
  Gx._();

  // --- Superficies sólidas (pila de profundidad) ---
  static const deepSpace = Color(0xFF080A18); // lienzo base / ZUI
  static const navRail = Color(0xFF0B1022); // riel de navegación
  static const panelSolid = Color(0xFF0E1426); // panel de datos
  static const cardInner = Color(0xFF11182E); // tarjeta interna
  static const surfaceRaised = Color(0xFF161E38); // hover de fila
  static const glassFill = Color(0x40F0F2FF); // vidrio Apple — tinte claro (~25%) sobre fondo oscuro
  static const gaugeTrack = Color(0xFF16203A); // riel de las barras de vitalidad

  // --- Estructura (bordes y separadores tintados) ---
  static const borderPanel = Color(0xFF1B2440); // hairline del panel sólido
  static const divider = Color(0xFF141C32); // separador interno

  // --- Texto ---
  static const textPrimary = Color(0xFFE6ECF8);
  static const textSecondary = Color(0xFFAEBBD6);
  static const textLabel = Color(0xFF8492B0);
  static const textMuted = Color(0xFF5C6B8C);
  static const pureWhite = Color(0xFFFFFFFF);

  // --- Espectro de vitalidad (neón semántico) ---
  static const optimaCyan = Color(0xFF54E8D0); // óptimo
  static const optimaTeal = Color(0xFF2DD4BF);
  static const reactorGreen = Color(0xFF7CF06A); // acción viva
  static const transitionIndigo = Color(0xFF9A8CFF); // transición / incubación
  static const transitionBlue = Color(0xFF56A8FF);
  static const transitionPurple = Color(0xFF8B83E8);
  static const alertAmber = Color(0xFFFFC94D); // alerta / volátil
  static const alertOrange = Color(0xFFF59423);
  static const criticalRed = Color(0xFFFF8A8A); // crítico
  static const criticalCrimson = Color(0xFFF0413F); // fallo / muerte

  // --- Fondos y bordes de chip por estado ---
  static const optimaChipBg = Color(0xFF08251F);
  static const optimaChipBorder = Color(0xFF1E5E4F);
  static const transitionChipBg = Color(0xFF0E1140);
  static const transitionChipBorder = Color(0xFF2E2F7A);
  static const alertChipBg = Color(0xFF2A1C06);
  static const alertChipBorder = Color(0xFF6E4A14);
  static const criticalChipBg = Color(0xFF2A0C0C);
  static const criticalChipBorder = Color(0xFF7A2A28);

  // --- Cristal y galaxia ---
  static const aberrationRed = Color(0xFFFF5C6B);
  static const aberrationGreen = Color(0xFF54E8D0);
  static const aberrationBlue = Color(0xFF56A8FF);
  static const cosmicA = Color(0xFFE59CFF); // gradiente cósmico (texto ceremonial)
  static const cosmicB = Color(0xFFB79CFF);
  static const cosmicC = Color(0xFF56A8FF);
  static const starField = Color(0xFFE6ECF8);

  // -------------------------------------------------------------------------
  // GxSurface — getters dinámicos que reflejan el modo global de superficie.
  //
  // solid → variantes del color de fondo de componentes (globalComponentBgColor)
  //   con ligeros ajustes de ligereza vía HSLColor para diferenciar fill/panel/card.
  // glass/tint/enhancedGlass → color de fondo de componentes como tinte del glass
  //   (la translucidez la manejan frosted() / GlassSurface al renderizar).
  //
  // Cambiar el color de componentes en SettingsDrawer → TODO pixel reacciona.
  // -------------------------------------------------------------------------

  // Color de fondo de componentes sin procesar (raw, sin opacidad ni ajustes).
  // Lo usan los wrappers (frosted, GlassSurface) para construir gradientes y
  // tintes con la opacidad adecuada a cada modo de superficie.
  // Público dentro del paquete: gallery_fx.dart lo consume directamente.
  static Color get componentBgBase => DrasusThemeState.globalComponentBgColor;

  // Verdadero si el modo activo es sólido (sin translucidez).
  static bool get _isSolidMode =>
      DrasusThemeState.globalSurfaceMode == DrasusSurfaceMode.solid;

  // Ajusta la ligereza de un color en un delta porcentual (positivo = más claro).
  // Usa HSLColor para preservar tono y saturación al variar solo la luminosidad.
  static Color _adjustLightness(Color c, double deltaPercent) {
    final hsl = HSLColor.fromColor(c);
    return hsl
        .withLightness((hsl.lightness + deltaPercent / 100).clamp(0.0, 1.0))
        .toColor();
  }

  /// Relleno base de superficie.
  /// solid → color de componentes tal cual; glass/tint/enhancedGlass → mismo color.
  /// Los wrappers aplican la opacidad adecuada según el modo al renderizar.
  static Color get surfaceFill => componentBgBase;

  /// Panel: ligeramente más claro que fill (+4% de ligereza en solid).
  /// En glass/tint/enhancedGlass se comporta igual que surfaceFill (los wrappers
  /// deciden la opacidad según el modo).
  static Color get surfacePanel =>
      _isSolidMode ? _adjustLightness(componentBgBase, 4) : componentBgBase;

  /// Card: un poco más claro que panel (+4% adicional en solid).
  /// En glass/tint/enhancedGlass se comporta igual que surfaceFill.
  static Color get surfaceCard =>
      _isSolidMode ? _adjustLightness(componentBgBase, 8) : componentBgBase;

  // -------------------------------------------------------------------------
  // Tokens dinámicos de texto — leen el espejo estático _globalTextColor.
  // Uso: Gx.textBase donde antes se usaba Gx.textPrimary (hardcoded).
  // Los const textPrimary/etc. siguen disponibles como referencia raw interna.
  // -------------------------------------------------------------------------

  // Color de texto base efectivo (override manual o auto por paleta activa).
  static Color get textBase => DrasusThemeState.globalTextColor;

  // Texto secundario: mismo color base a 75% de opacidad.
  static Color get textBaseSecondary =>
      DrasusThemeState.globalTextColor.withOpacity(0.75);

  // Etiqueta: mismo color base a 55% de opacidad.
  static Color get textBaseLabel =>
      DrasusThemeState.globalTextColor.withOpacity(0.55);

  // Texto inactivo/muted: mismo color base a 37% de opacidad.
  static Color get textBaseMuted =>
      DrasusThemeState.globalTextColor.withOpacity(0.37);

  // -------------------------------------------------------------------------
  // Tokens dinámicos de paleta — leen el espejo estático _globalCanvasBase.
  // Uso: Gx.canvasBase donde antes se usaba Gx.deepSpace (hardcoded) como
  // fondo de lienzo en secciones que no tienen acceso a Theme.of(context).

  // Color del lienzo base (deepSpace de la paleta activa).
  static Color get canvasBase => DrasusThemeState.globalCanvasBase;

  // Color raised/hover (surfaceRaised de la paleta activa).
  static Color get surfaceRaisedDynamic => DrasusThemeState.globalSurfaceRaised;

  // -------------------------------------------------------------------------
  // Tokens dinámicos de borde y énfasis.
  // Regla: borde estructural global = énfasis; los colores semánticos
  // (optimaCyan, alertAmber, criticalCrimson) se usan solo como señalización
  // interna del componente, nunca como borde global.
  // -------------------------------------------------------------------------

  // Color de énfasis dinámico (lee el espejo estático _globalAccent).
  static Color get accentDynamic => DrasusThemeState.globalAccent;

  // Borde estructural global tintado con el énfasis activo al 35% de opacidad.
  // Úsalo donde antes se usaba Gx.borderPanel como borde genérico.
  static Color get borderBase =>
      DrasusThemeState.globalAccent.withOpacity(0.35);

  // -------------------------------------------------------------------------
  // Grosor de borde — valores canónicos usados en toda la UI.
  // -------------------------------------------------------------------------

  // Hairline: borde estructural mínimo (paneles, chips, separadores).
  static const double borderHairline = 1.0;

  // Focus: borde de foco activo (inputs, controles seleccionados).
  static const double borderFocus = 1.5;

  // -------------------------------------------------------------------------
  // Escala de espaciado base 4px — tokens Dart de DESIGN.md §Spacing.
  // Úsalos en lugar de literales numéricos en padding/margin/gaps.
  // -------------------------------------------------------------------------
  static const double space4  = 4.0;
  static const double space8  = 8.0;
  static const double space12 = 12.0;
  static const double space16 = 16.0;
  static const double space24 = 24.0;
  static const double space32 = 32.0;
  static const double space48 = 48.0;
  static const double space64 = 64.0;

  // --- Vidrio Apple ---
  static const glassEdgeOpacity = 0.28;
  static const glassBlur = 36.0;

  // --- Radios (Border Radius de DESIGN.md) ---
  static const rPanel = 11.0;
  static const rChrome = 14.0;
  static const rButton = 10.0;
  static const rInput = 10.0;
  static const rChip = 8.0;
  static const rTooltip = 12.0;

  // --- Tipografía ---
  // Tres voces según DESIGN.md, servidas por los assets embebidos en
  // assets/fonts/ (declarados en pubspec.yaml). 100% offline, sin google_fonts
  // en runtime. Los nombres de familia deben coincidir con los de pubspec.yaml.
  //
  // displayGrotesque → Rajdhani (títulos)
  // uiSans           → Rajdhani (UI general)
  // dataMono         → Rajdhani (datos/números)
  //
  // Las firmas de los helpers son idénticas a las anteriores (con GoogleFonts)
  // para no tener que tocar ningún callsite en el resto de la galería.
  // NOTA: getters dinámicos, leen el espejo estático de DrasusThemeState.
  // Cambiar la fuente en SettingsDrawer → TODO texto de la galería reacciona.
  static String get fontDisplay => DrasusThemeState.globalFontDisplay;
  static String get fontSans => DrasusThemeState.globalFontSans;
  static String get fontMono => DrasusThemeState.globalFontMono;

  // Helper: retorna un TextStyle con Rajdhani (display grotesco).
  // NOTA: SIN default de color — el llamante DEBE elegir explícitamente
  // entre textBase / textBaseLabel / textBaseSecondary / textBaseMuted.
  static TextStyle displayGrotesque({
    double fontSize = 14,
    double height = 1.3,
    required Color color,
    FontWeight weight = FontWeight.w500,
    double letterSpacing = 0,
  }) =>
      TextStyle(
        fontFamily: fontDisplay,
        fontSize: fontSize,
        height: height,
        color: color,
        fontWeight: weight,
        letterSpacing: letterSpacing,
      );

  // Helper: retorna un TextStyle con Rajdhani (sans de UI).
  // NOTA: SIN default de color — el llamante DEBE elegir explícitamente.
  static TextStyle uiSans({
    double fontSize = 14,
    double height = 1.5,
    required Color color,
    FontWeight weight = FontWeight.w400,
  }) =>
      TextStyle(
        fontFamily: fontSans,
        fontSize: fontSize,
        height: height,
        color: color,
        fontWeight: weight,
      );

  // Helper: retorna un TextStyle con Rajdhani (datos y números).
  // NOTA: SIN default de color — el llamante DEBE elegir explícitamente.
  static TextStyle dataMono({
    double fontSize = 13,
    double height = 1.4,
    required Color color,
    FontWeight weight = FontWeight.w400,
  }) =>
      TextStyle(
        fontFamily: fontMono,
        fontSize: fontSize,
        height: height,
        color: color,
        fontWeight: weight,
      );

  // Type Scale de DESIGN.md — DELEGAN al theme provider vía DrasusThemeState.
  // Prohibido hardcodear fontSize/color aquí: todo viene de _syncStyleScale()
  // en DrasusThemeState, que a su vez deriva del TextTheme de buildThemeData().
  static TextStyle get microLabel => DrasusThemeState.globalMicroLabel;
  static TextStyle get label => DrasusThemeState.globalLabel;
  static TextStyle get body => DrasusThemeState.globalBody;
  static TextStyle get bodySecondary => DrasusThemeState.globalBodySecondary;
  static TextStyle get subheading => DrasusThemeState.globalSubheading;
  static TextStyle get panelTitle => DrasusThemeState.globalPanelTitle;
  static TextStyle get sectionHeading => DrasusThemeState.globalSectionHeading;
  static TextStyle get zuiTitle => DrasusThemeState.globalZuiTitle;
  static TextStyle get dataSmall => DrasusThemeState.globalDataSmall;
  static TextStyle get dataHero => DrasusThemeState.globalDataHero;

  // --- Gradientes (compatibles entre los colores del sistema) ---
  // Cada gradiente se queda DENTRO de una familia semántica, así el color sigue
  // significando un estado (Regla Cero) aunque ahora respire con degradado.
  static const gradOptima = [optimaCyan, optimaTeal];
  static const gradReactor = [reactorGreen, optimaCyan];
  static const gradTransition = [transitionIndigo, transitionBlue];
  static const gradAurora = [transitionPurple, transitionIndigo, transitionBlue];
  static const gradAlert = [alertAmber, alertOrange];
  static const gradCritical = [criticalRed, criticalCrimson];
  static const gradCosmic = [cosmicA, cosmicB, cosmicC];

  // Degradado lineal helper.
  static LinearGradient linear(List<Color> colors,
          {AlignmentGeometry begin = Alignment.topLeft,
          AlignmentGeometry end = Alignment.bottomRight}) =>
      LinearGradient(colors: colors, begin: begin, end: end);

  // --- Glow reutilizable (el "poder" Reflect) ---
  // Halo de color para botones, nodos, líneas, chips, focos.
  static List<BoxShadow> glow(Color c,
          {double blur = 16, double spread = 0, double opacity = 0.45}) =>
      [BoxShadow(color: c.withOpacity(opacity), blurRadius: blur, spreadRadius: spread)];

  // Glow doble: núcleo intenso + halo amplio, para acentos poderosos.
  static List<BoxShadow> glowStrong(Color c, [double k = 1.0]) => [
        BoxShadow(color: c.withOpacity((0.55 * k).clamp(0.0, 1.0)), blurRadius: 10 * k, spreadRadius: 0),
        BoxShadow(color: c.withOpacity((0.28 * k).clamp(0.0, 1.0)), blurRadius: 30 * k, spreadRadius: 2 * k),
      ];

  // Sombra de texto para neón "encendido" (glow en las letras).
  static List<Shadow> textGlow(Color c, [double blur = 8]) =>
      [Shadow(color: c.withOpacity(0.7), blurRadius: blur)];

  // Borde de luz tenue (rim-light Apple) para superficies de vidrio.
  static Border rimLight([double opacity = 0.10]) =>
      Border.all(color: textPrimary.withOpacity(opacity));

  // ---------------------------------------------------------------------------
  // Tokens de iconos — Iconsax Plus (estilo Linear, 896 iconos, const IconData).
  // Capa de indirección: cada token mapea un rol semántico a un IconData.
  // Para cambiar el icono de un rol, se edita solo aquí.
  // ---------------------------------------------------------------------------

  // Navegación y estructura.
  static const IconData iconHub = IconsaxPlusLinear.graph;
  static const IconData iconDashboard = IconsaxPlusLinear.element;
  static const IconData iconPalette = IconsaxPlusLinear.element_plus;
  static const IconData iconClock = IconsaxPlusLinear.clock;
  static const IconData iconJobs = IconsaxPlusLinear.element_1;
  static const IconData iconAudit = IconsaxPlusLinear.shield_tick;

  // Estado y vitalidad.
  static const IconData iconBolt = IconsaxPlusLinear.chart_1;
  static const IconData iconWarning = IconsaxPlusLinear.warning_2;
  static const IconData iconDanger = IconsaxPlusLinear.danger;
  static const IconData iconScience = IconsaxPlusLinear.magicpen;
  static const IconData iconChart = IconsaxPlusLinear.chart;
  static const IconData iconBlurOn = IconsaxPlusLinear.chart_2;

  // Acciones.
  static const IconData iconPlay = IconsaxPlusLinear.play;
  static const IconData iconPause = IconsaxPlusLinear.pause;
  static const IconData iconRefresh = IconsaxPlusLinear.refresh;
  static const IconData iconAdd = IconsaxPlusLinear.add;
  static const IconData iconCheck = IconsaxPlusLinear.tick_circle;
  static const IconData iconChevronDown = IconsaxPlusLinear.arrow_down;
}
