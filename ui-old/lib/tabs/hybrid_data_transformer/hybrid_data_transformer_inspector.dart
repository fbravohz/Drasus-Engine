// hybrid_data_transformer_inspector.dart — Panel Inspector de la feature
// `hybrid-data-transformer` (STORY-051 Etapa 4, `docs/features/hybrid-data-transformer.md`
// "## Cáscara Visual" / "Contexto de superficie").
//
// Por el "Contrato del nodo Canvas por-feature" (ADR-0136), esta superficie
// se abre al hacer clic en el card-node de la feature en el Canvas — Vista
// Relacional. **La infraestructura genérica ya existe** (STORY-073,
// 2026-07-31): `canvas_inspector_registry.dart` mapea nombre de feature ->
// constructor de panel, y `canvas_tab.dart` despacha por ese mapa sin
// ninguna rama por feature. Este panel estuvo meses construido y
// referenciado desde ninguna parte; se adoptó añadiendo UNA entrada al
// registro, que fue la prueba objetiva de que el mecanismo era genérico de
// verdad y no un caso especial disfrazado. La nota anterior de este
// encabezado decía lo contrario (cableado especial, infraestructura
// pendiente de EPIC-8) y quedó obsoleta -- corregida al detectarlo el gate
// de QA de STORY-073.
//
// Cáscara delgada (ADR-0117): cero lógica de negocio. Los únicos cómputos en
// este archivo son de presentación (formatear duración, derivar
// timeframe_value en mayúsculas desde el dropdown de agregación) — nunca se
// calcula, redondea ni deriva una magnitud financiera.

import 'dart:io' show Platform;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart' show Clipboard, ClipboardData;
import 'package:iconsax_plus/iconsax_plus.dart';

import '../../components/components.dart' as custom_ui;
import '../../gallery/gallery_fx.dart' show ScanRingWidget;
import '../../theme/gx_tokens.dart';
import '../../src/rust/api/hybrid_data_transformer.dart';
import 'optional_magnitude_text.dart';

// Ruta a la base de datos SQLite de Drasus — mismo valor que el resto de
// las Cáscaras Delgadas de EPIC-1 (jobs_tab.dart, sovereign_data_fetcher_section.dart).
const String _kDbPath = 'drasus.db';

// Vocabulario FIJO de agregación destino — idéntico al que valida
// `parse_target_aggregation` en el Núcleo (ver doc-comment de
// `submitHybridDataTransformation` en el binding generado).
const List<String> _kTargetAggregations = ['1m', '5m', '15m', '1h', '4h', '1d'];

// Máximo de filas de la tabla de barras que se renderizan — el conteo real
// siempre está disponible en el key-value "Barras generadas"; renderizar
// miles de filas en un panel lateral de ~400px no aporta lectura humana.
const int _kMaxBarsRenderizadas = 50;

// ---------------------------------------------------------------------------
// HybridDataTransformerInspectorPanel — widget principal del panel.
// ---------------------------------------------------------------------------

// StatefulWidget: gestiona el plan de transformación editable, el ciclo de
// vida del envío async (patrón G1: await directo, sin Timer.periodic) y el
// resultado de la última corrida.
class HybridDataTransformerInspectorPanel extends StatefulWidget {
  // Costura de prueba (STORY-073 Etapa 4, mismo patrón que `loadRecordsOverride`
  // en hybrid_data_transformer_dashboard_widget.dart): `flutter test` no tiene
  // la librería nativa compilada, así que un test de widget que alcance este
  // panel desde el lienzo (criterio de aceptación #9) inyecta un doble aquí
  // en vez de cruzar el puente FFI real en `initState()`.
  final Future<List<SourceDatasetDto>> Function({required String dbPath})?
      loadDatasetsOverride;

  const HybridDataTransformerInspectorPanel(
      {super.key, this.loadDatasetsOverride});

  @override
  State<HybridDataTransformerInspectorPanel> createState() =>
      _HybridDataTransformerInspectorPanelState();
}

