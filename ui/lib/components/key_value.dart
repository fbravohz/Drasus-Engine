// key_value.dart — Componente KeyValue (ADR-0138 enmienda 2026-06-29).
// Fila etiqueta → valor usada en paneles de datos, SVF y dashboards.
// El estilo lo decide el tema global vía tokens Gx; prohibido hardcodear color.

import 'package:flutter/material.dart';
import '../theme/gx_tokens.dart';
import 'copy_button.dart';

// Fila etiqueta → valor con separador inferior por token de borde.
// Contrato funcional: [label] texto descriptivo de la clave; [value] texto
// del valor a mostrar; [valueColor] color semántico del valor (null = usa
// el color de texto base dinámico — legible en todos los temas); [mono] si true
// renderiza el valor en tipografía monoespaciada (útil para números/hashes);
// [copyable] hace el valor seleccionable y añade un botón de copiado.
//
// Sobre [copyable] (2026-08-01): la etiqueta NUNCA es seleccionable —no sirve
// de nada fuera de la app y seleccionarla al arrastrar es ruido—, pero el
// VALOR sí puede necesitarse fuera: una huella SHA-256, un identificador de
// declaración, una ruta. Esos son justo los valores que nadie transcribe a
// mano sin equivocarse. Default `false`: un valor corto y legible no lo
// necesita.
class KeyValue extends StatelessWidget {
  final String label;
  final String value;
  final Color? valueColor;
  final bool mono;
  final bool copyable;

  // NO const: lee tokens dinámicos (Gx.textBaseLabel/borderBase) en build().
  // Un StatelessWidget const no se reconstruye al cambiar paleta/modo y
  // congelaría esos colores (antipatrón ADR-0138). Igual que el resto de
  // componentes Stateless que consumen el tema.
  KeyValue({
    super.key,
    required this.label,
    required this.value,
    this.valueColor,
    this.mono = false,
    this.copyable = false,
  });

  // Estilo del valor, compartido por la variante copiable y la normal para
  // que activar [copyable] no cambie cómo se ve el dato, solo qué se puede
  // hacer con él.
  TextStyle _valueStyle(Color vc) => (mono
          ? Gx.dataMono(fontSize: 13, color: vc)
          : Gx.uiSans(fontSize: 13, color: vc))
      .copyWith(shadows: valueColor != null ? Gx.textGlow(vc, 6) : null);

  @override
  // Fila con etiqueta en label muted a la izquierda y valor a la derecha.
  // Separador inferior via token borderBase; el valor usa color semántico si se provee.
  Widget build(BuildContext context) {
    final vc = valueColor ?? Gx.textBase;
    return Container(
      padding: const EdgeInsets.symmetric(vertical: 6),
      decoration: BoxDecoration(
        border: Border(bottom: BorderSide(color: Gx.borderBase)),
      ),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          // Etiqueta: texto descriptivo en color label (55% del texto base).
          Flexible(
            child: Text(
              label,
              overflow: TextOverflow.ellipsis,
              style: Gx.uiSans(fontSize: 13, color: Gx.textBaseLabel),
            ),
          ),
          const SizedBox(width: 8),
          // Valor: puede ser monoespaciado (datos numéricos) o sans (texto).
          // El glow solo aparece cuando hay un color semántico explícito.
          if (copyable)
            // Seleccionable + botón de copiado. `Flexible` porque un valor
            // copiable suele ser largo (hash, ruta) y debe poder envolver en
            // vez de desbordar la fila.
            Flexible(
              child: CopyableText(
                text: value,
                style: _valueStyle(vc),
                tooltip: 'Copiar $label',
              ),
            )
          else
            Text(value, style: _valueStyle(vc)),
        ],
      ),
    );
  }
}
