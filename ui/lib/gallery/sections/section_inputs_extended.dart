// Sección §6 Inputs extendidos — componentes de formulario faltantes.
// Render-only con estado de UI local y animación. Sin lógica de negocio ni FFI.

import 'package:flutter/material.dart';
import '../gallery_tokens.dart';
import '../gallery_fx.dart';

// ---------------------------------------------------------------------------
// Combobox / Autocomplete — input con sugerencias filtradas
// ---------------------------------------------------------------------------

// Muestra un campo de texto con lista de sugerencias que se filtra al escribir.
class GlowCombobox extends StatefulWidget {
  const GlowCombobox({super.key});
  @override
  State<GlowCombobox> createState() => _GlowComboboxState();
}

class _GlowComboboxState extends State<GlowCombobox> {
  final _ctrl = TextEditingController(text: 'SP');
  final _focus = FocusNode();
  // Lista completa de opciones hardcodeadas.
  static const _all = ['SPX', 'SPY', 'SPXL', 'SPXS', 'QQQ', 'GLD', 'G10'];
  bool _open = false;

  @override
  void initState() {
    super.initState();
    // Actualiza el estado al ganar/perder foco para animar el glow.
    _focus.addListener(() => setState(() => _open = _focus.hasFocus));
  }

  @override
  void dispose() {
    _ctrl.dispose();
    _focus.dispose();
    super.dispose();
  }

  // Filtra las opciones según el texto actual del input.
  List<String> get _filtered => _all
      .where((o) => o.toLowerCase().startsWith(_ctrl.text.toLowerCase()))
      .toList();

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        // Campo de texto con glow en foco.
        AnimatedContainer(
          duration: const Duration(milliseconds: 200),
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
          decoration: BoxDecoration(
            color: Gx.glassFill,
            borderRadius: BorderRadius.circular(Gx.rInput),
            border: Border.all(
                color: _open ? Gx.transitionIndigo : Gx.borderPanel,
                width: _open ? 1.5 : 1),
            boxShadow: _open
                ? Gx.glow(Gx.transitionIndigo, blur: 18, opacity: 0.4)
                : null,
          ),
          child: TextField(
            controller: _ctrl,
            focusNode: _focus,
            onChanged: (_) => setState(() {}),
            style: Gx.body,
            cursorColor: Gx.transitionIndigo,
            decoration: InputDecoration.collapsed(
              hintText: 'Símbolo…',
              hintStyle: Gx.uiSans(fontSize: 14, color: Gx.textMuted),
            ),
          ),
        ),
        // Lista de sugerencias animada.
        AnimatedSize(
          duration: const Duration(milliseconds: 200),
          curve: Curves.easeOut,
          child: (_open && _filtered.isNotEmpty)
              ? Padding(
                  padding: const EdgeInsets.only(top: 4),
                  child: frosted(
                    padding: const EdgeInsets.symmetric(vertical: 4),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.stretch,
                      children: _filtered
                          .map((o) => InkWell(
                                onTap: () {
                                  _ctrl.text = o;
                                  _focus.unfocus();
                                },
                                child: Padding(
                                  padding: const EdgeInsets.symmetric(
                                      horizontal: 12, vertical: 8),
                                  child: Text(o, style: Gx.dataMono(fontSize: 13)),
                                ),
                              ))
                          .toList(),
                    ),
                  ),
                )
              : const SizedBox.shrink(),
        ),
      ],
    );
  }
}

// ---------------------------------------------------------------------------
// Multiselect — selección múltiple con chips
// ---------------------------------------------------------------------------

// Permite seleccionar varias opciones; las seleccionadas aparecen como chips.
class GlowMultiSelect extends StatefulWidget {
  const GlowMultiSelect({super.key});
  @override
  State<GlowMultiSelect> createState() => _GlowMultiSelectState();
}

