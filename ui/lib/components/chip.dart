// chip.dart — Componente Chip (ADR-0138 enmienda 2026-06-29).
// Chip/etiqueta con estado semántico de vitalidad y glow neón.
// El estilo lo decide el tema global vía tokens Gx; prohibido hardcodear color.
//
// Nota: el nombre Chip colisiona con el widget Material del mismo nombre.
// Los consumidores importan con namespace `import ... as ui;` → `ui.Chip`.

import 'package:flutter/material.dart' hide Chip;
import '../theme/gx_tokens.dart';

// Estado semántico del chip — mapea a los colores de vitalidad del sistema.
enum ChipStatus {
  optima,      // ÓPTIMO — cian
  transition,  // INCUBA / TRANSICIÓN — índigo
  alert,       // VOLÁTIL / ALERTA — ámbar
  critical,    // FALLO / CRÍTICO — carmesí
}

// Chip/etiqueta con glow neón encendido y fondo semántico de chip.
// Contrato funcional: [label] texto del chip; [status] estado semántico
// (null = chip neutro con borde estructural global); [pill] si true usa
// forma pill completa (radio 999), si false usa radio de chip estándar.
class Chip extends StatelessWidget {
  final String label;
  final ChipStatus? status;
  final bool pill;

  // No es const: lee getters dinámicos de Gx que cambian con el tema.
  Chip({super.key, required this.label, this.status, this.pill = false});

  // Color de texto y glow según el estado semántico.
  Color _fg() => switch (status) {
        ChipStatus.optima     => Gx.optimaCyan,
        ChipStatus.transition => Gx.transitionIndigo,
        ChipStatus.alert      => Gx.alertAmber,
        ChipStatus.critical   => Gx.criticalCrimson,
        null                  => Gx.textBaseLabel,
      };

  // Color de fondo del chip según el estado semántico.
  Color _bg() => switch (status) {
        ChipStatus.optima     => Gx.optimaChipBg,
        ChipStatus.transition => Gx.transitionChipBg,
        ChipStatus.alert      => Gx.alertChipBg,
        ChipStatus.critical   => Gx.criticalChipBg,
        null                  => Gx.surfaceCard,
      };

  // Color del borde del chip según el estado semántico.
  Color _border() => switch (status) {
        ChipStatus.optima     => Gx.optimaChipBorder,
        ChipStatus.transition => Gx.transitionChipBorder,
        ChipStatus.alert      => Gx.alertChipBorder,
        ChipStatus.critical   => Gx.criticalChipBorder,
        null                  => Gx.borderBase,
      };

  @override
  // Chip con fondo semántico, borde coloreado y glow neón del color de estado.
  // Sin estado semántico usa los tokens globales de borde y superficie de tarjeta.
  Widget build(BuildContext context) {
    final fg = _fg();
    final hasSemantic = status != null;
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
      decoration: BoxDecoration(
        color: _bg(),
        border: Border.all(color: _border()),
        borderRadius: BorderRadius.circular(pill ? 999 : Gx.rChip),
        // Glow solo cuando hay estado semántico — refuerza la señal visual.
        boxShadow: hasSemantic
            ? Gx.glow(fg, blur: 12, opacity: 0.30)
            : null,
      ),
      child: Text(
        label,
        style: Gx.uiSans(fontSize: 12, color: fg, height: 1.2).copyWith(
          // Sombra de texto neón "encendido" solo cuando hay estado semántico.
          shadows: hasSemantic ? Gx.textGlow(fg) : null,
        ),
      ),
    );
  }
}
