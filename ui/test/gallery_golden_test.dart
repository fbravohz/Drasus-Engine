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
import 'package:drasus_ui/theme/theme_scope.dart';
import 'package:shared_preferences/shared_preferences.dart';

// Carga la tipografía embebida desde assets/fonts/. Debe llamarse una vez
// antes de los tests (setUpAll) para registrar la familia en el motor de
// texto y que los goldens muestren glifos reales.
//
// Por qué una sola familia (corregido 2026-07-31): este mapa listaba
// SpaceGrotesk, Inter y JetBrainsMono, tres familias que STORY-023 sustituyó
// por Rajdhani borrando sus .ttf. `File.readAsBytesSync` sobre un archivo
// inexistente lanzaba en `setUpAll`, así que ESTE ARCHIVO llevaba en rojo
// desde entonces y arrastraba a todo `flutter test` con él. Un suite que
// nunca puede ponerse verde convierte "Some tests failed" en ruido de fondo
// y esconde la siguiente regresión real: el mapa debe reflejar `pubspec.yaml`
// y nada más.
Future<void> _loadEmbeddedFonts() async {
  // Mapa: nombre de familia → lista de archivos .ttf asociados.
  // Refleja exactamente la declaración en pubspec.yaml.
  final fonts = {
    'Rajdhani': [
      'assets/fonts/Rajdhani-Regular.ttf',
      'assets/fonts/Rajdhani-Medium.ttf',
      'assets/fonts/Rajdhani-SemiBold.ttf',
      'assets/fonts/Rajdhani-Bold.ttf',
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
  // Carga fuentes e inicializa el tema una sola vez antes de todos los tests.
  //
  // Por qué también el tema (añadido 2026-07-31): registrar la familia con
  // FontLoader NO basta. La escala tipográfica (`Gx.panelTitle`, `Gx.body`,
  // `Gx.dataSmall`…) vive en espejos estáticos de `ThemeState` que nacen como
  // `TextStyle(fontSize: N)` SIN familia y solo la reciben cuando `load()`
  // dispara `_syncStyleScale()`. Sin esa llamada, todo texto que use la escala
  // caía en Ahem y se congelaba en el golden como cajas grises — justo lo que
  // el encabezado de este archivo dice que se quiere evitar. El golden medía
  // entonces media pantalla real y media pantalla de cajas.
  setUpAll(() async {
    await _loadEmbeddedFonts();
    // Sin valores persistidos: fuerza los defaults, que es lo que debe
    // capturar el golden (un tema que el usuario no ha tocado).
    SharedPreferences.setMockInitialValues({});
    await ThemeState().load();
  });

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
