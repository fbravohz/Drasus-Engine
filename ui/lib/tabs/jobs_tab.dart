// Pestaña de trabajos del Panel Operativo.
// Lista los últimos 20 trabajos registrados en la BD de Drasus con su estado.
// Consulta el Bridge al montar y permite refrescar manualmente.

import 'package:flutter/material.dart';
import '../src/rust/api/jobs.dart';
import '../theme/gx_tokens.dart';

// Ruta a la base de datos SQLite de Drasus en el sistema de archivos local.
// Esta constante centraliza la ruta para que sea fácil de cambiar si el
// usuario configura una ubicación diferente. La ruta es la misma que usa
// el Core Rust al inicializar shared::create_pool().
const String _kDbPath = 'drasus.db';

// JobsTab es StatefulWidget porque necesita re-lanzar la consulta al Bridge
// cuando el usuario pulsa "Refrescar", lo que implica cambiar el Future
// que alimenta al FutureBuilder.
class JobsTab extends StatefulWidget {
  const JobsTab({super.key});

  @override
  State<JobsTab> createState() => _JobsTabState();
}

// _JobsTabState aloja el Future actual de la consulta de trabajos.
// Reemplazar _futureJobs con un nuevo Future hace que FutureBuilder
// descarte el resultado anterior y muestre el nuevo.
class _JobsTabState extends State<JobsTab> {
  // Future que representa la consulta en curso al Bridge.
  // Se inicializa en initState() y se puede reemplazar en _refrescar().
  late Future<List<JobSummary>> _futureJobs;

  // initState() lanza la primera consulta al Bridge al montar el widget.
  @override
  void initState() {
    super.initState();
    _futureJobs = _consultarTrabajos();
  }

  // Llama al Bridge para obtener los últimos 20 trabajos de la BD.
  // getJobsSummary requiere la ruta de la base de datos.
  Future<List<JobSummary>> _consultarTrabajos() {
    // Consulta al Bridge: retorna los últimos 20 trabajos en cualquier estado,
    // ordenados del más reciente al más antiguo.
    return getJobsSummary(dbPath: _kDbPath);
  }

  // Reemplaza _futureJobs con una nueva consulta, forzando a FutureBuilder
  // a reconstruir con los datos frescos de la BD.
  void _refrescar() {
    setState(() {
      _futureJobs = _consultarTrabajos();
    });
  }

  // Devuelve el color del chip de estado según el valor del campo state.
  // Solo la presentación visual vive aquí — la regla de qué estado es cuál
  // la conoce Rust; Dart solo mapea el String a un color.
  Color _colorPorEstado(String estado) {
    switch (estado) {
      case 'QUEUED':
        return Gx.alertAmber;
      case 'RUNNING':
        return Gx.transitionIndigo;
      case 'COMPLETED':
        return Gx.reactorGreen;
      case 'FAILED':
      case 'CANCELLED':
        return Gx.criticalCrimson;
      default:
        return Gx.textBaseMuted;
    }
  }

  // build() muestra la lista de trabajos. FutureBuilder gestiona
  // automáticamente los tres estados posibles: cargando, error, datos listos.
  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        // Barra de acción superior con el botón de refresco manual.
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
          child: Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text(
                'Últimos 20 trabajos',
                style: Theme.of(context).textTheme.titleSmall?.copyWith(color: Gx.textBaseMuted),
              ),
              // Botón de refresco: lanza una nueva consulta al Bridge.
              IconButton(
                icon: const Icon(Icons.refresh),
                tooltip: 'Refrescar lista de trabajos',
                onPressed: _refrescar,
              ),
            ],
          ),
        ),
        // FutureBuilder escucha el Future y reconstruye su hijo cada vez
        // que el Future cambia de estado (esperando → completado → error).
        Expanded(
          child: FutureBuilder<List<JobSummary>>(
            future: _futureJobs,
            builder: (context, snapshot) {
              // ConnectionState.waiting: el Future aún no resolvió.
              // Mostramos un indicador de progreso para dar feedback visual.
              if (snapshot.connectionState == ConnectionState.waiting) {
                return const Center(child: CircularProgressIndicator());
              }
              // Si el Future terminó con error, mostramos el mensaje.
              // Cubre casos como: BD no encontrada, corrupción, error FFI.
              if (snapshot.hasError) {
                return Center(
                  child: Text(
                    'Error al consultar trabajos:\n${snapshot.error}',
                    style: Theme.of(context).textTheme.bodySmall?.copyWith(color: Gx.criticalRed),
                    textAlign: TextAlign.center,
                  ),
                );
              }
              // datos listos — snapshot.data es la List<JobSummary> del Bridge.
              final jobs = snapshot.data ?? [];
              if (jobs.isEmpty) {
                return Center(
                  child: Text(
                    'Sin trabajos registrados.',
                    style: Theme.of(context).textTheme.bodySmall?.copyWith(color: Gx.textBaseMuted),
                  ),
                );
              }
              // ListView.builder construye solo los ítems visibles en pantalla
              // (renderizado diferido). Para 20 ítems no es crítico, pero es
              // el patrón correcto para listas que podrían crecer.
              return ListView.builder(
                itemCount: jobs.length,
                itemBuilder: (context, index) {
                  final job = jobs[index];
                  // Convierte nanosegundos a fecha legible para mostrar al usuario.
                  final fecha = DateTime.fromMicrosecondsSinceEpoch(
                    job.createdAt ~/ 1000,
                  ).toUtc().toIso8601String();
                  // Trunca el UUID a los primeros 8 chars para ahorrar espacio visual.
                  final idCorto = job.id.length > 8
                      ? job.id.substring(0, 8)
                      : job.id;
                  return ListTile(
                    dense: true,
                    // ID truncado del trabajo — suficiente para identificarlo visualmente.
                    leading: Text(
                      idCorto,
                      style: Theme.of(context).textTheme.labelSmall?.copyWith(color: Gx.textBaseSecondary),
                    ),
                    // Tipo de trabajo (BACKTEST, INGEST, etc.)
                    title: Text(
                      job.jobType,
                      style: Theme.of(context).textTheme.labelMedium,
                    ),
                    // Fecha de creación del trabajo.
                    subtitle: Text(
                      fecha,
                      style: Theme.of(context).textTheme.labelSmall?.copyWith(color: Gx.textBaseMuted),
                    ),
                    // Chip de estado con color según el valor del campo.
                    trailing: Container(
                      padding: const EdgeInsets.symmetric(
                        horizontal: 8,
                        vertical: 4,
                      ),
                      decoration: BoxDecoration(
                        color: _colorPorEstado(job.state).withOpacity(0.2),
                        borderRadius: BorderRadius.circular(4),
                        border: Border.all(
                          color: _colorPorEstado(job.state).withOpacity(0.6),
                        ),
                      ),
                      child: Text(
                        job.state,
                        style: Theme.of(context).textTheme.labelSmall?.copyWith(
                          color: _colorPorEstado(job.state),
                        ),
                      ),
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
