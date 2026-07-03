// sheet.dart — Componente Sheet (ADR-0138 enmienda 2026-06-29).
// Bottom-sheet modal con superficie reactiva al modo global de tema.
// Incluye un tirador visual y el helper showAppSheet() para su uso como overlay.
// Migrado de _WidgetCatalogSheet (tabs/dashboard_tab.dart, Batch 4 STORY-025).

import 'dart:ui' as dartUi;
import 'package:flutter/material.dart';
import '../gallery/gallery_tokens.dart';
import '../theme/theme_scope.dart';

// Bottom-sheet estilizado con efecto glass/tint/solid según el modo global de tema.
// Contrato funcional:
//   [child]  widget de contenido que aparece debajo del tirador visual.
//   [height] altura total del sheet en píxeles lógicos; default: 400.
class Sheet extends StatelessWidget {
  final Widget child;
  final double height;

  // No es const: el build lee ThemeState.globalSurfaceMode y getters dinámicos de Gx.
  Sheet({super.key, required this.child, this.height = 400.0});

  @override
  // Muestra el sheet con esquinas superiores redondeadas y superficie reactiva al tema.
  // En modo glass/enhancedGlass: BackdropFilter + glassFill. En tint/solid: color panel.
  Widget build(BuildContext context) {
    // Radio de las esquinas superiores: rChrome (14px) — token más cercano al 20px original.
    const topRadius = BorderRadius.vertical(top: Radius.circular(Gx.rChrome));
    final mode = ThemeState.globalSurfaceMode;

    final content = SizedBox(
      height: height,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Tirador visual centrado: indica que el sheet es desplazable.
          Center(
            child: Padding(
              padding: const EdgeInsets.only(top: Gx.space12),
              child: Container(
                width: 36,
                height: 4,
                // 2px: radio decorativo menor a 3px — sin token equivalente para píldora de handle.
                decoration: BoxDecoration(
                  color: Gx.borderPanel,
                  borderRadius: BorderRadius.circular(2),
                ),
              ),
            ),
          ),
          // Contenido proporcionado por el consumidor.
          Expanded(child: child),
        ],
      ),
    );

    // Superficie glass con BackdropFilter: blur 36 + color glassFill sobre contenido detrás.
    if (mode == SurfaceMode.glass || mode == SurfaceMode.enhancedGlass) {
      return ClipRRect(
        borderRadius: topRadius,
        child: BackdropFilter(
          filter: dartUi.ImageFilter.blur(sigmaX: 36, sigmaY: 36),
          child: Container(
            decoration: BoxDecoration(
              // glassFill: tinte translúcido que deja ver el contenido tras el blur.
              color: Gx.surfaceFill,
              borderRadius: topRadius,
              border: Border(
                top: BorderSide(
                  // Borde superior del rim-light del vidrio Apple.
                  color: Gx.textBase.withOpacity(Gx.glassEdgeOpacity),
                  width: Gx.borderHairline,
                ),
              ),
            ),
            child: content,
          ),
        ),
      );
    }

    // Superficie sólida/tint: sin blur, solo color de panel con borde base.
    return ClipRRect(
      borderRadius: topRadius,
      child: Container(
        decoration: BoxDecoration(
          color: Gx.surfacePanel,
          borderRadius: topRadius,
          border: Border(
            top: BorderSide(color: Gx.borderBase, width: Gx.borderHairline),
          ),
        ),
        child: content,
      ),
    );
  }
}

// Muestra el Sheet como bottom-sheet modal sobre la pantalla actual.
// Devuelve el valor que el widget pase a Navigator.pop<T>().
// [child]  contenido del sheet (sin tirador: lo añade Sheet internamente).
// [height] altura total incluyendo el tirador; default: 400.
Future<T?> showAppSheet<T>(
  BuildContext context, {
  required Widget child,
  double height = 400.0,
}) {
  return showModalBottomSheet<T>(
    context: context,
    // isScrollControlled: permite alturas mayores a la mitad de la pantalla.
    isScrollControlled: true,
    // transparent para que el BackdropFilter del Sheet vea el contenido detrás.
    backgroundColor: Colors.transparent,
    builder: (_) => Sheet(child: child, height: height),
  );
}
