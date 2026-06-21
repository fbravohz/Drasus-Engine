// Punto de entrada de la aplicación Drasus Engine.
// Inicializa el bridge Rust↔Dart antes de montar el árbol de widgets.

import 'package:flutter/material.dart';
import 'src/rust/frb_generated.dart';
import 'panel_operativo.dart';

// main() es el punto de entrada de toda aplicación Dart.
// Es async porque debemos esperar a que la librería nativa de Rust cargue
// antes de mostrar cualquier widget.
Future<void> main() async {
  // Garantiza que el motor de Flutter esté listo para recibir llamadas
  // de plataforma antes de que cualquier plugin (incluido el bridge FFI)
  // intente usarlo. Sin esta línea, llamar a RustLib.init() en una app
  // desktop puede lanzar una excepción por orden de inicialización.
  WidgetsFlutterBinding.ensureInitialized();

  // Carga la librería nativa compilada de Rust (.so en Linux, .dylib en
  // macOS, .dll en Windows) y establece el canal de comunicación FFI.
  // A partir de este punto, todas las funciones del Bridge están disponibles.
  await RustLib.init();

  // Monta el árbol de widgets. A partir de aquí Flutter controla el ciclo
  // de vida completo de la interfaz.
  runApp(const DrasusApp());
}

// Widget raíz de la aplicación. Es StatelessWidget porque no tiene estado
// propio: solo configura el tema y delega la pantalla al PanelOperativo.
class DrasusApp extends StatelessWidget {
  const DrasusApp({super.key});

  // build() describe qué muestra este widget en pantalla.
  // Retorna un MaterialApp configurado en modo oscuro que carga el panel.
  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Drasus Engine',
      // Oculta el banner rojo "DEBUG" de la esquina superior derecha.
      debugShowCheckedModeBanner: false,
      // Tema oscuro con Material Design 3 — adecuado para interfaces de datos.
      // useMaterial3: true activa la paleta de colores y tipografía M3.
      theme: ThemeData.dark(useMaterial3: true),
      // Pantalla inicial: el Panel Operativo con sus 3 pestañas.
      home: const PanelOperativo(),
    );
  }
}
