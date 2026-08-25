// dropzone.dart — Componente Dropzone (ADR-0138 enmienda 2026-06-29).
// Zona de carga de archivos con dos estados de contenido (reposo y cargando)
// más un realce de puntero (hover) que NO altera la geometría.
// Migrado de GlowDropzone (gallery/sections/section_std_missing.dart, Batch 4 STORY-025).
//
// ---------------------------------------------------------------------------
// Corrección 2026-08-01 — parpadeo infinito al pasar el cursor (bug reportado
// por el propietario probando STORY-073).
// ---------------------------------------------------------------------------
// Síntoma: al acercar el cursor, la zona alternaba a toda velocidad entre su
// texto de reposo y "Suelta aquí"; por el lado izquierdo se quedaba encogida.
//
// Causa: la GEOMETRÍA dependía del estado de hover, por dos vías a la vez.
//   1. El texto cambiaba a uno más corto ("Suelta aquí") y la `Column` con
//      `mainAxisSize.min` encogía la caja.
//   2. El borde pasaba de `borderHairline` a `borderFocus`, y en Flutter el
//      grosor del borde consume espacio interior: la caja cambia de tamaño.
// El resultado se realimenta: el cursor entra -> la caja encoge -> el borde
// derecho se mueve por debajo del cursor -> se dispara `onExit` -> la caja
// crece -> el cursor queda dentro otra vez -> `onEnter`. A la derecha oscila
// sin parar; a la izquierda el cursor sigue dentro de la caja encogida y se
// estabiliza ahí, que es exactamente lo que se observó.
//
// Regla que deja este archivo: **un estado de puntero jamás puede cambiar el
// tamaño del objeto que lo produce.** El realce se expresa solo con color y
// glow; ancho, alto y grosor de borde son constantes en los tres estados.
//
// Además se retiró el texto "Suelta aquí" del hover: pertenece a un evento de
// arrastre real (drag-over), no a mover el ratón por encima. Hoy este
// componente NO acepta soltar archivos — no hay `DragTarget` ni plugin de
// arrastre de escritorio cableado — así que anunciarlo prometía algo que no
// ocurre. Cuando se cablee el arrastre real, entra aquí como un estado
// `dragOver` propio y ahí sí corresponde ese texto.

import 'package:flutter/material.dart';
import '../theme/gx_tokens.dart';
import '../theme/surfaces.dart';

// Estados internos de la zona de carga.
enum _DropStatus { idle, hover, loading }

// Zona interactiva de carga de archivos.
// Contrato funcional:
//   [onTap]  acción al tocar; null activa el modo demo (simula 2s de carga).
//   [label]  texto de instrucción en reposo; default: "Toca para cargar un archivo".
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

  // Color semántico del borde/ícono según estado: cyan=cargando, índigo=hover,
  // muted=reposo. El color es lo ÚNICO que cambia al pasar el cursor.
  Color get _stateColor => switch (_status) {
        _DropStatus.loading => Gx.optimaCyan,
        _DropStatus.hover => Gx.transitionIndigo,
        _DropStatus.idle => Gx.textBaseMuted,
      };

  @override
  // Zona de carga: MouseRegion para el realce + GestureDetector para el toque.
  // frosted() provee la superficie dinámica (glass/tint/solid según modo global).
  // AnimatedContainer anima color y glow — nunca tamaño.
  Widget build(BuildContext context) {
    final color = _stateColor;
    final isActive = _status != _DropStatus.idle;

    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) {
        if (_status == _DropStatus.idle) {
          setState(() => _status = _DropStatus.hover);
        }
      },
      onExit: (_) {
        if (_status == _DropStatus.hover) {
          setState(() => _status = _DropStatus.idle);
        }
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
            // Ancho completo del contenedor padre: la caja no encoge ni crece
            // con su contenido, así que ningún cambio de texto puede volver a
            // sacar el cursor de la zona.
            width: double.infinity,
            padding: const EdgeInsets.all(Gx.space24),
            decoration: BoxDecoration(
              borderRadius: BorderRadius.circular(Gx.rPanel),
              border: Border.all(
                // Grosor CONSTANTE en los tres estados: en Flutter el borde
                // ocupa espacio interior, así que engrosarlo al pasar el
                // cursor movía el propio límite que detecta al cursor.
                // La señal de "activo" va por color y glow, no por grosor.
                color: color.withOpacity(isActive ? 0.85 : 0.4),
                width: Gx.borderHairline,
              ),
              boxShadow:
                  isActive ? Gx.glow(color, blur: 14, opacity: 0.25) : null,
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
                // Texto de instrucción. Solo distingue "cargando" de "listo
                // para recibir un toque": el hover NO cambia el texto (ver
                // cabecera — era la mitad del bucle de parpadeo).
                Text(
                  _status == _DropStatus.loading
                      ? 'Cargando…'
                      : widget.label ?? 'Toca para cargar un archivo',
                  textAlign: TextAlign.center,
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
