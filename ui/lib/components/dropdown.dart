// dropdown.dart — Componente Dropdown<T> genérico (ADR-0138 enmienda 2026-06-29).
// Desplegable funcional con panel animado, ítem seleccionado y callback tipado.
// El estilo lo decide el tema global vía tokens Gx; prohibido hardcodear color.

import 'package:flutter/material.dart';
import '../gallery/gallery_tokens.dart';
import '../gallery/gallery_fx.dart';

// Ítem de un Dropdown — asocia un valor tipado a una etiqueta legible.
class DropdownItem<T> {
  final T value;
  final String label;
  const DropdownItem({required this.value, required this.label});
}

// Desplegable genérico con panel animado y glow en foco.
// Contrato funcional: [value] valor seleccionado actualmente (null = sin selección);
// [items] lista de opciones disponibles; [onChanged] callback al seleccionar una opción;
// [hint] texto guía cuando no hay selección.
// Modo no controlado: si [value] es null el componente gestiona el estado interno.
class Dropdown<T> extends StatefulWidget {
  final T? value;
  final List<DropdownItem<T>> items;
  final ValueChanged<T>? onChanged;
  final String? hint;

  // No es const: lee getters dinámicos de Gx que cambian con el tema.
  Dropdown({
    super.key,
    this.value,
    required this.items,
    this.onChanged,
    this.hint,
  });

  @override
  State<Dropdown<T>> createState() => _DropdownState<T>();
}

class _DropdownState<T> extends State<Dropdown<T>> {
  bool _open = false;
  // Estado interno para modo no controlado (cuando el padre no provee [value]).
  T? _internalValue;

  // Valor efectivo: el externo tiene prioridad sobre el interno.
  T? get _effectiveValue => widget.value ?? _internalValue;

  // Etiqueta del ítem seleccionado, o el hint si no hay selección.
  String get _displayLabel {
    if (_effectiveValue == null) return widget.hint ?? '';
    final item = widget.items
        .cast<DropdownItem<T>?>()
        .firstWhere((e) => e!.value == _effectiveValue, orElse: () => null);
    return item?.label ?? widget.hint ?? '';
  }

  void _selectItem(T value) {
    // En modo no controlado, actualiza el estado interno; en ambos casos llama al callback.
    if (widget.value == null) setState(() => _internalValue = value);
    widget.onChanged?.call(value);
    setState(() => _open = false);
  }

  @override
  // Desplegable con panel expandible/contraíble y glow cuando está abierto.
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // Cabecera del desplegable: texto seleccionado + flecha giratoria.
        GestureDetector(
          onTap: () => setState(() => _open = !_open),
          child: panelSurface(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
            // Glow cuando el panel está abierto — señal de foco activo.
            glow: _open
                ? Gx.glow(Gx.accentDynamic, blur: 16, opacity: 0.45)
                : null,
            child: Row(mainAxisSize: MainAxisSize.min, children: [
              Flexible(
                child: Text(
                  _displayLabel,
                  style: Gx.uiSans(
                    fontSize: 14,
                    // Hint en color muted; selección en color base dinámico.
                    color: _effectiveValue == null
                        ? Gx.textBaseMuted
                        : Gx.textBase,
                  ),
                  overflow: TextOverflow.ellipsis,
                ),
              ),
              const SizedBox(width: 8),
              // La flecha rota 180° cuando el panel está abierto.
              AnimatedRotation(
                turns: _open ? 0.5 : 0,
                duration: const Duration(milliseconds: 200),
                child: Icon(Gx.iconChevronDown,
                    size: 18, color: Gx.textBaseSecondary),
              ),
            ]),
          ),
        ),
        // Panel de opciones: aparece con AnimatedSize al abrir el desplegable.
        AnimatedSize(
          duration: const Duration(milliseconds: 220),
          curve: Curves.easeOut,
          child: _open
              ? Padding(
                  padding: const EdgeInsets.only(top: 6),
                  child: panelSurface(
                    padding: const EdgeInsets.symmetric(vertical: 4),
                    child: Column(
                      mainAxisSize: MainAxisSize.min,
                      crossAxisAlignment: CrossAxisAlignment.stretch,
                      children: widget.items
                          .map((item) => InkWell(
                                onTap: () => _selectItem(item.value),
                                child: Padding(
                                  padding: const EdgeInsets.symmetric(
                                      horizontal: 12, vertical: 8),
                                  // El ítem seleccionado usa el color de énfasis dinámico.
                                  child: Text(
                                    item.label,
                                    style: Gx.uiSans(
                                      fontSize: 14,
                                      color: item.value == _effectiveValue
                                          ? Gx.accentDynamic
                                          : Gx.textBaseSecondary,
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
