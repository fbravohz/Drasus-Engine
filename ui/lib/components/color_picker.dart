// color_picker.dart — Componente ColorPicker (ADR-0138 enmienda 2026-06-29).
//
// Selector de color híbrido que combina:
//   1. Swatches curados — una fila de chips de colores predefinidos.
//   2. Rueda HSV opcional — disco interactivo tono/saturación + deslizador brillo.
//
// Consolida GlowColorPicker (section_std_missing.dart) y ColorPickerWidget
// (widgets/color_picker.dart) en una única implementación canónica.
// Los consumidores anteriores de ColorPickerWidget deben migrar a ui.ColorPicker.
//
// Implementado con CustomPainter nativo; sin dependencias externas.
// Los colores del disco HSV son literales del espacio matemático del modelo HSV,
// no tokens de UI del sistema de diseño.

import 'dart:math';
import 'package:flutter/material.dart';
import '../theme/gx_tokens.dart';
import '../theme/surfaces.dart';

// Paleta semántica por defecto: espectro de vitalidad del sistema de diseño.
// Se usa cuando el consumidor no pasa swatches propios.
const List<Color> _kDefaultSwatches = [
  Gx.optimaCyan,
  Gx.optimaTeal,
  Gx.reactorGreen,
  Gx.transitionIndigo,
  Gx.transitionBlue,
  Gx.transitionPurple,
  Gx.alertAmber,
  Gx.alertOrange,
  Gx.criticalRed,
  Gx.criticalCrimson,
];

// Gradiente de tono para el disco HSV: 13 paradas de 0° a 360° en el espectro.
// Estos son colores matemáticos del espacio HSV, no tokens de UI del sistema.
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
// ColorPicker — selector de color híbrido principal.
// ---------------------------------------------------------------------------

/// Selector de color con swatches curados + rueda HSV expandible opcional.
///
/// Modo no controlado: [value] null; el widget gestiona internamente el color.
/// Modo controlado:    pasa [value] y recibe cambios en [onChanged].
///
/// [swatches] lista de colores predefinidos a mostrar como chips.
///            Si es null usa la paleta semántica del sistema de vitalidad.
/// [onChanged] se llama con el Color seleccionado al tocar un swatch o la rueda.
class ColorPicker extends StatefulWidget {
  // value: color actualmente seleccionado; null = modo no controlado.
  final Color? value;
  // onChanged: se llama con el nuevo color al seleccionarlo.
  final ValueChanged<Color>? onChanged;
  // swatches: lista de colores predefinidos; null usa la paleta semántica.
  final List<Color>? swatches;

  const ColorPicker({
    super.key,
    this.value,
    this.onChanged,
    this.swatches,
  });

  @override
  State<ColorPicker> createState() => _ColorPickerState();
}

class _ColorPickerState extends State<ColorPicker> {
  // Color activo internamente: en modo no controlado empieza en el primer swatch.
  late Color _color;

  // Componentes HSV del color activo en la rueda.
  late double _h; // tono 0–360
  late double _s; // saturación 0–1
  late double _v; // brillo 0–1

  // Controla si la sección de la rueda HSV está expandida.
  bool _showWheel = false;

  // Lista de swatches a mostrar.
  List<Color> get _swatches => widget.swatches ?? _kDefaultSwatches;

  // Convierte un Color a sus componentes HSV internos.
  void _fromColor(Color c) {
    final hsv = HSVColor.fromColor(c);
    _h = hsv.hue;
    _s = hsv.saturation;
    _v = hsv.value;
  }

  // Construye el Color actual desde los componentes HSV internos.
  Color get _currentHsvColor => HSVColor.fromAHSV(1.0, _h, _s, _v).toColor();

  @override
  void initState() {
    super.initState();
    // En modo controlado usa widget.value; en no controlado usa el primer swatch.
    _color = widget.value ?? _swatches.first;
    _fromColor(_color);
  }

