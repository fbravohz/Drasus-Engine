// Selector de color híbrido reutilizable del panel de configuración.
//
// Combina dos modos de selección:
//   1. Swatches curados — una fila de chips de colores predefinidos que
//      respetan el sistema de tokens (semánticos, UI y de énfasis).
//   2. Rueda HSV — disco interactivo de tono/saturación + deslizador de
//      brillo. Implementado con CustomPainter nativo (sin dependencias externas).
//
// Úsalo de forma uniforme en TODOS los controles de color del panel
// (énfasis, color de fuente, y futuros). Prohibido selectores ad-hoc.

import 'dart:math';
import 'package:flutter/material.dart';
import '../gallery/gallery_tokens.dart';

// Colores del SweepGradient para el eje de tono del disco HSV.
// Cubren el espectro completo (0°–360°) con 13 paradas uniformes.
const List<Color> _kHueGradient = [
  Color(0xFFFF0000), // 0°   rojo
  Color(0xFFFF7F00), // 30°  naranja
  Color(0xFFFFFF00), // 60°  amarillo
  Color(0xFF7FFF00), // 90°  amarillo-verde
  Color(0xFF00FF00), // 120° verde
  Color(0xFF00FF7F), // 150° verde-cian
  Color(0xFF00FFFF), // 180° cian
  Color(0xFF007FFF), // 210° azul-cian
  Color(0xFF0000FF), // 240° azul
  Color(0xFF7F00FF), // 270° violeta
  Color(0xFFFF00FF), // 300° magenta
  Color(0xFFFF007F), // 330° rosa
  Color(0xFFFF0000), // 360° rojo (cierre del ciclo)
];

// Diámetro del disco HSV en píxeles lógicos.
const double _kDiscSize = 160.0;

// Tamaño de cada chip de swatch en píxeles lógicos.
const double _kSwatchSize = 26.0;

// ---------------------------------------------------------------------------
// ColorPickerWidget — selector híbrido principal.
// ---------------------------------------------------------------------------

/// Selector de color que combina swatches curados y rueda HSV.
///
/// Params:
/// - [swatches]: lista de colores predefinidos a mostrar.
/// - [selectedColor]: color actualmente activo (viene del padre).
/// - [onColorChanged]: callback disparado al elegir un color (swatch o rueda).
class ColorPickerWidget extends StatefulWidget {
  final List<Color> swatches;
  final Color selectedColor;
  final ValueChanged<Color> onColorChanged;

  const ColorPickerWidget({
    super.key,
    required this.swatches,
    required this.selectedColor,
    required this.onColorChanged,
  });

  @override
  State<ColorPickerWidget> createState() => _ColorPickerWidgetState();
}

class _ColorPickerWidgetState extends State<ColorPickerWidget> {
  // Componentes HSV del color activo en la rueda.
  late double _h; // tono 0–360
  late double _s; // saturación 0–1
  late double _v; // brillo 0–1

  // Controla si la sección de la rueda HSV está expandida.
  bool _showWheel = false;

  // Convierte un Color a los componentes HSV internos.
  void _fromColor(Color c) {
    final hsv = HSVColor.fromColor(c);
    _h = hsv.hue;
    _s = hsv.saturation;
    _v = hsv.value;
  }

  // Construye el Color actual desde los componentes HSV internos.
  Color get _currentColor => HSVColor.fromAHSV(1.0, _h, _s, _v).toColor();

  @override
  void initState() {
    super.initState();
    // Sincronizar con el color externo al montar el widget.
    _fromColor(widget.selectedColor);
  }

