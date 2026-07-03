// Sección SVF del Sovereign Data Fetcher en el Banco de Verificación.
//
// Verifica el ciclo completo: el usuario configura broker/símbolo/fechas/
// timeframe/tipo, dispara submitDownloadJob() por FFI, y ve el resultado en
// la Zona B. La Zona C muestra el historial persistido en sovereign_download_records.
//
// Presentación (STORY-025): consume la librería de componentes `custom_ui.*`
// (Input, Dropdown, Segmented, Button, Banner, Chip, KeyValue, Empty, Table,
// Tooltip, Surface) en lugar de reimplementar cada control inline. La lógica
// FFI/estado es idéntica; solo cambia la capa de presentación. El único
// primitivo local es ScanRingWidget (animación de monitoreo, en gallery_fx).
//
// Patrón de job (Gap G1): NO usa Timer.periodic. submitDownloadJob() es await
// y resuelve con el resultado completo. Mientras espera: botón en `loading:` +
// ScanRingWidget activo. Al resolver: puebla Zona B y refresca Zona C.
//
// Columnas del historial (Gap G2): solo id, created_at y source_endpoint.
// No existen symbol/bytes/status en sovereign_download_records.
//
// Estados semánticos (Gap G3): COMPLETED/RUNNING/QUEUED/FAILED/CANCELLED.
// No hay estado "Retrying" en la API EPIC-1.

import 'package:flutter/material.dart';
import 'package:iconsax_plus/iconsax_plus.dart';

import '../../components/components.dart' as custom_ui;
// gallery_fx se importa SOLO por ScanRingWidget (primitivo de animación).
import '../../gallery/gallery_fx.dart' show ScanRingWidget;
import '../../gallery/gallery_tokens.dart';
import '../../src/rust/api/data_fetcher.dart';

// Ruta a la base de datos SQLite de Drasus — igual que jobs_tab.dart.
const String _kDbPath = 'drasus.db';

// Directorio donde Rust guardará los archivos descargados.
// Rust lo crea con create_dir_all si no existe.
const String _kDataDir = 'drasus_data';

// URL base del servidor de volcados Binance Vision.
const String _kBinanceVisionUrl = 'https://data.binance.vision';

// Brokers disponibles para el selector de broker.
const List<String> _kBrokers = ['Binance Vision'];

// Timeframes disponibles para el selector.
const List<String> _kTimeframes = ['1m', '5m', '15m', '1h', '4h', '1d', '1w'];

// ---------------------------------------------------------------------------
// SovereignDataFetcherSection — widget principal de la SVF.
// ---------------------------------------------------------------------------

// StatefulWidget porque gestiona el ciclo de vida del job async (espera,
// resultado, historial), el estado de los controles y la validación de fechas.
class SovereignDataFetcherSection extends StatefulWidget {
  const SovereignDataFetcherSection({super.key});

  @override
  State<SovereignDataFetcherSection> createState() =>
      _SovereignDataFetcherSectionState();
}

