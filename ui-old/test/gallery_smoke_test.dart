// Test de humo para la Galería de Componentes (pestaña "Components").
//
// POR QUÉ CAMBIÓ RESPECTO AL DISEÑO VIEJO:
//   El diseño anterior renderizaba TODOS los ~150 componentes a la vez en un
//   Column gigante con scroll. Cualquier texto de cualquier sección era hallable
//   en el árbol de widgets sin interacción, y el test comprobaba textos de
//   múltiples secciones no renderizadas (p.ej. 'EJECUTAR', 'Autopsia', 'node-07').
//
//   Tras STORY-022, la galería pasó a un modelo maestro-detalle:
//   - El panel lateral es un ListView.builder — solo construye los ítems visibles
//     en el viewport en ese momento (no todos los títulos de categoría están en
//     el árbol si están fuera del área visible).
//   - El panel de detalle solo renderiza la categoría/entrada seleccionada.
//     El resto son GalleryEntry.builder, funciones que no se invocan hasta navegar.
//
//   Por tanto, el test debe verificar invariantes del NUEVO diseño:
//   (a) la galería monta sin lanzar excepción
//   (b) el encabezado del panel lateral ("Drasus" + "Design System") está presente
//   (c) al menos una categoría visible en el panel lateral (las primeras se muestran)
//   (d) el panel de detalle muestra la categoría inicial con sus componentes
//   (e) la navegación funciona: tocar una categoría visible actualiza el detalle
//
// Nota sobre el viewport: 1280×900 garantiza que el layout de dos paneles
// (260px lateral + Expanded detalle) quepa sin overflow espurio.
//
// Correr con: flutter test test/gallery_smoke_test.dart

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:drasus_ui/gallery/gallery_tab.dart';

void main() {
  testWidgets('gallery_renders_and_navigates',
      (WidgetTester tester) async {
    // Viewport amplio para el layout maestro-detalle de dos paneles.
    tester.view.physicalSize = const Size(1280, 900);
    tester.view.devicePixelRatio = 1.0;
    addTearDown(tester.view.resetPhysicalSize);

    // (a) Monta la galería — si cualquier widget lanza durante build, pumpWidget lo propaga.
    await tester.pumpWidget(
      const MaterialApp(home: Scaffold(body: GalleryTab())),
    );
    await tester.pump();

    // (b) El hero del panel lateral tiene el título del design system.
    // "Drasus" y "Design System" son dos Text separados en el ShaderMask del hero.
    expect(find.text('Drasus'), findsOneWidget);
    expect(find.text('Design System'), findsOneWidget);

    // (c) La primera categoría del catálogo ('Fundamentos') aparece en el panel
    // lateral (siempre visible por ser la primera en la lista) y en el panel de
    // detalle (es la categoría seleccionada por defecto).
    // findsWidgets porque puede aparecer en ambos paneles a la vez.
    expect(find.text('Fundamentos'), findsWidgets);

    // (d) El panel de detalle renderiza la categoría inicial: algunos componentes
    // de 'Fundamentos' deben estar en el árbol como texto de sus ítems en el sidebar.
    // 'Paleta — superficies' es la primera entrada de Fundamentos.
    expect(find.text('Paleta — superficies'), findsWidgets);

    // (e) Navegación real: la segunda categoría visible es 'Layout y estructura'.
    // Al tocarla, el detalle debe mostrar su título como encabezado de sección.
    // El sidebar siempre la muestra (está en el viewport inicial de 900px de alto).
    await tester.tap(find.text('Layout y estructura').first);
    await tester.pump();

    // Tras la navegación, 'Layout y estructura' aparece en el encabezado del detalle
    // además de en el sidebar → al menos 2 instancias en el árbol.
    expect(find.text('Layout y estructura'), findsAtLeastNWidgets(2));
  });
}
