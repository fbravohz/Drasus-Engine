// Punto de entrada de la aplicación Drasus Engine.
// Inicializa el bridge Rust↔Dart antes de montar el árbol de widgets.

import 'package:flutter/material.dart';
import 'app_meta.dart';
import 'src/rust/frb_generated.dart';
import 'theme/theme_scope.dart';
import 'operational_panel.dart';

// main() es el punto de entrada de toda aplicación Dart.
// Es async porque debemos esperar a que la librería nativa de Rust cargue
// antes de mostrar cualquier widget.
Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();

  // Inicializa el estado del tema y carga preferencias de SharedPreferences.
  final themeState = ThemeState();
  await themeState.load();

  // Carga la librería nativa compilada de Rust (.so en Linux, .dylib en
  // macOS, .dll en Windows) y establece el canal de comunicación FFI.
  await RustLib.init();

  runApp(AppRoot(state: themeState));
}

// Widget raíz de la aplicación. Usa ThemeState para acento + paleta de fondo.
class AppRoot extends StatelessWidget {
  final ThemeState state;
  const AppRoot({super.key, required this.state});

  @override
  Widget build(BuildContext context) {
    return ThemeScope(
      state: state,
      child: MaterialApp(
        title: kAppName,
        debugShowCheckedModeBanner: false,
        theme: state.buildThemeData(),
        home: const OperationalPanel(),
      ),
    );
  }
}
