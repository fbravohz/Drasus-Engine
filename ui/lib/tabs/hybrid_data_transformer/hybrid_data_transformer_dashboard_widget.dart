// hybrid_data_transformer_dashboard_widget.dart — Widget read-only del
// Dashboard para hybrid-data-transformer (STORY-051 Etapa 4,
// docs/features/hybrid-data-transformer.md "## Cáscara Visual" →
// "Widget de Tablero").
//
// Muestra el registro más reciente de transformation_records:
//   - rows_out de la última corrida como dato héroe.
//   - Dataset (data_snapshot_id truncado a 8 chars).
//   - Duración (transform_duration_ns formateada).
//   - logic_hash truncado a 8 chars.
//
// NOTA DE DISEÑO: la Cáscara Visual original describe un color de fallo
// (criticalCrimson) para "si la última corrida falló". TransformationRecordDto
// (list_transformation_records) no trae ningún campo de éxito/fallo — el
// ledger de transformation_records es append-only y solo se escribe en el
// camino Ok de execute() (ver hybrid_data_transformer.rs, la Cáscara nunca
// persiste un TransformOutcomeDto con error). Por eso todo registro
// presente en la tabla es, por construcción, una corrida completada: el
// dato héroe siempre usa el color de éxito (optimaCyan), igual que
// SovereignDataFetcherDashboardWidget cuando no hay campo de estado en su
// propia tabla.
//
// Sin callbacks — widget de solo lectura para el bento grid del Dashboard
// (regla FIJO ADR-0136: "El Dashboard es read-only").

import 'package:flutter/material.dart';
import 'package:iconsax_plus/iconsax_plus.dart';

import '../../src/rust/api/hybrid_data_transformer.dart';
import '../../theme/gx_tokens.dart';
import '../../theme/surfaces.dart';

// Ruta a la base de datos SQLite de Drasus — igual que jobs_tab.dart.
const String _kDbPathDashboard = 'drasus.db';

// HybridDataTransformerDashboardWidget — tarjeta compacta del bento grid.
// Carga el historial en initState() y muestra el registro más reciente.
class HybridDataTransformerDashboardWidget extends StatefulWidget {
  const HybridDataTransformerDashboardWidget({super.key, this.loadRecordsOverride});

  // Costura de prueba (mismo patrón que `verifyOverride` en
  // GenericVerificationSection, ver generic_verification_section.dart): en
  // producción es siempre `null` y el widget llama a `listTransformationRecords`
  // real por FFI. Un widget test la sustituye por un doble que lee de un
  // "almacén" externo al árbol de widgets — así el test de persistencia
  // (criterio 10 de STORY-051) puede mutar ese almacén entre un desmontaje y
  // un remontaje sin necesitar la librería nativa compilada.
  final Future<List<TransformationRecordDto>> Function({required String dbPath})?
      loadRecordsOverride;

  @override
  State<HybridDataTransformerDashboardWidget> createState() =>
      _HybridDataTransformerDashboardWidgetState();
}

class _HybridDataTransformerDashboardWidgetState
    extends State<HybridDataTransformerDashboardWidget> {
  // Último registro (null si la tabla está vacía o la BD no existe).
  TransformationRecordDto? _lastRecord;

  // true mientras el Future de listTransformationRecords no ha resuelto.
  bool _loading = true;

  @override
  // initState: carga el historial al montar el widget.
  void initState() {
    super.initState();
    _cargar();
  }

  // Llama al Bridge para obtener el historial y toma el registro más
  // reciente (listTransformationRecords ya devuelve del más nuevo al más
  // antiguo — ver doc-comment del binding). Siempre invoca la fuente de
  // datos de nuevo en cada montaje (nunca guarda el resultado fuera de este
  // State, que muere al desmontar) — es la garantía que el criterio 10
  // exige probada, no solo estructural.
  Future<void> _cargar() async {
    final loader = widget.loadRecordsOverride ?? listTransformationRecords;
    final records = await loader(dbPath: _kDbPathDashboard);
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
            _buildHeader(),
            const SizedBox(height: Gx.space8),
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
      Icon(IconsaxPlusLinear.convert, size: 14, color: Gx.textBaseLabel),
      const SizedBox(width: 6),
      Text(
        'Transformación de Datos',
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
      'Sin transformaciones registradas.',
      style: Gx.uiSans(fontSize: 11, color: Gx.textBaseMuted),
    );
  }

  // Muestra los datos del último registro disponible.
  Widget _buildConDatos(TransformationRecordDto record) {
    // Trunca dataset y hash a 8 chars como dato compacto (mismo patrón que
    // SovereignDataFetcherDashboardWidget con el UUID de descarga).
    final datasetCorto = _truncar(record.dataSnapshotId ?? 'sin linaje', 8);
    final hashCorto = _truncar(record.logicHash, 8);

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [
        // Dato héroe: filas de salida de la última corrida, con glow óptimo.
        Text(
          record.rowsOut.toString(),
          style: Gx.dataMono(fontSize: 22, color: Gx.optimaCyan)
              .copyWith(shadows: Gx.textGlow(Gx.optimaCyan)),
        ),
        const SizedBox(height: Gx.space4),
        _buildKV('Dataset', datasetCorto),
        _buildKV('Duración', _formatDuracion(record.transformDurationNs)),
        _buildKV('logic_hash', hashCorto),
      ],
    );
  }

  // Trunca una cadena a [n] caracteres, con elipsis si excede.
  String _truncar(String s, int n) => s.length > n ? s.substring(0, n) : s;

  // Formatea duración: ns → ms/s legibles. Presentación pura, sin cálculo
  // de negocio (mismo criterio que el panel inspector de esta feature).
  String _formatDuracion(int ns) {
    final ms = ns / 1e6;
    if (ms >= 1000) return '${(ms / 1000).toStringAsFixed(2)} s';
    return '${ms.toStringAsFixed(1)} ms';
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
