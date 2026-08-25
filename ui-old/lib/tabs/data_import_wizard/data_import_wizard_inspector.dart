// data_import_wizard_inspector.dart — Panel Inspector de la feature
// `data-import-wizard` (STORY-073 Etapa 4, `docs/features/data-import-wizard.md`
// "## Cáscara Visual"). Seis pasos: Archivo → Mapeo → Reloj de Origen →
// Unidades → Rango Declarado → Vista Previa y Confirmación.
//
// Por ADR-0136 (FIJO) esta superficie es un panel lateral que el lienzo abre
// al pulsar el nodo de la feature — nunca pantalla completa. El cableado
// genérico que lo hace alcanzable vive en `canvas_inspector_registry.dart` y
// `canvas_tab.dart`.
//
// Frontera lectura/escritura (LDR-0008 condición 1, ver el doc-comment de
// `crates/bridge/src/api/data_import_wizard.rs`): de las cinco operaciones
// que este panel consume, solo `confirmManualBarImport` recibe `db_path` y
// puede escribir. Las otras cuatro se piden y descartan sin efecto — el Paso
// 6 las usa así: la vista previa (barata) se repite en cada ajuste, el
// análisis de rango (caro) se pide UNA vez cuando el mapeo ya es estable.
//
// Cáscara delgada (ADR-0117): cero lógica de negocio. Los únicos cómputos en
// este archivo son de presentación (formatear ns a fecha legible, mirar
// campos vacíos para bloquear el avance) — nunca se calcula, redondea ni
// deriva una magnitud financiera. La conversión de fecha de calendario a
// nanosegundos UTC es serialización de frontera FFI, no cálculo de dominio
// (mismo criterio que documenta `docs/features/data-import-wizard.md`
// "Notas de implementación" para `DateRangePicker.range`).

import 'dart:typed_data' show Uint8List;

import 'package:file_selector/file_selector.dart';
import 'package:flutter/material.dart';

import '../../components/components.dart' as custom_ui;
import '../../theme/gx_tokens.dart';
import '../../src/rust/api/data_import_wizard.dart';
import '../hybrid_data_transformer/optional_magnitude_text.dart'
    show formatScaledInt;

// Ruta a la base de datos SQLite de Drasus — mismo valor que el resto de las
// Cáscaras Delgadas de EPIC-1 (jobs_tab.dart, hybrid_data_transformer_inspector.dart).
const String _kDbPath = 'drasus.db';

// Los siete campos canónicos SIN relación con la marca de tiempo — vocabulario
// FIJO que espeja `CanonicalBarField::ALL_NON_TIMESTAMP` del Núcleo (ver
// `crates/features/data/data-import-wizard/src/domain.rs`). Par
// (token del vocabulario FFI, etiqueta visible).
const List<(String, String)> _kNonTimestampFields = [
  ('OPEN', 'Apertura'),
  ('HIGH', 'Máximo'),
  ('LOW', 'Mínimo'),
  ('CLOSE', 'Cierre'),
  ('VOLUME_TICKS', 'Volumen de ticks'),
  ('VOLUME_TRADED', 'Volumen operado'),
  ('SPREAD_REPORTED', 'Spread reportado'),
];

// Vía de declaración de la marca de tiempo — dos formas mutuamente
// excluyentes que el Núcleo ya modela como `CanonicalBarField::Timestamp` vs.
// `TimestampDate`+`TimestampTime` (ver domain.rs). El archivo real del
// propietario (`<DATE>` TAB `<TIME>`) es la forma separada — la más común de
// la industria, no un caso raro.
enum _TimestampMode { combined, separate }

// Preset de temporalidad: rótulo visible -> (timeframe_value, bar_interval_ns).
// Tabla de presentación (lookup fijo), no un cálculo de dominio — mismo
// criterio que `_kTargetAggregations` en hybrid_data_transformer_inspector.dart.
class _TimeframePreset {
  final String label;
  final String timeframeValue;
  final int barIntervalNs;
  const _TimeframePreset(this.label, this.timeframeValue, this.barIntervalNs);
}

const List<_TimeframePreset> _kTimeframePresets = [
  _TimeframePreset('1 minuto', '1M', 60000000000),
  _TimeframePreset('5 minutos', '5M', 300000000000),
  _TimeframePreset('15 minutos', '15M', 900000000000),
  _TimeframePreset('1 hora', '1H', 3600000000000),
  _TimeframePreset('4 horas', '4H', 14400000000000),
  _TimeframePreset('1 día', '1D', 86400000000000),
];

const List<String> _kStepLabels = [
  'Archivo',
  'Mapeo',
  'Reloj de Origen',
  'Unidades',
  'Rango Declarado',
  'Vista Previa y Confirmación',
];

// Archivo elegido por el usuario — nombre + bytes reales. Costura pública
// (no privada) para que un test de widget pueda inyectar un archivo sin
// pasar por el diálogo nativo del sistema operativo.
class PickedImportFile {
  final String fileName;
  final List<int> fileBytes;
  const PickedImportFile({required this.fileName, required this.fileBytes});
}

// ---------------------------------------------------------------------------
// DataImportWizardInspectorPanel — widget principal del panel.
// ---------------------------------------------------------------------------

