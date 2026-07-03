// combobox.dart — Componente Combobox<T> (ADR-0138 enmienda 2026-06-29).
// Autocomplete/combobox genérico: campo de texto con filtrado de sugerencias.
// El estilo lo decide el tema global vía tokens Gx; prohibido hardcodear color.

import 'package:flutter/material.dart';
import '../gallery/gallery_tokens.dart';
import '../gallery/gallery_fx.dart';

// Ítem de un Combobox — asocia un valor tipado a una etiqueta visible.
class ComboboxItem<T> {
  final T value;
  final String label;
  const ComboboxItem({required this.value, required this.label});
}

// Combobox genérico con campo de texto y lista de sugerencias filtradas.
// Contrato funcional:
//   [value]    valor seleccionado actualmente (null = modo no controlado).
//   [items]    lista completa de opciones disponibles.
//   [onChanged] callback al seleccionar una opción (T seleccionado).
//   [hint]     texto guía cuando no hay selección.
// El filtrado compara el texto escrito con el inicio de cada label (insensible a mayúsculas).
// Modo no controlado: el componente gestiona [_selectedValue] internamente.
class Combobox<T> extends StatefulWidget {
  final T? value;
  final List<ComboboxItem<T>> items;
  final ValueChanged<T>? onChanged;
  final String? hint;

  // No es const: lee getters dinámicos de Gx que cambian con el tema.
  Combobox({
    super.key,
    this.value,
    required this.items,
    this.onChanged,
    this.hint,
  });

  @override
  State<Combobox<T>> createState() => _ComboboxState<T>();
}

class _ComboboxState<T> extends State<Combobox<T>> {
  final _ctrl = TextEditingController();
  final _focus = FocusNode();
  // Valor seleccionado en modo no controlado (el externo tiene prioridad).
  T? _internalValue;
  bool _open = false;

  // Valor efectivo: el externo si fue provisto, el interno si no.
  T? get _effectiveValue => widget.value ?? _internalValue;

  @override
  void initState() {
    super.initState();
    // Al recibir/perder foco, abre/cierra el panel de sugerencias.
    _focus.addListener(() {
      setState(() => _open = _focus.hasFocus);
    });
    // Si hay valor inicial, muestra su label en el campo de texto.
    _syncTextFromValue();
  }

  @override
  void didUpdateWidget(Combobox<T> oldWidget) {
    super.didUpdateWidget(oldWidget);
    // En modo controlado, sincroniza el texto cuando cambia el value externo.
    if (widget.value != oldWidget.value) _syncTextFromValue();
  }

  // Pone el label del valor seleccionado en el TextField, o lo vacía si no hay selección.
  void _syncTextFromValue() {
    final v = _effectiveValue;
    if (v == null) return;
    final item = widget.items.cast<ComboboxItem<T>?>().firstWhere(
        (e) => e!.value == v,
        orElse: () => null);
    if (item != null) _ctrl.text = item.label;
  }

  // Opciones filtradas: solo las que empiezan con el texto actual del campo.
  List<ComboboxItem<T>> get _filtered => widget.items
      .where((o) => o.label.toLowerCase().startsWith(_ctrl.text.toLowerCase()))
      .toList();

  // Selecciona un ítem, actualiza el estado y llama al callback.
  void _select(ComboboxItem<T> item) {
    _ctrl.text = item.label;
    if (widget.value == null) setState(() => _internalValue = item.value);
    widget.onChanged?.call(item.value);
    _focus.unfocus(); // cierra el panel al seleccionar
  }

  @override
  void dispose() {
    _ctrl.dispose();
    _focus.dispose();
    super.dispose();
  }

  @override
  // Campo con panel de sugerencias animado; glow al ganar foco.
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        // Campo de texto con borde de énfasis al ganar foco.
        panelSurface(
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
          radius: Gx.rInput,
          glow: _open
              ? Gx.glow(Gx.accentDynamic, blur: 18, opacity: 0.40)
              : null,
          child: AnimatedContainer(
            duration: const Duration(milliseconds: 200),
            decoration: BoxDecoration(
              borderRadius: BorderRadius.circular(Gx.rInput),
              border: _open
                  ? Border.all(color: Gx.accentDynamic, width: Gx.borderFocus)
                  : null,
            ),
            child: Row(children: [
              Expanded(
                child: TextField(
                  controller: _ctrl,
                  focusNode: _focus,
                  // Redibuja al escribir para actualizar la lista filtrada.
                  onChanged: (_) => setState(() {}),
                  style: Gx.uiSans(fontSize: 14, color: Gx.textBase),
                  cursorColor: Gx.accentDynamic,
                  decoration: InputDecoration.collapsed(
                    hintText: widget.hint,
                    hintStyle: Gx.uiSans(fontSize: 14, color: Gx.textBaseMuted),
                  ),
                ),
              ),
              // Flecha que indica si el panel está abierto o cerrado.
              AnimatedRotation(
                turns: _open ? 0.5 : 0,
                duration: const Duration(milliseconds: 200),
                child: Icon(Gx.iconChevronDown,
                    size: 16, color: Gx.textBaseSecondary),
              ),
            ]),
          ),
        ),
        // Panel de sugerencias filtradas — aparece y desaparece con animación.
        AnimatedSize(
          duration: const Duration(milliseconds: 200),
          curve: Curves.easeOut,
          child: (_open && _filtered.isNotEmpty)
              ? Padding(
                  padding: const EdgeInsets.only(top: 4),
                  child: panelSurface(
                    padding: const EdgeInsets.symmetric(vertical: 4),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.stretch,
                      children: _filtered
                          .map((item) => InkWell(
                                onTap: () => _select(item),
                                child: Padding(
                                  padding: const EdgeInsets.symmetric(
                                      horizontal: 12, vertical: 8),
                                  // Ítem seleccionado resaltado con el énfasis dinámico.
                                  child: Text(
                                    item.label,
                                    style: Gx.dataMono(
                                      fontSize: 13,
                                      color: item.value == _effectiveValue
                                          ? Gx.accentDynamic
                                          : Gx.textBase,
                                    ),
                                  ),
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
