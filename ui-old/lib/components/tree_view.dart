// tree_view.dart — Componente TreeView (ADR-0138 enmienda 2026-06-29).
// Árbol de navegación con nodos raíz expandibles y selección de hoja.
// Estilo 100% por tema; prohibido hardcodear color.

import 'package:flutter/material.dart';
import '../theme/gx_tokens.dart';

// Nodo del árbol de navegación.
// Puede ser raíz (con hijos) o hoja (sin hijos, seleccionable).
class TreeViewNode {
  // label: texto visible del nodo en el árbol.
  final String label;
  // id: identificador único pasado a onSelect al seleccionar la hoja.
  final int id;
  // children: nodos hijo. Vacío = este nodo es hoja (sin triángulo plegable).
  final List<TreeViewNode> children;
  // initiallyExpanded: true abre el nodo raíz al montar el widget.
  final bool initiallyExpanded;

  const TreeViewNode({
    required this.label,
    required this.id,
    this.children = const [],
    this.initiallyExpanded = false,
  });
}

// Árbol de navegación interactivo con expansión de raíces y selección de hojas.
// Modo controlado:    TreeView(nodes: ..., selected: _id, onSelect: ...)
// Modo no controlado: TreeView(nodes: ..., onSelect: ...)
class TreeView extends StatefulWidget {
  // nodes: lista de nodos raíz que forman el árbol.
  final List<TreeViewNode> nodes;
  // selected: id del nodo hoja actualmente seleccionado (modo controlado).
  final int? selected;
  // onSelect: se llama con el id de la hoja al tocarla.
  final ValueChanged<int>? onSelect;

  const TreeView({
    super.key,
    required this.nodes,
    this.selected,
    this.onSelect,
  });

  @override
  State<TreeView> createState() => _TreeViewState();
}

class _TreeViewState extends State<TreeView> {
  // Estado de expansión de los nodos raíz, mapeado por su id.
  late final Map<int, bool> _expanded;
  // Id del nodo hoja seleccionado actualmente (estado interno).
  int? _selected;

  @override
  void initState() {
    super.initState();
    _selected = widget.selected;
    _expanded = {
      for (final n in widget.nodes) n.id: n.initiallyExpanded,
    };
  }

  @override
  void didUpdateWidget(TreeView old) {
    super.didUpdateWidget(old);
    // Modo controlado: sincroniza la selección cuando el padre la cambia.
    if (widget.selected != _selected) {
      setState(() => _selected = widget.selected);
    }
  }

  // Selecciona una hoja y notifica al padre.
  void _select(int id) {
    setState(() => _selected = id);
    widget.onSelect?.call(id);
  }

  @override
  // Dibuja la lista de nodos raíz con sus hojas colapsadas/expandidas.
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: widget.nodes.map(_buildRoot).toList(),
    );
  }

  // Construye un nodo raíz con su lista de hojas animada.
  Widget _buildRoot(TreeViewNode node) {
    final isOpen = _expanded[node.id] ?? false;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // Fila del nodo raíz: el clic alterna la expansión/colapso.
        GestureDetector(
          onTap: () => setState(() => _expanded[node.id] = !isOpen),
          child: Padding(
            padding: const EdgeInsets.symmetric(vertical: 5, horizontal: 8),
            child: Row(children: [
              // Triángulo que rota 90° al expandir el nodo.
              AnimatedRotation(
                turns: isOpen ? 0.25 : 0,
                duration: const Duration(milliseconds: 180),
                child: Icon(
                  Gx.iconChevronDown,
                  size: 12,
                  color: Gx.textBaseSecondary,
                ),
              ),
              const SizedBox(width: 6),
              Text(
                node.label,
                style: Gx.uiSans(
                  fontSize: 13,
                  color: Gx.textBaseSecondary,
                  weight: FontWeight.w500,
                ),
              ),
            ]),
          ),
        ),
        // Hojas: solo visibles cuando el nodo raíz está abierto.
        AnimatedSize(
          duration: const Duration(milliseconds: 200),
          curve: Curves.easeOut,
          child: isOpen
              ? Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: node.children.map(_buildLeaf).toList(),
                )
              : const SizedBox.shrink(),
        ),
      ],
    );
  }

  // Construye una hoja con indicador de selección (filo de énfasis izquierdo).
  Widget _buildLeaf(TreeViewNode leaf) {
    final isSel = leaf.id == _selected;
    return GestureDetector(
      onTap: () => _select(leaf.id),
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 140),
        padding: const EdgeInsets.symmetric(vertical: 5, horizontal: 8),
        // Las hojas están indentadas 20px respecto al nodo raíz.
        margin: const EdgeInsets.only(left: 20),
        decoration: BoxDecoration(
          // Fondo tenue del énfasis cuando la hoja está seleccionada.
          color: isSel ? Gx.accentDynamic.withAlpha(20) : Colors.transparent,
          borderRadius: BorderRadius.circular(Gx.rChip),
          // Filo de énfasis de 2px en el lado izquierdo: indica selección activa.
          border: isSel
              ? Border(left: BorderSide(color: Gx.accentDynamic, width: 2))
              : null,
        ),
        child: Text(
          leaf.label,
          style: Gx.dataMono(
            fontSize: 12,
            color: isSel ? Gx.accentDynamic : Gx.textBaseLabel,
          ),
        ),
      ),
    );
  }
}