// StatefulWidget: gestiona el paso activo, el mapeo en construcción y el
// ciclo de vida de las cinco llamadas async (patrón G1: await directo, sin
// Timer.periodic). Cada llamada al puente tiene una costura `...Override`
// inyectable — mismo patrón que `loadRecordsOverride` en
// hybrid_data_transformer_dashboard_widget.dart y `verifyOverride` en
// generic_verification_section.dart: `flutter test` no tiene la librería
// nativa compilada, así que los tests de este panel inyectan dobles y nunca
// cruzan el puente FFI real.
class DataImportWizardInspectorPanel extends StatefulWidget {
  final Future<PickedImportFile?> Function()? pickFileOverride;
  final Future<SelectManualImportFileResultDto> Function(
      {required List<int> fileBytes})? selectFileOverride;
  final Future<ManualImportColumnsResultDto> Function(
      {required List<int> fileBytes})? getColumnsOverride;
  final Future<ManualImportPreviewResultDto> Function({
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
  })? previewOverride;
  final Future<ManualImportContentRangeResultDto> Function({
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
  })? analyzeRangeOverride;
  final Future<ManualImportConfirmResultDto> Function({
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
  })? confirmOverride;

  const DataImportWizardInspectorPanel({
    super.key,
    this.pickFileOverride,
    this.selectFileOverride,
    this.getColumnsOverride,
    this.previewOverride,
    this.analyzeRangeOverride,
    this.confirmOverride,
  });

  @override
  State<DataImportWizardInspectorPanel> createState() =>
      _DataImportWizardInspectorPanelState();
}