  @override
  void didUpdateWidget(ColorPicker old) {
    super.didUpdateWidget(old);
    // Modo controlado: sincroniza el color interno cuando el padre lo cambia.
    if (widget.value != null && widget.value != _color) {
      _color = widget.value!;
      _fromColor(_color);
    }
  }

  // Selecciona un color y notifica al padre.
  void _pick(Color c) {
    _fromColor(c);
    setState(() => _color = c);
    widget.onChanged?.call(c);
  }

  @override
  // Muestra swatches + toggle de la rueda + rueda HSV expandible + franja de preview.
  Widget build(BuildContext context) {
    return panelSurface(
      padding: const EdgeInsets.all(Gx.space12),
      glow: Gx.glow(_color, blur: 12, opacity: 0.2),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          // Fila de swatches curados + botón de toggle de la rueda HSV.
          Wrap(
            spacing: 8,
            runSpacing: 8,
            children: [
              ..._swatches.map((color) {
                // El chip se marca como seleccionado si coincide con el color activo.
                final isSel = color.toARGB32() == _color.toARGB32();
                return _SwatchChip(
                  color: color,
                  isSelected: isSel,
                  size: _kSwatchSize,
                  onTap: () => _pick(color),
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
                          _color = _currentHsvColor;
                        });
                        widget.onChanged?.call(_currentHsvColor);
                      },
                    ),
                  )
                : const SizedBox.shrink(),
          ),

          const SizedBox(height: 10),

          // Franja de preview: muestra el color actualmente seleccionado.
          Container(
            height: 4,
            width: double.infinity,
            decoration: BoxDecoration(
              color: _color,
              borderRadius: BorderRadius.circular(2),
            ),
          ),
        ],
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// _SwatchChip — chip cuadrado de un color predefinido.
// ---------------------------------------------------------------------------

// Muestra borde blanco + glow cuando está seleccionado; borde transparente en reposo.
// Gx.pureWhite es el token canónico para blanco puro — necesario para contraste
// sobre cualquier color semántico del espectro de vitalidad.
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
  // Chip cuadrado con borde de selección blanco y glow cuando está activo.
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: onTap,
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 180),
        width: size,
        height: size,
        decoration: BoxDecoration(
          color: color,
          // Radio decorativo 6px para chip de swatch 26×26px.
          borderRadius: BorderRadius.circular(6),
          border: Border.all(
            // Borde blanco puro para máximo contraste con cualquier color semántico.
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
// _WheelToggle — botón de apertura/cierre de la rueda HSV.
// ---------------------------------------------------------------------------

// Icono paleta (cerrado) o X (abierto). Fondo de énfasis dinámico cuando está abierto.
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
  // Toggle con fondo y borde del énfasis dinámico cuando está abierto.
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: onTap,
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 180),
        width: size,
        height: size,
        decoration: BoxDecoration(
          color: isOpen
              ? Gx.accentDynamic.withAlpha(77)
              : Gx.gaugeTrack,
          borderRadius: BorderRadius.circular(6),
          border: Border.all(
            color: isOpen
                ? Gx.accentDynamic.withAlpha(179)
                : Gx.borderPanel,
          ),
        ),
        alignment: Alignment.center,
        child: Icon(
          isOpen ? Icons.close : Icons.palette_outlined,
          size: 14,
          color: isOpen ? Gx.accentDynamic : Gx.textBaseMuted,
        ),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// _HsvSection — sección con disco de tono/saturación + deslizador de brillo.
// ---------------------------------------------------------------------------

// Retorna los tres componentes (h, s, v) al cambiar cualquiera.
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
  // Fila con el disco HSV (izquierda) + deslizador de brillo y preview (derecha).
  Widget build(BuildContext context) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // Disco de tono/saturación interactivo con CustomPainter nativo.
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
              Text('Brillo',
                  style: Gx.uiSans(fontSize: 11, color: Gx.textBaseLabel)),
              const SizedBox(height: 8),
              // Deslizador del componente V (brillo) del modelo HSV.
              _ValueSlider(
                value: val,
                hue: hue,
                sat: sat,
                onChanged: (v) => onChanged(hue, sat, v),
              ),
              const SizedBox(height: 12),
              // Caja de preview del color resultante de la rueda HSV.
              Container(
                height: 32,
                decoration: BoxDecoration(
                  color: HSVColor.fromAHSV(1.0, hue, sat, val).toColor(),
                  // Radio decorativo 6px para caja de preview.
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

// Tres capas superpuestas sobre el disco circular:
//   1. SweepGradient (tono por ángulo, 0°–360°).
//   2. RadialGradient blanco → transparente (saturación por radio; S=0 en centro).
//   3. Overlay negro opaco según 1-val (brillo; negro total cuando val=0).
//   4. Indicador circular blanco en la posición (hue, sat) actual.
//
// Colors.white y Colors.black aquí son constantes del espacio HSV (saturación
// y brillo respectivamente), NO tokens de UI del sistema de diseño.
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

    // Capa 1: tono por ángulo (SweepGradient de 0° a 360°).
    canvas.drawCircle(
      center,
      radius,
      Paint()
        ..shader = SweepGradient(colors: _kHueGradient).createShader(rect),
    );

    // Capa 2: saturación por radio.
    // Blanco en el centro (S=0) → transparente en el borde (S=1, tono puro visible).
    // Colors.white.withAlpha(0) = blanco totalmente transparente (constante HSV, no UI).
    canvas.drawCircle(
      center,
      radius,
      Paint()
        ..shader = RadialGradient(
          colors: [Colors.white, Colors.white.withAlpha(0)],
        ).createShader(rect),
    );

    // Capa 3: overlay de brillo.
    // Negro opaco según 1-val: negro total cuando val=0, invisible cuando val=1.
    // Colors.black es el negro del espacio HSV, no un token de UI.
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
    // Sombra oscura exterior para visibilidad sobre colores claros del disco.
    canvas.drawCircle(
      selOffset,
      8,
      Paint()
        ..color = Colors.black.withOpacity(0.5)
        ..style = PaintingStyle.stroke
        ..strokeWidth = 3,
    );
    // Anillo blanco interior del indicador.
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
  // Repinta solo cuando cambia tono, saturación o brillo.
  bool shouldRepaint(_HsvDiscPainter old) =>
      old.hue != hue || old.sat != sat || old.val != val;
}

// ---------------------------------------------------------------------------
// _ValueSlider — deslizador horizontal del componente V (brillo) del HSV.
// ---------------------------------------------------------------------------

// El track muestra un gradiente de negro (izquierda) al color de máximo brillo
// (derecha). El thumb es un círculo blanco.
// Colors.black es el negro del espacio matemático HSV, no un token de UI.
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
    // Color puro del tono/saturación actuales al máximo brillo: extremo derecho.
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
            // Track: degradado de negro (brillo 0) a fullColor (brillo máximo).
            Container(
              height: 8,
              decoration: BoxDecoration(
                gradient: LinearGradient(colors: [Colors.black, fullColor]),
                borderRadius: BorderRadius.circular(4),
              ),
            ),
            // Thumb: círculo blanco en la posición del valor actual.
            // El clamp evita que el thumb salga del área visible del track.
            Positioned(
              left: (value * (w - 16)).clamp(0, w - 16),
              child: Container(
                width: 16,
                height: 16,
                decoration: BoxDecoration(
                  shape: BoxShape.circle,
                  // Gx.pureWhite = blanco del sistema (contraste garantizado).
                  color: Gx.pureWhite,
                  border: Border.all(color: Gx.borderPanel),
                  boxShadow: [
                    BoxShadow(
                      // Sombra oscura del thumb: constante de contraste visual, no UI.
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
