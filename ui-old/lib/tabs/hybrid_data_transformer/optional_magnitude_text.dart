// optional_magnitude_text.dart — Garantía visual central de
// hybrid-data-transformer (ADR-0155, STORY-051 criterio 14).
//
// El esquema distingue "el bróker no reporta esta magnitud" de "esta
// magnitud vale cero real": son hechos distintos y la interfaz NO puede
// colapsarlos. En Rust esa distinción es `Option<i64>` — `None` viaja a
// Dart como `null` (nunca como `0`, ver doc-comment de BarDto en
// hybrid_data_transformer.dart). Este widget es el único punto de la
// Cáscara Visual que renderiza una magnitud que puede faltar: si aquí se
// tratara `null` como `0`, se rompería la garantía que costó dos rondas de
// retrabajo construir en el Núcleo.

import 'package:flutter/material.dart';
import '../../theme/gx_tokens.dart';

// [scaledValue] entero escalado ×10⁸ (ADR-0141) o `null` si el bróker no
// distingue esta magnitud para el intervalo. `null` se muestra como texto
// muted en cursiva con guion — nunca como "0". Un `0` real (magnitud
// conocida y su valor es cero) se muestra como cualquier otro valor.
class OptionalMagnitudeText extends StatelessWidget {
  final int? scaledValue;
  final String absentLabel;

  OptionalMagnitudeText({
    super.key,
    required this.scaledValue,
    this.absentLabel = 'No reportado',
  });

  @override
  Widget build(BuildContext context) {
    if (scaledValue == null) {
      // Ausente: marca visual explícita, distinta de cualquier valor numérico.
      return Text(
        '— $absentLabel',
        style: Gx.dataMono(fontSize: 12, color: Gx.textBaseMuted)
            .copyWith(fontStyle: FontStyle.italic),
      );
    }
    // Presente — incluido el caso "vale cero real": texto normal, sin
    // ninguna marca de ausencia.
    return Text(
      formatScaledInt(scaledValue!),
      style: Gx.dataMono(fontSize: 12, color: Gx.textBase),
    );
  }
}

// Formatea un entero escalado ×10⁸ (ADR-0141) a texto decimal exacto.
// Manipulación de string pura — nunca coma flotante — así que no hay
// pérdida de precisión ni cálculo financiero: es formateo de presentación,
// igual que truncar un hash o convertir nanosegundos a milisegundos.
String formatScaledInt(int scaled) {
  final negative = scaled < 0;
  final digits = scaled.abs().toString().padLeft(9, '0');
  final integerPart = digits.substring(0, digits.length - 8);
  final fractionPart = digits.substring(digits.length - 8);
  return '${negative ? '-' : ''}$integerPart.$fractionPart';
}
