// Pruebas del copiado al portapapeles (`CopyButton` / `CopyableText`) y de su
// aplicación en `Banner`.
//
// Origen (2026-08-01): el propietario encontró un fallo inesperado en el
// asistente de importación y tuvo que TRANSCRIBIR el mensaje a mano para
// reportarlo, porque el texto no se dejaba seleccionar. El texto que más
// necesita salir de la aplicación es justamente el que describe un fallo.
//
// La regla que estas pruebas fijan: etiquetas y botones NO se seleccionan
// (seleccionarlos al arrastrar es ruido y no sirven fuera de la app), pero
// todo texto que el usuario pueda necesitar fuera —errores, huellas,
// identificadores— es seleccionable y se copia de un clic.

import 'package:flutter/material.dart' hide Banner;
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:drasus_ui/components/components.dart' as custom_ui;

// Intercepta el canal de plataforma para observar qué se envió al
// portapapeles. Devuelve la lista viva de llamadas registradas.
List<MethodCall> _espiarPortapapeles() {
  final llamadas = <MethodCall>[];
  TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
      .setMockMethodCallHandler(SystemChannels.platform, (call) async {
    llamadas.add(call);
    return null;
  });
  addTearDown(() {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(SystemChannels.platform, null);
  });
  return llamadas;
}

Widget _host(Widget child) =>
    MaterialApp(home: Scaffold(body: Center(child: child)));

// Extrae el texto que se envió al portapapeles, o null si nunca se envió nada.
String? _textoCopiado(List<MethodCall> llamadas) {
  for (final c in llamadas) {
    if (c.method == 'Clipboard.setData') {
      return (c.arguments as Map)['text'] as String?;
    }
  }
  return null;
}

void main() {
  testWidgets('el botón de copiado envía el texto EXACTO al portapapeles',
      (tester) async {
    final llamadas = _espiarPortapapeles();
    const mensaje = 'Fallo inesperado durante la operación: algo se rompió';

    await tester.pumpWidget(_host(const custom_ui.CopyButton(text: mensaje)));
    await tester.tap(find.byType(custom_ui.CopyButton));
    await tester.pump();

    expect(_textoCopiado(llamadas), mensaje,
        reason: 'Se copia el mensaje completo, sin recortes ni reformateo: '
            'el usuario lo va a pegar en un reporte.');

    // Drena el temporizador del acuse visual: el framework de pruebas falla
    // si queda un `Timer` vivo al terminar.
    await tester.pump(const Duration(seconds: 3));
  });

  testWidgets('tras copiar se muestra un acuse visible y luego se revierte',
      (tester) async {
    _espiarPortapapeles();
    await tester
        .pumpWidget(_host(const custom_ui.CopyButton(text: 'lo que sea')));

    // Icono en reposo. Se compara por identidad contra el de después de
    // copiar en vez de fijar un `IconData` concreto: el token del icono puede
    // cambiar en el tema sin que esta prueba deba enterarse — lo que se
    // afirma es que el estado CAMBIA y luego VUELVE, no cuál es el glifo.
    final iconoInicial = tester.widget<Icon>(find.descendant(
      of: find.byType(custom_ui.CopyButton),
      matching: find.byType(Icon),
    ));

    await tester.tap(find.byType(custom_ui.CopyButton));
    await tester.pump();

    final iconoTrasCopiar = tester.widget<Icon>(find.descendant(
      of: find.byType(custom_ui.CopyButton),
      matching: find.byType(Icon),
    ));
    expect(iconoTrasCopiar.icon, isNot(iconoInicial.icon),
        reason: 'Copiar al portapapeles no produce ningún efecto observable '
            'por sí mismo; sin acuse el usuario no sabe si funcionó.');

    // El acuse se revierte solo, sin dejar el botón en un estado permanente.
    await tester.pump(const Duration(seconds: 3));
    final iconoFinal = tester.widget<Icon>(find.descendant(
      of: find.byType(custom_ui.CopyButton),
      matching: find.byType(Icon),
    ));
    expect(iconoFinal.icon, iconoInicial.icon);
  });

  testWidgets(
      'un Banner de error copiable es seleccionable y copia su mensaje entero',
      (tester) async {
    final llamadas = _espiarPortapapeles();
    const mensaje =
        'Fallo inesperado durante la operación: Exception: no se pudo abrir el archivo';

    await tester.pumpWidget(_host(SizedBox(
      width: 420,
      child: custom_ui.Banner(
        message: mensaje,
        type: custom_ui.BannerType.error,
        copyable: true,
      ),
    )));
    await tester.pumpAndSettle();

    // Seleccionable con el ratón: es un `SelectableText`, no un `Text` inerte.
    expect(find.byType(SelectableText), findsOneWidget);

    await tester.tap(find.byType(custom_ui.CopyButton));
    await tester.pump();
    expect(_textoCopiado(llamadas), mensaje);
    await tester.pump(const Duration(seconds: 3));
  });

  testWidgets('un Banner normal NO es seleccionable ni ofrece copiado',
      (tester) async {
    // El default se mantiene inerte a propósito: un aviso corto no se lleva a
    // ningún lado, y hacerlo seleccionable solo añade ruido al arrastrar.
    await tester.pumpWidget(_host(SizedBox(
      width: 420,
      child: custom_ui.Banner(
        message: 'Importación sellada con éxito.',
        type: custom_ui.BannerType.success,
      ),
    )));

    expect(find.byType(SelectableText), findsNothing);
    expect(find.byType(custom_ui.CopyButton), findsNothing);
  });

  testWidgets('KeyValue copiable copia el VALOR, nunca la etiqueta',
      (tester) async {
    final llamadas = _espiarPortapapeles();
    const huella =
        'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855';

    await tester.pumpWidget(_host(SizedBox(
      width: 420,
      child: custom_ui.KeyValue(
        label: 'Huella SHA-256',
        value: huella,
        mono: true,
        copyable: true,
      ),
    )));
    await tester.pumpAndSettle();

    await tester.tap(find.byType(custom_ui.CopyButton));
    await tester.pump();

    expect(_textoCopiado(llamadas), huella,
        reason: 'La etiqueta describe el dato dentro de la app y no sirve '
            'fuera; lo que se pega en otro sitio es el valor.');
    await tester.pump(const Duration(seconds: 3));
  });
}