  @override
  // Muestra swatches + toggle + rueda HSV expandible + franja de preview.
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [
        // Fila de swatches curados + botón de toggle de la rueda.
        Wrap(
          spacing: 8,
          runSpacing: 8,
          children: [
            ...widget.swatches.map((color) {
              // El chip se marca como seleccionado si coincide con el color externo.
              final isSel =
                  color.toARGB32() == widget.selectedColor.toARGB32();
              return _SwatchChip(
                color: color,
                isSelected: isSel,
                size: _kSwatchSize,
                onTap: () {
                  // Al elegir un swatch: sincroniza estado interno y notifica al padre.
                  _fromColor(color);
                  widget.onColorChanged(color);
                },
              );
            }),
            // Botón para abrir/cerrar la rueda HSV personalizada.
            _WheelToggle(
              isOpen: _showWheel,
              size: _kSwatchSize,
              onTap: () => setState(() => _showWheel = !_showWheel),
            ),
          ],
        ),

        // Sección de la rueda HSV: se expande/contrae con animación suave.
        AnimatedSize(
          duration: const Duration(milliseconds: 240),
          curve: Curves.easeOut,
          child: _showWheel
              ? Padding(
                  padding: const EdgeInsets.only(top: 14),
                  child: _HsvSection(
                    hue: _h,
                    sat: _s,
                    val: _v,
                    // Al cambiar en la rueda: actualiza estado interno y notifica.
                    onChanged: (h, s, v) {
                      setState(() {
                        _h = h;
                        _s = s;
                        _v = v;
                      });
                      widget.onColorChanged(_currentColor);
                    },
                  ),
                )
              : const SizedBox.shrink(),
        ),

        const SizedBox(height: 10),

        // Franja de preview: muestra el color actualmente seleccionado (desde el padre).
        Container(
          height: 4,
          width: double.infinity,
          decoration: BoxDecoration(
            color: widget.selectedColor,
            borderRadius: BorderRadius.circular(2),
          ),
        ),
      ],
    );
  }
}

// ---------------------------------------------------------------------------
// _SwatchChip — chip cuadrado de un color predefinido.
// ---------------------------------------------------------------------------

// Muestra borde blanco + glow cuando está seleccionado; transparente en reposo.
class _SwatchChip extends StatelessWidget {
  final Color color;
  final bool isSelected;
  final double size;
  final VoidCallback onTap;

  const _SwatchChip({
    required this.color,
    required this.isSelected,
    required this.size,
    required this.onTap,
  });

  @override
  // Chip cuadrado con borde de selección blanco y glow al estar activo.
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: onTap,
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 180),
        width: size,
        height: size,
        decoration: BoxDecoration(
          color: color,
          borderRadius: BorderRadius.circular(6),
          border: Border.all(
            // Gx.pureWhite como borde activo (literal físico, no color de UI).
            color: isSelected ? Gx.pureWhite : Colors.transparent,
            width: 2,
          ),
          boxShadow: isSelected ? Gx.glow(color, blur: 12, opacity: 0.7) : null,
        ),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// _WheelToggle — botón para abrir/cerrar la rueda HSV personalizada.
// ---------------------------------------------------------------------------

// Muestra un ícono de paleta (cerrado) o X (abierto).
class _WheelToggle extends StatelessWidget {
  final bool isOpen;
  final double size;
  final VoidCallback onTap;

  const _WheelToggle({
    required this.isOpen,
    required this.size,
    required this.onTap,
  });

  @override
  // Toggle con fondo de énfasis cuando está abierto; fondo neutro cuando está cerrado.
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: onTap,
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 180),
        width: size,
        height: size,
        decoration: BoxDecoration(
          color: isOpen
              ? Gx.transitionIndigo.withOpacity(0.3)
              : Gx.gaugeTrack,
          borderRadius: BorderRadius.circular(6),
          border: Border.all(
            color: isOpen
                ? Gx.transitionIndigo.withOpacity(0.7)
                : Gx.borderPanel,
          ),
        ),
        alignment: Alignment.center,
        child: Icon(
          isOpen ? Icons.close : Icons.palette_outlined,
          size: 14,
          color: isOpen ? Gx.transitionIndigo : Gx.textBaseMuted,
        ),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// _HsvSection — sección interna con disco HSV + deslizador de brillo.