class _SovereignDataFetcherSectionState
    extends State<SovereignDataFetcherSection> {
  // ---- Controles de Zona A ----

  // Broker seleccionado — en EPIC-1 solo hay uno.
  String _broker = _kBrokers.first;

  // Símbolo de mercado que el usuario escribe (ej. BTCUSDT).
  final TextEditingController _symbolCtrl =
      TextEditingController(text: 'BTCUSDT');

  // Rango de fechas: desde y hasta para la descarga.
  DateTime _fechaDesde = DateTime.now().subtract(const Duration(days: 7));
  DateTime _fechaHasta = DateTime.now();

  // true cuando _fechaDesde >= _fechaHasta; deshabilita "Descargar".
  bool _fechaError = false;

  // Timeframe seleccionado (ej. "1m").
  String _timeframe = '1m';

  // 0 = Trades (ticks), 1 = Klines (bars).
  int _outputTypeIndex = 0;

  // ---- Estado del job ----

  // true mientras el Future de submitDownloadJob() no ha resuelto.
  bool _isRunning = false;

  // Resultado del último job ejecutado. null = ningún job ejecutado aún.
  DownloadJobResult? _lastResult;

  // Estado del job leído con getJobStatus() tras resolver el Future.
  // null si no se ha ejecutado ningún job o si getJobStatus() retornó null.
  JobStatusDto? _lastStatus;

  // Error de FFI o de validación para mostrar en el banner de Zona B.
  String? _error;

  // ---- Historial (Zona C) ----

  // Lista de registros de sovereign_download_records — los más recientes primero.
  List<DownloadRecordDto> _records = [];

  @override
  // initState: carga el historial desde la BD al montar el widget.
  void initState() {
    super.initState();
    _cargarHistorial();
  }

  @override
  // dispose: libera el controlador de texto para evitar fugas de memoria.
  void dispose() {
    _symbolCtrl.dispose();
    super.dispose();
  }

  // Lee sovereign_download_records por FFI y actualiza la Zona C.
  // Devuelve lista vacía si la BD aún no existe o la tabla está vacía.
  Future<void> _cargarHistorial() async {
    // Consulta al Bridge: lista de registros del más reciente al más antiguo.
    final records = await listDownloadRecords(dbPath: _kDbPath);
    // Guarda contra setState tras dispose (patrón defensivo).
    if (!mounted) return;
    setState(() {
      _records = records;
    });
  }

  // Valida las fechas y actualiza _fechaError.
  // Regla: fechaDesde debe ser estrictamente anterior a fechaHasta.
  void _validarFechas() {
    setState(() {
      _fechaError = !_fechaDesde.isBefore(_fechaHasta);
    });
  }

  // Abre el DatePicker nativo de Flutter para seleccionar la fecha Desde.
  Future<void> _seleccionarDesde() async {
    final picked = await showDatePicker(
      context: context,
      // Fecha inicial = la actualmente seleccionada.
      initialDate: _fechaDesde,
      firstDate: DateTime(2017),
      lastDate: _fechaHasta,
    );
    if (picked != null && mounted) {
      setState(() {
        _fechaDesde = picked;
        _validarFechas();
      });
    }
  }

  // Abre el DatePicker nativo de Flutter para seleccionar la fecha Hasta.
  Future<void> _seleccionarHasta() async {
    final picked = await showDatePicker(
      context: context,
      initialDate: _fechaHasta,
      // La fecha mínima de "Hasta" es el día siguiente a "Desde".
      firstDate: _fechaDesde.add(const Duration(days: 1)),
      lastDate: DateTime(2030),
    );
    if (picked != null && mounted) {
      setState(() {
        _fechaHasta = picked;
        _validarFechas();
      });
    }
  }

  // Dispara la descarga soberana por FFI.
  // Patrón G1: await directo — no hay polling. La UI bloquea el botón y
  // muestra ScanRingWidget mientras el Future no resuelve.
  Future<void> _descargar() async {
    // Validación mínima: símbolo no vacío.
    final symbol = _symbolCtrl.text.trim().toUpperCase();
    if (symbol.isEmpty) {
      setState(() => _error = 'El símbolo no puede estar vacío.');
      return;
    }
    if (_fechaError) {
      setState(() => _error = 'La fecha Desde debe ser anterior a Hasta.');
      return;
    }

    // Convierte fechas a nanosegundos desde epoch: µs × 1000 = ns.
    // PlatformInt64 = int en Linux nativo (plataforma IO de flutter_rust_bridge).
    final startNs = _fechaDesde.toUtc().microsecondsSinceEpoch * 1000;
    final endNs = _fechaHasta.toUtc().microsecondsSinceEpoch * 1000;

    // outputType: "ticks" cuando _outputTypeIndex == 0, "bars" cuando es 1.
    final outputType = _outputTypeIndex == 0 ? 'ticks' : 'bars';

    setState(() {
      _isRunning = true;
      _error = null;
      _lastResult = null;
      _lastStatus = null;
    });

    // Llamada al Bridge: bloquea el botón hasta que el job termina o falla.
    // No hay actualizaciones intermedias en EPIC-1 (sin Canal de Progreso).
    final result = await submitDownloadJob(
      dbPath: _kDbPath,
      dataDir: _kDataDir,
      symbol: symbol,
      brokerUrl: _kBinanceVisionUrl,
      startNs: startNs,
      endNs: endNs,
      timeframe: _timeframe,
      outputType: outputType,
    );

    // Guarda contra setState tras dispose.
    if (!mounted) return;

    // Si hay error FFI, lo mostramos en el banner de Zona B.
    if (result.error != null) {
      setState(() {
        _isRunning = false;
        _lastResult = result;
        _error = result.error;
      });
      return;
    }

    // Job exitoso: consulta el estado final UNA vez (patrón G1).
    JobStatusDto? status;
    if (result.jobId.isNotEmpty) {
      status = await getJobStatus(dbPath: _kDbPath, jobId: result.jobId);
    }

    if (!mounted) return;
    setState(() {
      _isRunning = false;
      _lastResult = result;
      _lastStatus = status;
    });

    // Refresca el historial para mostrar el nuevo registro en Zona C.
    await _cargarHistorial();
  }

  // ---------------------------------------------------------------------------
  // Build
  // ---------------------------------------------------------------------------

  // build(): ensambla las tres zonas en columna vertical con gaps de space8.
  // La Zona C toma el espacio restante con Expanded.
  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        // Encabezado de sección.
        _buildHeader(),
        const SizedBox(height: Gx.space16),

        // Zona A — controles de entrada.
        _buildZonaA(),
        const SizedBox(height: Gx.space8),

        // Zona B — resultado del job activo o banner de error.
        _buildZonaB(),
        const SizedBox(height: Gx.space8),

        // Zona C — historial persistido; toma el espacio sobrante.
        Expanded(child: _buildZonaC()),
      ],
    );
  }

  // Encabezado con ícono y título de la sección.
  Widget _buildHeader() {
    return Row(children: [
      Icon(IconsaxPlusLinear.document_download,
          size: 16, color: Gx.textBaseLabel),
      const SizedBox(width: 8),
      Text(
        'Datos Soberanos — Verificación FFI',
        style: Gx.uiSans(
            fontSize: 14,
            color: Gx.textBase,
            weight: FontWeight.w500),
      ),
    ]);
  }

  // ---------------------------------------------------------------------------
  // Zona A — Panel de Control (controles de entrada)
  // ---------------------------------------------------------------------------

  // Construye el panel con todos los controles: broker, símbolo, fechas,
  // timeframe, tipo de salida y botón de descarga.
  Widget _buildZonaA() {
    return custom_ui.Surface(
      padding: const EdgeInsets.all(Gx.space16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Etiqueta de zona.
          Text('Panel de Control', style: Gx.microLabel),
          const SizedBox(height: Gx.space12),

          // Fila 1: Broker + Símbolo.
          Row(children: [
            Expanded(child: _buildDropdownBroker()),
            const SizedBox(width: Gx.space12),
            Expanded(child: _buildInputSimbolo()),
          ]),
          const SizedBox(height: Gx.space12),

          // Fila 2: Desde + Hasta.
          Row(children: [
            Expanded(child: _buildDateButton('Desde', _fechaDesde, _seleccionarDesde, _fechaError)),
            const SizedBox(width: Gx.space12),
            Expanded(child: _buildDateButton('Hasta', _fechaHasta, _seleccionarHasta, false)),
          ]),
          const SizedBox(height: Gx.space12),

          // Fila 3: Timeframe + Segmented (Trades/Klines).
          Row(children: [
            Expanded(child: _buildDropdownTimeframe()),
            const SizedBox(width: Gx.space12),
            Expanded(child: _buildSegmentedOutputType()),
          ]),
          const SizedBox(height: Gx.space16),

          // Botón "Descargar" centrado.
          Center(child: _buildBotonDescargar()),
        ],
      ),
    );
  }

  // Selector de broker (custom_ui.Dropdown). Solo hay un broker (Binance Vision).
  Widget _buildDropdownBroker() {
    return _buildDropdownLocal(
      label: 'Broker',
      value: _broker,
      options: _kBrokers,
      onChanged: (v) => setState(() => _broker = v),
    );
  }

  // Campo de símbolo (custom_ui.Input). La conversión a uppercase se aplica al
  // enviar (_descargar), no al escribir.
  Widget _buildInputSimbolo() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text('Símbolo', style: Gx.microLabel),
        const SizedBox(height: 4),
        custom_ui.Input(
          controller: _symbolCtrl,
          hint: 'Ej. BTCUSDT',
        ),
      ],
    );
  }

  // Botón selector de fecha que abre el DatePicker nativo de Flutter.
  // El diálogo nativo cubre el rango multi-año (2017–2030) que necesita el
  // fetcher; custom_ui.DateRangePicker es de un solo mes y no aplica aquí.
  // [isError] pinta el borde con glow crimson si las fechas son inválidas.
  Widget _buildDateButton(
    String label,
    DateTime fecha,
    VoidCallback onTap,
    bool isError,
  ) {
    final fechaStr = '${fecha.year.toString().padLeft(4, '0')}-'
        '${fecha.month.toString().padLeft(2, '0')}-'
        '${fecha.day.toString().padLeft(2, '0')}';

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(label, style: Gx.microLabel),
        const SizedBox(height: 4),
        GestureDetector(
          onTap: onTap,
          child: custom_ui.Surface(
            padding: const EdgeInsets.symmetric(
                horizontal: Gx.space12, vertical: 10),
            glow: isError
                ? Gx.glow(Gx.criticalCrimson, blur: 12, opacity: 0.4)
                : null,
            child: Row(children: [
              Expanded(
                child: Text(
                  fechaStr,
                  style: Gx.uiSans(
                    fontSize: 13,
                    color:
                        isError ? Gx.criticalCrimson : Gx.textBase,
                  ),
                ),
              ),
              Icon(
                Icons.calendar_today_outlined,
                size: 14,
                color:
                    isError ? Gx.criticalCrimson : Gx.textBaseSecondary,
              ),
            ]),
          ),
        ),
        // Mensaje de error inline cuando las fechas son inválidas.
        if (isError)
          Padding(
            padding: const EdgeInsets.only(top: 4),
            child: Text(
              'Desde debe ser anterior a Hasta',
              style:
                  Gx.uiSans(fontSize: 11, color: Gx.criticalCrimson),
            ),
          ),
      ],
    );
  }

  // Selector de timeframe (intervalo temporal: 1m, 5m, 15m, 1h, 4h, 1d, 1w).
  Widget _buildDropdownTimeframe() {
    return _buildDropdownLocal(
      label: 'Timeframe',
      value: _timeframe,
      options: _kTimeframes,
      onChanged: (v) => setState(() => _timeframe = v),
    );
  }

  // Conmutador Trades/Klines (custom_ui.Segmented) — 0 → ticks, 1 → bars.
  Widget _buildSegmentedOutputType() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text('Tipo de salida', style: Gx.microLabel),
        const SizedBox(height: 4),
        custom_ui.Segmented(
          options: const ['Trades (Tick)', 'Klines (Bars)'],
          selected: _outputTypeIndex,
          onChanged: (i) => setState(() => _outputTypeIndex = i),
        ),
      ],
    );
  }

  // Botón "Descargar" (custom_ui.Button). El componente gestiona el estado
  // deshabilitado y el spinner de `loading:` — sin gradiente ni spinner inline.
  Widget _buildBotonDescargar() {
    final enabled = !_isRunning && !_fechaError;

    return custom_ui.Button(
      label: _isRunning ? 'Descargando...' : 'Descargar',
      onPressed: enabled ? _descargar : null,
      variant: custom_ui.ButtonVariant.primary,
      enabled: enabled,
      loading: _isRunning,
    );
  }

  // ---------------------------------------------------------------------------
  // Zona B — Panel de Resultados
  // ---------------------------------------------------------------------------

  // Muestra el resultado del job activo o el último completado.
  // Si _isRunning: ScanRingWidget + chip "En progreso".
  // Si hay error: custom_ui.Banner.
  // Si hay resultado exitoso: 4 key-value + chip de estado final.
  // Si no hay ningún job ejecutado: mensaje informativo.
  Widget _buildZonaB() {
    return custom_ui.Surface(
      padding: const EdgeInsets.all(Gx.space12),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text('Último Job', style: Gx.microLabel),
          const SizedBox(height: Gx.space8),

          // Caso: error de FFI o validación.
          if (_error != null)
            _buildBannerError(_error!)
          // Caso: job en curso — ScanRingWidget + indicador.
          else if (_isRunning)
            _buildJobEnCurso()
          // Caso: resultado disponible del último job.
          else if (_lastResult != null)
            _buildJobResultado(_lastResult!, _lastStatus)
          // Caso inicial: ningún job ejecutado aún.
          else
            Text(
              'Sin jobs ejecutados. Configura los parámetros y pulsa Descargar.',
              style: Gx.uiSans(fontSize: 12, color: Gx.textBaseMuted),
            ),
        ],
      ),
    );
  }

  // Banner de error (custom_ui.Banner) + acción para descartarlo. El error
  // también se limpia solo al reintentar la descarga.
  Widget _buildBannerError(String mensaje) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        custom_ui.Banner(
          message: mensaje,
          type: custom_ui.BannerType.error,
        ),
        Align(
          alignment: Alignment.centerRight,
          child: Padding(
            padding: const EdgeInsets.only(top: 4),
            child: GestureDetector(
              onTap: () => setState(() => _error = null),
              child: Text(
                'Descartar',
                style: Gx.uiSans(fontSize: 11, color: Gx.textBaseMuted),
              ),
            ),
          ),
        ),
      ],
    );
  }

  // Muestra el ScanRingWidget y un chip "En progreso" mientras el job corre.
  Widget _buildJobEnCurso() {
    return Row(children: [
      ScanRingWidget(
        // El ScanRingWidget envuelve el chip que indica el estado running.
        color: Gx.transitionIndigo,
        maxRadius: 24,
        period: const Duration(milliseconds: 2800),
        child: custom_ui.Chip(
          label: 'En progreso',
          status: custom_ui.ChipStatus.transition,
          pill: true,
        ),
      ),
      const SizedBox(width: Gx.space16),
      Expanded(
        child: Text(
          'Descargando datos... El proceso puede tardar varios segundos.',
          style: Gx.uiSans(fontSize: 12, color: Gx.textBaseSecondary),
        ),
      ),
    ]);
  }

  // Muestra los 4 key-value del resultado del job + chip de estado final.
  Widget _buildJobResultado(
      DownloadJobResult result, JobStatusDto? status) {
    // Estado del job: viene de getJobStatus() o se infiere del result.error.
    final estadoStr = status?.state ??
        (result.error != null ? 'FAILED' : 'COMPLETED');
    final (estadoLabel, estadoStatus, pill) = _mapearEstado(estadoStr);
    final estadoColor = _statusColor(estadoStatus);

    // Formatea BigInt de bytes para mostrar al usuario.
    final totalBytesStr = _formatBytes(result.totalBytes);
    final bulkFilesStr = result.bulkFilesDownloaded.toString();
    final jobIdCorto = result.jobId.length > 8
        ? result.jobId.substring(0, 8)
        : result.jobId;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // Grid 2×2 de key-value con los datos del job.
        GridView.count(
          crossAxisCount: 2,
          childAspectRatio: 4.0,
          // shrinkWrap: el Grid no toma espacio infinito dentro de la Column.
          shrinkWrap: true,
          physics: const NeverScrollableScrollPhysics(),
          children: [
            custom_ui.KeyValue(
                label: 'Job ID', value: jobIdCorto, mono: true),
            custom_ui.KeyValue(
                label: 'Estado',
                value: estadoLabel,
                valueColor: estadoColor,
                mono: true),
            custom_ui.KeyValue(
                label: 'Archivos Bulk', value: bulkFilesStr, mono: true),
            custom_ui.KeyValue(
                label: 'Total bytes', value: totalBytesStr, mono: true),
          ],
        ),
        const SizedBox(height: Gx.space8),
        // Chip de estado al pie del panel.
        custom_ui.Chip(label: estadoLabel, status: estadoStatus, pill: pill),
      ],
    );
  }

  // ---------------------------------------------------------------------------
  // Zona C — Historial de Descargas
  // ---------------------------------------------------------------------------

  // Muestra la tabla de sovereign_download_records con 3 columnas reales:
  // id (truncado 8 chars), created_at (ISO 8601) y source_endpoint (truncado 40).
  Widget _buildZonaC() {
    return custom_ui.Surface(
      padding: const EdgeInsets.all(Gx.space12),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Cabecera de zona con botón de refresco.
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text('Historial de Descargas', style: Gx.microLabel),
              GestureDetector(
                onTap: _cargarHistorial,
                child: Icon(Icons.refresh,
                    size: 16, color: Gx.textBaseMuted),
              ),
            ],
          ),
          const SizedBox(height: Gx.space8),

          // Estado vacío — cuando no hay registros en la BD.
          if (_records.isEmpty)
            _buildEstadoVacio()
          else
            // Tabla con los 3 campos disponibles del historial.
            Expanded(
              child: _buildTablaHistorial(),
            ),
        ],
      ),
    );
  }

  // Estado vacío de la Zona C (custom_ui.Empty).
  Widget _buildEstadoVacio() {
    return custom_ui.Empty(
      message: 'Sin descargas aún.',
      icon: IconsaxPlusLinear.document_download,
    );
  }

  // Tabla de registros (custom_ui.Table) con scroll vertical.
  // Columnas (Gap G2): solo id, created_at, source_endpoint.
  Widget _buildTablaHistorial() {
    return SingleChildScrollView(
      child: custom_ui.Table(
        columns: const [
          custom_ui.TableColumn(label: 'ID'),
          custom_ui.TableColumn(label: 'Fecha (UTC)'),
          custom_ui.TableColumn(label: 'Endpoint'),
        ],
        rows: [
          for (final r in _records) _filaHistorial(r),
        ],
      ),
    );
  }

  // Construye una fila del historial: id | created_at | source_endpoint.
  // El endpoint se trunca a 40 chars y muestra la URL completa en Tooltip.
  List<Widget> _filaHistorial(DownloadRecordDto r) {
    // Trunca el UUID a 8 chars para ahorrar espacio visual.
    final idCorto = r.id.length > 8 ? r.id.substring(0, 8) : r.id;
    // Convierte ns → microsegundos → DateTime UTC → ISO 8601.
    final fecha = DateTime.fromMicrosecondsSinceEpoch(
      r.createdAt ~/ 1000,
    ).toUtc().toIso8601String();
    // Trunca el endpoint a 40 chars para la celda.
    final endpointTruncado = r.sourceEndpoint.length > 40
        ? '${r.sourceEndpoint.substring(0, 40)}…'
        : r.sourceEndpoint;

    final cellStyle = Gx.dataMono(fontSize: 12, color: Gx.textBase);

    return [
      Text(idCorto, style: cellStyle, overflow: TextOverflow.ellipsis),
      Text(fecha, style: cellStyle, overflow: TextOverflow.ellipsis),
      // Tooltip con la URL completa al hacer hover sobre el texto truncado.
      custom_ui.Tooltip(
        message: r.sourceEndpoint,
        child: Text(endpointTruncado,
            style: cellStyle, overflow: TextOverflow.ellipsis),
      ),
    ];
  }

  // ---------------------------------------------------------------------------
  // Helpers de presentación
  // ---------------------------------------------------------------------------

  // Dropdown genérico (custom_ui.Dropdown) con label, valor y opciones.
  Widget _buildDropdownLocal({
    required String label,
    required String value,
    required List<String> options,
    required ValueChanged<String> onChanged,
  }) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(label, style: Gx.microLabel),
        const SizedBox(height: 4),
        custom_ui.Dropdown<String>(
          value: value,
          items: [
            for (final o in options)
              custom_ui.DropdownItem(value: o, label: o),
          ],
          onChanged: onChanged,
        ),
      ],
    );
  }

  // Mapea el estado string del job a su etiqueta + ChipStatus semántico + pill.
  // Gap G3: no existe "Retrying" en EPIC-1; "CANCELLED" → Fallido.
  // ChipStatus null = chip neutro (estado desconocido).
  (String, custom_ui.ChipStatus?, bool) _mapearEstado(String state) {
    switch (state) {
      case 'COMPLETED':
        return ('Completado', custom_ui.ChipStatus.optima, false);
      case 'RUNNING':
        return ('En progreso', custom_ui.ChipStatus.transition, true);
      case 'QUEUED':
        return ('En cola', custom_ui.ChipStatus.transition, false);
      case 'FAILED':
      case 'CANCELLED':
        return ('Fallido', custom_ui.ChipStatus.critical, false);
      default:
        return (state, null, false);
    }
  }

  // Color semántico asociado a un ChipStatus (para el valor del KeyValue).
  Color _statusColor(custom_ui.ChipStatus? status) {
    switch (status) {
      case custom_ui.ChipStatus.optima:
        return Gx.optimaCyan;
      case custom_ui.ChipStatus.transition:
        return Gx.transitionIndigo;
      case custom_ui.ChipStatus.alert:
        return Gx.alertAmber;
      case custom_ui.ChipStatus.critical:
        return Gx.criticalCrimson;
      case null:
        return Gx.textBaseMuted;
    }
  }

  // Formatea un BigInt de bytes a una cadena legible (KB/MB/GB).
  // Solo presentación — la lógica de negocio de cuánto se descargó vive en Rust.
  String _formatBytes(BigInt bytes) {
    if (bytes == BigInt.zero) return '0 B';
    final gb = BigInt.from(1024 * 1024 * 1024);
    final mb = BigInt.from(1024 * 1024);
    final kb = BigInt.from(1024);
    if (bytes >= gb) {
      final v = (bytes * BigInt.from(100) ~/ gb).toInt() / 100;
      return '$v GB';
    } else if (bytes >= mb) {
      final v = (bytes * BigInt.from(100) ~/ mb).toInt() / 100;
      return '$v MB';
    } else if (bytes >= kb) {
      final v = (bytes * BigInt.from(100) ~/ kb).toInt() / 100;
      return '$v KB';
    }
    return '$bytes B';
  }
}
