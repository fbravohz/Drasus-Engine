// Test de humo para la Galería de Componentes (pestaña "Components").
//
// La galería es render-only y NO llama al Bridge Rust, así que se puede montar
// directamente. Verifica que el árbol completo construye sin lanzar excepción
// y que las secciones y varios componentes representativos están presentes.
//
// Correr con: flutter test ui/test/gallery_smoke_test.dart

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:drasus_ui/gallery/gallery_tab.dart';

void main() {
  testWidgets('gallery_renders_sections_and_components',
      (WidgetTester tester) async {
    // Monta la galería dentro de un Scaffold para darle restricciones de tamaño.
    // Si cualquier componente lanza durante build/paint, pumpWidget lo propaga.
    await tester.pumpWidget(
      const MaterialApp(home: Scaffold(body: GalleryTab())),
    );

    // Encabezado ceremonial de la vitrina.
    expect(find.text('Drasus Design System'), findsOneWidget);

    // Encabezados de sección (el Column construye todos, aunque estén fuera de
    // pantalla, así que find.text los encuentra en el árbol).
    expect(find.text('Fundamentos'), findsOneWidget);
    expect(find.text('Botones y acciones'), findsOneWidget);
    expect(find.text('Data-viz (dominio Drasus)'), findsOneWidget);
    expect(find.text('Núcleo Drasus'), findsOneWidget);

    // Componentes representativos de distintas categorías.
    // findsWidgets en lugar de findsOneWidget porque con el catálogo completo
    // hay múltiples secciones que pueden repetir estos textos de muestra.
    expect(find.text('EJECUTAR'), findsWidgets); // botón de acción viva
    expect(find.text('Autopsia'), findsWidgets); // portada de autopsia
    expect(find.text('node-07'), findsWidgets); // fila de tabla densa
  });
}
