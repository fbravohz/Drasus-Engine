// Pruebas de widget del mecanismo GENÉRICO de apertura de paneles del
// lienzo (STORY-073 Etapa 4) — criterios de aceptación 8 y 9.
//
// Criterio #8: pulsar un nodo del lienzo abre el panel de esa feature —
// probado con el mecanismo real (`onTap` -> `_openPanel` -> registro ->
// `_InspectorPanelOverlay`) contra el asistente de importación, que expone
// costuras de prueba para las cinco operaciones del puente y por tanto puede
// montarse COMPLETO en `flutter test` sin cruzar el FFI real.
//
// Criterio #9: el panel huérfano de `hybrid-data-transformer` deja de
// estarlo. Se prueba en DOS mitades separadas a propósito:
//   (a) el REGISTRO asocia la feature con la clase REAL del panel (prueba
//       de tipo, sin montar el árbol — `HybridDataTransformerInspectorPanel`
//       llama a `listSourceDatasets` de verdad en su `initState()` sin
//       costura en este camino, porque el registro usa los defaults de
//       producción a propósito — ver canvas_inspector_registry.dart; montarlo
//       sin `RustLib.init()` lanza `StateError` de forma síncrona durante el
//       build, sin importar cuántos `pump()` seguidos).
//   (b) el panel, montado DIRECTO (no vía el lienzo) con su propia costura
//       `loadDatasetsOverride`, sigue siendo el mismo panel completo y
//       funcional de STORY-051 — adoptarlo en el registro no lo dejó a
//       medias.
// Juntas, (a) y (b) prueban lo mismo que "tocar el nodo abre el panel real"
// probaría de punta a punta, sin depender de que `flutter test` tenga la
// librería nativa cargada.
//
// `CanvasTab.debugInitialNodeLabels` es la costura de prueba que coloca
// nodos sin simular el gesto real de arrastrar-y-soltar (`WidgetTester` no
// tiene una forma práctica de simular coordenadas de `DragTarget`); el
// mecanismo que se prueba (`onTap` -> registro -> panel) es el mismo código
// que corre cuando el nodo se soltó por arrastre real.
//
// Correr con: flutter test test/canvas_inspector_mechanism_test.dart

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:drasus_ui/components/components.dart' as custom_ui;
import 'package:drasus_ui/src/rust/api/hybrid_data_transformer.dart';
import 'package:drasus_ui/tabs/canvas_inspector_registry.dart';
import 'package:drasus_ui/tabs/canvas_tab.dart';
import 'package:drasus_ui/tabs/hybrid_data_transformer/hybrid_data_transformer_inspector.dart';

// Encuentra el nodo del lienzo por su Key (`canvas-node-<label>-<x>-<y>`),
// nunca por su texto: el panel lateral de features muestra EL MISMO texto
// para su propio ítem de lista, así que `find.text(label)` encontraría dos
// candidatos y `tap()` fallaría por ambigüedad. La posición coincide con la
// que usa `CanvasTab.debugInitialNodeLabels` (Offset(40, 40) fijo).
Finder nodoDelLienzo(String label) =>
    find.byKey(ValueKey('canvas-node-$label-40.0-40.0'));

