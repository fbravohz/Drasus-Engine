// Test de widget de hybrid-data-transformer: distinción visual entre
// "magnitud ausente" (el bróker no la reportó) y "magnitud cero real".
//
// Es la prueba central de la Cáscara Visual de esta feature (STORY-051
// Etapa 4, ADR-0155 criterio 14). El esquema de datos transporta esa
// distinción como `Option<i64>` en Rust — `None` cruza a Dart como `null`,
// nunca como `0` (ver doc-comment de BarDto en
// ui/lib/src/rust/api/hybrid_data_transformer.dart). Si la interfaz
// colapsara ambos casos al mismo texto "0", se rompería la garantía que
// costó dos rondas de retrabajo construir en el Núcleo.
//
// No cruza el puente FFI real: OptionalMagnitudeText es un widget de
// presentación puro (recibe un `int?` ya resuelto), así que se puede montar
// directo en un widget test sin librería nativa — mismo criterio que
// justifica por qué este archivo, y no el panel completo, es la prueba
// unitaria de esta garantía (el panel/inspector y el widget de dashboard sí
// llaman al Bridge en initState y no se pumpean directo en tests, ver
// operational_panel_smoke_test.dart).
//
// Correr con: flutter test test/hybrid_data_transformer_optional_magnitude_test.dart

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:drasus_ui/tabs/hybrid_data_transformer/optional_magnitude_text.dart';
import 'package:drasus_ui/theme/gx_tokens.dart';

// Monta OptionalMagnitudeText solo (sin Scaffold/AppBar innecesarios) y
// devuelve el único Text que renderiza, para inspeccionar texto y estilo.
Future<Text> _pumpAndFindText(
  WidgetTester tester, {
  required int? scaledValue,
}) async {
  await tester.pumpWidget(
    MaterialApp(
      home: Scaffold(
        body: OptionalMagnitudeText(scaledValue: scaledValue),
      ),
    ),
  );
  return tester.widget<Text>(find.byType(Text));
}

void main() {
  testWidgets(
    'magnitud ausente (null) se muestra con marca de ausencia — NUNCA como "0"',
    (WidgetTester tester) async {
      final texto = await _pumpAndFindText(tester, scaledValue: null);

      // El texto debe llevar la marca explícita de ausencia...
      expect(texto.data, contains('No reportado'));
      // ...y en ningún caso puede colapsar a un cero desnudo.
      expect(texto.data, isNot(equals('0')));
      expect(texto.data, isNot(equals('0.00000000')));

      // Tratamiento visual distinto: muted + cursiva (nunca el color de
      // texto normal que usa un valor real).
      expect(texto.style?.color, Gx.textBaseMuted);
      expect(texto.style?.fontStyle, FontStyle.italic);
    },
  );

  testWidgets(
    'magnitud cero REAL (0) se muestra como valor normal — NUNCA como ausente',
    (WidgetTester tester) async {
      final texto = await _pumpAndFindText(tester, scaledValue: 0);

      // El valor cero real se formatea como cualquier otro número escalado
      // ×10⁸ — sin la marca de ausencia ni el texto "No reportado".
      expect(texto.data, equals('0.00000000'));
      expect(texto.data, isNot(contains('No reportado')));

      // Tratamiento visual normal: color de texto base, sin cursiva.
      expect(texto.style?.color, Gx.textBase);
      expect(texto.style?.fontStyle, isNot(FontStyle.italic));
    },
  );

  testWidgets(
    'magnitud presente distinta de cero se formatea exacta, sin coma flotante',
    (WidgetTester tester) async {
      // 123456789012 escalado ×10⁸ → 1234.56789012 (manipulación de string
      // pura: el test fija un valor cuya representación en f64 podría
      // perder precisión si la Cáscara usara división flotante).
      final texto =
          await _pumpAndFindText(tester, scaledValue: 123456789012);
      expect(texto.data, equals('1234.56789012'));
    },
  );

  testWidgets(
    'el mismo widget con null y con 0 produce salidas visualmente distintas',
    (WidgetTester tester) async {
      // Regresión directa contra "si falta, usa cero": monta ambos casos en
      // el mismo árbol y confirma que no colapsan al mismo string.
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Column(children: [
              OptionalMagnitudeText(scaledValue: null, key: const Key('ausente')),
              OptionalMagnitudeText(scaledValue: 0, key: const Key('cero')),
            ]),
          ),
        ),
      );

      final ausente =
          tester.widget<Text>(find.descendant(
        of: find.byKey(const Key('ausente')),
        matching: find.byType(Text),
      ));
      final cero = tester.widget<Text>(find.descendant(
        of: find.byKey(const Key('cero')),
        matching: find.byType(Text),
      ));

      expect(ausente.data, isNot(equals(cero.data)));
    },
  );
}
