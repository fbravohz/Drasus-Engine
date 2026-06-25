// Design tokens centralizados de Drasus Engine (ADR-0138).
// Cuatro ThemeExtension<T>: vidrio, motion, superficies y paleta.
// Ningún token de estas familias vive como constante suelta fuera de aquí.
//
// Los defaults se tomaron de gallery_tokens.dart y drasus_theme.dart
// (no se inventó ningún valor). Flutter exige copyWith + lerp para que
// las transiciones animadas entre temas funcionen.

import 'package:flutter/material.dart';
import '../drasus_theme.dart';

// ---------------------------------------------------------------------------
// DrasusGlass — vidrio Apple (chrome translúcido + rim-light).
// ---------------------------------------------------------------------------
class DrasusGlass extends ThemeExtension<DrasusGlass> {
  // Relleno base del vidrio (60% alpha). Default: Gx.glassFill.
  final Color fill;
  // Sigma del desenfoque del BackdropFilter. Default: 36 (Gx.glassBlur).
  final double blurSigma;
  // Tinte blanco-azulado interior (milk glass). Default: Gx.glassTint.
  final Color tint;
  // Sigma del halo del borde (rim-light). Default: 30 (Gx.glassRimBlur).
  final double rimBlur;
  // Opacidad del borde luminoso. Default: 0.22 (Gx.glassEdgeOpacity).
  final double edgeOpacity;
  // Color del rim-light. Default: 0x0DA096FF (canónico ADR-0138).
  final Color rimColor;

  const DrasusGlass({
    required this.fill,
    required this.blurSigma,
    required this.tint,
    required this.rimBlur,
    required this.edgeOpacity,
    required this.rimColor,
  });

  // Instancia por defecto con los valores canónicos del bunker nocturno.
  static const defaults = DrasusGlass(
    fill: Color(0x40F0F2FF),
    blurSigma: 36.0,
    tint: Color(0x14AAAAFF),
    rimBlur: 30.0,
    edgeOpacity: 0.28,
    rimColor: Color(0x20A096FF),
  );

  @override
  DrasusGlass copyWith({
    Color? fill,
    double? blurSigma,
    Color? tint,
    double? rimBlur,
    double? edgeOpacity,
    Color? rimColor,
  }) =>
      DrasusGlass(
        fill: fill ?? this.fill,
        blurSigma: blurSigma ?? this.blurSigma,
        tint: tint ?? this.tint,
        rimBlur: rimBlur ?? this.rimBlur,
        edgeOpacity: edgeOpacity ?? this.edgeOpacity,
        rimColor: rimColor ?? this.rimColor,
      );

  @override
  DrasusGlass lerp(ThemeExtension<DrasusGlass>? other, double t) {
    if (other is! DrasusGlass) return this;
    return DrasusGlass(
      fill: Color.lerp(fill, other.fill, t)!,
      blurSigma: _lerpDouble(blurSigma, other.blurSigma, t),
      tint: Color.lerp(tint, other.tint, t)!,
      rimBlur: _lerpDouble(rimBlur, other.rimBlur, t),
      edgeOpacity: _lerpDouble(edgeOpacity, other.edgeOpacity, t),
      rimColor: Color.lerp(rimColor, other.rimColor, t)!,
    );
  }

  @override
  bool operator ==(Object other) =>
      other is DrasusGlass &&
      other.fill == fill &&
      other.blurSigma == blurSigma &&
      other.tint == tint &&
      other.rimBlur == rimBlur &&
      other.edgeOpacity == edgeOpacity &&
      other.rimColor == rimColor;

  @override
  int get hashCode => Object.hash(fill, blurSigma, tint, rimBlur, edgeOpacity, rimColor);
}

// ---------------------------------------------------------------------------
// DrasusMotion — duraciones universales de animación (Motion Philosophy).
// ---------------------------------------------------------------------------
class DrasusMotion extends ThemeExtension<DrasusMotion> {
  // Duración del odómetro numérico. Default: 1000ms (QuantKpiOdometerRow).
  final int odometerMs;
  // Duración del arco animado. Default: 1000ms (QuantRadialGauge).
  final int arcMs;
  // Duración del scan eléctrico. Default: 1400ms (ElectricLineChart).
  final int scanMs;
  // Decaimiento exponencial del scan. Default: 8.0 (electricIntensity).
  final double scanDecay;
  // Duración del path drawing. Default: 1500ms (STORY-019).
  final int pathDrawMs;

  const DrasusMotion({
    required this.odometerMs,
    required this.arcMs,
    required this.scanMs,
    required this.scanDecay,
    required this.pathDrawMs,
  });

  static const defaults = DrasusMotion(
    odometerMs: 1000,
    arcMs: 1000,
    scanMs: 1400,
    scanDecay: 8.0,
    pathDrawMs: 1500,
  );

  @override
  DrasusMotion copyWith({
    int? odometerMs,
    int? arcMs,
    int? scanMs,
    double? scanDecay,
    int? pathDrawMs,
  }) =>
      DrasusMotion(
        odometerMs: odometerMs ?? this.odometerMs,
        arcMs: arcMs ?? this.arcMs,
        scanMs: scanMs ?? this.scanMs,
        scanDecay: scanDecay ?? this.scanDecay,
        pathDrawMs: pathDrawMs ?? this.pathDrawMs,
      );

  @override
  DrasusMotion lerp(ThemeExtension<DrasusMotion>? other, double t) {
    if (other is! DrasusMotion) return this;
    return DrasusMotion(
      odometerMs: _lerpInt(odometerMs, other.odometerMs, t),
      arcMs: _lerpInt(arcMs, other.arcMs, t),
      scanMs: _lerpInt(scanMs, other.scanMs, t),
      scanDecay: _lerpDouble(scanDecay, other.scanDecay, t),
      pathDrawMs: _lerpInt(pathDrawMs, other.pathDrawMs, t),
    );
  }