class _GlowMultiSelectState extends State<GlowMultiSelect> {
  // Conjunto de opciones actualmente seleccionadas.
  final _selected = <String>{'SPX', 'G10'};
  static const _options = ['SPX', 'QQQ', 'GLD', 'G10', 'DXY'];

  @override
  Widget build(BuildContext context) {
    return frosted(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Chips de las seleccionadas + botón de borrado.
          Wrap(
            spacing: 6,
            runSpacing: 6,
            children: [
              ..._selected.map((s) => GestureDetector(
                    onTap: () => setState(() => _selected.remove(s)),
                    child: Container(
                      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                      decoration: BoxDecoration(
                        color: Gx.transitionChipBg,
                        borderRadius: BorderRadius.circular(Gx.rChip),
                        border: Border.all(color: Gx.transitionChipBorder),
                        boxShadow: Gx.glow(Gx.transitionIndigo, blur: 8, opacity: 0.3),
                      ),
                      child: Row(mainAxisSize: MainAxisSize.min, children: [
                        Text(s,
                            style: Gx.uiSans(
                                fontSize: 12, color: Gx.transitionIndigo)),
                        const SizedBox(width: 4),
                        Icon(Gx.iconAdd, size: 10, color: Gx.transitionIndigo),
                      ]),
                    ),
                  )),
            ],
          ),
          const SizedBox(height: 8),
          const Divider(color: Gx.divider, height: 1),
          const SizedBox(height: 8),
          // Opciones disponibles para añadir.
          Wrap(
            spacing: 6,
            runSpacing: 6,
            children: _options
                .where((o) => !_selected.contains(o))
                .map((o) => GestureDetector(
                      onTap: () => setState(() => _selected.add(o)),
                      child: Container(
                        padding: const EdgeInsets.symmetric(
                            horizontal: 8, vertical: 4),
                        decoration: BoxDecoration(
                          color: Colors.transparent,
                          borderRadius: BorderRadius.circular(Gx.rChip),
                          border: Border.all(color: Gx.borderPanel),
                        ),
                        child: Text(o,
                            style: Gx.uiSans(
                                fontSize: 12, color: Gx.textLabel)),
                      ),
                    ))
                .toList(),
          ),
        ],
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Number Input — campo numérico con botones +/−
// ---------------------------------------------------------------------------

// Input numérico con controles +/- y glow en foco; valor restringido al rango.
class GlowNumberInput extends StatefulWidget {
  const GlowNumberInput({super.key, this.initial = 5, this.min = 1, this.max = 20});
  final int initial;
  final int min;
  final int max;
  @override
  State<GlowNumberInput> createState() => _GlowNumberInputState();
}

class _GlowNumberInputState extends State<GlowNumberInput> {
  late int _value = widget.initial;

  @override
  Widget build(BuildContext context) {
    return frosted(
      padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 4),
      child: Row(mainAxisSize: MainAxisSize.min, children: [
        _btn(Gx.iconChevronDown, () {
          if (_value > widget.min) setState(() => _value--);
        }, rotate: true),
        SizedBox(
          width: 48,
          child: Text(
            '$_value',
            textAlign: TextAlign.center,
            style: Gx.dataMono(fontSize: 14, color: Gx.textPrimary),
          ),
        ),
        _btn(Gx.iconChevronDown, () {
          if (_value < widget.max) setState(() => _value++);
        }),
      ]),
    );
  }

  Widget _btn(IconData icon, VoidCallback onTap, {bool rotate = false}) =>
      GestureDetector(
        onTap: onTap,
        child: Container(
          width: 28,
          height: 28,
          alignment: Alignment.center,
          decoration: BoxDecoration(
            color: Gx.glassFill,
            borderRadius: BorderRadius.circular(6),
          ),
          child: Transform.rotate(
            angle: rotate ? 3.14159 : 0,
            child: Icon(icon, size: 14, color: Gx.textSecondary),
          ),
        ),
      );
}

// ---------------------------------------------------------------------------
// Textarea — campo de texto multilínea
// ---------------------------------------------------------------------------

// Textarea: igual que GlowInput pero multilínea, con glow en foco.
class GlowTextarea extends StatefulWidget {
  const GlowTextarea({super.key});
  @override
  State<GlowTextarea> createState() => _GlowTextareaState();
}

class _GlowTextareaState extends State<GlowTextarea> {
  final _focus = FocusNode();

