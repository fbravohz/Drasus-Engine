// Pestaña del reloj determinista de Drasus.
// Muestra el timestamp actual en nanosegundos y su equivalente en fecha/hora
// legible. Actualiza cada 1 segundo mediante polling al Bridge.

import 'dart:async';
import 'package:flutter/material.dart';
// Importa la función getClockTimestampNs() expuesta por el Bridge Rust.
import '../src/rust/api/clock.dart';

// ClockTab es un StatefulWidget porque necesita actualizar lo que muestra
// en pantalla cada segundo. Un StatelessWidget no puede cambiar por sí solo.
class ClockTab extends StatefulWidget {
  const ClockTab({super.key});

  // createState() conecta el widget con su objeto de estado.
  // Flutter llama a este método exactamente una vez al insertar ClockTab
  // en el árbol de widgets.
  @override
  State<ClockTab> createState() => _ClockTabState();
}

// _ClockTabState aloja el estado mutable de ClockTab: el timestamp actual
// y el Timer que lo actualiza. El guión bajo (_) indica que es privado al archivo.
class _ClockTabState extends State<ClockTab> {
  // Timestamp más reciente recibido del Bridge, en nanosegundos desde Unix epoch.
  // Valor inicial 0 para evitar null — antes de la primera lectura del Bridge
  // se muestra "cargando..."
  int _timestampNs = 0;

  // Timer que dispara el polling al Bridge cada 1 segundo.
  // Es nullable (Timer?) porque aún no existe cuando se crea el State.
  Timer? _timer;

  // initState() es el constructor del State. Flutter lo llama una vez,
  // justo después de insertar este State en el árbol.
  // Aquí se arranca el Timer de polling.
  @override
  void initState() {
    super.initState();
    // Hace la primera lectura inmediatamente para no mostrar 0 durante 1 segundo.
    _actualizarReloj();
    // Timer.periodic dispara el callback cada 1 segundo indefinidamente.
    // Duration(seconds: 1) = el intervalo entre llamadas.
    _timer = Timer.periodic(const Duration(seconds: 1), (_) {
      _actualizarReloj();
    });
  }

  // Llama al Bridge para obtener el timestamp actual y actualiza el estado.
  // getClockTimestampNs() es síncrona (#[frb(sync)] en Rust) — no necesita await.
  void _actualizarReloj() {
    // Llamada al Bridge: obtiene el timestamp del reloj determinista de Drasus
    // en nanosegundos desde el Unix epoch.
    final ns = getClockTimestampNs();
    // setState() notifica a Flutter que el estado cambió y que debe reconstruir
    // el widget. Sin esta llamada, la pantalla no se actualiza aunque _timestampNs cambie.
    setState(() {
      _timestampNs = ns;
    });
  }

  // dispose() se llama cuando Flutter retira este State del árbol (p.ej. al
  // navegar a otra pestaña permanentemente o al cerrar la app).
  // CRÍTICO: cancelar el Timer aquí evita que siga ejecutándose después de
  // que el widget fue destruido, lo que provocaría un error de "setState on
  // disposed widget" o una fuga de memoria.
  @override
  void dispose() {
    _timer?.cancel();
    super.dispose();
  }

  // build() describe lo que muestra esta pestaña. Flutter lo llama cada vez
  // que setState() se invoca — en este caso, cada segundo.
  // Muestra el timestamp en nanosegundos y su conversión a fecha/hora legible.
  @override
  Widget build(BuildContext context) {
    // Convierte nanosegundos a microsegundos (÷1000) para DateTime.
    // DateTime.fromMicrosecondsSinceEpoch() es la conversión estándar Dart
    // para timestamps de alta resolución.
    final fechaHora = _timestampNs > 0
        ? DateTime.fromMicrosecondsSinceEpoch(_timestampNs ~/ 1000).toUtc()
        : null;

    return Padding(
      padding: const EdgeInsets.all(24.0),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Etiqueta descriptiva — no cambia entre frames.
          const Text(
            'Reloj determinista Drasus',
            style: TextStyle(
              fontFamily: 'monospace',
              fontSize: 14,
              color: Colors.grey,
            ),
          ),
          const SizedBox(height: 12),
          // Timestamp en nanosegundos — valor bruto del Bridge.
          Text(
            _timestampNs > 0 ? '$_timestampNs ns' : 'cargando...',
            style: const TextStyle(
              fontFamily: 'monospace',
              fontSize: 22,
              // Verde claro para datos "vivos" — convención de terminales.
              color: Color(0xFF80FF80),
            ),
          ),
          const SizedBox(height: 8),
          // Fecha y hora legible en UTC.
          Text(
            fechaHora != null
                ? fechaHora.toIso8601String()
                : '—',
            style: const TextStyle(
              fontFamily: 'monospace',
              fontSize: 16,
              color: Colors.white70,
            ),
          ),
        ],
      ),
    );
  }
}
