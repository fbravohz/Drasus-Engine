// tree_table.dart — Componente TreeTable (ADR-0138 enmienda 2026-06-29).
// Tabla jerárquica con filas raíz expandibles que revelan filas hijo con sangría.
// Estilo 100% por tema; prohibido hardcodear color.

import 'package:flutter/material.dart';
import '../theme/gx_tokens.dart';

// Definición de una columna de cabecera para la tabla jerárquica.
class TreeTableColumn {
  // label: texto mostrado en la cabecera.
  final String label;
  // numeric: true alinea el contenido a la derecha (cifras y métricas).
  final bool numeric;
  const TreeTableColumn({required this.label, this.numeric = false});
}

// Nodo de datos de la tabla jerárquica.
// Puede tener hijos (sub-filas indentadas) o ser un nodo hoja (sin triángulo).
class TreeTableNode {
  // id: identificador único; se pasa a onNodeTap al hacer clic.
  final String id;
  // cells: lista de widgets en el orden de las columnas; debe coincidir con columns.
  final List<Widget> cells;
  // children: sub-nodos que aparecen indentados al expandir esta fila.
  final List<TreeTableNode> children;
  // initiallyExpanded: true abre el nodo al montar el widget.
  final bool initiallyExpanded;

  const TreeTableNode({
    required this.id,
    required this.cells,
    this.children = const [],
    this.initiallyExpanded = false,
  });
}

// Tabla jerárquica con expansión por fila raíz y selección de nodo.
class TreeTable extends StatefulWidget {
  // columns: lista de cabeceras en el orden de aparición de las columnas.
  final List<TreeTableColumn> columns;
  // nodes: lista de nodos raíz; cada nodo puede tener sub-nodos.
  final List<TreeTableNode> nodes;
  // onNodeTap: se llama con el id del nodo al hacer clic en cualquier fila.
  final ValueChanged<String>? onNodeTap;

  const TreeTable({
    super.key,
    required this.columns,
    required this.nodes,
    this.onNodeTap,
  });

  @override
  State<TreeTable> createState() => _TreeTableState();
}

class _TreeTableState extends State<TreeTable> {
  // Mapa de id de nodo a si está expandido; inicializado desde initiallyExpanded.
  late final Map<String, bool> _expanded;

  @override
  void initState() {
    super.initState();
    _expanded = {
      for (final n in widget.nodes) n.id: n.initiallyExpanded,
    };
  }

  // Construye la cabecera de una columna con tipografía de etiqueta del tema.
  Widget _headerCell(TreeTableColumn col) => Expanded(
        child: Text(
          col.label,
          textAlign: col.numeric ? TextAlign.right : TextAlign.left,
          style: Gx.uiSans(fontSize: 11, color: Gx.textBaseLabel),
        ),
      );

  // Fila de datos con separador inferior por token de borde del tema.
  Widget _row({
    required List<Widget> cells,
    bool isChild = false,
    VoidCallback? onTap,
  }) {
    return GestureDetector(
      onTap: onTap,
      child: Container(
        // Los nodos hijo llevan sangría de 20px para indicar jerarquía.
        padding: EdgeInsets.symmetric(vertical: 6, horizontal: isChild ? 20 : 8),
        decoration: BoxDecoration(
          border: Border(bottom: BorderSide(color: Gx.borderBase)),
        ),
        child: Row(children: cells),
      ),
    );
  }

  @override
  // Dibuja la cabecera + todos los nodos raíz con sus hijos.
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        // Fila de cabecera.
        _row(cells: widget.columns.map(_headerCell).toList()),
        // Nodos raíz.
        ...widget.nodes.map(_buildNode),
      ],
    );
  }

  // Construye un nodo raíz: fila de datos + hijos colapsables.
  Widget _buildNode(TreeTableNode node) {
    final hasChildren = node.children.isNotEmpty;
    final isExp = _expanded[node.id] ?? false;

    return Column(
      children: [
        // Fila del nodo raíz: triángulo plegable si tiene hijos.
        _row(
          cells: [
            Expanded(
              child: Row(children: [
                // Triángulo que rota 90° al expandir: solo si el nodo tiene hijos.
                if (hasChildren)
                  AnimatedRotation(
                    turns: isExp ? 0.25 : 0,
                    duration: const Duration(milliseconds: 180),
                    child: Icon(
                      Gx.iconChevronDown,
                      size: 10,
                      color: Gx.textBaseSecondary,
                    ),
                  ),
                if (hasChildren) const SizedBox(width: 4),
                // Primera celda de datos del nodo (columna 0).
                Flexible(
                  child: node.cells.isNotEmpty ? node.cells.first : const SizedBox(),
                ),
              ]),
            ),
            // Celdas restantes (columnas 1..n), una por columna definida.
            ...node.cells.skip(1).map((c) => Expanded(child: c)),
          ],
          onTap: () {
            if (hasChildren) setState(() => _expanded[node.id] = !isExp);
            widget.onNodeTap?.call(node.id);
          },
        ),
        // Sub-nodos: aparecen con sangría cuando el nodo está expandido.
        AnimatedSize(
          duration: const Duration(milliseconds: 200),
          child: isExp && hasChildren
              ? Column(
                  children: node.children
                      .map(
                        (child) => _row(
                          cells: child.cells.map((c) => Expanded(child: c)).toList(),
                          isChild: true,
                          onTap: () => widget.onNodeTap?.call(child.id),
                        ),
                      )
                      .toList(),
                )
              : const SizedBox.shrink(),
        ),
      ],
    );
  }
}