// ---------------------------------------------------------------------------

// Devuelve los tres componentes (h, s, v) al cambiar cualquiera.
class _HsvSection extends StatelessWidget {
  final double hue; // 0–360
  final double sat; // 0–1
  final double val; // 0–1
  final void Function(double h, double s, double v) onChanged;

  const _HsvSection({
    required this.hue,
    required this.sat,
    required this.val,
    required this.onChanged,
  });

  // Convierte coordenadas locales del disco a (hue, sat) y notifica al padre.
  void _handleDisc(Offset localPos) {
    const half = _kDiscSize / 2;
    final dx = localPos.dx - half;
    final dy = localPos.dy - half;
    final dist = sqrt(dx * dx + dy * dy);
    // Tolerancia de 8px en el borde del disco para gestos imprecisos.
    if (dist > half + 8) return;
    final angle = (atan2(dy, dx) * 180 / pi + 360) % 360;
    final newSat = (dist / half).clamp(0.0, 1.0);
    onChanged(angle, newSat, val);
  }

  @override
  // Fila con el disco HSV a la izquierda y el deslizador de brillo + preview a la derecha.
  Widget build(BuildContext context) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // Disco de tono/saturación interactivo.
        SizedBox(
          width: _kDiscSize,
          height: _kDiscSize,
          child: GestureDetector(
            onPanDown: (d) => _handleDisc(d.localPosition),
            onPanUpdate: (d) => _handleDisc(d.localPosition),
            child: CustomPaint(
              painter: _HsvDiscPainter(hue: hue, sat: sat, val: val),
              size: const Size(_kDiscSize, _kDiscSize),
            ),
          ),
        ),
        const SizedBox(width: 12),

        // Columna derecha: etiqueta + deslizador de brillo + caja de preview.
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text('Brillo', style: Gx.uiSans(fontSize: 11, color: Gx.textBaseLabel)),
              const SizedBox(height: 8),

              // Deslizador de brillo (componente V del modelo HSV).
              _ValueSlider(
                value: val,
                hue: hue,
                sat: sat,
                onChanged: (v) => onChanged(hue, sat, v),
              ),
              const SizedBox(height: 12),

              // Caja de preview del color resultante en la rueda.
              Container(
                height: 32,
                decoration: BoxDecoration(
                  color: HSVColor.fromAHSV(1.0, hue, sat, val).toColor(),
                  borderRadius: BorderRadius.circular(6),
                  border: Border.all(color: Gx.borderPanel),
                ),
              ),
            ],
          ),
        ),
      ],
    );
  }
}

// ---------------------------------------------------------------------------
// _HsvDiscPainter — pinta el disco de tono y saturación con overlay de brillo.
// ---------------------------------------------------------------------------

// Tres capas superpuestas:
//   1. SweepGradient (tono por ángulo, 0°–360°).
//   2. RadialGradient blanco → transparente (saturación por radio; 0 en centro).
//   3. Overlay negro opaco según 1-val (brillo; negro total cuando val=0).
//   4. Indicador circular blanco en la posición (hue, sat) actual.
class _HsvDiscPainter extends CustomPainter {
  final double hue; // 0–360
  final double sat; // 0–1
  final double val; // 0–1

  const _HsvDiscPainter({
    required this.hue,
    required this.sat,
    required this.val,
  });

