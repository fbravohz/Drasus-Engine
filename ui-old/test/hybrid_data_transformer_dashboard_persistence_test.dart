// Test de persistencia del Widget de Tablero de hybrid-data-transformer.
//
// Cierra el bloqueo del Flutter-Engineer en el veredicto NO APTO de STORY-051
// (docs/execution/STORY-051-hybrid-data-transformer.md, criterio 10): "el
// observable persiste tras recargar". Hasta ahora esa garantía era solo
// ESTRUCTURAL — HybridDataTransformerDashboardWidget relee vía
// listTransformationRecords() en cada initState(), así que en teoría
// sobrevive a un remontaje — pero ningún test desmontaba y volvía a montar
// el widget para comprobarlo. Es exactamente la clase de cosa que se rompe
// sin que nadie se entere: el día que alguien mueva la carga a una caché en
// memoria (un `static` a nivel de archivo, o un singleton que recuerda el
// último resultado y deja de volver a preguntar), el comportamiento visible
// NO cambia hasta que el usuario reinicia el proceso completo — y un test
// que solo verifica "el primer montaje muestra el dato" no lo detecta.
//
// Nivel de prueba elegido: widget test con la capa de datos simulada, NO
// integración contra el puente FFI real. Justificación (ver reporte del
// Flutter-Engineer al invocador): `flutter test` no tiene la librería nativa
// compilada — el mismo criterio que ya usa verification_bench_status_test.dart
// con `verifyOverride`. HybridDataTransformerDashboardWidget gana aquí el
// mismo tipo de costura: `loadRecordsOverride` (ver
// hybrid_data_transformer_dashboard_widget.dart). El widget en sí no sabe
// que es un doble — solo invoca la función que le llega por constructor,
// igual que en producción invoca listTransformationRecords real.
//
// Por qué el test SÍ discrimina (y no pasaría igual con una caché en
// memoria): en vez de reutilizar el mismo almacén simulado entre el primer y
// el segundo montaje, cada montaje recibe una función `loadRecordsOverride`
// DISTINTA que lee de un almacén DISTINTO. Si la implementación cacheara el
// resultado en algún lugar que sobrevive al `dispose()` del primer State (un
// campo `static`, un singleton de módulo) e ignorara o ni siquiera llamara
// la nueva función inyectada, el segundo montaje seguiría mostrando el dato
// del PRIMER almacén — el test fallaría exactamente ahí. La implementación
// correcta (relee del `loadRecordsOverride` vigente en cada `initState()`,
// sin nada persistido fuera del State) muestra el dato del SEGUNDO almacén.
//
// Correr con: flutter test test/hybrid_data_transformer_dashboard_persistence_test.dart

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:drasus_ui/src/rust/api/hybrid_data_transformer.dart';
import 'package:drasus_ui/tabs/hybrid_data_transformer/hybrid_data_transformer_dashboard_widget.dart';

// Fabrica un TransformationRecordDto mínimo pero completo — solo varía
// `rowsOut` entre los casos de prueba, que es el "dato héroe" que el widget
// pinta en pantalla y por tanto el observable más directo de inspeccionar.
TransformationRecordDto _record({required int rowsOut, required String id}) {
  return TransformationRecordDto(
    id: id,
    createdAt: 1700000000000000000,
    dataSnapshotId: 'binance_btcusdt_1h_202607',
    logicHash: 'hash-$id',
    rowsIn: rowsOut * 10,
    rowsOut: rowsOut,
    transformDurationNs: 5000000,
    schemaVersion: '3.0',
  );
}

// Monta HybridDataTransformerDashboardWidget con un doble de
// listTransformationRecords que lee del `store` recibido, deja que el
// Future de _cargar() resuelva y asienta el frame resultante.
Future<void> _pumpDashboard(
  WidgetTester tester, {
  required List<TransformationRecordDto> store,
}) async {
  await tester.pumpWidget(
    MaterialApp(
      home: Scaffold(
        body: HybridDataTransformerDashboardWidget(
          // Cierre nuevo por cada llamada a _pumpDashboard — deliberado: no
          // se reutiliza la misma función entre montajes en los tests de
          // abajo, para que un remontaje solo pueda ver datos nuevos si de
          // verdad vuelve a invocar la costura en vez de recordar la vieja.
          loadRecordsOverride: ({required dbPath}) async => store,
        ),
      ),
    ),
  );
  await tester.pumpAndSettle();
}

// Desmonta el widget actual sustituyéndolo por un árbol vacío — simula que
// el usuario navegó fuera del Dashboard (o recargó la ventana) antes de que
// _pumpDashboard() del siguiente montaje construya una instancia de State
// nueva desde cero.
Future<void> _desmontar(WidgetTester tester) async {
  await tester.pumpWidget(const MaterialApp(home: Scaffold(body: SizedBox.shrink())));
  await tester.pumpAndSettle();
}

void main() {
  testWidgets(
    'un registro escrito DESPUÉS del primer montaje se ve al recargar — '
    'no se queda pegado al dato del montaje anterior',
    (WidgetTester tester) async {
      // Primer montaje: el "backend" solo tiene una corrida antigua.
      await _pumpDashboard(tester, store: [_record(id: 'run-viejo', rowsOut: 111)]);
      expect(find.text('111'), findsOneWidget);

      // El usuario navega fuera del Dashboard (desmontaje real: dispose()).
      await _desmontar(tester);

      // Mientras el widget estaba desmontado, corrió una transformación
      // nueva y quedó persistida — modelado aquí como un almacén DISTINTO
      // (no una mutación del mismo store) para que la única forma de que el
      // segundo montaje muestre el valor nuevo sea invocando de verdad la
      // costura otra vez, nunca reciclando algo que el primer State guardó.
      await _pumpDashboard(tester, store: [_record(id: 'run-nuevo', rowsOut: 222)]);

      // El observable persistido (rowsOut de la corrida MÁS RECIENTE) debe
      // verse tras "recargar" — y el dato del montaje anterior no debe seguir
      // presente en pantalla.
      expect(find.text('222'), findsOneWidget);
      expect(find.text('111'), findsNothing);
    },
  );

  testWidgets(
    'un registro escrito DESPUÉS de un primer montaje en vacío deja de '
    'mostrar "sin transformaciones" al recargar',
    (WidgetTester tester) async {
      // Primer montaje: BD sin ninguna transformación registrada todavía —
      // el widget debe mostrar el estado vacío explícito, nunca un dato falso.
      await _pumpDashboard(tester, store: const []);
      expect(find.text('Sin transformaciones registradas.'), findsOneWidget);

      await _desmontar(tester);

      // El usuario corrió su primera transformación (vía el panel inspector)
      // y quedó persistida antes de volver a esta tarjeta.
      await _pumpDashboard(tester, store: [_record(id: 'primera-corrida', rowsOut: 777)]);

      expect(find.text('777'), findsOneWidget);
      expect(find.text('Sin transformaciones registradas.'), findsNothing);
    },
  );
}
