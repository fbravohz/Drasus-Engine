// OdometerNumber — primitivo de odómetro numérico (ADR-0138).
// Anima un double desde 0.0 hasta [value] con Curves.easeOutCubic y
// formatea el valor interpolado cada frame. Lee la duración por defecto
// de DrasusMotion.odometerMs vía Theme.of(context).
// Extraído de _QuantKpiOdometer en section_dataviz_quant.dart.

import 'package:flutter/material.dart';
import '../theme/drasus_tokens.dart';

/// Firma del formateador: recibe el double interpolado y retorna el texto.
typedef OdometerFormatter = String Function(double value);

/// Odómetro que anima de 0.0 a [value] al montarse.
class OdometerNumber extends StatelessWidget {
  /// Valor destino de la cuenta.
  final double value;

  /// Formateador del valor interpolado. Si es null se usa toStringAsFixed
  /// con [decimals].
  final OdometerFormatter? format;

  /// Decimales usados por el formateador por defecto.
  final int decimals;

  /// Aplica separador de miles con punto (convención es-ES) cuando el valor
  /// es entero y grande. Solo se usa si [format] es null.
  final bool thousands;

  /// Sufijo fijo tras el número (unidad, signo de porcentaje).
  final String suffix;

  /// Duración de la animación. Default: DrasusMotion.odometerMs.
  final Duration? duration;

  /// Estilo del texto del número.
  final TextStyle style;

  const OdometerNumber({
    super.key,
    required this.value,
    this.format,
    this.decimals = 0,
    this.thousands = false,
    this.suffix = '',
    this.duration,
    required this.style,
  });

  // Formateo por defecto (mismo algoritmo que _QuantKpiOdometer._format).
  String _defaultFormat(double v) {
    if (decimals == 0) {
      final n = v.toInt();
      if (thousands && n >= 10000) {
        final s = n.toString();
        final buf = StringBuffer();
        var count = 0;
        for (var i = s.length - 1; i >= 0; i--) {
          if (count > 0 && count % 3 == 0) buf.write('.');
          buf.write(s[i]);
          count++;
        }
        return buf.toString().split('').reversed.join();
      }
      return n.toString();
    }
    return v.toStringAsFixed(decimals);
  }

  @override
  Widget build(BuildContext context) {
    final motion =
        Theme.of(context).extension<DrasusMotion>() ?? DrasusMotion.defaults;
    final dur = duration ?? Duration(milliseconds: motion.odometerMs);
    return TweenAnimationBuilder<double>(
      tween: Tween(begin: 0.0, end: value),
      duration: dur,
      curve: Curves.easeOutCubic,
      builder: (_, double v, __) {
        final text = format != null ? format!(v) : _defaultFormat(v);
        return Text('$text$suffix', style: style);
      },
    );
  }
}