  @override
  void initState() {
    super.initState();
    _focus.addListener(() => setState(() {}));
  }

  @override
  void dispose() {
    _focus.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final focused = _focus.hasFocus;
    return AnimatedContainer(
      duration: const Duration(milliseconds: 200),
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
      decoration: BoxDecoration(
        color: Gx.glassFill,
        borderRadius: BorderRadius.circular(Gx.rInput),
        border: Border.all(
            color: focused ? Gx.transitionIndigo : Gx.borderPanel,
            width: focused ? 1.5 : 1),
        boxShadow:
            focused ? Gx.glow(Gx.transitionIndigo, blur: 18, opacity: 0.4) : null,
      ),
      child: TextField(
        focusNode: _focus,
        maxLines: 3,
        style: Gx.body,
        cursorColor: Gx.transitionIndigo,
        decoration: InputDecoration.collapsed(
          hintText: 'Descripción de la estrategia…',
          hintStyle: Gx.uiSans(fontSize: 14, color: Gx.textMuted),
        ),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// OTP / PIN Input — campos de dígito único para código de acceso
// ---------------------------------------------------------------------------

// Muestra 6 cajas de un dígito para ingreso de código OTP; el activo brilla.
class GlowOtpInput extends StatefulWidget {
  const GlowOtpInput({super.key});
  @override
  State<GlowOtpInput> createState() => _GlowOtpInputState();
}

class _GlowOtpInputState extends State<GlowOtpInput> {
  // Dígitos hardcodeados para la vitrina (sin lógica de validación real).
  final _digits = ['4', '2', '', '', '', ''];
  // Índice del campo "activo" (simulado en la vitrina).
  int _active = 2;

  @override
  Widget build(BuildContext context) {
    return Wrap(
      spacing: 4,
      children: List.generate(6, (i) {
        final isActive = i == _active;
        final hasVal = _digits[i].isNotEmpty;
        return GestureDetector(
          onTap: () => setState(() => _active = i),
          child: AnimatedContainer(
            duration: const Duration(milliseconds: 180),
            width: 34,
            height: 44,
            alignment: Alignment.center,
            decoration: BoxDecoration(
              color: Gx.glassFill,
              borderRadius: BorderRadius.circular(8),
              border: Border.all(
                color: isActive ? Gx.transitionIndigo : Gx.borderPanel,
                width: isActive ? 1.5 : 1,
              ),
              boxShadow: isActive
                  ? Gx.glow(Gx.transitionIndigo, blur: 14, opacity: 0.5)
                  : null,
            ),
            child: Text(
              hasVal ? _digits[i] : (isActive ? '|' : ''),
              style: Gx.dataMono(
                fontSize: 16,
                color: isActive ? Gx.transitionIndigo : Gx.textPrimary,
              ),
            ),
          ),
        );
      }),
    );
  }
}

// ---------------------------------------------------------------------------
// Rating — valoración por "estrellas" (circulos neón en Drasus)
// ---------------------------------------------------------------------------

// Muestra 5 indicadores de puntuación; los activos brillan en amber de alerta.
class GlowRating extends StatefulWidget {
  const GlowRating({super.key, this.initial = 3});
  final int initial;
  @override
  State<GlowRating> createState() => _GlowRatingState();
}

class _GlowRatingState extends State<GlowRating> {
  late int _rating = widget.initial;

  @override
  Widget build(BuildContext context) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: List.generate(5, (i) {
        final active = i < _rating;
        return GestureDetector(
          onTap: () => setState(() => _rating = i + 1),
          child: AnimatedContainer(
            duration: const Duration(milliseconds: 160),
            width: 22,
            height: 22,
            margin: const EdgeInsets.symmetric(horizontal: 3),
            decoration: BoxDecoration(
              shape: BoxShape.circle,
              color: active ? Gx.alertAmber.withAlpha(40) : Colors.transparent,
              border: Border.all(
                color: active ? Gx.alertAmber : Gx.borderPanel,
              ),
              boxShadow: active
                  ? Gx.glow(Gx.alertAmber, blur: 10, opacity: 0.6)
                  : null,
            ),
          ),
        );
      }),
    );
  }
}

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
      frosted(
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
        radius: Gx.rPanel,
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            _fmtBtn('B'),
            _fmtBtn('I'),
            _fmtBtn('U'),
            Container(width: 1, height: 14, color: Gx.borderPanel,
                margin: const EdgeInsets.symmetric(horizontal: 6)),
            _fmtBtn('H1'),
            _fmtBtn('H2'),
          ],
        ),
      ),
      const SizedBox(height: 6),
      // Área de contenido editable (placeholder estático).
      Container(
        padding: const EdgeInsets.all(12),
        decoration: BoxDecoration(
          color: Gx.panelSolid,
          borderRadius: BorderRadius.circular(Gx.rPanel),
          border: Border.all(color: Gx.borderPanel),
        ),
        child: Text(
          'Notas de la estrategia node-07…',
          style: Gx.uiSans(fontSize: 13, color: Gx.textMuted),
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
        borderRadius: BorderRadius.circular(4),
        color: label == 'B' ? Gx.transitionIndigo.withAlpha(40) : Colors.transparent,
      ),
      child: Text(label,
          style: Gx.dataMono(
              fontSize: 11,
              color: label == 'B' ? Gx.transitionIndigo : Gx.textLabel)),
    );

// ---------------------------------------------------------------------------
// Form Field — campo completo con label, input, helper y error
// ---------------------------------------------------------------------------

// Muestra un campo de formulario completo con label, input, mensaje de ayuda
// y estado de error (borde carmesí + mensaje de error).
class GlowFormField extends StatefulWidget {
  const GlowFormField({super.key, this.error = false});
  final bool error;
  @override
  State<GlowFormField> createState() => _GlowFormFieldState();
}

class _GlowFormFieldState extends State<GlowFormField> {
  final _focus = FocusNode();

