// carousel.dart — Componente Carousel (ADR-0138 enmienda 2026-06-29).
// Carrusel de ítems deslizables con puntos de navegación animados.
// Estilo 100% por tema; prohibido hardcodear color.

import 'package:flutter/material.dart';
import '../theme/gx_tokens.dart';

// Carrusel de widgets deslizables con puntos de navegación.
// Modo no controlado: gestiona internamente el índice de página activa.
class Carousel extends StatefulWidget {
  // items: lista de widgets que componen cada página del carrusel.
  final List<Widget> items;
  // itemHeight: altura de cada página en píxeles lógicos.
  final double itemHeight;
  // onPageChanged: se llama con el índice de la página visible al cambiar.
  final ValueChanged<int>? onPageChanged;

  const Carousel({
    super.key,
    required this.items,
    this.itemHeight = 80.0,
    this.onPageChanged,
  });

  @override
  State<Carousel> createState() => _CarouselState();
}

class _CarouselState extends State<Carousel> {
  // Índice de la página actualmente visible (0-indexado).
  int _current = 0;
  late final PageController _ctrl;

  @override
  void initState() {
    super.initState();
    _ctrl = PageController();
  }

  @override
  void dispose() {
    _ctrl.dispose();
    super.dispose();
  }

  @override
  // Dibuja la vista de páginas deslizables y los puntos de navegación.
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        // Vista de páginas: desliza horizontal entre los ítems.
        SizedBox(
          height: widget.itemHeight,
          child: PageView.builder(
            controller: _ctrl,
            itemCount: widget.items.length,
            onPageChanged: (i) {
              setState(() => _current = i);
              widget.onPageChanged?.call(i);
            },
            // Cada ítem lleva margen lateral para separación visual entre páginas.
            itemBuilder: (_, i) => Padding(
              padding: const EdgeInsets.symmetric(horizontal: 8),
              child: widget.items[i],
            ),
          ),
        ),
        const SizedBox(height: 8),
        // Puntos de navegación: el punto activo se ensancha (pill) con glow de énfasis.
        Row(
          mainAxisSize: MainAxisSize.min,
          children: List.generate(widget.items.length, (i) {
            final active = i == _current;
            return GestureDetector(
              onTap: () => _ctrl.animateToPage(
                i,
                duration: const Duration(milliseconds: 300),
                curve: Curves.easeOut,
              ),
              child: AnimatedContainer(
                duration: const Duration(milliseconds: 180),
                // Punto activo: ancho 18px (pill). Punto inactivo: 8px (círculo).
                width: active ? 18 : 8,
                height: 8,
                margin: const EdgeInsets.symmetric(horizontal: 3),
                decoration: BoxDecoration(
                  color: active ? Gx.accentDynamic : Gx.borderBase,
                  borderRadius: BorderRadius.circular(999),
                  boxShadow: active
                      ? Gx.glow(Gx.accentDynamic, blur: 8, opacity: 0.6)
                      : null,
                ),
              ),
            );
          }),
        ),
      ],
    );
  }
}
