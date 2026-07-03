// scrollspy.dart — Componente Scrollspy (ADR-0138 enmienda 2026-06-29).
// Índice lateral de secciones con la sección activa resaltada.
// Simula el comportamiento de un ancla lateral que sigue la posición del scroll.

import 'package:flutter/material.dart';
import '../gallery/gallery_tokens.dart';
import '../gallery/gallery_fx.dart';

// Lista de anclas de secciones con indicador de posición activa.
// Modo controlado:    Scrollspy(sections: ..., active: _i, onChanged: ...)
// Modo no controlado: Scrollspy(sections: ..., onChanged: ...)  (arranca en sección 0)
class Scrollspy extends StatefulWidget {
  // sections: nombres de las secciones del documento en orden de aparición.
  final List<String> sections;
  // active: índice de la sección activa (modo controlado).
  final int? active;
  // onChanged: se llama con el índice al tocar una sección.
  final ValueChanged<int>? onChanged;

  const Scrollspy({
    super.key,
    required this.sections,
    this.active,
    this.onChanged,
  });

  @override
  State<Scrollspy> createState() => _ScrollspyState();
}

class _ScrollspyState extends State<Scrollspy> {
  // Índice activo interno para el modo no controlado.
  late int _active;

  @override
  void initState() {
    super.initState();
    _active = widget.active ?? 0;
  }

  @override
  void didUpdateWidget(Scrollspy old) {
    super.didUpdateWidget(old);
    // Modo controlado: sincroniza el índice activo cuando el padre lo cambia.
    if (widget.active != null && widget.active != _active) {
      setState(() => _active = widget.active!);
    }
  }

  @override
  // Dibuja un panel con la lista de secciones; la activa tiene filo de énfasis izquierdo.
  Widget build(BuildContext context) {
    return panelSurface(
      padding: const EdgeInsets.symmetric(vertical: 8, horizontal: 0),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: widget.sections.asMap().entries.map((e) {
          final isActive = e.key == _active;
          return GestureDetector(
            onTap: () {
              setState(() => _active = e.key);
              widget.onChanged?.call(e.key);
            },
            child: AnimatedContainer(
              duration: const Duration(milliseconds: 180),
              curve: Curves.easeOut,
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 9),
              decoration: BoxDecoration(
                // Fondo tenue del énfasis dinámico en la sección activa.
                color: isActive ? Gx.accentDynamic.withAlpha(20) : Colors.transparent,
                // Filo del énfasis de 2px a la izquierda: indica posición actual.
                border: Border(
                  left: BorderSide(
                    color: isActive ? Gx.accentDynamic : Colors.transparent,
                    width: 2,
                  ),
                ),
              ),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  // Punto de estado: relleno y con glow en la sección activa; atenuado en las demás.
                  AnimatedContainer(
                    duration: const Duration(milliseconds: 200),
                    width: 5,
                    height: 5,
                    margin: const EdgeInsets.only(right: 8),
                    decoration: BoxDecoration(
                      shape: BoxShape.circle,
                      color: isActive ? Gx.accentDynamic : Gx.textBaseMuted,
                      boxShadow: isActive
                          ? Gx.glow(Gx.accentDynamic, blur: 6, opacity: 0.8)
                          : null,
                    ),
                  ),
                  // Etiqueta de la sección: negrita y color base en la activa; muted en las demás.
                  Text(
                    e.value,
                    style: Gx.uiSans(
                      fontSize: 12,
                      color: isActive ? Gx.textBase : Gx.textBaseLabel,
                      weight: isActive ? FontWeight.w500 : FontWeight.w400,
                    ),
                  ),
                ],
              ),
            ),
          );
        }).toList(),
      ),
    );
  }
}