class _DataImportWizardInspectorPanelState
    extends State<DataImportWizardInspectorPanel> {
  // ---- Paso activo ----
  int _currentStep = 0;

  // ---- Paso 1: Archivo ----
  Uint8List? _fileBytes;
  String? _fileName;
  int? _fileSizeBytes;
  String? _fileHash;
  List<String> _columns = [];
  String? _fileError;
  bool _loadingFile = false;

  // ---- Paso 2: Mapeo ----
  final TextEditingController _instrumentIdCtrl = TextEditingController();
  final TextEditingController _venueCtrl = TextEditingController();
  final TextEditingController _sourceProviderCtrl = TextEditingController();
  int _timeframeIndex = 0;
  _TimestampMode _timestampMode = _TimestampMode.separate;
  String? _timestampColumn;
  String? _timestampDateColumn;
  String? _timestampTimeColumn;
  final Map<String, String?> _fieldColumns = {
    for (final f in _kNonTimestampFields) f.$1: null
  };

  // ---- Paso 3: Reloj de Origen ----
  final TextEditingController _anchorTimezoneCtrl =
      TextEditingController(text: 'UTC');
  double _offsetMinutes = 0;

  // ---- Paso 4: Unidades ----
  final TextEditingController _spreadUnitCtrl = TextEditingController();
  final TextEditingController _volumeUnitCtrl = TextEditingController();

  // ---- Paso 5: Rango Declarado ----
  custom_ui.DateRange? _declaredRange;

  // ---- Paso 6: Vista Previa y Confirmación ----
  ManualImportPreviewResultDto? _previewResult;
  bool _loadingPreview = false;
  ManualImportContentRangeResultDto? _rangeAnalysis;
  bool _loadingRangeAnalysis = false;
  ManualImportConfirmResultDto? _confirmResult;
  bool _loadingConfirm = false;

  // Fallo INESPERADO de cualquiera de las cinco operaciones: una excepción,
  // no un rechazo de negocio. Los rechazos de negocio viajan dentro del
  // propio resultado (`.error`) y ya se muestran donde corresponde; esto es
  // para lo que ni siquiera llega a devolver resultado.
  //
  // Por qué existe (gate de QA de STORY-073): un pánico del Núcleo Rust
  // cruza la frontera FFI como excepción de Dart. Sin captura, el botón se
  // quedaba en "Analizando…" para siempre y el usuario NUNCA veía un error
  // -- ni falso éxito, que estaba bien resuelto, pero tampoco el fallo. Con
  // 9,2 millones de filas basta una corrupta para llegar aquí.
  String? _unexpectedError;

  @override
  void dispose() {
    _instrumentIdCtrl.dispose();
    _venueCtrl.dispose();
    _sourceProviderCtrl.dispose();
    _anchorTimezoneCtrl.dispose();
    _spreadUnitCtrl.dispose();
    _volumeUnitCtrl.dispose();
    super.dispose();
  }

  // Cualquier cambio en el mapeo, el reloj, las unidades o el rango deja
  // obsoleta la vista previa y el análisis ya pedidos — nunca se muestra un
  // resultado que ya no corresponde a lo que el usuario acaba de declarar.
  // También cierra la puerta a confirmar con un análisis viejo (ver
  // `_confirmEnabled`).
  void _invalidateDownstreamResults() {
    _previewResult = null;
    _rangeAnalysis = null;
    _confirmResult = null;
  }

  // ---------------------------------------------------------------------------
  // Paso 1 — Archivo
  // ---------------------------------------------------------------------------

  // Diálogo nativo real del sistema operativo (file_selector) — fuera de la
  // librería de componentes por diseño (ver docs/features/data-import-wizard.md
  // "Notas de implementación"): Dropzone.onTap solo señala la intención, abrir
  // el selector de archivos es responsabilidad de esta llamada.
  //
  // Por qué `file_selector` y no `file_picker` (corregido 2026-08-01, reporte
  // del propietario): en Linux `file_picker` NO abre un diálogo propio —
  // lanza el ejecutable externo `zenity` (o `kdialog`/`qarma`) y falla con
  // "Couldn't find the executable zenity in the path" si no está instalado.
  // Eso es exactamente lo que pasó en Fedora Silverblue, y es una dependencia
  // que no podemos exigirle a la máquina de un usuario: el escritorio mínimo
  // de KDE no trae `zenity`, y una imagen atómica obliga a reiniciar para
  // instalarlo. `file_selector_linux` compila el diálogo GTK dentro del propio
  // plugin (`GtkFileChooserNative`, que además enruta al portal XDG cuando la
  // app corre en un sandbox), así que no depende de ningún binario externo.
  Future<PickedImportFile?> _pickFileReal() async {
    const typeGroup = XTypeGroup(
      label: 'Datos de mercado',
      extensions: ['csv', 'txt', 'tsv'],
    );
    final file = await openFile(acceptedTypeGroups: [typeGroup]);
    if (file == null) return null;
    // Lectura completa a memoria: mismo comportamiento que antes, y misma
    // deuda (DEBT-040) — el camino de streaming se diseña en la Story de
    // persistencia, no aquí.
    final bytes = await file.readAsBytes();
    return PickedImportFile(fileName: file.name, fileBytes: bytes);
  }

  // Dispara el selector de archivo real (Paso 1). Al resolver, encadena
  // operación 1/5 (huella) y 2/5 (columnas detectadas) — ambas NO reciben
  // db_path, se pueden pedir sin ningún efecto sobre el ledger.
  Future<void> _onDropzoneTap() async {
    await _runGuarded(_onDropzoneTapInner, () => _loadingFile = false);
  }

  Future<void> _onDropzoneTapInner() async {
    final picker = widget.pickFileOverride ?? _pickFileReal;
    final picked = await picker();
    if (picked == null || !mounted) return;

    setState(() {
      _fileBytes = Uint8List.fromList(picked.fileBytes);
      _fileName = picked.fileName;
      _columns = [];
      _fileError = null;
      _unexpectedError = null;
      _loadingFile = true;
      _invalidateDownstreamResults();
    });

    final selectFn = widget.selectFileOverride ?? selectManualImportFile;
    final columnsFn = widget.getColumnsOverride ?? getManualImportColumns;
    final selected = await selectFn(fileBytes: _fileBytes!);
    final cols = await columnsFn(fileBytes: _fileBytes!);
    if (!mounted) return;

    setState(() {
      _fileSizeBytes = selected.fileSizeBytes.toInt();
      _fileHash = selected.sourceFileHash;
      _columns = cols.columns;
      _fileError = cols.error;
      _loadingFile = false;
    });
  }

  Widget _buildPasoArchivo() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text('Archivo de origen', style: Gx.microLabel),
        const SizedBox(height: Gx.space12),
        if (_fileBytes == null)
          custom_ui.Dropzone(
            onTap: _onDropzoneTap,
            // El texto NO promete arrastrar: soltar un archivo aquí todavía
            // no hace nada (ver cabecera de `components/dropzone.dart`).
            label: 'Toca para elegir tu archivo CSV/TXT/TSV',
          )
        else if (_loadingFile)
          const _CargandoRow(mensaje: 'Leyendo columnas del archivo…')
        else ...[
          custom_ui.KeyValue(label: 'Nombre', value: _fileName ?? '—'),
          custom_ui.KeyValue(
            label: 'Tamaño',
            value: _fileSizeBytes != null
                ? '${(_fileSizeBytes! / (1024 * 1024)).toStringAsFixed(1)} MB'
                : '—',
            mono: true,
          ),
          custom_ui.KeyValue(
              label: 'Huella SHA-256',
              value: _fileHash ?? '—',
              mono: true,
              // 64 caracteres hexadecimales: el ejemplo canónico de valor que
              // no se transcribe a mano sin equivocarse.
              copyable: true),
          const SizedBox(height: Gx.space12),
          if (_fileError != null)
            custom_ui.Banner(
                message: _fileError!,
                type: custom_ui.BannerType.error,
                copyable: true)
          else
            custom_ui.KeyValue(
                label: 'Columnas detectadas',
                value: '${_columns.length}',
                mono: true),
          const SizedBox(height: Gx.space12),
          custom_ui.Button(
            label: 'Elegir otro archivo',
            variant: custom_ui.ButtonVariant.ghost,
            onPressed: _onDropzoneTap,
          ),
        ],
      ],
    );
  }

  // ---------------------------------------------------------------------------
  // Paso 2 — Mapeo
  // ---------------------------------------------------------------------------

  // Construye el mapeo completo tal como el puente lo espera, o `null` si
  // falta declarar algún campo obligatorio — nunca se asume ni se completa a
  // medias (criterio de aceptación #6 de STORY-073).
  List<ManualColumnMappingDto>? _buildColumnMappings() {
    final mappings = <ManualColumnMappingDto>[];
    if (_timestampMode == _TimestampMode.combined) {
      if (_timestampColumn == null) return null;
      mappings.add(ManualColumnMappingDto(
          sourceColumn: _timestampColumn!, canonicalField: 'TIMESTAMP'));
    } else {
      if (_timestampDateColumn == null || _timestampTimeColumn == null) {
        return null;
      }
      mappings.add(ManualColumnMappingDto(
          sourceColumn: _timestampDateColumn!,
          canonicalField: 'TIMESTAMP_DATE'));
      mappings.add(ManualColumnMappingDto(
          sourceColumn: _timestampTimeColumn!,
          canonicalField: 'TIMESTAMP_TIME'));
    }
    for (final f in _kNonTimestampFields) {
      final col = _fieldColumns[f.$1];
      if (col == null) return null;
      mappings
          .add(ManualColumnMappingDto(sourceColumn: col, canonicalField: f.$1));
    }
    return mappings;
  }

  Widget _buildPasoMapeo() {
    if (_columns.isEmpty) {
      return custom_ui.Empty(
        message: 'Sin columnas para mapear.',
        subtitle: 'Selecciona un archivo en el Paso 1 primero.',
        icon: Icons.table_chart_outlined,
      );
    }

    final dropdownItems = [
      for (final c in _columns) custom_ui.DropdownItem(value: c, label: c)
    ];

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text('Identidad de la serie', style: Gx.microLabel),
        const SizedBox(height: Gx.space8),
        custom_ui.FormField(
          label: 'Instrumento',
          // onChanged solo dispara un rebuild — el valor real vive en el
          // controller. Sin este `setState`, el gate de "Siguiente"
          // (`_canAdvanceFromStep`) queda con el veredicto del último build
          // y no reacciona a lo que el usuario tecleó (bug real hallado por
          // esta misma Story: sin este callback, el botón se queda
          // deshabilitado aunque el campo ya tenga texto).
          child: custom_ui.Input(
            controller: _instrumentIdCtrl,
            hint: 'Ej. EURUSD',
            onChanged: (_) => setState(_invalidateDownstreamResults),
          ),
        ),
        const SizedBox(height: Gx.space12),
        custom_ui.FormField(
          label: 'Venue',
          child: custom_ui.Input(
            controller: _venueCtrl,
            hint: 'Ej. GENERIC_BROKER',
            onChanged: (_) => setState(_invalidateDownstreamResults),
          ),
        ),
        const SizedBox(height: Gx.space12),
        custom_ui.FormField(
          label: 'Proveedor de datos',
          child: custom_ui.Input(
            controller: _sourceProviderCtrl,
            hint: 'Ej. MANUAL_UPLOAD',
            onChanged: (_) => setState(_invalidateDownstreamResults),
          ),
        ),
        const SizedBox(height: Gx.space16),
        Text('Temporalidad de la barra', style: Gx.microLabel),
        const SizedBox(height: Gx.space8),
        custom_ui.Dropdown<int>(
          value: _timeframeIndex,
          items: [
            for (var i = 0; i < _kTimeframePresets.length; i++)
              custom_ui.DropdownItem(
                  value: i, label: _kTimeframePresets[i].label),
          ],
          onChanged: (v) => setState(() {
            _timeframeIndex = v;
            _invalidateDownstreamResults();
          }),
        ),
        const SizedBox(height: Gx.space16),
        Text('Marca de tiempo', style: Gx.microLabel),
        const SizedBox(height: Gx.space8),
        custom_ui.Tooltip(
          message:
              'Columna combinada: una sola columna trae fecha y hora juntas. '
              'Fecha y hora separadas: el archivo trae dos columnas — la forma '
              'más común de la industria (ej. MetaTrader: <DATE> y <TIME>).',
          child: custom_ui.Segmented(
            options: const ['Columna combinada', 'Fecha y hora separadas'],
            selected: _timestampMode == _TimestampMode.combined ? 0 : 1,
            onChanged: (i) => setState(() {
              _timestampMode =
                  i == 0 ? _TimestampMode.combined : _TimestampMode.separate;
              _timestampColumn = null;
              _timestampDateColumn = null;
              _timestampTimeColumn = null;
              _invalidateDownstreamResults();
            }),
          ),
        ),
        const SizedBox(height: Gx.space12),
        if (_timestampMode == _TimestampMode.combined)
          custom_ui.FormField(
            label: 'Columna de Timestamp',
            child: custom_ui.Dropdown<String>(
              value: _timestampColumn,
              hint: 'Selecciona la columna',
              items: dropdownItems,
              onChanged: (v) => setState(() {
                _timestampColumn = v;
                _invalidateDownstreamResults();
              }),
            ),
          )
        else ...[
          custom_ui.FormField(
            label: 'Columna de Fecha',
            child: custom_ui.Dropdown<String>(
              value: _timestampDateColumn,
              hint: 'Selecciona la columna',
              items: dropdownItems,
              onChanged: (v) => setState(() {
                _timestampDateColumn = v;
                _invalidateDownstreamResults();
              }),
            ),
          ),
          const SizedBox(height: Gx.space12),
          custom_ui.FormField(
            label: 'Columna de Hora',
            child: custom_ui.Dropdown<String>(
              value: _timestampTimeColumn,
              hint: 'Selecciona la columna',
              items: dropdownItems,
              onChanged: (v) => setState(() {
                _timestampTimeColumn = v;
                _invalidateDownstreamResults();
              }),
            ),
          ),
        ],
        const SizedBox(height: Gx.space16),
        Text('Campos de la barra', style: Gx.microLabel),
        const SizedBox(height: Gx.space8),
        for (final f in _kNonTimestampFields) ...[
          custom_ui.FormField(
            label: f.$2,
            child: custom_ui.Dropdown<String>(
              value: _fieldColumns[f.$1],
              hint: 'Selecciona la columna',
              items: dropdownItems,
              onChanged: (v) => setState(() {
                _fieldColumns[f.$1] = v;
                _invalidateDownstreamResults();
              }),
            ),
          ),
          const SizedBox(height: Gx.space12),
        ],
      ],
    );
  }

  // ---------------------------------------------------------------------------
  // Paso 3 — Reloj de Origen
  // ---------------------------------------------------------------------------

  Widget _buildPasoRelojOrigen() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text('Reloj de origen', style: Gx.microLabel),
        const SizedBox(height: Gx.space8),
        custom_ui.FormField(
          label: 'Zona ancla (IANA)',
          helperText:
              'Ej. "America/New_York", "Europe/London", "UTC". Un reloj sin '
              'desfase se declara con zona UTC y desplazamiento 0.',
          child: custom_ui.Input(
            controller: _anchorTimezoneCtrl,
            hint: 'UTC',
            onChanged: (_) => setState(_invalidateDownstreamResults),
          ),
        ),
        const SizedBox(height: Gx.space16),
        custom_ui.FormField(
          label: 'Desplazamiento fijo (minutos)',
          helperText:
              'Corrección declarada sobre la zona ancla — 0 si no aplica.',
          child: custom_ui.NumberInput(
            value: _offsetMinutes,
            min: -1440,
            max: 1440,
            step: 1,
            onChanged: (v) => setState(() {
              _offsetMinutes = v;
              _invalidateDownstreamResults();
            }),
          ),
        ),
      ],
    );
  }

  // ---------------------------------------------------------------------------
  // Paso 4 — Unidades
  // ---------------------------------------------------------------------------

  Widget _buildPasoUnidades() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text('Unidades declaradas', style: Gx.microLabel),
        const SizedBox(height: Gx.space8),
        custom_ui.FormField(
          label: 'Unidad del spread reportado',
          helperText:
              'Obligatoria — ej. "POINTS_MT5", "PIPS". Sin ella la importación se rechaza.',
          child: custom_ui.Input(
            controller: _spreadUnitCtrl,
            hint: 'Ej. PIPS',
            // Este `onChanged` es el que hace que "Siguiente" reaccione en
            // vivo al tecleo — ver la nota completa junto al Input de
            // "Instrumento" en el Paso 2.
            onChanged: (_) => setState(_invalidateDownstreamResults),
          ),
        ),
        const SizedBox(height: Gx.space16),
        custom_ui.FormField(
          label: 'Unidad real del volumen',
          helperText:
              'Opcional — ej. "CONTRACTS", "LOTS". Si la dejas vacía, el volumen '
              'se conserva pero se marca como unidad NO verificada — nunca se asume.',
          child: custom_ui.Input(
            controller: _volumeUnitCtrl,
            hint: 'Ej. CONTRACTS',
            onChanged: (_) => setState(_invalidateDownstreamResults),
          ),
        ),
      ],
    );
  }

  // ---------------------------------------------------------------------------
  // Paso 5 — Rango Declarado
  // ---------------------------------------------------------------------------

  Widget _buildPasoRangoDeclarado() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text('Rango que este tramo declara', style: Gx.microLabel),
        const SizedBox(height: Gx.space8),
        custom_ui.Tooltip(
          message:
              'Un tramo es una afirmación de procedencia, no de presencia de '
              'dato: declarar un rango con huecos internos es legítimo. La '
              'Etapa 6 te muestra el contenido real para que contrastes.',
          child: custom_ui.DateRangePicker(
            range: _declaredRange,
            onChanged: (r) => setState(() {
              _declaredRange = r;
              _invalidateDownstreamResults();
            }),
          ),
        ),
      ],
    );
  }

  // ---------------------------------------------------------------------------
  // Paso 6 — Vista Previa y Confirmación
  // ---------------------------------------------------------------------------

  // Conversión de fecha de calendario a nanosegundos UTC — serialización de
  // frontera FFI, no cálculo de dominio (ver doc-comment de módulo).
  int _startOfDayNs(DateTime d) =>
      DateTime.utc(d.year, d.month, d.day).microsecondsSinceEpoch * 1000;
  int _endOfDayNs(DateTime d) =>
      DateTime.utc(d.year, d.month, d.day, 23, 59, 59, 999)
          .microsecondsSinceEpoch *
      1000;

  // Formatea nanosegundos UTC a fecha legible — presentación pura (cambio de
  // unidad para lectura humana), mismo criterio que `_formatDuracion` en
  // hybrid_data_transformer_inspector.dart.
  String _formatTimestampNs(int ns) {
    final dt = DateTime.fromMicrosecondsSinceEpoch(ns ~/ 1000, isUtc: true);
    String p2(int v) => v.toString().padLeft(2, '0');
    return '${dt.year}-${p2(dt.month)}-${p2(dt.day)} ${p2(dt.hour)}:${p2(dt.minute)} UTC';
  }

  // Ejecuta una operación del puente capturando cualquier excepción. Sin
  // esto, una excepción deja el indicador de carga encendido para siempre y
  // el usuario no ve nada -- ver `_unexpectedError`. `onFailure` apaga el
  // indicador concreto de quien llamó; la excepción se muestra siempre.
  Future<void> _runGuarded(Future<void> Function() body, void Function() onFailure) async {
    try {
      await body();
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _unexpectedError = e.toString();
        onFailure();
      });
    }
  }

  Future<void> _fetchPreview() async {
    await _runGuarded(_fetchPreviewInner, () => _loadingPreview = false);
  }

  Future<void> _fetchPreviewInner() async {
    final mappings = _buildColumnMappings();
    if (mappings == null || _fileBytes == null) return;
    setState(() {
      _loadingPreview = true;
      _unexpectedError = null;
    });
    final fn = widget.previewOverride ?? previewManualBarImport;
    final result = await fn(
      fileBytes: _fileBytes!,
      columnMappings: mappings,
      instrumentId: _instrumentIdCtrl.text.trim(),
      venue: _venueCtrl.text.trim(),
      sourceProvider: _sourceProviderCtrl.text.trim(),
      sourceClockAnchorTimezoneRef: _anchorTimezoneCtrl.text.trim().isEmpty
          ? null
          : _anchorTimezoneCtrl.text.trim(),
      sourceClockFixedOffsetMinutes: _offsetMinutes.round(),
      realVolumeUnitDeclared: _volumeUnitCtrl.text.trim().isEmpty
          ? null
          : _volumeUnitCtrl.text.trim(),
      spreadReportedUnitDeclared: _spreadUnitCtrl.text.trim().isEmpty
          ? null
          : _spreadUnitCtrl.text.trim(),
      timeframeValue: _kTimeframePresets[_timeframeIndex].timeframeValue,
      barIntervalNs: _kTimeframePresets[_timeframeIndex].barIntervalNs,
    );
    if (!mounted) return;
    setState(() {
      _previewResult = result;
      _loadingPreview = false;
    });
  }

  // Operación CARA — se pide UNA vez, con el mapeo ya estable (ver
  // doc-comment de módulo y `crates/bridge/.../data_import_wizard.rs`). Da el
  // dato que el criterio 5b de STORY-073 / DEBT-039 exige: la primera y la
  // última fecha que el archivo REALMENTE contiene, junto al rango declarado.
  Future<void> _fetchRangeAnalysis() async {
    await _runGuarded(_fetchRangeAnalysisInner, () => _loadingRangeAnalysis = false);
  }

  Future<void> _fetchRangeAnalysisInner() async {
    final mappings = _buildColumnMappings();
    if (mappings == null || _fileBytes == null) return;
    setState(() {
      _loadingRangeAnalysis = true;
      _unexpectedError = null;
    });
    final fn = widget.analyzeRangeOverride ?? analyzeManualBarFileContentRange;
    final result = await fn(
      fileBytes: _fileBytes!,
      columnMappings: mappings,
      instrumentId: _instrumentIdCtrl.text.trim(),
      venue: _venueCtrl.text.trim(),
      sourceProvider: _sourceProviderCtrl.text.trim(),
      sourceClockAnchorTimezoneRef: _anchorTimezoneCtrl.text.trim().isEmpty
          ? null
          : _anchorTimezoneCtrl.text.trim(),
      sourceClockFixedOffsetMinutes: _offsetMinutes.round(),
      realVolumeUnitDeclared: _volumeUnitCtrl.text.trim().isEmpty
          ? null
          : _volumeUnitCtrl.text.trim(),
      spreadReportedUnitDeclared: _spreadUnitCtrl.text.trim().isEmpty
          ? null
          : _spreadUnitCtrl.text.trim(),
      timeframeValue: _kTimeframePresets[_timeframeIndex].timeframeValue,
      barIntervalNs: _kTimeframePresets[_timeframeIndex].barIntervalNs,
    );
    if (!mounted) return;
    setState(() {
      _rangeAnalysis = result;
      _loadingRangeAnalysis = false;
    });
  }

  // La ÚNICA de las seis operaciones que escribe. Solo se habilita cuando el
  // usuario ya vio el contraste rango declarado / contenido real (criterio 5b
  // — LDR-0008: lo que se firma se vio antes).
  bool get _confirmEnabled {
    return !_loadingConfirm &&
        _fileBytes != null &&
        _buildColumnMappings() != null &&
        _instrumentIdCtrl.text.trim().isNotEmpty &&
        _venueCtrl.text.trim().isNotEmpty &&
        _sourceProviderCtrl.text.trim().isNotEmpty &&
        _spreadUnitCtrl.text.trim().isNotEmpty &&
        _declaredRange != null &&
        _rangeAnalysis != null &&
        _rangeAnalysis!.error == null;
  }

  Future<void> _confirmar() async {
    await _runGuarded(_confirmarInner, () => _loadingConfirm = false);
  }

  Future<void> _confirmarInner() async {
    final mappings = _buildColumnMappings();
    if (mappings == null || _fileBytes == null || _declaredRange == null) {
      return;
    }
    setState(() {
      _loadingConfirm = true;
      _confirmResult = null;
      _unexpectedError = null;
    });
    final fn = widget.confirmOverride ?? confirmManualBarImport;
    final result = await fn(
      dbPath: _kDbPath,
      sourceFileName: _fileName ?? 'archivo.csv',
      fileBytes: _fileBytes!,
      instrumentId: _instrumentIdCtrl.text.trim(),
      venue: _venueCtrl.text.trim(),
      sourceProvider: _sourceProviderCtrl.text.trim(),
      declaredRangeStartNs: _startOfDayNs(_declaredRange!.start),
      declaredRangeEndNs: _endOfDayNs(_declaredRange!.end),
      sourceClockAnchorTimezoneRef: _anchorTimezoneCtrl.text.trim().isEmpty
          ? null
          : _anchorTimezoneCtrl.text.trim(),
      sourceClockFixedOffsetMinutes: _offsetMinutes.round(),
      realVolumeUnitDeclared: _volumeUnitCtrl.text.trim().isEmpty
          ? null
          : _volumeUnitCtrl.text.trim(),
      spreadReportedUnitDeclared: _spreadUnitCtrl.text.trim().isEmpty
          ? null
          : _spreadUnitCtrl.text.trim(),
      // venue_operational_since_ns: sin control en esta Cáscara Visual — el
      // DTO lo declara opcional y ninguna fila del catálogo de componentes
      // sellado pide un control para él en el Paso 5. Reportado al Tech-Lead.
      venueOperationalSinceNs: null,
      timeframeValue: _kTimeframePresets[_timeframeIndex].timeframeValue,
      barIntervalNs: _kTimeframePresets[_timeframeIndex].barIntervalNs,
      columnMappings: mappings,
    );
    if (!mounted) return;
    setState(() {
      _confirmResult = result;
      _loadingConfirm = false;
    });
  }

  Widget _buildPasoVistaPreviaConfirmacion() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text('Vista previa (dato ya transformado)', style: Gx.microLabel),
        const SizedBox(height: Gx.space8),
        custom_ui.Button(
          label: _loadingPreview ? 'Actualizando…' : 'Actualizar vista previa',
          variant: custom_ui.ButtonVariant.secondary,
          loading: _loadingPreview,
          onPressed: _loadingPreview ? null : _fetchPreview,
        ),
        const SizedBox(height: Gx.space12),
        if (_previewResult != null) _buildPreviewResult(_previewResult!),
        const SizedBox(height: Gx.space24),
        Container(height: 1, color: Gx.divider),
        const SizedBox(height: Gx.space24),
        Text('Rango real del archivo — criterio 5b (DEBT-039)',
            style: Gx.microLabel),
        const SizedBox(height: Gx.space8),
        custom_ui.Button(
          label: _loadingRangeAnalysis
              ? 'Analizando…'
              : 'Analizar archivo completo (una vez)',
          variant: custom_ui.ButtonVariant.secondary,
          loading: _loadingRangeAnalysis,
          onPressed: _loadingRangeAnalysis ? null : _fetchRangeAnalysis,
        ),
        const SizedBox(height: Gx.space12),
        if (_rangeAnalysis != null) _buildRangeAnalysisResult(_rangeAnalysis!),
        const SizedBox(height: Gx.space24),
        Container(height: 1, color: Gx.divider),
        const SizedBox(height: Gx.space24),
        Center(
          child: custom_ui.Button(
            label: _loadingConfirm ? 'Confirmando…' : 'Confirmar e Importar',
            variant: custom_ui.ButtonVariant.primary,
            enabled: _confirmEnabled,
            loading: _loadingConfirm,
            onPressed: _confirmEnabled ? _confirmar : null,
          ),
        ),
        const SizedBox(height: Gx.space12),
        if (_confirmResult != null) _buildConfirmResult(_confirmResult!),
      ],
    );
  }

  Widget _buildPreviewResult(ManualImportPreviewResultDto preview) {
    if (preview.error != null) {
      return custom_ui.Banner(
          message: preview.error!,
          type: custom_ui.BannerType.error,
          copyable: true);
    }
    if (preview.rows.isEmpty) {
      return custom_ui.Empty(message: 'La vista previa no produjo filas.');
    }
    final cellStyle = Gx.dataMono(fontSize: 12, color: Gx.textBase);
    return custom_ui.Surface(
      padding: const EdgeInsets.all(Gx.space8),
      child: SingleChildScrollView(
        scrollDirection: Axis.horizontal,
        child: custom_ui.Table(
          columns: const [
            custom_ui.TableColumn(label: 'Apertura (ns UTC)', numeric: true),
            custom_ui.TableColumn(label: 'Cierre', numeric: true),
            custom_ui.TableColumn(label: 'Volumen operado', numeric: true),
            custom_ui.TableColumn(label: 'Spread', numeric: true),
            custom_ui.TableColumn(label: 'Estado'),
          ],
          rows: [
            for (final row in preview.rows)
              [
                Text(row.barOpenTimestampNs.toString(), style: cellStyle),
                Text(formatScaledInt(row.close), style: cellStyle),
                Row(mainAxisSize: MainAxisSize.min, children: [
                  // La unidad se imprime JUNTO al número, igual que en la
                  // columna del spread. Sale de lo que el usuario declaró en
                  // el paso anterior: esta es la pantalla donde confirma su
                  // dato ya transformado (LDR-0008 condición 1), y un número
                  // desnudo no es confirmable. Si no declaró unidad, no se
                  // inventa ninguna -- se marca con el chip de abajo.
                  Text(
                      row.volumeUnitUnverified
                          ? row.volumeTradedRaw.toString()
                          : '${row.volumeTradedRaw} ${_volumeUnitCtrl.text.trim()}',
                      style: cellStyle),
                  if (row.volumeUnitUnverified) ...[
                    const SizedBox(width: 6),
                    // Unidad no verificada: marca visual distinta — nunca se
                    // colapsa "no verificada" en "verificada" (lección
                    // "unidad declarada junto al dato").
                    custom_ui.Chip(
                        label: 'unidad no verificada',
                        status: custom_ui.ChipStatus.alert),
                  ],
                ]),
                Text('${row.spreadReportedRaw} ${row.spreadReportedUnit}',
                    style: cellStyle),
                row.isActivityAnomaly
                    ? custom_ui.Chip(
                        label: 'anomalía de actividad',
                        status: custom_ui.ChipStatus.critical)
                    : Text('normal', style: cellStyle),
              ],
          ],
        ),
      ),
    );
  }

  Widget _buildRangeAnalysisResult(ManualImportContentRangeResultDto analysis) {
    if (analysis.error != null) {
      return custom_ui.Banner(
          message: analysis.error!,
          type: custom_ui.BannerType.error,
          copyable: true);
    }
    final declared = _declaredRange;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        custom_ui.KeyValue(
            label: 'Filas totales del archivo',
            value: analysis.rowsParsedTotal.toString(),
            mono: true),
        const SizedBox(height: Gx.space8),
        Text('Contenido real vs. declarado',
            style: Gx.uiSans(fontSize: 12, color: Gx.textBaseLabel)),
        const SizedBox(height: 4),
        custom_ui.KeyValue(
          label: 'Primera fecha REAL del archivo',
          value: analysis.contentMinOpenTimestampNs != null
              ? _formatTimestampNs(analysis.contentMinOpenTimestampNs!)
              : 'sin filas',
          mono: true,
        ),
        custom_ui.KeyValue(
          label: 'Última fecha REAL del archivo',
          value: analysis.contentMaxCloseTimestampNs != null
              ? _formatTimestampNs(analysis.contentMaxCloseTimestampNs!)
              : 'sin filas',
          mono: true,
        ),
        custom_ui.KeyValue(
          label: 'Inicio DECLARADO por ti (Paso 5)',
          value: declared != null
              ? _formatTimestampNs(_startOfDayNs(declared.start))
              : '—',
          mono: true,
        ),
        custom_ui.KeyValue(
          label: 'Fin DECLARADO por ti (Paso 5)',
          value: declared != null
              ? _formatTimestampNs(_endOfDayNs(declared.end))
              : '—',
          mono: true,
        ),
      ],
    );
  }

  Widget _buildConfirmResult(ManualImportConfirmResultDto result) {
    if (!result.ok) {
      // Rechazo: mensaje literal del backend, nunca una importación parcial
      // visible (criterio de aceptación #11 de STORY-073).
      return custom_ui.Banner(
          message: result.error ?? 'La importación fue rechazada.',
          type: custom_ui.BannerType.error,
          copyable: true);
    }
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        custom_ui.Banner(
          message: result.wasNoopReimport
              ? 'Este tramo ya estaba importado — no se duplicó nada.'
              : 'Importación sellada con éxito.',
          type: result.wasNoopReimport
              ? custom_ui.BannerType.info
              : custom_ui.BannerType.success,
        ),
        const SizedBox(height: Gx.space12),
        custom_ui.KeyValue(
            label: 'mapping_origin', value: result.mappingOrigin, mono: true),
        custom_ui.KeyValue(
            label: 'declaration_id', value: result.declarationId, mono: true),
        custom_ui.KeyValue(
            label: 'event_sequence_id',
            value: result.eventSequenceId.toString(),
            mono: true),
        custom_ui.KeyValue(
            label: 'audit_hash', value: result.auditHash, mono: true),
        custom_ui.KeyValue(
            label: 'Filas importadas',
            value: result.rowsImported.toString(),
            mono: true),
      ],
    );
  }

  // ---------------------------------------------------------------------------
  // Navegación del stepper — bloquea el avance con datos incompletos
  // (criterio de aceptación #10 de STORY-073).
  // ---------------------------------------------------------------------------

  bool _canAdvanceFromStep(int step) {
    switch (step) {
      case 0:
        return _fileBytes != null && _columns.isNotEmpty && _fileError == null;
      case 1:
        return _buildColumnMappings() != null &&
            _instrumentIdCtrl.text.trim().isNotEmpty &&
            _venueCtrl.text.trim().isNotEmpty &&
            _sourceProviderCtrl.text.trim().isNotEmpty;
      case 2:
        // El desplazamiento siempre tiene un valor válido (default 0); la
        // zona ancla es opcional en el contrato (`Option<String>`).
        return true;
      case 3:
        return _spreadUnitCtrl.text.trim().isNotEmpty;
      case 4:
        return _declaredRange != null;
      default:
        return true;
    }
  }

  String? _blockedHint(int step) {
    switch (step) {
      case 0:
        return 'Selecciona un archivo válido para continuar.';
      case 1:
        return 'Declara la identidad de la serie y mapea todos los campos obligatorios.';
      case 3:
        return 'Declara la unidad del spread reportado — es obligatoria.';
      case 4:
        return 'Declara el rango de fechas que este tramo cubre.';
      default:
        return null;
    }
  }

  Widget _buildStepContent() {
    switch (_currentStep) {
      case 0:
        return _buildPasoArchivo();
      case 1:
        return _buildPasoMapeo();
      case 2:
        return _buildPasoRelojOrigen();
      case 3:
        return _buildPasoUnidades();
      case 4:
        return _buildPasoRangoDeclarado();
      default:
        return _buildPasoVistaPreviaConfirmacion();
    }
  }

  Widget _buildNavBar() {
    final canAdvance = _canAdvanceFromStep(_currentStep);
    final hint = !canAdvance ? _blockedHint(_currentStep) : null;
    final isLastStep = _currentStep == _kStepLabels.length - 1;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        if (hint != null) ...[
          Text(hint, style: Gx.uiSans(fontSize: 11, color: Gx.criticalCrimson)),
          const SizedBox(height: Gx.space8),
        ],
        Row(
          children: [
            if (_currentStep > 0)
              custom_ui.Button(
                label: 'Atrás',
                variant: custom_ui.ButtonVariant.ghost,
                onPressed: () => setState(() => _currentStep--),
              ),
            const Spacer(),
            if (!isLastStep)
              custom_ui.Button(
                label: 'Siguiente',
                variant: custom_ui.ButtonVariant.primary,
                enabled: canAdvance,
                onPressed:
                    canAdvance ? () => setState(() => _currentStep++) : null,
              ),
          ],
        ),
      ],
    );
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        // Cabecera fija: stepper de 6 pasos (currentStep externo — sin
        // onStepTapped, nunca navega libre, solo Atrás/Siguiente lo mueven).
        custom_ui.Stepper(steps: _kStepLabels, currentStep: _currentStep),
        const SizedBox(height: Gx.space16),
        // Fallo inesperado (excepción, no rechazo de negocio): se muestra
        // SIEMPRE y en la cabecera, sea cual sea el paso en que ocurrió --
        // el usuario tiene que enterarse aunque haya navegado a otro paso.
        if (_unexpectedError != null) ...[
          custom_ui.Banner(
              message:
                  'Fallo inesperado durante la operación: ${_unexpectedError!}',
              type: custom_ui.BannerType.error,
              // Copiable: este es el texto que el propietario tuvo que
              // transcribir a mano para reportarlo (2026-08-01).
              copyable: true),
          const SizedBox(height: Gx.space16),
        ],
        Expanded(
          child: SingleChildScrollView(
            child: _buildStepContent(),
          ),
        ),
        const SizedBox(height: Gx.space16),
        _buildNavBar(),
      ],
    );
  }
}

// Fila de carga: spinner discreto + mensaje — presentación pura.
class _CargandoRow extends StatelessWidget {
  final String mensaje;
  const _CargandoRow({required this.mensaje});

  @override
  Widget build(BuildContext context) {
    return Row(children: [
      const SizedBox(
        width: 14,
        height: 14,
        child: CircularProgressIndicator(strokeWidth: 1.5),
      ),
      const SizedBox(width: Gx.space12),
      Text(mensaje,
          style: Gx.uiSans(fontSize: 12, color: Gx.textBaseSecondary)),
    ]);
  }
}
