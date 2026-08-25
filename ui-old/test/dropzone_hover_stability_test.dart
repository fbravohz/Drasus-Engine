// Prueba de regresión del parpadeo infinito de `Dropzone` (bug reportado por
// el propietario el 2026-08-01 probando STORY-073).
//
// Qué reprodujo el bug en producto: al acercar el cursor a la zona de carga,
// esta alternaba a toda velocidad entre su texto de reposo y "Suelta aquí";
// por el lado izquierdo se quedaba encogida y sin poder pulsarse.
//
// Por qué ocurría: la geometría de la zona dependía del estado del puntero
// (texto más corto en hover + borde más grueso), así que al entrar el cursor
// la caja encogía, el borde derecho pasaba por debajo del cursor, se disparaba
// `onExit`, la caja crecía y el cursor volvía a quedar dentro. Bucle.
//
// Estas pruebas montan la zona con las MISMAS restricciones de layout que la
// hacían encoger —una `Column` con `crossAxisAlignment.start`, que es como la
// usa el panel del asistente de importación— y afirman lo que el bug rompía:
// el tamaño no cambia al pasar el cursor. Sobre el código anterior fallan.

import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:drasus_ui/components/components.dart' as custom_ui;

// Ancho del contenedor que aloja la zona: holgado y con restricción SUELTA en
// horizontal, igual que el panel real. Con restricción tight el bug no se
// reproduce (el padre impone el ancho), así que la prueba no discriminaría.
const double _kHostWidth = 400;

Widget _host(Widget child) => MaterialApp(
      home: Scaffold(
        body: SizedBox(
          width: _kHostWidth,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [child],
          ),
        ),
      ),
    );

void main() {
  testWidgets(
      'el tamaño de la zona de carga NO cambia al pasar el cursor por encima',
      (tester) async {
    await tester.pumpWidget(_host(custom_ui.Dropzone(
      onTap: () {},
      label: 'Toca para elegir tu archivo CSV/TXT/TSV',
    )));
    await tester.pumpAndSettle();

    final zona = find.byType(custom_ui.Dropzone);
    final tamanoEnReposo = tester.getSize(zona);

    // Puntero de ratón real: `MouseRegion.onEnter` solo reacciona a un
    // dispositivo de tipo mouse, no a un toque.
    final cursor = await tester.createGesture(kind: PointerDeviceKind.mouse);
    await cursor.addPointer(location: Offset.zero);
    addTearDown(cursor.removePointer);
    await tester.pump();

    await cursor.moveTo(tester.getCenter(zona));
    await tester.pumpAndSettle();

    expect(
      tester.getSize(zona),
      tamanoEnReposo,
      reason: 'Un estado de puntero no puede cambiar el tamaño del objeto que '
          'lo produce: si encoge, el borde se mueve por debajo del cursor y '
          'se dispara el bucle enter/exit que causó el parpadeo.',
    );
  });

  testWidgets('pasar el cursor no cambia el texto de la zona de carga',
      (tester) async {
    const etiqueta = 'Toca para elegir tu archivo CSV/TXT/TSV';
    await tester.pumpWidget(_host(custom_ui.Dropzone(
      onTap: () {},
      label: etiqueta,
    )));
    await tester.pumpAndSettle();

    final cursor = await tester.createGesture(kind: PointerDeviceKind.mouse);
    await cursor.addPointer(location: Offset.zero);
    addTearDown(cursor.removePointer);
    await tester.pump();

    await cursor.moveTo(tester.getCenter(find.byType(custom_ui.Dropzone)));
    await tester.pumpAndSettle();

    expect(find.text(etiqueta), findsOneWidget);
    // "Suelta aquí" pertenece a un evento de arrastre real, que hoy no está
    // cableado: anunciarlo al mover el ratón prometía algo que no ocurre.
    expect(find.text('Suelta aquí'), findsNothing);
  });

  testWidgets('la zona sigue siendo pulsable con el cursor encima',
      (tester) async {
    // El bug dejaba la zona inservible: parpadeaba tan rápido que el clic no
    // llegaba a registrarse. Esta prueba fija que el toque funciona en el
    // mismo estado en el que antes se perdía.
    var toques = 0;
    await tester.pumpWidget(_host(custom_ui.Dropzone(
      onTap: () => toques++,
      label: 'Toca para elegir tu archivo CSV/TXT/TSV',
    )));
    await tester.pumpAndSettle();

    final zona = find.byType(custom_ui.Dropzone);
    final cursor = await tester.createGesture(kind: PointerDeviceKind.mouse);
    await cursor.addPointer(location: Offset.zero);
    addTearDown(cursor.removePointer);
    await tester.pump();
    await cursor.moveTo(tester.getCenter(zona));
    await tester.pumpAndSettle();

    await tester.tap(zona);
    await tester.pumpAndSettle();

    expect(toques, 1);
  });
}
