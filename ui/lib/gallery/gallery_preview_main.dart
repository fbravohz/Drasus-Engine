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
// Cómo usarlo:
//   cd ui
//   flutter run -d linux -t lib/gallery/gallery_preview_main.dart
//
//   Para golden tests:
//   flutter test --update-goldens test/gallery_golden_test.dart

import 'package:flutter/material.dart';
import 'gallery_tab.dart';

// Punto de entrada: monta solo la galería sin inicializar el Bridge Rust.
void main() {
  runApp(const _GalleryPreviewApp());
}

// Aplicación minimal que envuelve la galería en un tema oscuro.
class _GalleryPreviewApp extends StatelessWidget {
  const _GalleryPreviewApp();

  @override
  Widget build(BuildContext context) {
    // MaterialApp con tema oscuro para que el fondo del sistema no
    // choque con el deep space de la galería.
    return MaterialApp(
      title: 'Drasus — Preview de Galería',
      debugShowCheckedModeBanner: false,
      theme: ThemeData.dark(useMaterial3: false),
      // Scaffold raíz: body = GalleryTab, que ya lleva su propio
      // telón cósmico y scroll. Sin AppBar para maximizar el lienzo.
      home: const Scaffold(
        body: GalleryTab(),
      ),
    );
  }
}
