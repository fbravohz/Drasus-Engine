// dropzone.dart — Componente Dropzone (ADR-0138 enmienda 2026-06-29).
// Zona de arrastre/soltura de archivos con tres estados visuales:
// reposo (idle), hover (arrastrando sobre la zona) y cargando (loading).
// Migrado de GlowDropzone (gallery/sections/section_std_missing.dart, Batch 4 STORY-025).

import 'package:flutter/material.dart';
import '../theme/gx_tokens.dart';
import '../theme/surfaces.dart';

// Estados internos de la zona de drop.
enum _DropStatus { idle, hover, loading }

// Zona interactiva de arrastre/soltura de archivos.
// Contrato funcional:
//   [onTap]  acción al tocar o soltar; null activa el modo demo (simula 2s de carga).
//   [label]  texto de instrucción en estado reposo; default: "Arrastra o toca para cargar".
class Dropzone extends StatefulWidget {
  final VoidCallback? onTap;
  final String? label;

  // No es const: el build lee getters dinámicos de Gx que cambian con el tema.
  Dropzone({super.key, this.onTap, this.label});

  @override
  State<Dropzone> createState() => _DropzoneState();
}

class _DropzoneState extends State<Dropzone> {
  _DropStatus _status = _DropStatus.idle;

  // Color semántico del borde/ícono según estado: cyan=cargando, índigo=hover, muted=reposo.
  Color get _stateColor => _status == _DropStatus.loading
      ? Gx.optimaCyan
      : _status == _DropStatus.hover
          ? Gx.transitionIndigo
          : Gx.textBaseMuted;

  @override
  // Zona de drop: MouseRegion para hover + GestureDetector para tap.
  // frosted() provee la superficie dinámica (glass/tint/solid según modo global).
  // AnimatedContainer controla el borde/glow animado conforme al estado activo.
  Widget build(BuildContext context) {
    final color = _stateColor;
    final isActive = _status != _DropStatus.idle;

    return MouseRegion(
      onEnter: (_) {
        if (_status == _DropStatus.idle) setState(() => _status = _DropStatus.hover);
      },
      onExit: (_) {
        if (_status == _DropStatus.hover) setState(() => _status = _DropStatus.idle);
      },
      child: GestureDetector(
        onTap: () async {
          if (widget.onTap != null) {
            widget.onTap!();
          } else {
            // Modo demo: simula carga 2s y vuelve al reposo.
            setState(() => _status = _DropStatus.loading);
            await Future.delayed(const Duration(seconds: 2));
            if (mounted) setState(() => _status = _DropStatus.idle);
          }
        },
        // frosted() actúa como superficie reactiva; padding=zero para que
        // el borde animado quede en el límite visual del glass.
        child: frosted(
          radius: Gx.rPanel,
          padding: EdgeInsets.zero,
          child: AnimatedContainer(
            duration: const Duration(milliseconds: 200),
            padding: const EdgeInsets.all(Gx.space24),
            decoration: BoxDecoration(
              borderRadius: BorderRadius.circular(Gx.rPanel),
              border: Border.all(
                // Borde semántico: más opaco y grueso cuando la zona está activa.
                color: color.withOpacity(isActive ? 0.8 : 0.4),
                width: isActive ? Gx.borderFocus : Gx.borderHairline,
              ),
              boxShadow: isActive
                  ? Gx.glow(color, blur: 14, opacity: 0.25)
                  : null,
            ),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                // Ícono central: spinner en carga, "+" en reposo/hover.
                Icon(
                  _status == _DropStatus.loading ? Gx.iconRefresh : Gx.iconAdd,
                  size: 28,
                  color: color,
                  shadows: isActive ? Gx.textGlow(color, 12) : null,
                ),
                const SizedBox(height: Gx.space8),
                // Texto de instrucción adaptado al estado actual.
                Text(
                  _status == _DropStatus.loading
                      ? 'Cargando…'
                      : _status == _DropStatus.hover
                          ? 'Suelta aquí'
                          : widget.label ?? 'Arrastra o toca para cargar',
                  style: Gx.uiSans(fontSize: 13, color: color),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
