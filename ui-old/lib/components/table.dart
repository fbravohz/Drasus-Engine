// table.dart — Componente Table (ADR-0138 enmienda 2026-06-29).
// Tabla de datos con cabecera, separadores por token, hover de fila y columnas tipadas.
// El estilo lo decide el tema global vía tokens Gx; prohibido hardcodear color.
//
// Nota: el nombre Table colisiona con el widget Flutter del mismo nombre.
// Los consumidores importan con namespace `import ... as ui;` → `ui.Table`.

import 'package:flutter/material.dart' hide Table;
import '../theme/gx_tokens.dart';
import '../theme/surfaces.dart';

// Definición de columna para la tabla.
class TableColumn {
  final String label;      // texto del encabezado
  final bool numeric;      // si true, alinea el contenido a la derecha
  const TableColumn({required this.label, this.numeric = false});
}

// Tabla de datos con cabecera, hover de fila y separadores por token.
// Contrato funcional: [columns] lista de columnas con label y tipo;
// [rows] filas de widgets (cada lista interna tiene un widget por columna);
// [onRowTap] callback opcional al pulsar una fila (índice 0-based).
class Table extends StatefulWidget {
  final List<TableColumn> columns;
  final List<List<Widget>> rows;
  final ValueChanged<int>? onRowTap;

  // No es const: lee getters dinámicos de Gx que cambian con el tema.
  Table({
    super.key,
    required this.columns,
    required this.rows,
    this.onRowTap,
  });

  @override
  State<Table> createState() => _TableState();
}

class _TableState extends State<Table> {
  // Índice de la fila bajo el cursor (-1 = ninguna).
  int _hoverIndex = -1;

  // Construye la fila de cabecera con etiquetas de columna en microLabel.
  Widget _buildHeader() {
    return Container(
      padding: const EdgeInsets.symmetric(vertical: 7, horizontal: 8),
      decoration: BoxDecoration(
        border: Border(bottom: BorderSide(color: Gx.borderBase)),
      ),
      child: Row(
        children: widget.columns.map((col) {
          return Expanded(
            child: Text(
              col.label,
              textAlign: col.numeric ? TextAlign.right : TextAlign.left,
              style: Gx.microLabel,
            ),
          );
        }).toList(),
      ),
    );
  }

  // Construye una fila de datos con hover highlight y separador inferior.
  Widget _buildRow(int index, List<Widget> cells) {
    final isHover = _hoverIndex == index;
    return MouseRegion(
      onEnter: (_) => setState(() => _hoverIndex = index),
      onExit: (_) => setState(() => _hoverIndex = -1),
      cursor: widget.onRowTap != null
          ? SystemMouseCursors.click
          : SystemMouseCursors.basic,
      child: GestureDetector(
        onTap: widget.onRowTap != null ? () => widget.onRowTap!(index) : null,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 150),
          padding: const EdgeInsets.symmetric(vertical: 7, horizontal: 8),
          decoration: BoxDecoration(
            // Fondo de hover dinámico: token surfaceRaisedDynamic (se adapta a la paleta).
            color: isHover ? Gx.surfaceRaisedDynamic : Colors.transparent,
            border: Border(bottom: BorderSide(color: Gx.borderBase)),
          ),
          child: Row(
            children: widget.columns.asMap().entries.map((entry) {
              final col = entry.value;
              final cell = cells.length > entry.key
                  ? cells[entry.key]
                  : const SizedBox.shrink();
              return Expanded(
                child: Align(
                  alignment: col.numeric
                      ? Alignment.centerRight
                      : Alignment.centerLeft,
                  child: cell,
                ),
              );
            }).toList(),
          ),
        ),
      ),
    );
  }

  @override
  // Tabla con cabecera fija y filas con hover; todo envuelto en panelSurface.
  Widget build(BuildContext context) {
    return panelSurface(
      padding: EdgeInsets.zero,
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          _buildHeader(),
          ...widget.rows.asMap().entries.map((entry) {
            return _buildRow(entry.key, entry.value);
          }),
        ],
      ),
    );
  }
}
