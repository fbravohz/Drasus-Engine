// Punto de entrada aislado para previsualizar la Galería de Componentes.
//
// Por qué existe este archivo:
//   La app principal (lib/main.dart) llama a RustLib.init() para iniciar
//   el motor Rust/FFI antes de mostrar la interfaz. Ese init falla si el
//   binario nativo no está compilado o si no existe el entorno de trading.
//
//   Este main monta SOLO la galería (render-only, cero FFI, cero red,
//   cero persistencia) para que se pueda lanzar en cualquier máquina sin
//   tener el backend Rust listo.
//
//   Incluye selector de temas (acento + paleta de fondo) vía SettingsDrawer.
//
// Cómo usarlo:
//   cd ui
//   flutter run -d linux -t lib/gallery/gallery_preview_main.dart
//
//   Para golden tests:
//   flutter test --update-goldens test/gallery_golden_test.dart

import 'package:flutter/material.dart';
import '../drasus_theme.dart';
import '../tabs/settings_drawer.dart';
import 'gallery_tokens.dart';
import 'gallery_tab.dart';

// Punto de entrada: monta solo la galería con selector de temas.
void main() {
  WidgetsFlutterBinding.ensureInitialized();
  final state = DrasusThemeState();
  state.load().then((_) {
    runApp(_GalleryPreviewApp(state: state));
  });
}

// Aplicación con DrasusTheme + galería + selector de temas en settings drawer.
class _GalleryPreviewApp extends StatefulWidget {
  final DrasusThemeState state;
  const _GalleryPreviewApp({required this.state});

  @override
  State<_GalleryPreviewApp> createState() => _GalleryPreviewAppState();
}

// Estado reactivo de la app preview: escucha cambios de tema y reconstruye el árbol.
class _GalleryPreviewAppState extends State<_GalleryPreviewApp> {
  final _scaffoldKey = GlobalKey<ScaffoldState>();

  @override
  // Suscribe al listener del tema para triggerear setState al cambiar acento o modo de fondo.
  void initState() {
    super.initState();
    widget.state.addListener(() => setState(() {}));
  }

  @override
  // Monta DrasusTheme → MaterialApp con AppBar + SettingsDrawer + GalleryTab.
  // Los datos de tema vienen de DrasusThemeState (cargado en main()); GalleryTab no usa FFI.
  Widget build(BuildContext context) {
    return DrasusTheme(
      state: widget.state,
      child: MaterialApp(
        title: 'Drasus — Preview de Galería',
        debugShowCheckedModeBanner: false,
        theme: widget.state.buildThemeData(),
        home: Scaffold(
          key: _scaffoldKey,
          appBar: AppBar(
            title: Text('Galería de Componentes',
                style: TextStyle(fontFamily: Gx.fontDisplay, fontSize: 16)),
            backgroundColor: Gx.deepSpace,
            actions: [
              IconButton(
                icon: const Icon(Icons.settings),
                tooltip: 'Temas',
                onPressed: () =>
                    _scaffoldKey.currentState!.openEndDrawer(),
              ),
            ],
          ),
          endDrawer: const SettingsDrawer(),
          body: const GalleryTab(),
        ),
      ),
    );
  }
}
