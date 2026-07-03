// accordion.dart — Componente Accordion (ADR-0138 enmienda 2026-06-29).
// Lista de secciones plegables con cabecera clicable y cuerpo animado.
// El estilo lo decide el tema global vía tokens Gx; prohibido hardcodear color.

import 'package:flutter/material.dart';
import '../gallery/gallery_tokens.dart';
import '../gallery/gallery_fx.dart';

// Elemento de datos de cada sección del acordeón.
// [title] es la cabecera clicable; [content] es el texto del cuerpo expandido.
class AccordionItem {
  final String title;
  final String content;
  const AccordionItem({required this.title, required this.content});
}

// Lista de secciones plegables; solo una puede estar abierta a la vez.
// Cabecera activa: fondo surfaceRaisedDynamic + borde inferior semántico (transitionIndigo).
// Cabecera inactiva: fondo surfacePanel + borde estructural global.
// Cuerpo abierto: cardSurface() — reacciona a los modos del tema.
// La flecha del encabezado rota 180° al abrir/cerrar.
//
// Contrato funcional:
//   [items]     lista de AccordionItem (título + contenido).
//   [openIndex] índice de la sección abierta (null = no controlado; arranca en 0).
//               Pasar -1 para empezar con ninguna sección abierta.
//   [onChanged] callback con el índice de la nueva sección abierta (-1 = ninguna).
class Accordion extends StatefulWidget {
  final List<AccordionItem> items;
  final int? openIndex;
  final ValueChanged<int>? onChanged;

  // No es const: los getters dinámicos de Gx cambian con el tema.
  Accordion({
    super.key,
    required this.items,
    this.openIndex,
    this.onChanged,
  });

  @override
  State<Accordion> createState() => _AccordionState();
}

class _AccordionState extends State<Accordion> {
  // Índice abierto interno para modo no controlado (-1 = ninguna abierta).
  int _internalOpen = 0;

  // Índice efectivo: el externo (openIndex) tiene prioridad.
  int get _open => widget.openIndex ?? _internalOpen;

  // Alterna la sección: si ya estaba abierta se cierra (pasa a -1); si no, se abre.
  void _toggle(int index) {
    final next = _open == index ? -1 : index;
    if (widget.openIndex == null) setState(() => _internalOpen = next);
    widget.onChanged?.call(next);
  }

  @override
  // Columna de secciones; cada una tiene cabecera + cuerpo expandible animado.
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: widget.items.asMap().entries.map((e) {
        final isOpen = e.key == _open;
        return Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            // Cabecera: clicable, cambia fondo y borde según estado de apertura.
            GestureDetector(
              onTap: () => _toggle(e.key),
              child: Container(
                padding: const EdgeInsets.symmetric(
                    horizontal: Gx.space12,
                    vertical: Gx.space8 + Gx.space4),
                decoration: BoxDecoration(
                  // surfaceRaisedDynamic (token de hover/activo) en abierta; panel en cerrada.
                  color: isOpen ? Gx.surfaceRaisedDynamic : Gx.surfacePanel,
                  border: Border(
                    bottom: BorderSide(
                      // Borde semántico (transitionIndigo) en activa; estructural global en cerrada.
                      color: isOpen ? Gx.transitionIndigo : Gx.borderBase,
                      width:
                          isOpen ? Gx.borderFocus : Gx.borderHairline,
                    ),
                  ),
                ),
                child: Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    // Título: token base dinámico (abierta) o secundario (cerrada).
                    Expanded(
                      child: Text(
                        e.value.title,
                        overflow: TextOverflow.ellipsis,
                        style: Gx.uiSans(
                          fontSize: 13,
                          color: isOpen
                              ? Gx.textBase
                              : Gx.textBaseSecondary,
                          weight: isOpen
                              ? FontWeight.w500
                              : FontWeight.w400,
                        ),
                      ),
                    ),
                    // Flecha: rota 180° al abrir (turns 0 → 0.5).
                    AnimatedRotation(
                      turns: isOpen ? 0.5 : 0,
                      duration: const Duration(milliseconds: 200),
                      child: Icon(Gx.iconChevronDown,
                          size: 14, color: Gx.textBaseSecondary),
                    ),
                  ],
                ),
              ),
            ),
            // Cuerpo expandible: cardSurface() reacciona a los modos del tema.
            AnimatedSize(
              duration: const Duration(milliseconds: 220),
              curve: Curves.easeOut,
              child: isOpen
                  ? cardSurface(
                      padding: const EdgeInsets.symmetric(
                          horizontal: Gx.space12,
                          vertical: Gx.space8 + Gx.space4),
                      child: Text(
                        e.value.content,
                        style: Gx.uiSans(
                          fontSize: 12,
                          color: Gx.textBaseSecondary,
                        ),
                      ),
                    )
                  : const SizedBox.shrink(),
            ),
          ],
        );
      }).toList(),
    );
  }
}
