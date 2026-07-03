// Sección §6 Inputs extendidos — componentes migrados a components/ en Batch 2.
// Este archivo conserva únicamente richTextEditorPlaceholder() (placeholder estático)
// que no tiene clase equivalente en la librería. Las clases Glow* fueron migradas.

import 'package:flutter/material.dart';
import '../gallery_tokens.dart';
import '../gallery_fx.dart';

// ---------------------------------------------------------------------------
// Rich Text Editor — placeholder de área de edición enriquecida
// ---------------------------------------------------------------------------

// Vitrina estática del editor enriquecido: barra de formato + área de texto.
// Es render-only (placeholder); no implementa edición real, que requeriría
// paquete dedicado fuera del scope de la Cáscara Delgada.
Widget richTextEditorPlaceholder() {
  return Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      // Barra de formato simplificada.
      panelSurface(
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
        radius: Gx.rPanel,
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            _fmtBtn('B'),
            _fmtBtn('I'),
            _fmtBtn('U'),
            Container(width: 1, height: 14, color: Gx.divider,
                margin: const EdgeInsets.symmetric(horizontal: 6)),
            _fmtBtn('H1'),
            _fmtBtn('H2'),
          ],
        ),
      ),
      const SizedBox(height: 6),
      // Área de contenido editable (placeholder estático).
      cardSurface(
        padding: const EdgeInsets.all(12),
        child: Text(
          'Notas de la estrategia node-07…',
          style: Gx.uiSans(fontSize: 13, color: Gx.textBaseMuted),
        ),
      ),
    ],
  );
}

// Botón de formato individual de la barra del editor.
Widget _fmtBtn(String label) => Container(
      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 3),
      margin: const EdgeInsets.symmetric(horizontal: 2),
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(Gx.rChip),
        color: label == 'B' ? Gx.transitionIndigo.withAlpha(40) : Colors.transparent,
      ),
      child: Text(label,
          style: Gx.dataMono(
              fontSize: 11,
              color: label == 'B' ? Gx.transitionIndigo : Gx.textBaseLabel)),
    );

// GlowFormField migrado a ui/lib/components/form_field.dart (Batch 2, STORY-025).