void main() {
  Future<void> conViewportAmplio(
      WidgetTester tester, Future<void> Function() body) async {
    tester.view.physicalSize = const Size(1400, 1000);
    tester.view.devicePixelRatio = 1.0;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    await body();
  }

  testWidgets(
    'criterio #8 — pulsar el nodo del asistente de importación abre SU panel, '
    'y el botón de cierre de la cabecera GENÉRICA (no una por feature) lo cierra',
    (WidgetTester tester) async {
      await conViewportAmplio(tester, () async {
        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(
              body: CanvasTab(
                  debugInitialNodeLabels: ['Asistente de Importación']),
            ),
          ),
        );
        await tester.pumpAndSettle();

        // Antes de pulsar: el panel NO está montado (solo el nodo en el lienzo).
        expect(find.text('Archivo de origen'), findsNothing);

        await tester.tap(nodoDelLienzo('Asistente de Importación'));
        await tester.pumpAndSettle();

        // Después de pulsar: el contenido REAL del Paso 1 del asistente está
        // en pantalla — no un placeholder, el panel de verdad.
        expect(find.text('Archivo de origen'), findsOneWidget);

        // El botón de cierre es de la cabecera GENÉRICA (_InspectorPanelOverlay,
        // la misma para cualquier feature) — cerrarlo prueba que el mecanismo
        // completo (abrir Y cerrar) no es exclusivo de esta feature.
        await tester.tap(find.byIcon(Icons.close));
        await tester.pumpAndSettle();
        expect(find.text('Archivo de origen'), findsNothing);
      });
    },
  );

  testWidgets(
    'criterio #9 (a) — el registro genérico asocia la feature huérfana con '
    'la clase REAL de su panel, no un stub',
    (WidgetTester tester) async {
      // Invoca el builder registrado para construir el Widget (sin montarlo
      // en el árbol — un `Builder` descartable solo aporta el BuildContext
      // que la firma de `CanvasPanelBuilder` exige). No dispara `initState()`
      // porque el widget nunca llega a insertarse en el árbol de elementos;
      // esta prueba verifica la ASOCIACIÓN del registro, no el ciclo de vida
      // del panel (eso lo prueba la prueba (b), con su propia costura).
      late Widget built;
      await tester.pumpWidget(Builder(builder: (context) {
        built = resolveCanvasPanelBuilder('Transformador de Datos de Mercado')(
            context);
        return const SizedBox.shrink();
      }));

      expect(built, isA<HybridDataTransformerInspectorPanel>());
      expect(
        kCanvasFeaturePanels.containsKey('Transformador de Datos de Mercado'),
        isTrue,
        reason: 'la feature huérfana debe estar en el registro genérico, '
            'no resuelta por su fallback "sin construir"',
      );
    },
  );

  testWidgets(
    'criterio #9 (b) — el panel del transformador híbrido, montado con su '
    'propia costura de datos, sigue siendo el panel completo y funcional de '
    'STORY-051: adoptarlo en el registro no lo dejó a medias',
    (WidgetTester tester) async {
      await conViewportAmplio(tester, () async {
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: HybridDataTransformerInspectorPanel(
                loadDatasetsOverride: ({required dbPath}) async => [
                  const SourceDatasetDto(
                    id: 'download-record-de-prueba',
                    dataSnapshotId: 'binance_btcusdt_1h_202607',
                    sourceEndpoint: 'https://no-usado.example/klines',
                    createdAt: 0,
                  ),
                ],
              ),
            ),
          ),
        );
        await tester.pumpAndSettle();

        expect(find.text('Plan de transformación'), findsOneWidget);

        // El nombre del dataset solo aparece en el árbol una vez que el
        // Dropdown de "Dataset de origen" se abre (su lista de ítems no se
        // construye cerrada) — abrirlo prueba que el doble inyectado sí
        // llegó hasta la UI real. `.first`: el panel tiene un SEGUNDO
        // Dropdown<String> (Agregación destino, valores '1m'/'5m'/...), así
        // que el finder por tipo solo no basta.
        await tester.tap(find.byType(custom_ui.Dropdown<String>).first);
        await tester.pumpAndSettle();
        expect(find.text('binance_btcusdt_1h_202607'), findsOneWidget);
      });
    },
  );

  testWidgets(
    'un nodo de una feature sin panel construido abre un panel honesto, '
    'nunca un no-op silencioso',
    (WidgetTester tester) async {
      await conViewportAmplio(tester, () async {
        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(
              body: CanvasTab(debugInitialNodeLabels: ['Reloj Determinista']),
            ),
          ),
        );
        await tester.pumpAndSettle();

        await tester.tap(nodoDelLienzo('Reloj Determinista'));
        await tester.pumpAndSettle();

        expect(
          find.textContaining('todavía no tiene un Inspector Panel construido'),
          findsOneWidget,
        );
      });
    },
  );
}
