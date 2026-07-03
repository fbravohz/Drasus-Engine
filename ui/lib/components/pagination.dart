// pagination.dart — Componente Pagination (ADR-0138 enmienda 2026-06-29).
// Controles de paginación con flechas y páginas numeradas.
// La página activa lleva glow del énfasis dinámico del tema.

import 'package:flutter/material.dart';
import '../theme/gx_tokens.dart';

// Controles de paginación con soporte controlado y no controlado.
// Modo controlado:    Pagination(page: _p, total: 5, onChanged: ...)
// Modo no controlado: Pagination(total: 5, onChanged: ...)  (arranca en página 1)
class Pagination extends StatefulWidget {
  // page: página activa 1-indexada para modo controlado; null = no controlado.
  final int? page;
  // total: número total de páginas.
  final int total;
  // onChanged: se llama con el nuevo número de página al cambiar.
  final ValueChanged<int>? onChanged;

  const Pagination({
    super.key,
    this.page,
    required this.total,
    this.onChanged,
  });

  @override
  State<Pagination> createState() => _PaginationState();
}

class _PaginationState extends State<Pagination> {
  // Estado interno de la página activa para el modo no controlado.
  late int _page;

  @override
  void initState() {
    super.initState();
    // Modo controlado usa widget.page; no controlado comienza en 1.
    _page = widget.page ?? 1;
  }

  @override
  void didUpdateWidget(Pagination old) {
    super.didUpdateWidget(old);
    // En modo controlado, sincroniza la página interna cuando el padre cambia.
    if (widget.page != null && widget.page != _page) {
      setState(() => _page = widget.page!);
    }
  }

  // Navega a la página indicada si está dentro del rango válido.
  void _go(int p) {
    if (p < 1 || p > widget.total) return;
    setState(() => _page = p);
    widget.onChanged?.call(p);
  }

  @override
  // Dibuja las flechas de navegación + los botones numerados de página.
  Widget build(BuildContext context) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        // Flecha izquierda: deshabilitada en la primera página (sin callback).
        _navBtn(
          Gx.iconChevronDown,
          rotate: true,
          enabled: _page > 1,
          onTap: () => _go(_page - 1),
        ),
        const SizedBox(width: 6),
        // Botones numerados: uno por página, el activo con glow de énfasis.
        ...List.generate(widget.total, (i) {
          final p = i + 1;
          final active = p == _page;
          return GestureDetector(
            onTap: () => _go(p),
            child: AnimatedContainer(
              duration: const Duration(milliseconds: 180),
              width: 28,
              height: 28,
              margin: const EdgeInsets.symmetric(horizontal: 3),
              alignment: Alignment.center,
              decoration: BoxDecoration(
                shape: BoxShape.circle,
                // Fondo tenue del énfasis en el botón activo.
                color: active ? Gx.accentDynamic.withAlpha(40) : Colors.transparent,
                border: active
                    ? Border.all(color: Gx.accentDynamic)
                    : Border.all(color: Colors.transparent),
                boxShadow: active
                    ? Gx.glow(Gx.accentDynamic, blur: 10, opacity: 0.5)
                    : null,
              ),
              child: Text(
                '$p',
                style: Gx.dataMono(
                  fontSize: 12,
                  color: active ? Gx.accentDynamic : Gx.textBaseLabel,
                ),
              ),
            ),
          );
        }),
        const SizedBox(width: 6),
        // Flecha derecha: deshabilitada en la última página.
        _navBtn(
          Gx.iconChevronDown,
          enabled: _page < widget.total,
          onTap: () => _go(_page + 1),
        ),
      ],
    );
  }

  // Botón de flecha circular con estado habilitado/deshabilitado.
  // rotate: true gira 180° la flecha (para la dirección izquierda).
  Widget _navBtn(
    IconData icon, {
    required VoidCallback onTap,
    bool rotate = false,
    bool enabled = true,
  }) {
    return GestureDetector(
      onTap: enabled ? onTap : null,
      child: Container(
        width: 28,
        height: 28,
        alignment: Alignment.center,
        decoration: BoxDecoration(
          shape: BoxShape.circle,
          color: Gx.surfaceFill,
          border: Border.all(color: Gx.borderBase),
        ),
        child: Transform.rotate(
          angle: rotate ? 3.14159 : 0,
          child: Icon(
            icon,
            size: 14,
            // Icono atenuado cuando está deshabilitado.
            color: enabled ? Gx.textBaseSecondary : Gx.textBaseMuted,
          ),
        ),
      ),
    );
  }
}
