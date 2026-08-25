// Pruebas de widget del Inspector Panel del asistente de importación
// (STORY-073 Etapa 4) — criterios de aceptación 5b, 10 y 11.
//
// Nivel de prueba: widget test con las cinco operaciones del puente
// simuladas por costura (`...Override`) — mismo criterio que
// `hybrid_data_transformer_dashboard_persistence_test.dart` y
// `verification_bench_status_test.dart`: `flutter test` no tiene la
// librería nativa compilada, así que ningún test de este archivo cruza el
// puente FFI real.
//
// Correr con: flutter test test/data_import_wizard_panel_test.dart

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:drasus_ui/components/components.dart' as custom_ui;
import 'package:drasus_ui/src/rust/api/data_import_wizard.dart';
import 'package:drasus_ui/tabs/data_import_wizard/data_import_wizard_inspector.dart';

// Columnas arbitrarias del archivo simulado — fecha y hora SEPARADAS, la
// forma real del propietario (`<DATE>` TAB `<TIME>`), que es el modo por
// defecto del Paso 2.
const List<String> _kColumns = [
  'Fecha',
  'Hora',
  'Apertura',
  'Maximo',
  'Minimo',
  'Cierre',
  'VolumenTicks',
  'VolumenReal',
  'SpreadPts',
];

// Construye el panel con dobles para las cinco operaciones — ninguno cruza
// el FFI real. `analyzeRangeOverride`/`confirmOverride` son parametrizables
// porque cada test necesita una respuesta distinta de esas dos operaciones.
DataImportWizardInspectorPanel _buildPanel({
  Future<ManualImportContentRangeResultDto> Function({
    required List<int> fileBytes,
    required List<ManualColumnMappingDto> columnMappings,
    required String instrumentId,
    required String venue,
    required String sourceProvider,
    String? sourceClockAnchorTimezoneRef,
    required int sourceClockFixedOffsetMinutes,
    String? realVolumeUnitDeclared,
    String? spreadReportedUnitDeclared,
    required String timeframeValue,
    required int barIntervalNs,
  })? analyzeRangeOverride,
  Future<ManualImportConfirmResultDto> Function({
    required String dbPath,
    required String sourceFileName,
    required List<int> fileBytes,
    required String instrumentId,
    required String venue,
    required String sourceProvider,
    required int declaredRangeStartNs,
    required int declaredRangeEndNs,
    String? sourceClockAnchorTimezoneRef,
    required int sourceClockFixedOffsetMinutes,
    String? realVolumeUnitDeclared,
    String? spreadReportedUnitDeclared,
    int? venueOperationalSinceNs,
    required String timeframeValue,
    required int barIntervalNs,
    required List<ManualColumnMappingDto> columnMappings,
  })? confirmOverride,
}) {
  return DataImportWizardInspectorPanel(
    pickFileOverride: () async => const PickedImportFile(
      fileName: 'broker_generico_export.csv',
      fileBytes: [1, 2, 3, 4],
    ),
    selectFileOverride: ({required fileBytes}) async =>
        SelectManualImportFileResultDto(
      fileSizeBytes: BigInt.from(fileBytes.length),
      sourceFileHash: 'hash-de-prueba',
    ),
    getColumnsOverride: ({required fileBytes}) async =>
        const ManualImportColumnsResultDto(columns: _kColumns),
    previewOverride: ({
      required fileBytes,
      required columnMappings,
      required instrumentId,
      required venue,
      required sourceProvider,
      sourceClockAnchorTimezoneRef,
      required sourceClockFixedOffsetMinutes,
      realVolumeUnitDeclared,
      spreadReportedUnitDeclared,
      required timeframeValue,
      required barIntervalNs,
    }) async =>
        const ManualImportPreviewResultDto(rows: []),
    analyzeRangeOverride: analyzeRangeOverride ??
        ({
          required fileBytes,
          required columnMappings,
          required instrumentId,
          required venue,
          required sourceProvider,
          sourceClockAnchorTimezoneRef,
          required sourceClockFixedOffsetMinutes,
          realVolumeUnitDeclared,
          spreadReportedUnitDeclared,
          required timeframeValue,
          required barIntervalNs,
        }) async =>
            ManualImportContentRangeResultDto(rowsParsedTotal: BigInt.zero),
    confirmOverride: confirmOverride ??
        ({
          required dbPath,
          required sourceFileName,
          required fileBytes,
          required instrumentId,
          required venue,
          required sourceProvider,
          required declaredRangeStartNs,
          required declaredRangeEndNs,
          sourceClockAnchorTimezoneRef,
          required sourceClockFixedOffsetMinutes,
          realVolumeUnitDeclared,
          spreadReportedUnitDeclared,
          venueOperationalSinceNs,
          required timeframeValue,
          required barIntervalNs,
          required columnMappings,
        }) async =>
            ManualImportConfirmResultDto(
              ok: false,
              declarationId: '',
              mappingOrigin: '',
              eventSequenceId: 0,
              auditHash: '',
              rowsImported: BigInt.zero,
              wasNoopReimport: false,
              error: 'confirmOverride no configurado para este test',
            ),
  );
}

