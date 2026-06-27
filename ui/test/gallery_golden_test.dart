// Golden tests de la Galería de Componentes de Drasus Engine.
//
// Por qué secciones separadas y no la galería completa:
//   La galería es un scroll largo (~4000px de alto). El viewport de test
//   por defecto (800x600) solo muestra el primer fragmento y parte del
//   contenido queda fuera del área renderizable. Renderizar cada sección
//   en su propio golden garantiza cobertura total sin overflow.
//
// Por qué se cargan fuentes manualmente (FontLoader):
//   El harness de test de Flutter carga assets declarados en pubspec.yaml pero
//   NO registra automáticamente las familias tipográficas; el motor de texto
//   usa "Ahem" por defecto y renderiza cajas en lugar de glifos reales.
//   Cargar cada .ttf con FontLoader y llamar loadFontFromList() registra la
//   familia en el motor antes de hacer el primer pump, de modo que los goldens
//   muestran tipografía real legible.
//
// Cómo generar los PNG:
//   cd ui
//   flutter test --update-goldens test/gallery_golden_test.dart
//
// Los PNG quedan en: ui/test/goldens/
// Están versionados en git para detectar regresiones visuales.

import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:drasus_ui/gallery/gallery_tab.dart';

// Carga las tres familias tipográficas embebidas desde assets/fonts/.
// Debe llamarse una vez antes de los tests (setUpAll) para registrar
// las familias en el motor de texto y que los goldens muestren glifos reales.
Future<void> _loadEmbeddedFonts() async {
  // Mapa: nombre de familia → lista de archivos .ttf asociados.
  // Refleja exactamente la declaración en pubspec.yaml.
  final fonts = {
    'SpaceGrotesk': ['assets/fonts/SpaceGrotesk-Medium.ttf'],
    'Inter': [
      'assets/fonts/Inter-Regular.ttf',
      'assets/fonts/Inter-Medium.ttf',
    ],
    'JetBrainsMono': [
      'assets/fonts/JetBrainsMono-Regular.ttf',
      'assets/fonts/JetBrainsMono-Medium.ttf',
    ],
  };

  for (final entry in fonts.entries) {
    final loader = FontLoader(entry.key);
    for (final path in entry.value) {
      // Lee el .ttf desde disco (ruta relativa al directorio ui/).
      // En el harness de test la cwd es el directorio del paquete (ui/).
      final bytes = File(path).readAsBytesSync();
      loader.addFont(Future.value(ByteData.view(bytes.buffer)));
    }
    // Registra la familia en el motor de texto de Flutter.
    await loader.load();
  }
}

void main() {
  // Carga fuentes una sola vez antes de todos los tests de este archivo.
  setUpAll(_loadEmbeddedFonts);

  testWidgets('gallery_full_scroll', (WidgetTester tester) async {
    // Viewport para el layout maestro-detalle (STORY-022):
    // 1440px de ancho captura el panel lateral de 260px + el panel de detalle.
    // 1200px de alto es suficiente para ver la categoría inicial completa;
    // ya no se necesitan 5000px porque el contenido no es un scroll único —
    // cada categoría tiene su propio SingleChildScrollView en el panel de detalle.
    tester.view.physicalSize = const Size(1440, 1200);
    tester.view.devicePixelRatio = 1.0;

    await tester.pumpWidget(
      const MaterialApp(
        debugShowCheckedModeBanner: false,
        home: Scaffold(body: GalleryTab()),
      ),
    );

    // Deja que los timers y animaciones de estado inicial se resuelvan.
    await tester.pump();

    // Compara con el golden o lo genera si no existe (--update-goldens).
    await expectLater(
      find.byType(GalleryTab),
      matchesGoldenFile('goldens/gallery_full_scroll.png'),
    );

    addTearDown(tester.view.resetPhysicalSize);
  });

  testWidgets('gallery_fundamentos', (WidgetTester tester) async {
    // Viewport enfocado en la sección superior (hero + fundamentos).
    tester.view.physicalSize = const Size(1200, 900);
    tester.view.devicePixelRatio = 1.0;

    await tester.pumpWidget(
      const MaterialApp(
        debugShowCheckedModeBanner: false,
        home: Scaffold(body: GalleryTab()),
      ),
    );
    await tester.pump();

    // Golden de la sección superior visible sin scroll.
    await expectLater(
      find.byType(GalleryTab),
      matchesGoldenFile('goldens/gallery_fundamentos.png'),
    );

    addTearDown(tester.view.resetPhysicalSize);
  });
}
