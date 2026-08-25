// bento_card.dart — Componente BentoCard (ADR-0138 enmienda 2026-06-29).
// Celda bento-grid con efecto glass/tint/solid según el modo global de superficie.
// Muestra ícono + título + contenido opcional (o texto placeholder).
// Migrado de GlassBentoCard (tabs/dashboard_tab.dart, Batch 4 STORY-025).
// Diferencia con GlassBentoCard: usa frosted() en lugar de BackdropFilter+Color hardcodeado.

import 'package:flutter/material.dart';
import '../theme/gx_tokens.dart';
import '../theme/surfaces.dart';

// Celda bento-grid con superficie reactiva al modo global de tema.
// Contrato funcional:
//   [icon]   ícono del área temática de la celda.
//   [title]  nombre de la celda (cabecera).
//   [height] altura total en píxeles lógicos; default: 200.
//   [child]  contenido personalizado; si es null muestra texto "Sin datos — próximamente".
class BentoCard extends StatelessWidget {
  final IconData icon;
  final String title;
  final double height;
  final Widget? child;

  // No es const: el build lee getters dinámicos de Gx (textBaseSecondary, textBase, textBaseMuted).
  BentoCard({
    super.key,
    required this.icon,
    required this.title,
    this.height = 200.0,
    this.child,
  });

  @override
  // Muestra una celda bento-grid de [height]px con frosted() reactivo al modo de tema global.
  Widget build(BuildContext context) {
    return SizedBox(
      height: height,
      child: frosted(
        radius: Gx.rPanel,
        // El espaciado interno lo aporta frosted vía padding; sin borde extra.
        padding: const EdgeInsets.all(Gx.space16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // Ícono temático de la celda: color secundario del texto base.
            Icon(icon, size: 22, color: Gx.textBaseSecondary),
            const SizedBox(height: Gx.space8),
            // Título de la celda con peso medio para destacar sobre el contenido.
            Text(
              title,
              style: Gx.uiSans(
                  fontSize: 14,
                  color: Gx.textBase,
                  weight: FontWeight.w500),
            ),
            const Spacer(),
            // Contenido real o texto placeholder si no se proporcionó widget.
            child ??
                Text(
                  'Sin datos — próximamente',
                  style: Gx.uiSans(fontSize: 11, color: Gx.textBaseMuted),
                ),
          ],
        ),
      ),
    );
  }
}
