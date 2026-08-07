// Pruebas de los paneles laterales del lienzo: ancho por defecto medido,
// ajuste por arrastre con topes, y descripción de cada feature.
//
// Por qué existen: el propietario abrió el panel del *Transformador de Datos
// de Mercado* creyendo que era el *Asistente de Importación*, no encontró el
// selector de archivos y concluyó que la pantalla estaba inservible — con
// razón sobre lo que veía. Las dos features son vecinas en la lista y ninguna
// decía qué hacía (DEBT-045). La descripción existe para que esa confusión no
// sea posible sin abrir nada.

import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:drasus_ui/tabs/canvas_tab.dart';
import 'package:drasus_ui/theme/gx_tokens.dart';

// Monta el lienzo con espacio de sobra, para que el ancho de los paneles lo
// decida el propio widget y no la restricción del contenedor.
Future<void> _pumpCanvas(WidgetTester tester) async {
  tester.view.physicalSize = const Size(1600, 900);
  tester.view.devicePixelRatio = 1.0;
  addTearDown(tester.view.resetPhysicalSize);

  await tester.pumpWidget(const MaterialApp(
    home: Scaffold(body: CanvasTab()),
  ));
  await tester.pump();
}

// Ancho actual del panel de features, medido sobre el widget real.
double _sidePanelWidth(WidgetTester tester) =>
    tester.getSize(find.byKey(const ValueKey('canvas-side-panel'))).width;

// Arrastra una manija de ajuste horizontalmente.
Future<void> _dragHandle(
    WidgetTester tester, Finder handle, double dx) async {
  final gesture = await tester.startGesture(tester.getCenter(handle),
      kind: PointerDeviceKind.mouse);
  await gesture.moveBy(Offset(dx, 0));
  await gesture.up();
  await tester.pump();
}

void main() {
  testWidgets(
      'el panel de features nace tan ancho como el nombre de feature más largo',
      (tester) async {
    await _pumpCanvas(tester);

    // Oráculo calculado con la misma tipografía con que se dibuja, NO una
    // constante repetida: repetir el número no discriminaría — la prueba
    // mutaría junto al código.
    final painter = TextPainter(
      text: TextSpan(
        text: 'Transformador de Datos de Mercado',
        style: Gx.uiSans(fontSize: 12, color: Gx.textBase),
      ),
      textDirection: TextDirection.ltr,
      maxLines: 1,
    )..layout();

    // El nombre más largo del catálogo debe caber entero, sin recorte.
    expect(_sidePanelWidth(tester), greaterThanOrEqualTo(painter.width),
        reason: 'Si el panel nace más estrecho que el nombre más largo, ese '
            'nombre nace recortado — que es justo lo que el ancho medido '
            'existe para impedir.');

    // Y no puede ser desproporcionado respecto a lo que necesita.
    expect(_sidePanelWidth(tester), lessThan(painter.width + 100));
  });

  testWidgets('arrastrar la manija ensancha el panel de features',
      (tester) async {
    await _pumpCanvas(tester);
    final antes = _sidePanelWidth(tester);

    await _dragHandle(
        tester, find.byKey(const ValueKey('canvas-side-resize')), 80);

    expect(_sidePanelWidth(tester), greaterThan(antes));
  });

  testWidgets('el panel de features no se puede encoger hasta desaparecer',
      (tester) async {
    await _pumpCanvas(tester);

    // Arrastre exagerado hacia la izquierda: debe topar, no colapsar.
    await _dragHandle(
        tester, find.byKey(const ValueKey('canvas-side-resize')), -2000);

    expect(_sidePanelWidth(tester), greaterThan(100),
        reason: 'Un panel colapsado a cero deja las features inalcanzables y '
            'sin forma visible de recuperarlas.');
  });

  testWidgets('el panel de features no se puede ensanchar hasta comerse el lienzo',
      (tester) async {
    await _pumpCanvas(tester);

    await _dragHandle(
        tester, find.byKey(const ValueKey('canvas-side-resize')), 5000);

    expect(_sidePanelWidth(tester), lessThan(600));
  });

  testWidgets('cada feature muestra qué hace bajo su nombre, sin abrir nada',
      (tester) async {
    await _pumpCanvas(tester);

    // Las dos features que el propietario confundió deben distinguirse por lo
    // que dicen: una toma un archivo del disco, la otra trabaja sobre datos
    // ya importados.
    expect(find.textContaining('archivo de tu disco'), findsOneWidget);
    expect(find.textContaining('datos YA importados'), findsOneWidget);
  });

  testWidgets('la descripción completa también está disponible al posar el cursor',
      (tester) async {
    await _pumpCanvas(tester);

    // El texto bajo el nombre se recorta a dos líneas y depende del ancho;
    // el tooltip lleva nombre y descripción completos, sin esa dependencia.
    final tooltips = tester
        .widgetList<Tooltip>(find.byType(Tooltip))
        .map((t) => t.message ?? '')
        .toList();

    expect(
      tooltips.any((m) =>
          m.contains('Asistente de Importación') &&
          m.contains('archivo de tu disco')),
      isTrue,
    );
  });

  testWidgets('el panel del inspector se puede ensanchar arrastrando su borde',
      (tester) async {
    tester.view.physicalSize = const Size(1600, 900);
    tester.view.devicePixelRatio = 1.0;
    addTearDown(tester.view.resetPhysicalSize);

    // Se abre el panel usando la costura de prueba del lienzo.
    await tester.pumpWidget(const MaterialApp(
      home: Scaffold(
        body: CanvasTab(
          debugInitialNodeLabels: ['Asistente de Importación'],
        ),
      ),
    ));
    await tester.pump();
    // Se pulsa el NODO del lienzo, no su fila del panel de features: el
    // mismo texto aparece en los dos sitios (misma costura que usa
    // `canvas_inspector_mechanism_test.dart`).
    await tester.tap(find
        .byKey(const ValueKey('canvas-node-Asistente de Importación-40.0-40.0')));
    await tester.pump();

    final panel = find.byKey(const ValueKey('canvas-inspector-panel'));
    expect(panel, findsOneWidget);
    final antes = tester.getSize(panel).width;

    // El inspector crece hacia la izquierda: se arrastra su manija en esa
    // dirección para ensancharlo.
    await _dragHandle(
        tester, find.byKey(const ValueKey('canvas-inspector-resize')), -120);

    expect(tester.getSize(panel).width, greaterThan(antes));
  });
}