class _HybridDataTransformerInspectorPanelState
    extends State<HybridDataTransformerInspectorPanel> {
  // ---- Dataset de origen ----
  List<SourceDatasetDto> _datasets = [];
  String? _selectedDataSnapshotId;
  bool _loadingDatasets = true;

  // ---- Plan de transformación ----
  String _targetAggregation = '1h';
  double _interopThreshold = 100000;
  bool _hilosAutomatico = true;
  late double _hilosManual;

  // ---- Identidad de la serie ----
  // NOTA DE DISEÑO (Flutter-Engineer, STORY-051 Etapa 4): la Cáscara Visual
  // original (docs/features/hybrid-data-transformer.md, 2026-07-17) solo
  // especificaba 4 controles (dataset/agregación/umbral/hilos) porque en ese
  // momento el contrato FFI no exigía identidad de serie. El retrofit
  // STORY-057 (posterior) volvió obligatorios `instrument_id`/`venue`/
  // `source_provider` en `submitHybridDataTransformation` — `SourceDatasetDto`
  // (`listSourceDatasets`) no los transporta y no existe ninguna otra fuente
  // para ellos: sin estos campos el botón "Ejecutar Transformación" no puede
  // construir una llamada válida. Se añaden aquí como inputs mínimos, con el
  // mismo vocabulario que ya usa el JSON de ejemplo del harness genérico
  // (BTCUSDT/BINANCE) — no se inventan. `bar_type` se fija en `"TIME"`
  // (única salida que esta feature produce agregando `TradeTick`, ver
  // doc-comment de `BarDto.barType`) y `timeframe_value` se deriva del
  // dropdown de agregación en mayúsculas (formateo de texto, no un control
  // nuevo). Gap reportado al Tech-Lead en el cierre de la Story para que se
  // decida si `SourceDatasetDto` se enriquece en una Story futura.
  final TextEditingController _instrumentIdCtrl = TextEditingController();
  final TextEditingController _venueCtrl = TextEditingController();
  final TextEditingController _sourceProviderCtrl = TextEditingController();
  // Alternativa a enviar `ticks` explícitos: SourceDatasetDto no incluye una
  // ruta local de archivo, así que se pide aquí como campo manual (mismo rol
  // que documenta el binding: "alternativa a ticks — ruta a un Parquet ya
  // descargado").
  final TextEditingController _sourceParquetPathCtrl = TextEditingController();

  // ---- Estado de la corrida ----
  bool _procesando = false;
  TransformOutcomeDto? _outcome;
  String? _error;

  @override
  void initState() {
    super.initState();
    // Núm. de núcleos lógicos como techo del modo manual de hilos.
    _hilosManual = Platform.numberOfProcessors.toDouble();
    _cargarDatasets();
  }

  @override
  void dispose() {
    _instrumentIdCtrl.dispose();
    _venueCtrl.dispose();
    _sourceProviderCtrl.dispose();
    _sourceParquetPathCtrl.dispose();
    super.dispose();
  }

  // Lee list_source_datasets() por FFI (o el doble inyectado en pruebas) y
  // puebla el Dropdown "Dataset de origen".
  Future<void> _cargarDatasets() async {
    setState(() => _loadingDatasets = true);
    final loader = widget.loadDatasetsOverride ?? listSourceDatasets;
    final datasets = await loader(dbPath: _kDbPath);
    if (!mounted) return;
    setState(() {
      _datasets = datasets;
      _loadingDatasets = false;
    });
  }

  // Dispara la transformación real por FFI. Patrón G1: await directo, sin
  // sondeo — el botón queda en loading: true mientras el Future no resuelve
  // (mismo patrón que submitDownloadJob en sovereign_data_fetcher_section.dart).
  Future<void> _ejecutar() async {
    if (_selectedDataSnapshotId == null) return;

    setState(() {
      _procesando = true;
      _error = null;
      _outcome = null;
    });

    // timeframe_value: forma de texto legible del intervalo — se deriva de
    // target_aggregation en mayúsculas (formateo de texto puro, ver nota de
    // diseño arriba; no es un control separado del usuario).
    final timeframeValue = _targetAggregation.toUpperCase();
    final sourceParquetPath = _sourceParquetPathCtrl.text.trim();

    final outcome = await submitHybridDataTransformation(
      dbPath: _kDbPath,
      instrumentId: _instrumentIdCtrl.text.trim(),
      venue: _venueCtrl.text.trim(),
      sourceProvider: _sourceProviderCtrl.text.trim(),
      // "TIME" es el único bar_type que esta feature produce agregando
      // TradeTick — invariante de la feature, no una elección del usuario.
      barType: 'TIME',
      timeframeValue: timeframeValue,
      dataSnapshotId: _selectedDataSnapshotId!,
      targetAggregation: _targetAggregation,
      interopThreshold: BigInt.from(_interopThreshold.round()),
      polarsThreads: _hilosAutomatico ? null : _hilosManual.round(),
      ticks: const [],
      sourceParquetPath: sourceParquetPath.isEmpty ? null : sourceParquetPath,
    );

    if (!mounted) return;

    setState(() {
      _procesando = false;
      _outcome = outcome;
      _error = outcome.error;
    });
  }

  // ---------------------------------------------------------------------------
  // Build
  // ---------------------------------------------------------------------------

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        _buildHeader(),
        const SizedBox(height: Gx.space16),
        Expanded(
          child: SingleChildScrollView(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                _buildPlanSection(),
                const SizedBox(height: Gx.space16),
                Container(height: 1, color: Gx.divider),
                const SizedBox(height: Gx.space16),
                _buildResultSection(),
              ],
            ),
          ),
        ),
      ],
    );
  }

  // Header: ícono + nombre de la feature + chip con el estado semántico actual.
  Widget _buildHeader() {
    final (label, status) = _estadoActual();
    return Row(children: [
      Icon(IconsaxPlusLinear.convert, size: 18, color: Gx.textBaseLabel),
      const SizedBox(width: 8),
      Expanded(
        child: Text(
          'Transformador de Datos de Mercado',
          style: Gx.displayGrotesque(
              fontSize: 14, color: Gx.textBase, weight: FontWeight.w500),
          overflow: TextOverflow.ellipsis,
        ),
      ),
      custom_ui.Chip(label: label, status: status, pill: true),
    ]);
  }

  (String, custom_ui.ChipStatus?) _estadoActual() {
    if (_procesando) return ('Procesando', custom_ui.ChipStatus.transition);
    if (_error != null) return ('Falló', custom_ui.ChipStatus.critical);
    if (_outcome != null) return ('Completado', custom_ui.ChipStatus.optima);
    return ('Sin ejecutar', null);
  }

  // ---------------------------------------------------------------------------
  // Sección "Plan de transformación"
  // ---------------------------------------------------------------------------

  Widget _buildPlanSection() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text('Plan de transformación', style: Gx.microLabel),
        const SizedBox(height: Gx.space12),
        // Los 4 controles del plan (más identidad de serie) se deshabilitan
        // por completo mientras hay una corrida en curso — IgnorePointer +
        // Opacity: composición estándar de Flutter, no reimplementa ningún
        // componente de custom_ui (Dropdown/NumberInput/Switch no exponen un
        // parámetro `enabled` propio hoy).
        IgnorePointer(
          ignoring: _procesando,
          child: Opacity(
            opacity: _procesando ? 0.5 : 1,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                _buildDatasetDropdown(),
                const SizedBox(height: Gx.space16),
                _buildAgregacionDropdown(),
                const SizedBox(height: Gx.space16),
                _buildIdentidadSerie(),
                const SizedBox(height: Gx.space16),
                _buildUmbralInteroperabilidad(),
                const SizedBox(height: Gx.space16),
                _buildHilos(),
              ],
            ),
          ),
        ),
        const SizedBox(height: Gx.space16),
        Center(child: _buildBotonEjecutar()),
      ],
    );
  }

  Widget _buildDatasetDropdown() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(children: [
          Text('Dataset de origen',
              style: Gx.uiSans(fontSize: 12, color: Gx.textBaseLabel)),
          const Spacer(),
          GestureDetector(
            onTap: _cargarDatasets,
            child: Icon(Icons.refresh, size: 14, color: Gx.textBaseMuted),
          ),
        ]),
        const SizedBox(height: 4),
        if (_loadingDatasets)
          Text('Cargando datasets...',
              style: Gx.uiSans(fontSize: 12, color: Gx.textBaseMuted))
        else if (_datasets.isEmpty)
          Text('Sin datasets disponibles.',
              style: Gx.uiSans(fontSize: 12, color: Gx.textBaseMuted))
        else
          custom_ui.Dropdown<String>(
            value: _selectedDataSnapshotId,
            hint: 'Selecciona un dataset',
            items: [
              for (final d in _datasets)
                custom_ui.DropdownItem(
                    value: d.dataSnapshotId, label: d.dataSnapshotId),
            ],
            onChanged: (v) => setState(() => _selectedDataSnapshotId = v),
          ),
      ],
    );
  }

  Widget _buildAgregacionDropdown() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text('Agregación destino',
            style: Gx.uiSans(fontSize: 12, color: Gx.textBaseLabel)),
        const SizedBox(height: 4),
        custom_ui.Dropdown<String>(
          value: _targetAggregation,
          items: [
            for (final tf in _kTargetAggregations)
              custom_ui.DropdownItem(value: tf, label: tf),
          ],
          onChanged: (v) => setState(() => _targetAggregation = v),
        ),
      ],
    );
  }

  Widget _buildIdentidadSerie() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text('Identidad de la serie', style: Gx.microLabel),
        const SizedBox(height: Gx.space8),
        custom_ui.FormField(
          label: 'Instrumento',
          child: custom_ui.Input(
              controller: _instrumentIdCtrl, hint: 'Ej. BTCUSDT'),
        ),
        const SizedBox(height: Gx.space12),
        custom_ui.FormField(
          label: 'Venue',
          child: custom_ui.Input(controller: _venueCtrl, hint: 'Ej. BINANCE'),
        ),
        const SizedBox(height: Gx.space12),
        custom_ui.FormField(
          label: 'Proveedor de datos',
          child: custom_ui.Input(
              controller: _sourceProviderCtrl, hint: 'Ej. BINANCE'),
        ),
        const SizedBox(height: Gx.space12),
        custom_ui.FormField(
          label: 'Ruta al Parquet de origen',
          helperText: 'Alternativa a ticks explícitos — Parquet ya descargado.',
          child: custom_ui.Input(
              controller: _sourceParquetPathCtrl,
              hint: '/ruta/al/archivo.parquet'),
        ),
      ],
    );
  }

  Widget _buildUmbralInteroperabilidad() {
    return custom_ui.FormField(
      label: 'Umbral de interoperabilidad',
      helperText: 'Filas máx. antes de rechazar conversión a matriz',
      child: custom_ui.NumberInput(
        value: _interopThreshold,
        min: 0,
        // ponytail: max arbitrario (100M) — el contrato de la feature
        // declara el rango como "-" (sin definir, docs/features/
        // hybrid-data-transformer.md tabla de Parámetros Configurables).
        // Ajustar cuando Rust-Engineer/Quant fije el rango real.
        max: 100000000,
        step: 1000,
        onChanged: (v) => setState(() => _interopThreshold = v),
      ),
    );
  }

  Widget _buildHilos() {
    final coresLogicos = Platform.numberOfProcessors.toDouble();
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text('Hilos', style: Gx.uiSans(fontSize: 12, color: Gx.textBaseLabel)),
        const SizedBox(height: 4),
        Row(children: [
          custom_ui.Switch(
            value: _hilosAutomatico,
            onChanged: (v) => setState(() => _hilosAutomatico = v),
          ),
          const SizedBox(width: Gx.space8),
          Text('Automático',
              style: Gx.uiSans(fontSize: 13, color: Gx.textBaseSecondary)),
          const Spacer(),
          if (!_hilosAutomatico)
            custom_ui.NumberInput(
              value: _hilosManual,
              min: 1,
              max: coresLogicos,
              step: 1,
              onChanged: (v) => setState(() => _hilosManual = v),
            ),
        ]),
      ],
    );
  }

  Widget _buildBotonEjecutar() {
    final habilitado = !_procesando && _selectedDataSnapshotId != null;
    return custom_ui.Button(
      label: _procesando ? 'Ejecutando...' : 'Ejecutar Transformación',
      onPressed: habilitado ? _ejecutar : null,
      variant: custom_ui.ButtonVariant.primary,
      enabled: habilitado,
      loading: _procesando,
    );
  }

  // ---------------------------------------------------------------------------
  // Sección "Resultado"
  // ---------------------------------------------------------------------------

  Widget _buildResultSection() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text('Resultado', style: Gx.microLabel),
        const SizedBox(height: Gx.space12),
        if (_procesando)
          _buildProcesando()
        else if (_error != null)
          _buildError(_error!)
        else if (_outcome != null)
          _buildOutcome(_outcome!)
        else
          custom_ui.Empty(
            message: 'Sin transformación ejecutada.',
            subtitle: 'Configura el plan y pulsa Ejecutar Transformación.',
            icon: IconsaxPlusLinear.convert,
          ),
      ],
    );
  }

  Widget _buildProcesando() {
    return Row(children: [
      ScanRingWidget(
        color: Gx.transitionIndigo,
        maxRadius: 24,
        child: custom_ui.Chip(
          label: 'Transformando...',
          status: custom_ui.ChipStatus.transition,
          pill: true,
        ),
      ),
      const SizedBox(width: Gx.space16),
      Expanded(
        child: Text(
          'La transformación puede tardar sobre datasets grandes.',
          style: Gx.uiSans(fontSize: 12, color: Gx.textBaseSecondary),
        ),
      ),
    ]);
  }

  Widget _buildError(String detalle) {
    return custom_ui.Banner(
      message: 'Transformación falló: $detalle',
      type: custom_ui.BannerType.error,
    );
  }

  Widget _buildOutcome(TransformOutcomeDto outcome) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        custom_ui.Banner(
            message: 'Transformación completada.',
            type: custom_ui.BannerType.success),
        const SizedBox(height: Gx.space12),
        custom_ui.KeyValue(
            label: 'Filas de entrada',
            value: outcome.rowsIn.toString(),
            mono: true),
        custom_ui.KeyValue(
            label: 'Filas de salida',
            value: outcome.rowsOut.toString(),
            mono: true),
        custom_ui.KeyValue(
            label: 'Duración',
            value: _formatDuracion(outcome.transformDurationNs),
            mono: true),
        custom_ui.KeyValue(
            label: 'Barras generadas',
            value: outcome.bars.length.toString(),
            mono: true),
        const SizedBox(height: Gx.space12),
        Text('Esquema resultante', style: Gx.microLabel),
        const SizedBox(height: Gx.space8),
        _buildBarsTable(outcome.bars),
        const SizedBox(height: Gx.space12),
        _buildHashRow('logic_hash', outcome.logicHash),
        _buildHashRow('audit_hash', outcome.auditHash),
      ],
    );
  }

  // Tabla read-only de las barras producidas. "Volumen comprador" usa
  // OptionalMagnitudeText: la garantía central del esquema — un volumen
  // ausente (el bróker no distinguió el lado) jamás se confunde con un
  // volumen cero real.
  Widget _buildBarsTable(List<BarDto> bars) {
    if (bars.isEmpty) {
      return custom_ui.Empty(message: 'La transformación no produjo barras.');
    }
    final visibles = bars.take(_kMaxBarsRenderizadas).toList();
    return custom_ui.Surface(
      padding: const EdgeInsets.all(Gx.space8),
      child: SingleChildScrollView(
        scrollDirection: Axis.horizontal,
        child: custom_ui.Table(
          columns: const [
            custom_ui.TableColumn(label: 'Inicio (ns UTC)', numeric: true),
            custom_ui.TableColumn(label: 'Cierre', numeric: true),
            custom_ui.TableColumn(label: 'Volumen operado', numeric: true),
            custom_ui.TableColumn(label: 'Volumen comprador', numeric: true),
          ],
          rows: [
            for (final b in visibles) _filaBarra(b),
          ],
        ),
      ),
    );
  }

  List<Widget> _filaBarra(BarDto b) {
    final cellStyle = Gx.dataMono(fontSize: 12, color: Gx.textBase);
    return [
      Text(b.barTimestampNs.toString(), style: cellStyle),
      // close/volume_traded son campos obligatorios de BarDto (nunca
      // Option): se formatean directo, sin el tratamiento visual de
      // ausencia que sí necesita volume_buy.
      Text(formatScaledInt(b.close), style: cellStyle),
      Text(formatScaledInt(b.volumeTraded), style: cellStyle),
      OptionalMagnitudeText(scaledValue: b.volumeBuy),
    ];
  }

  Widget _buildHashRow(String label, String hash) {
    final corto = hash.length > 12 ? '${hash.substring(0, 12)}…' : hash;
    return Row(children: [
      Expanded(
          child: custom_ui.KeyValue(label: label, value: corto, mono: true)),
      GestureDetector(
        onTap: () => Clipboard.setData(ClipboardData(text: hash)),
        child: Icon(Icons.copy, size: 14, color: Gx.textBaseMuted),
      ),
    ]);
  }

  // Formatea duración: ns → ms/s legibles. Presentación pura (mismo criterio
  // que la nota 5 de la Cáscara Visual): no es una métrica analítica, solo
  // cambio de unidad de texto para lectura humana.
  String _formatDuracion(int ns) {
    final ms = ns / 1e6;
    if (ms >= 1000) return '${(ms / 1000).toStringAsFixed(2)} s';
    return '${ms.toStringAsFixed(1)} ms';
  }
}