// Selecciona la columna [columnLabel] en el Dropdown<String> número [index]
// (0-based, en el orden en que aparecen en el Paso 2: Fecha, Hora, Apertura,
// Máximo, Mínimo, Cierre, VolumenTicks, VolumenReal, SpreadPts). Abre el
// desplegable, toca el ítem DENTRO de su propio subárbol (evita colisión con
// el texto que otros Dropdown ya cerrados muestran como valor seleccionado)
// y deja que la animación de cierre se asiente.
Future<void> _selectColumn(
    WidgetTester tester, int index, String columnLabel) async {
  final dropdown = find.byType(custom_ui.Dropdown<String>).at(index);
  // El Paso 2 tiene 9 desplegables dentro de un SingleChildScrollView — hay
  // que desplazar cada uno a la vista antes de tocarlo, `tap()` no
  // desplaza por su cuenta.
  await tester.ensureVisible(dropdown);
  await tester.pumpAndSettle();
  await tester.tap(dropdown);
  await tester.pumpAndSettle();
  final item = find.descendant(of: dropdown, matching: find.text(columnLabel));
  await tester.ensureVisible(item);
  await tester.pumpAndSettle();
  await tester.tap(item);
  await tester.pumpAndSettle();
}

// Navega el panel completo desde el Paso 1 (Archivo) hasta el Paso 6 (Vista
// Previa y Confirmación) con un mapeo válido y completo — la ruta que
// cualquier test que necesite llegar al Paso 6 recorre primero. Devuelve el
// rango de fechas real que quedó declarado en el Paso 5 (para que el test
// pueda construir sus propias aserciones sobre las fechas mostradas).
Future<custom_ui.DateRange> _navegarHastaVistaPreviaYConfirmacion(
    WidgetTester tester) async {
  // ---- Paso 1: Archivo ----
  await tester.tap(find.byType(custom_ui.Dropzone));
  await tester.pumpAndSettle();
  await tester.tap(find.text('Siguiente'));
  await tester.pumpAndSettle();

  // ---- Paso 2: Mapeo ----
  await tester.enterText(find.byType(TextField).at(0), 'EURUSD');
  await tester.enterText(find.byType(TextField).at(1), 'GENERIC_BROKER');
  await tester.enterText(find.byType(TextField).at(2), 'MANUAL_UPLOAD');
  await tester.pump();
  // Modo por defecto ya es "Fecha y hora separadas" (el archivo real del
  // propietario) — no hace falta tocar el Segmented.
  await _selectColumn(tester, 0, 'Fecha'); // TimestampDate
  await _selectColumn(tester, 1, 'Hora'); // TimestampTime
  await _selectColumn(tester, 2, 'Apertura'); // OPEN
  await _selectColumn(tester, 3, 'Maximo'); // HIGH
  await _selectColumn(tester, 4, 'Minimo'); // LOW
  await _selectColumn(tester, 5, 'Cierre'); // CLOSE
  await _selectColumn(tester, 6, 'VolumenTicks'); // VOLUME_TICKS
  await _selectColumn(tester, 7, 'VolumenReal'); // VOLUME_TRADED
  await _selectColumn(tester, 8, 'SpreadPts'); // SPREAD_REPORTED
  await tester.tap(find.text('Siguiente'));
  await tester.pumpAndSettle();

  // ---- Paso 3: Reloj de Origen (deja el default UTC / desplazamiento 0) ----
  await tester.tap(find.text('Siguiente'));
  await tester.pumpAndSettle();

  // ---- Paso 4: Unidades ----
  await tester.enterText(find.byType(TextField).at(0), 'PIPS');
  await tester.pump();
  await tester.tap(find.text('Siguiente'));
  await tester.pumpAndSettle();

  // ---- Paso 5: Rango Declarado ----
  // Toca dos días del mini-calendario para forzar `onChanged` — el rango
  // interno por defecto del componente (días 3-17) nunca llega al padre sin
  // interacción real (DateRangePicker.range solo se lee en su initState).
  await tester.ensureVisible(find.text('1'));
  await tester.pumpAndSettle();
  await tester.tap(find.text('1'));
  await tester.pumpAndSettle();
  await tester.ensureVisible(find.text('25'));
  await tester.pumpAndSettle();
  await tester.tap(find.text('25'));
  await tester.pumpAndSettle();

  final now = DateTime.now();
  final declared = custom_ui.DateRange(
    start: DateTime(now.year, now.month, 1),
    end: DateTime(now.year, now.month, 25),
  );

  await tester.tap(find.text('Siguiente'));
  await tester.pumpAndSettle();

  return declared;
}

