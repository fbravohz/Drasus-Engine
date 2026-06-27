// Pestaña de bitácora de auditoría del Panel Operativo.
// Lista los últimos 50 eventos de auditoría con su tipo, entidad, fecha
// y los últimos 8 caracteres del hash de cadena para verificación visual.

import 'package:flutter/material.dart';
import '../src/rust/api/audit.dart';
import '../gallery/gallery_tokens.dart';

// Ruta a la base de datos SQLite de Drasus. La misma constante que jobs_tab.dart.
// Si en el futuro se configura dinámicamente, se pasa como parámetro al widget.
const String _kDbPath = 'drasus.db';

// Número de eventos a mostrar. Rust u64 → Dart BigInt (no es const-constructible).
final BigInt _kLimitAudit = BigInt.from(50);

// AuditTab es StatefulWidget porque necesita relanzar la consulta al Bridge
// al refrescar y tiene estado propio (_futureEventos).
class AuditTab extends StatefulWidget {
  const AuditTab({super.key});

  @override
  State<AuditTab> createState() => _AuditTabState();
}

// _AuditTabState aloja el Future de la consulta de eventos de auditoría.
class _AuditTabState extends State<AuditTab> {
  // Future con la lista de eventos más recientes de la bitácora.
  late Future<List<AuditEventSummary>> _futureEventos;

  // initState() lanza la primera consulta al montar el widget.
  @override
  void initState() {
    super.initState();
    _futureEventos = _consultarEventos();
  }

  // Llama al Bridge para obtener los últimos _kLimitAudit eventos de auditoría.
  // getRecentAuditEvents acepta dbPath y limit como parámetros nombrados.
  Future<List<AuditEventSummary>> _consultarEventos() {
    // Consulta al Bridge: retorna los últimos 50 eventos de la cadena de
    // auditoría, ordenados del más reciente al más antiguo.
    return getRecentAuditEvents(dbPath: _kDbPath, limit: _kLimitAudit);
  }

  // Reemplaza _futureEventos para forzar un refresco de la lista.
  void _refrescar() {
    setState(() {
      _futureEventos = _consultarEventos();
    });
  }

  // Extrae los últimos 8 caracteres del hash de cadena para verificación visual.
  // El hash completo es SHA-256 (64 chars en hex). Mostrar los últimos 8 permite
  // al usuario comprobar de un vistazo si la cadena está íntegra sin necesitar
  // el hash completo.
  // Si el hash está vacío (evento génesis sin predecesor), devuelve "——genesis".
  String _hashCorto(String hash) {
    if (hash.isEmpty) return '——genesis';
    return hash.length > 8 ? hash.substring(hash.length - 8) : hash;
  }

  // build() muestra la lista de eventos de auditoría con FutureBuilder.
  // Estructura idéntica a jobs_tab.dart: waiting → error → datos.
  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        // Barra superior con título y botón de refresco.
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
          child: Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text(
                'Últimos 50 eventos de auditoría',
                style: Theme.of(context).textTheme.titleSmall?.copyWith(color: Gx.textBaseMuted),
              ),
              IconButton(
                icon: const Icon(Icons.refresh),
                tooltip: 'Refrescar bitácora de auditoría',
                onPressed: _refrescar,
              ),
            ],
          ),
        ),
        // FutureBuilder escucha _futureEventos y reconstruye la lista
        // cada vez que el Future cambia de estado o es reemplazado.
        Expanded(
          child: FutureBuilder<List<AuditEventSummary>>(
            future: _futureEventos,
            builder: (context, snapshot) {
              // El Future aún no resolvió — mostramos spinner de carga.
              if (snapshot.connectionState == ConnectionState.waiting) {
                return const Center(child: CircularProgressIndicator());
              }
              // El Future terminó con error — mostramos el mensaje al usuario.
              if (snapshot.hasError) {
                return Center(
                  child: Text(
                    'Error al consultar bitácora:\n${snapshot.error}',
                    style: Theme.of(context).textTheme.bodySmall?.copyWith(color: Gx.criticalRed),
                    textAlign: TextAlign.center,
                  ),
                );
              }
              // Datos listos — snapshot.data es la List<AuditEventSummary>.
              final eventos = snapshot.data ?? [];
              if (eventos.isEmpty) {
                return Center(
                  child: Text(
                    'Sin eventos de auditoría registrados.',
                    style: Theme.of(context).textTheme.bodySmall?.copyWith(color: Gx.textBaseMuted),
                  ),
                );
              }
              // ListView.builder: renderizado diferido, construye solo los
              // ítems visibles en la ventana de scroll.
              return ListView.builder(
                itemCount: eventos.length,
                itemBuilder: (context, index) {
                  final evento = eventos[index];
                  // Convierte nanosegundos a fecha UTC legible.
                  final fecha = DateTime.fromMicrosecondsSinceEpoch(
                    evento.createdAt ~/ 1000,
                  ).toUtc().toIso8601String();
                  // Últimos 8 chars del hash para verificación visual de la cadena.
                  final hashVis = _hashCorto(evento.auditChainHash);
                  return ListTile(
                    dense: true,
                    // Tipo de acción (ORDER_STATE_CHANGE, USER_VETO, etc.)
                    title: Text(
                      evento.actionType,
                      style: Theme.of(context).textTheme.labelMedium,
                    ),
                    // Tipo de entidad afectada por la acción.
                    subtitle: Text(
                      '${evento.entityType}  ·  $fecha',
                      style: Theme.of(context).textTheme.labelSmall?.copyWith(color: Gx.textBaseMuted),
                    ),
                    // Últimos 8 chars del hash de cadena en verde — convención
                    // de terminal para datos de integridad criptográfica.
                    trailing: Text(
                      hashVis,
                      style: Theme.of(context).textTheme.labelSmall?.copyWith(color: Gx.reactorGreen),
                    ),
                  );
                },
              );
            },
          ),
        ),
      ],
    );
  }
}