  @override
  bool operator ==(Object other) =>
      other is DrasusMotion &&
      other.odometerMs == odometerMs &&
      other.arcMs == arcMs &&
      other.scanMs == scanMs &&
      other.scanDecay == scanDecay &&
      other.pathDrawMs == pathDrawMs;

  @override
  int get hashCode => Object.hash(odometerMs, arcMs, scanMs, scanDecay, pathDrawMs);
}

// ---------------------------------------------------------------------------
// DrasusSurfaces — pila sólida escalada desde deepSpace.
// panelRaised y panelBorder se mapean desde surfaceRaised y borderPanel de Gx.
// ---------------------------------------------------------------------------
class DrasusSurfaces extends ThemeExtension<DrasusSurfaces> {
  final Color deepSpace;
  final Color navRail;
  final Color panelSolid;
  final Color panelRaised;
  final Color panelBorder;

  const DrasusSurfaces({
    required this.deepSpace,
    required this.navRail,
    required this.panelSolid,
    required this.panelRaised,
    required this.panelBorder,
  });

  // Defaults del bunker nocturno (drasus_theme.dart _kPalettes[bunker] + Gx).
  static const defaults = DrasusSurfaces(
    deepSpace: Color(0xFF04050E),
    navRail: Color(0xFF060819),
    panelSolid: Color(0xFF090D1F),
    panelRaised: Color(0xFF111833), // Gx.surfaceRaised
    panelBorder: Color(0xFF17213A), // Gx.borderPanel
  );

  // Construye la pila desde una DrasusSurfacePalette activa.
  // panelRaised toma surfaceRaised; panelBorder se conserva (invariante).
  factory DrasusSurfaces.fromPalette(DrasusSurfacePalette p) => DrasusSurfaces(
        deepSpace: p.deepSpace,
        navRail: p.navRail,
        panelSolid: p.panelSolid,
        panelRaised: p.surfaceRaised,
        panelBorder: defaults.panelBorder,
      );

  @override
  DrasusSurfaces copyWith({
    Color? deepSpace,
    Color? navRail,
    Color? panelSolid,
    Color? panelRaised,
    Color? panelBorder,
  }) =>
      DrasusSurfaces(
        deepSpace: deepSpace ?? this.deepSpace,
        navRail: navRail ?? this.navRail,
        panelSolid: panelSolid ?? this.panelSolid,
        panelRaised: panelRaised ?? this.panelRaised,
        panelBorder: panelBorder ?? this.panelBorder,
      );

  @override
  DrasusSurfaces lerp(ThemeExtension<DrasusSurfaces>? other, double t) {
    if (other is! DrasusSurfaces) return this;
    return DrasusSurfaces(
      deepSpace: Color.lerp(deepSpace, other.deepSpace, t)!,
      navRail: Color.lerp(navRail, other.navRail, t)!,
      panelSolid: Color.lerp(panelSolid, other.panelSolid, t)!,
      panelRaised: Color.lerp(panelRaised, other.panelRaised, t)!,
      panelBorder: Color.lerp(panelBorder, other.panelBorder, t)!,
    );
  }

  @override
  bool operator ==(Object other) =>
      other is DrasusSurfaces &&
      other.deepSpace == deepSpace &&
      other.navRail == navRail &&
      other.panelSolid == panelSolid &&
      other.panelRaised == panelRaised &&
      other.panelBorder == panelBorder;

  @override
  int get hashCode => Object.hash(deepSpace, navRail, panelSolid, panelRaised, panelBorder);
}

// ---------------------------------------------------------------------------
// DrasusPalette — acento dinámico + paleta de fondo activa.
// ---------------------------------------------------------------------------
class DrasusPalette extends ThemeExtension<DrasusPalette> {
  // Acento elegido por el usuario (chrome interactivo). Default: transitionIndigo.
  final Color accentColor;
  // Paleta de fondo activa (enum existente en drasus_theme.dart).
  final DrasusBackgroundPalette backgroundPalette;

  const DrasusPalette({
    required this.accentColor,
    required this.backgroundPalette,
  });

  static const defaults = DrasusPalette(
    accentColor: Color(0xFF9A8CFF), // transitionIndigo
    backgroundPalette: DrasusBackgroundPalette.bunker,
  );

  @override
  DrasusPalette copyWith({
    Color? accentColor,
    DrasusBackgroundPalette? backgroundPalette,
  }) =>
      DrasusPalette(
        accentColor: accentColor ?? this.accentColor,
        backgroundPalette: backgroundPalette ?? this.backgroundPalette,
      );

  @override
  DrasusPalette lerp(ThemeExtension<DrasusPalette>? other, double t) {
    if (other is! DrasusPalette) return this;
    // El enum no interpola: usa step al 50% (convención Flutter para enums).
    return DrasusPalette(
      accentColor: Color.lerp(accentColor, other.accentColor, t)!,
      backgroundPalette: t < 0.5 ? backgroundPalette : other.backgroundPalette,
    );
  }

  @override
  bool operator ==(Object other) =>
      other is DrasusPalette &&
      other.accentColor == accentColor &&
      other.backgroundPalette == backgroundPalette;

  @override
  int get hashCode => Object.hash(accentColor, backgroundPalette);
}

// Helpers de lerp internos (enteros y dobles con tolerancia).
double _lerpDouble(double a, double b, double t) => a + (b - a) * t;
int _lerpInt(int a, int b, double t) => (a + (b - a) * t).round();