  @override
  void paint(Canvas canvas, Size size) {
    final center = Offset(size.width / 2, size.height / 2);
    final radius = size.shortestSide / 2;
    final rect = Rect.fromCircle(center: center, radius: radius);

    // Capa 1: tono por ángulo (SweepGradient).
    canvas.drawCircle(
      center,
      radius,
      Paint()..shader = SweepGradient(colors: _kHueGradient).createShader(rect),
    );

    // Capa 2: saturación por radio.
    // El gradiente blanco → transparente superpone blanco en el centro (S=0)
    // y lo desvanece hacia el borde (S=1), dejando el tono puro visible.
    // Colors.white y Colors.white.withAlpha(0) son literales del espacio HSV,
    // no colores de UI del sistema de tokens.
    canvas.drawCircle(
      center,
      radius,
      Paint()
        ..shader = RadialGradient(
          colors: [Colors.white, Colors.white.withAlpha(0)],
        ).createShader(rect),
    );

    // Capa 3: overlay de brillo (Colors.black es literal del espacio HSV).
    if (val < 1.0) {
      canvas.drawCircle(
        center,
        radius,
        Paint()..color = Colors.black.withOpacity(1.0 - val),
      );
    }

    // Indicador de posición actual: anillo blanco con sombra oscura exterior.
    final angle = hue * pi / 180.0;
    final selRadius = sat * radius;
    final selOffset = Offset(
      center.dx + selRadius * cos(angle),
      center.dy + selRadius * sin(angle),
    );
    // Sombra oscura para visibilidad sobre colores claros.
    canvas.drawCircle(
      selOffset,
      8,
      Paint()
        ..color = Colors.black.withOpacity(0.5)
        ..style = PaintingStyle.stroke
        ..strokeWidth = 3,
    );
    // Anillo blanco interior.
    canvas.drawCircle(
      selOffset,
      8,
      Paint()
        ..color = Colors.white
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2,
    );
  }

  @override
  // Solo repinta cuando cambia tono, saturación o brillo.
  bool shouldRepaint(_HsvDiscPainter old) =>
      old.hue != hue || old.sat != sat || old.val != val;
}

// ---------------------------------------------------------------------------
// _ValueSlider — deslizador horizontal de brillo (componente V del HSV).
// ---------------------------------------------------------------------------

// El track muestra un gradiente de negro (izquierda) al color de máximo brillo
// del tono/saturación actuales (derecha). El thumb es un círculo blanco.
class _ValueSlider extends StatelessWidget {
  final double value; // 0–1
  final double hue;   // 0–360 (para calcular el extremo derecho del track)
  final double sat;   // 0–1
  final ValueChanged<double> onChanged;

  const _ValueSlider({
    required this.value,
    required this.hue,
    required this.sat,
    required this.onChanged,
  });

  @override
  // Slider de brillo con track degradado y thumb circular. Sin estado propio.
  Widget build(BuildContext context) {
    // Color puro del tono/saturación actuales al brillo máximo: extremo derecho del track.
    final fullColor = HSVColor.fromAHSV(1.0, hue, sat, 1.0).toColor();
    return LayoutBuilder(builder: (_, box) {
      final w = box.maxWidth;
      return GestureDetector(
        onPanDown: (d) =>
            onChanged((d.localPosition.dx / w).clamp(0.0, 1.0)),
        onPanUpdate: (d) =>
            onChanged((d.localPosition.dx / w).clamp(0.0, 1.0)),
        child: SizedBox(
          height: 24,
          child: Stack(alignment: Alignment.centerLeft, children: [
            // Track: degradado de negro (Colors.black literal de espacio HSV) a fullColor.
            Container(
              height: 8,
              decoration: BoxDecoration(
                gradient: LinearGradient(colors: [Colors.black, fullColor]),
                borderRadius: BorderRadius.circular(4),
              ),
            ),
            // Thumb: círculo blanco en la posición del valor actual.
            // El clamp evita que el thumb salga del área del track.
            Positioned(
              left: (value * (w - 16)).clamp(0, w - 16),
              child: Container(
                width: 16,
                height: 16,
                decoration: BoxDecoration(
                  shape: BoxShape.circle,
                  color: Gx.pureWhite,
                  border: Border.all(color: Gx.borderPanel),
                  boxShadow: [
                    BoxShadow(
                      color: Colors.black.withOpacity(0.4),
                      blurRadius: 4,
                    ),
                  ],
                ),
              ),
            ),
          ]),
        ),
      );
    });
  }
}
