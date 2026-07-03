// Widget read-only del Dashboard para el Sovereign Data Fetcher.
//
// Muestra el registro más reciente de sovereign_download_records:
//   - ID truncado (8 chars) como dato héroe.
//   - Timestamp de creación en ISO 8601.
//   - Source endpoint truncado a 40 chars.
//
// Gap G2: listDownloadRecords() solo devuelve id, created_at y source_endpoint.
// No existen bytes, symbol ni status en esa tabla, por eso el widget muestra
// los tres campos disponibles en vez de la spec original.
//
// Sin callbacks — widget de solo lectura para el bento grid del Dashboard.

import 'package:flutter/material.dart';
import 'package:iconsax_plus/iconsax_plus.dart';
import '../src/rust/api/data_fetcher.dart';
import '../theme/gx_tokens.dart';
import '../theme/surfaces.dart';

// Ruta a la base de datos SQLite de Drasus — igual que jobs_tab.dart.
const String _kDbPathDashboard = 'drasus.db';

// SovereignDataFetcherDashboardWidget — tarjeta compacta del bento grid.
// Carga el historial en initState() y muestra el registro más reciente.
class SovereignDataFetcherDashboardWidget extends StatefulWidget {
  const SovereignDataFetcherDashboardWidget({super.key});

  @override
  State<SovereignDataFetcherDashboardWidget> createState() =>
      _SovereignDataFetcherDashboardWidgetState();
}

class _SovereignDataFetcherDashboardWidgetState
    extends State<SovereignDataFetcherDashboardWidget> {
  // Último registro (null si la tabla está vacía o la BD no existe).
  DownloadRecordDto? _lastRecord;

  // true mientras el Future de listDownloadRecords no ha resuelto.
  bool _loading = true;

  @override
  // initState: carga el historial al montar el widget.
  void initState() {
    super.initState();
    _cargar();
  }

  // Llama al Bridge para obtener el historial y toma el primer elemento.
  Future<void> _cargar() async {
    // Consulta al Bridge: lista de registros del más reciente al más antiguo.
    final records =
        await listDownloadRecords(dbPath: _kDbPathDashboard);
    // Guarda contra setState tras dispose.
    if (!mounted) return;
    setState(() {
      _lastRecord = records.isNotEmpty ? records.first : null;
      _loading = false;
    });
  }

  // build(): tarjeta compacta con header, dato héroe, key-values.
  // Alto mínimo 110 px. Sin scroll. Sin callbacks.
  @override
  Widget build(BuildContext context) {
    return ConstrainedBox(
      constraints: const BoxConstraints(minHeight: 110),
      child: panelSurface(
        padding: const EdgeInsets.all(Gx.space12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisSize: MainAxisSize.min,
          children: [
            // Header: ícono + label "Datos Soberanos".
            _buildHeader(),
            const SizedBox(height: Gx.space8),

            // Contenido condicional: cargando / sin datos / con datos.
            if (_loading)
              _buildCargando()
            else if (_lastRecord == null)
              _buildSinDatos()
            else
              _buildConDatos(_lastRecord!),
          ],
        ),
      ),
    );
  }

  // Encabezado con ícono y título del widget.
  Widget _buildHeader() {
    return Row(children: [
      Icon(
        IconsaxPlusLinear.document_download,
        size: 14,
        color: Gx.textBaseLabel,
      ),
      const SizedBox(width: 6),
      Text(
        'Datos Soberanos',
        style: Gx.displayGrotesque(
          fontSize: 12,
          color: Gx.textBaseLabel,
          weight: FontWeight.w500,
        ),
      ),
    ]);
  }

  // Indicador de carga mientras el Future resuelve.
  Widget _buildCargando() {
    return Row(children: [
      const SizedBox(
        width: 12,
        height: 12,
        child: CircularProgressIndicator(
          strokeWidth: 1.5,
          color: Gx.transitionIndigo,
        ),
      ),
      const SizedBox(width: 8),
      Text(
        'Cargando...',
        style: Gx.uiSans(fontSize: 11, color: Gx.textBaseMuted),
      ),
    ]);
  }

  // Estado vacío cuando no hay registros en la BD.
  Widget _buildSinDatos() {
    return Text(
      'Sin descargas registradas.',
      style: Gx.uiSans(fontSize: 11, color: Gx.textBaseMuted),
    );
  }

  // Muestra los datos del último registro disponible.
  Widget _buildConDatos(DownloadRecordDto record) {
    // Trunca el UUID a 8 chars como dato héroe (identificador visual compacto).
    final idCorto =
        record.id.length > 8 ? record.id.substring(0, 8) : record.id;

    // Convierte nanosegundos a ISO 8601 UTC.
    final fecha = DateTime.fromMicrosecondsSinceEpoch(
      record.createdAt ~/ 1000,
    ).toUtc().toIso8601String();

    // Trunca el endpoint a 40 chars para evitar desbordamiento.
    final endpoint = record.sourceEndpoint.length > 40
        ? '${record.sourceEndpoint.substring(0, 40)}…'
        : record.sourceEndpoint;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [
        // Dato héroe: ID del último registro con glow optimaCyan.
        Text(
          idCorto,
          style: Gx.dataMono(fontSize: 22, color: Gx.optimaCyan)
              .copyWith(
                  shadows: Gx.textGlow(Gx.optimaCyan)),
        ),
        const SizedBox(height: Gx.space4),
        // Key-value: timestamp de la última descarga.
        _buildKV('Fecha', fecha),
        // Key-value: endpoint fuente truncado.
        _buildKV('Fuente', endpoint),
      ],
    );
  }

  // Fila clave-valor compacta para el widget de Dashboard.
  Widget _buildKV(String key, String value) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 2),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            key,
            style: Gx.uiSans(fontSize: 10, color: Gx.textBaseLabel),
          ),
          const SizedBox(width: 4),
          Flexible(
            child: Text(
              value,
              textAlign: TextAlign.right,
              overflow: TextOverflow.ellipsis,
              style: Gx.dataMono(fontSize: 10, color: Gx.textBaseSecondary),
            ),
          ),
        ],
      ),
    );
  }
}
