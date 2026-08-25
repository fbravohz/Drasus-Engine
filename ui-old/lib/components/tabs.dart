// tabs.dart — Componente Tabs (ADR-0138 enmienda 2026-06-29).
// Barra de pestañas estilizada con tokens Gx + contenido de cada pestaña.
// Encapsula DefaultTabController + TabBar (estilizado) + TabBarView en un solo widget.
// Reemplaza el uso directo de Material TabBar/TabBarView + DefaultTabController.

import 'package:flutter/material.dart';
import '../theme/gx_tokens.dart';

// Definición de una pestaña: ícono opcional, etiqueta de texto y widget de contenido.
class TabItem {
  // icon: widget del ícono (por ejemplo Icon(Icons.clock)); null = solo texto.
  final Widget? icon;
  // label: texto de la etiqueta de la pestaña.
  final String label;
  // child: widget que se muestra cuando esta pestaña está activa.
  final Widget child;

  const TabItem({this.icon, required this.label, required this.child});
}

// Barra de pestañas + contenido en un único widget autocontenido.
// Gestiona su propio DefaultTabController internamente.
// Contrato funcional:
//   [tabs]          lista de definiciones de pestaña (TabItem).
//   [isScrollable]  si la barra de pestañas hace scroll horizontal cuando hay muchas; default true.
//   [onChanged]     callback con el índice de la pestaña activa al cambiar.
//   [initialIndex]  índice activo al construir el widget; default 0.
class Tabs extends StatelessWidget {
  final List<TabItem> tabs;
  final bool isScrollable;
  final ValueChanged<int>? onChanged;
  final int initialIndex;

  // No es const: el build lee getters dinámicos de Gx (accentDynamic, textBaseSecondary).
  Tabs({
    super.key,
    required this.tabs,
    this.isScrollable = true,
    this.onChanged,
    this.initialIndex = 0,
  });

  @override
  // Muestra la barra de pestañas estilizada (arriba) + el contenido de la activa (abajo).
  // DefaultTabController gestiona el índice activo; TabBarView anima el cambio.
  Widget build(BuildContext context) {
    return DefaultTabController(
      length: tabs.length,
      initialIndex: initialIndex,
      child: Column(
        children: [
          // Barra de pestañas con colores y marcador de Gx.
          _StyledTabBar(tabs: tabs, isScrollable: isScrollable, onChanged: onChanged),
          // Contenido de la pestaña activa; Expanded para llenar el espacio disponible.
          Expanded(
            child: TabBarView(
              children: tabs.map((t) => t.child).toList(),
            ),
          ),
        ],
      ),
    );
  }
}

// Barra de pestañas interna estilizada con tokens Gx.
// Usa Material TabBar con overrides de color y estilo tipográfico.
class _StyledTabBar extends StatelessWidget {
  final List<TabItem> tabs;
  final bool isScrollable;
  final ValueChanged<int>? onChanged;

  const _StyledTabBar({
    required this.tabs,
    required this.isScrollable,
    this.onChanged,
  });

  @override
  // Barra de pestañas Material con labelColor, indicatorColor y tipografía Gx.
  Widget build(BuildContext context) {
    return TabBar(
      isScrollable: isScrollable,
      onTap: onChanged,
      // Color de la pestaña activa: énfasis dinámico del tema global.
      labelColor: Gx.accentDynamic,
      // Color de pestañas inactivas: texto secundario base del tema global.
      unselectedLabelColor: Gx.textBaseSecondary,
      // Color del indicador inferior: igual que la pestaña activa.
      indicatorColor: Gx.accentDynamic,
      // Grosor del indicador: usa borderFocus (1.5px) — más discreto que el default de 2px.
      indicatorWeight: Gx.borderFocus,
      // El color en los estilos es sobrescrito por labelColor/unselectedLabelColor de arriba;
      // se pasa accentDynamic y textBaseSecondary para que el TextStyle sea tipográficamente correcto.
      labelStyle: Gx.uiSans(fontSize: 13, weight: FontWeight.w500, color: Gx.accentDynamic),
      unselectedLabelStyle: Gx.uiSans(fontSize: 13, color: Gx.textBaseSecondary),
      tabs: tabs
          .map((t) => Tab(
                icon: t.icon,
                text: t.label,
                iconMargin: const EdgeInsets.only(bottom: 2),
              ))
          .toList(),
    );
  }
}
