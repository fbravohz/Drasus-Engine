// Tokens de diseño de la Galería de Componentes.
// ESTE es el ÚNICO archivo donde los hex de docs/DESIGN.md se vuelven Color(0xFF…).
// Todos los componentes de la galería leen de aquí: cambiar un token en
// docs/DESIGN.md se refleja editando solo este archivo (spec-driven).
//
// Tipografía: los helpers displayGrotesque / uiSans / dataMono usan los .ttf
// embebidos en assets/fonts/ (declarados en pubspec.yaml). Google Fonts ya NO
// se usa para servir las familias en runtime: las fuentes son 100% offline.
// google_fonts sigue presente en pubspec.yaml por si otras partes del proyecto
// lo necesitan, pero los helpers de la galería NO lo invocan.

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
  // glass:  glassFill (0x40F0F2FF) — translúcido uniforme en TODOS los niveles.
  //         El BackdropFilter + rim-light lo añaden frosted() y GlassSurface.
  // tint:   glassFill — mismo fill, sin blur ni rim.
  // solid:  panelSolid / cardInner — colores oscuros tradicionales.
  //
  // Cambiar el modo en SettingsDrawer → TODO pixel de la UI reacciona.
  // -------------------------------------------------------------------------

  /// glass/tint → glassFill, solid → panelSolid
  static Color get surfaceFill {
    final mode = DrasusThemeState.globalSurfaceMode;
    return mode == DrasusSurfaceMode.solid ? Gx.panelSolid : Gx.glassFill;
  }

  /// glass/tint → glassFill, solid → panelSolid
  static Color get surfacePanel {
    final mode = DrasusThemeState.globalSurfaceMode;
    return mode == DrasusSurfaceMode.solid ? Gx.panelSolid : Gx.glassFill;
  }

  /// glass/tint → glassFill, solid → cardInner
  static Color get surfaceCard {
    final mode = DrasusThemeState.globalSurfaceMode;
    return mode == DrasusSurfaceMode.solid ? Gx.cardInner : Gx.glassFill;
  }

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
  // displayGrotesque → SpaceGrotesk (sabor técnico/terminal, w500)
  // uiSans           → Inter (fuerza de trabajo, w400/500)
  // dataMono         → JetBrainsMono (números e IDs, w400/500)
  //
  // Las firmas de los helpers son idénticas a las anteriores (con GoogleFonts)
  // para no tener que tocar ningún callsite en el resto de la galería.
  static const fontDisplay = 'SpaceGrotesk'; // nombre de familia en pubspec.yaml
  static const fontSans = 'Inter'; // nombre de familia en pubspec.yaml
  static const fontMono = 'JetBrainsMono'; // nombre de familia en pubspec.yaml

  // Helper: retorna un TextStyle con Space Grotesk (display grotesco).
  // Usa la familia embebida en assets/fonts/SpaceGrotesk-Medium.ttf.
  static TextStyle displayGrotesque({
    double fontSize = 14,
    double height = 1.3,
    Color color = textPrimary,
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

  // Helper: retorna un TextStyle con Inter (sans de UI).
  // Usa las familias embebidas en assets/fonts/Inter-Regular.ttf y Inter-Medium.ttf.
  static TextStyle uiSans({
    double fontSize = 14,
    double height = 1.5,
    Color color = textPrimary,
    FontWeight weight = FontWeight.w400,
  }) =>
      TextStyle(
        fontFamily: fontSans,
        fontSize: fontSize,
        height: height,
        color: color,
        fontWeight: weight,
      );

  // Helper: retorna un TextStyle con JetBrains Mono (datos y números).
  // Usa las familias embebidas en assets/fonts/JetBrainsMono-Regular.ttf y -Medium.ttf.
  static TextStyle dataMono({
    double fontSize = 13,
    double height = 1.4,
    Color color = textPrimary,
    FontWeight weight = FontWeight.w400,
  }) =>
      TextStyle(
        fontFamily: fontMono,
        fontSize: fontSize,
        height: height,
        color: color,
        fontWeight: weight,
      );

  // Type Scale de DESIGN.md — se usan los helpers arriba para asignar familias.
  // Las constantes que usan google_fonts no pueden ser 'const' porque los
  // TextStyle devueltos incluyen referencias a FontLoader, que no es const.
  static TextStyle get microLabel =>
      uiSans(fontSize: 13, height: 1.3, color: textLabel);
  static TextStyle get label =>
      uiSans(fontSize: 14, height: 1.4, color: textLabel);
  static TextStyle get body =>
      uiSans(fontSize: 14, height: 1.5, color: textPrimary);
  static TextStyle get bodySecondary =>
      uiSans(fontSize: 14, height: 1.5, color: textSecondary);
  static TextStyle get subheading =>
      uiSans(fontSize: 16, height: 1.5, color: textPrimary);
  static TextStyle get panelTitle => displayGrotesque(
      fontSize: 16, height: 1.3, color: textSecondary, weight: FontWeight.w500);
  static TextStyle get sectionHeading => displayGrotesque(
      fontSize: 22,
      height: 1.15,
      color: textPrimary,
      weight: FontWeight.w500,
      letterSpacing: -0.4);
  static TextStyle get zuiTitle => displayGrotesque(
      fontSize: 40,
      height: 1.1,
      color: textPrimary,
      weight: FontWeight.w500,
      letterSpacing: -0.8);

  // Datos en JetBrains Mono (numStyle de DESIGN.md).
  static TextStyle get dataSmall =>
      dataMono(fontSize: 14, height: 1.4, color: textPrimary);
  static TextStyle get dataHero =>
      dataMono(fontSize: 28, height: 1.1, color: textPrimary);

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