// Formatea ns UTC igual que el panel (`_formatTimestampNs` es privado —
// esta es una réplica de solo lectura para construir la aserción esperada,
// no una reimplementación del componente bajo prueba).
String _formatEsperado(DateTime utcDate, {int hour = 0, int minute = 0}) {
  String p2(int v) => v.toString().padLeft(2, '0');
  return '${utcDate.year}-${p2(utcDate.month)}-${p2(utcDate.day)} ${p2(hour)}:${p2(minute)} UTC';
}

void main() {
  // Viewport amplio (mismo criterio que verification_bench_status_test.dart):
  // el panel de 6 pasos con Table interna desborda el tamaño de test 800×600.
  setUp(() {});

  Future<void> conViewportAmplio(
      WidgetTester tester, Future<void> Function() body) async {
    tester.view.physicalSize = const Size(1280, 1000);
    tester.view.devicePixelRatio = 1.0;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    await body();
  }

  testWidgets(
    'criterio #10 — el Paso 1 bloquea "Siguiente" hasta que hay archivo y columnas',
    (WidgetTester tester) async {
      await conViewportAmplio(tester, () async {
        await tester
            .pumpWidget(MaterialApp(home: Scaffold(body: _buildPanel())));
        await tester.pumpAndSettle();

        // Antes de elegir archivo: el hint de bloqueo es visible y "Siguiente"
        // no avanza al Paso 2 (el título de esa sección nunca aparece).
        expect(find.text('Selecciona un archivo válido para continuar.'),
            findsOneWidget);
        await tester.tap(find.text('Siguiente'));
        await tester.pumpAndSettle();
        expect(find.text('Identidad de la serie'), findsNothing);

        // Tras elegir archivo (Dropzone → los dobles resuelven columnas):
        // el hint desaparece y "Siguiente" SÍ avanza.
        await tester.tap(find.byType(custom_ui.Dropzone));
        await tester.pumpAndSettle();
        expect(find.text('Selecciona un archivo válido para continuar.'),
            findsNothing);
        await tester.tap(find.text('Siguiente'));
        await tester.pumpAndSettle();
        expect(find.text('Identidad de la serie'), findsOneWidget);
      });
    },
  );

  testWidgets(
    'una excepción del puente se muestra como error y apaga el indicador de '
    'carga, en vez de dejar el botón colgado para siempre',
    (WidgetTester tester) async {
      await conViewportAmplio(tester, () async {
        // Reproduce la cadena que encontró el gate de QA de STORY-073: el
        // Núcleo entra en pánico ante una fecha extrema, el pánico cruza el
        // FFI como excepción, y sin captura el usuario se queda mirando
        // "Analizando…" para siempre SIN ver jamás un error. No es un
        // rechazo de negocio (esos viajan dentro del resultado): es que no
        // llega resultado ninguno.
        final panel = _buildPanel(
          analyzeRangeOverride: ({
            required fileBytes,
            required columnMappings,
            required instrumentId,
            required venue,
            required sourceProvider,
            sourceClockAnchorTimezoneRef,
            required sourceClockFixedOffsetMinutes,
            realVolumeUnitDeclared,
            spreadReportedUnitDeclared,
            required timeframeValue,
            required barIntervalNs,
          }) async =>
              throw Exception('panico del nucleo cruzando el FFI'),
        );

        await tester.pumpWidget(MaterialApp(home: Scaffold(body: panel)));
        await tester.pumpAndSettle();
        await _navegarHastaVistaPreviaYConfirmacion(tester);

        await tester.tap(find.text('Analizar archivo completo (una vez)'));
        await tester.pumpAndSettle();

        // El usuario VE el fallo...
        expect(find.textContaining('Fallo inesperado durante la operación'),
            findsOneWidget);
        // ...y el indicador de carga NO sigue encendido, que es la mitad que
        // dejaba la pantalla inservible.
        expect(find.textContaining('Analizando'), findsNothing);
      });
    },
  );

  testWidgets(
    'criterio 5b — el Paso 6 muestra la fecha REAL del archivo junto a la '
    'declarada, con una discrepancia de años visible y contrastable',
    (WidgetTester tester) async {
      await conViewportAmplio(tester, () async {
        // Rango real medido MUY por dentro del rango declarado (2019-2020),
        // reproduciendo el caso raíz de DEBT-039 ("declarar 2007 cuando el
        // tramo empieza en 2010") con años de diferencia, no solo días.
        final contentStart = DateTime.utc(2019, 3, 10, 8, 30);
        final contentEnd = DateTime.utc(2020, 6, 20, 17, 45);

        final panel = _buildPanel(
          analyzeRangeOverride: ({
            required fileBytes,
            required columnMappings,
            required instrumentId,
            required venue,
            required sourceProvider,
            sourceClockAnchorTimezoneRef,
            required sourceClockFixedOffsetMinutes,
            realVolumeUnitDeclared,
            spreadReportedUnitDeclared,
            required timeframeValue,
            required barIntervalNs,
          }) async =>
              ManualImportContentRangeResultDto(
            rowsParsedTotal: BigInt.from(9231696),
            contentMinOpenTimestampNs:
                contentStart.microsecondsSinceEpoch * 1000,
            contentMaxCloseTimestampNs:
                contentEnd.microsecondsSinceEpoch * 1000,
          ),
        );

        await tester.pumpWidget(MaterialApp(home: Scaffold(body: panel)));
        await tester.pumpAndSettle();

        final declared = await _navegarHastaVistaPreviaYConfirmacion(tester);

        // Paso 6: pide el análisis del rango real (operación CARA, una vez).
        await tester.tap(find.text('Analizar archivo completo (una vez)'));
        await tester.pumpAndSettle();

        // Las CUATRO fechas deben estar visibles y ser distinguibles entre sí
        // — el contraste que el criterio exige: real vs. declarada.
        expect(find.text(_formatEsperado(contentStart, hour: 8, minute: 30)),
            findsOneWidget);
        expect(find.text(_formatEsperado(contentEnd, hour: 17, minute: 45)),
            findsOneWidget);
        expect(find.text(_formatEsperado(declared.start)), findsOneWidget);
        // Fin declarado se serializa a las 23:59 del día (fin de jornada
        // inclusive) — mismo criterio de _endOfDayNs del panel.
        expect(find.text(_formatEsperado(declared.end, hour: 23, minute: 59)),
            findsOneWidget);

        // Discriminación mínima: el año real (2019) y el año declarado (el
        // mes actual, casi con certeza distinto de 2019) no colapsan al mismo
        // texto — si el panel mostrara solo una de las dos fechas, una de
        // estas dos aserciones fallaría.
        expect(contentStart.year, isNot(equals(declared.start.year)));
      });
    },
  );

  testWidgets(
    'criterio #11 — un rechazo del backend se muestra literal, nunca una '
    'importación parcial visible',
    (WidgetTester tester) async {
      await conViewportAmplio(tester, () async {
        const mensajeLiteral =
            'el Gate de Homogeneidad de Procedencia rechazó el tramo: '
            'fractura de magnitud detectada en 2019-06-01';

        final panel = _buildPanel(
          confirmOverride: ({
            required dbPath,
            required sourceFileName,
            required fileBytes,
            required instrumentId,
            required venue,
            required sourceProvider,
            required declaredRangeStartNs,
            required declaredRangeEndNs,
            sourceClockAnchorTimezoneRef,
            required sourceClockFixedOffsetMinutes,
            realVolumeUnitDeclared,
            spreadReportedUnitDeclared,
            venueOperationalSinceNs,
            required timeframeValue,
            required barIntervalNs,
            required columnMappings,
          }) async =>
              ManualImportConfirmResultDto(
            ok: false,
            declarationId: '',
            mappingOrigin: '',
            eventSequenceId: 0,
            auditHash: '',
            rowsImported: BigInt.zero,
            wasNoopReimport: false,
            error: mensajeLiteral,
          ),
        );

        await tester.pumpWidget(MaterialApp(home: Scaffold(body: panel)));
        await tester.pumpAndSettle();
        await _navegarHastaVistaPreviaYConfirmacion(tester);

        // El análisis debe tener éxito para habilitar "Confirmar e Importar"
        // — el gate de secuencia del propio panel (ver `_confirmEnabled`).
        await tester.tap(find.text('Analizar archivo completo (una vez)'));
        await tester.pumpAndSettle();

        await tester.tap(find.text('Confirmar e Importar'));
        await tester.pumpAndSettle();

        // El mensaje literal del backend está en pantalla...
        expect(find.text(mensajeLiteral), findsOneWidget);
        // ...y NINGÚN observable de una importación exitosa (identidad del
        // ledger) está presente — nunca una confirmación parcial visible.
        expect(find.text('declaration_id'), findsNothing);
        expect(find.text('event_sequence_id'), findsNothing);
        expect(
            find.textContaining('Importación sellada con éxito'), findsNothing);
      });
    },
  );
}
