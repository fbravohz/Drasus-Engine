// back_to_top.dart — Componente BackToTop (ADR-0138 enmienda 2026-06-29).
// Botón flotante circular "volver arriba" con superficie de panel y glow de énfasis.
// No es const: lee Gx.* en build(); un const freezaría los tokens del tema.

import 'package:flutter/material.dart';
import '../gallery/gallery_tokens.dart';
import '../gallery/gallery_fx.dart';

// Botón circular "Volver arriba" con superficie de panel y glow de énfasis dinámico.
// Uso típico: envuelto en Align(alignment: Alignment.bottomRight) sobre un Stack.
// Si se pasa controller, lleva el scroll hasta 0.0 al presionar.
// Si no hay controller, llama a onPressed como callback alternativo.
//
// Sin const en el constructor: este widget lee Gx.* en build();
// un constructor const freezaría los tokens del tema al montar el widget.
class BackToTop extends StatelessWidget {
  // controller: ScrollController del contenido; anima el scroll a 0.0 al presionar.
  final ScrollController? controller;
  // onPressed: callback alternativo cuando no se pasa controller.
  final VoidCallback? onPressed;

  // Constructor sin const — lee tokens dinámicos en build().
  // ignore: prefer_const_constructors_in_immutables
  BackToTop({super.key, this.controller, this.onPressed});

  // Lleva el scroll a 0.0 con animación (si hay controller) o llama al callback.
  void _handleTap() {
    if (controller != null) {
      controller!.animateTo(
        0,
        duration: const Duration(milliseconds: 400),
        curve: Curves.easeOut,
      );
    }
    onPressed?.call();
  }

  @override
  // Botón circular con ClipOval + panelSurface para reaccionar al modo global del tema.
  Widget build(BuildContext context) {
    return Align(
      alignment: Alignment.bottomRight,
      child: GestureDetector(
        onTap: _handleTap,
        child: ClipOval(
          child: panelSurface(
            radius: 999,
            padding: const EdgeInsets.all(Gx.space4),
            glow: Gx.glow(Gx.accentDynamic, blur: 14, opacity: 0.35),
            child: SizedBox(
              width: 34,
              height: 34,
              child: Center(
                // Icono con token dinámico secundario — legible tanto en bunker como en paper.
                child: Icon(
                  Icons.keyboard_arrow_up,
                  size: 20,
                  color: Gx.textBaseSecondary,
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}