  @override
  void initState() {
    super.initState();
    _focus.addListener(() => setState(() {}));
  }

  @override
  void dispose() {
    _focus.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final focused = _focus.hasFocus;
    final errorColor = Gx.criticalCrimson;
    final activeColor = widget.error ? errorColor : Gx.transitionIndigo;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [
        Text('Símbolo de activo',
            style: Gx.uiSans(fontSize: 12, color: Gx.textLabel)),
        const SizedBox(height: 4),
        AnimatedContainer(
          duration: const Duration(milliseconds: 200),
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
          decoration: BoxDecoration(
            color: Gx.glassFill,
            borderRadius: BorderRadius.circular(Gx.rInput),
            border: Border.all(
                color: (focused || widget.error) ? activeColor : Gx.borderPanel,
                width: focused ? 1.5 : 1),
            boxShadow: focused
                ? Gx.glow(activeColor, blur: 16, opacity: 0.4)
                : null,
          ),
          child: TextField(
            focusNode: _focus,
            style: Gx.body,
            cursorColor: activeColor,
            decoration: InputDecoration.collapsed(
              hintText: 'SPX',
              hintStyle: Gx.uiSans(fontSize: 14, color: Gx.textMuted),
            ),
          ),
        ),
        const SizedBox(height: 4),
        Text(
          widget.error ? 'Símbolo no reconocido.' : 'Cualquier ticker de futuros.',
          style: Gx.uiSans(
              fontSize: 11,
              color: widget.error ? errorColor : Gx.textMuted),
        ),
      ],
    );
  }
}
