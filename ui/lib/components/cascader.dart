// cascader.dart — Componente Cascader<T> (ADR-0138 enmienda 2026-06-29).
// Selector jerárquico en cascada: dos columnas (nivel 1 y nivel 2).
// El estilo lo decide el tema global vía tokens Gx; prohibido hardcodear color.

import 'package:flutter/material.dart';
import '../gallery/gallery_tokens.dart';
import '../gallery/gallery_fx.dart';

// Nodo del árbol del Cascader — value tipado + label + hijos opcionales.
class CascaderNode<T> {
  final T value;
  final String label;
  final List<CascaderNode<T>> children;

  const CascaderNode({
    required this.value,
    required this.label,
    this.children = const [],
  });
}

// Selector jerárquico de dos niveles (expandible a N niveles en futuras versiones).
// Contrato funcional:
//   [value]     valor actualmente seleccionado en el nivel hoja (null = sin selección).
//   [nodes]     árbol de nodos raíz con sus hijos.
//   [onChanged] callback con el valor del nodo hoja seleccionado al tocar nivel 2.
// Modo no controlado: el componente rastrea [_selectedLevel1] y [_selectedLeaf] internamente.
// El color semántico del nivel 1 activo es el énfasis dinámico del tema.
class Cascader<T> extends StatefulWidget {
  final T? value;
  final List<CascaderNode<T>> nodes;
  final ValueChanged<T>? onChanged;

  // No es const: lee getters dinámicos de Gx que cambian con el tema.
  Cascader({
    super.key,
    this.value,
    required this.nodes,
    this.onChanged,
  });

  @override
  State<Cascader<T>> createState() => _CascaderState<T>();
}

class _CascaderState<T> extends State<Cascader<T>> {
  // Índice del nodo de nivel 1 seleccionado actualmente (0 = primero por defecto).
  int _sel1 = 0;
  // Valor del nodo hoja seleccionado en modo no controlado.
  T? _internalLeaf;

  // Valor hoja efectivo: el externo tiene prioridad sobre el interno.
  T? get _effectiveLeaf => widget.value ?? _internalLeaf;

  // Selecciona un nodo hoja (nivel 2) y notifica al padre.
  void _selectLeaf(T leafValue) {
    if (widget.value == null) setState(() => _internalLeaf = leafValue);
    widget.onChanged?.call(leafValue);
  }

  @override
  // Panel de dos columnas separadas por un hairline vertical.
  Widget build(BuildContext context) {
    final children = _sel1 < widget.nodes.length
        ? widget.nodes[_sel1].children
        : <CascaderNode<T>>[];

    return panelSurface(
      padding: EdgeInsets.zero,
      child: SizedBox(
        height: 130,
        child: Row(
          children: [
            // Columna 1 — nodos raíz; el activo tiene borde izquierdo del énfasis dinámico.
            Expanded(
              child: Column(
                children: widget.nodes.asMap().entries.map((e) {
                  final isActive = e.key == _sel1;
                  return GestureDetector(
                    onTap: () => setState(() => _sel1 = e.key),
                    child: AnimatedContainer(
                      duration: const Duration(milliseconds: 160),
                      padding: const EdgeInsets.symmetric(
                          horizontal: 12, vertical: 10),
                      decoration: BoxDecoration(
                        // Fondo tenue del énfasis dinámico cuando el ítem está activo.
                        color: isActive
                            ? Gx.accentDynamic.withOpacity(0.12)
                            : Colors.transparent,
                        // Borde lateral del énfasis dinámico como indicador de activo.
                        border: isActive
                            ? Border(
                                right: BorderSide(
                                    color: Gx.accentDynamic, width: 2))
                            : null,
                      ),
                      child: Row(
                        mainAxisAlignment: MainAxisAlignment.spaceBetween,
                        children: [
                          // Flexible evita desbordamiento con textos largos.
                          Flexible(
                            child: Text(
                              e.value.label,
                              overflow: TextOverflow.ellipsis,
                              style: Gx.uiSans(
                                fontSize: 13,
                                // Activo: énfasis dinámico. Inactivo: token dinámico secundario.
                                color: isActive
                                    ? Gx.accentDynamic
                                    : Gx.textBaseSecondary,
                              ),
                            ),
                          ),
                          // Indicador de subnivel: flecha pequeña con token apropiado.
                          Icon(
                            Gx.iconChevronDown,
                            size: 10,
                            color: isActive
                                ? Gx.accentDynamic
                                : Gx.textBaseMuted,
                          ),
                        ],
                      ),
                    ),
                  );
                }).toList(),
              ),
            ),
            // Separador vertical hairline con borde estructural global.
            Container(width: Gx.borderHairline, color: Gx.borderBase),
            // Columna 2 — hijos del nodo de nivel 1 seleccionado.
            Expanded(
              child: children.isEmpty
                  ? Center(
                      child: Text(
                        '—',
                        style: Gx.uiSans(
                            fontSize: 12, color: Gx.textBaseMuted),
                      ),
                    )
                  : Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: children.map((child) {
                        final isLeafSelected =
                            child.value == _effectiveLeaf;
                        return GestureDetector(
                          onTap: () => _selectLeaf(child.value),
                          child: AnimatedContainer(
                            duration: const Duration(milliseconds: 140),
                            width: double.infinity,
                            padding: const EdgeInsets.symmetric(
                                horizontal: 12, vertical: 10),
                            // Fondo de énfasis dinámico cuando este nodo hoja está seleccionado.
                            color: isLeafSelected
                                ? Gx.accentDynamic.withOpacity(0.10)
                                : Colors.transparent,
                            child: Text(
                              child.label,
                              style: Gx.dataMono(
                                fontSize: 13,
                                // Seleccionado: énfasis dinámico. Reposo: token dinámico base.
                                color: isLeafSelected
                                    ? Gx.accentDynamic
                                    : Gx.textBase,
                              ),
                            ),
                          ),
                        );
                      }).toList(),
                    ),
            ),
          ],
        ),
      ),
    );
  }
}
