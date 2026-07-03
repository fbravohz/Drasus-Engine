// stepper.dart — Componente Stepper (ADR-0138 enmienda 2026-06-29).
// Indicador de progreso en N pasos: completado / activo / pendiente.
// El estilo lo decide el tema global vía tokens Gx; prohibido hardcodear color.
//
// Nota: el nombre Stepper colisiona con el widget de Material del mismo nombre.
// Los consumidores importan con namespace `import ... as ui;` → `ui.Stepper`.
// Dentro de este archivo se oculta el Stepper de Material con `hide Stepper`.

// ignore: undefined_hidden_name — se oculta Stepper de Material para evitar colisión.
import 'package:flutter/material.dart' hide Stepper;
import '../gallery/gallery_tokens.dart';

// Indicador de progreso por pasos con barra de avance inferior.
// Estados de cada paso:
//   - Completado (índice < currentStep): relleno optimaCyan + ícono check.
//   - Activo (índice == currentStep): borde + fondo tenue + glow transitionIndigo.
//   - Pendiente (índice > currentStep): riel gaugeTrack, texto muted.
//
// Contrato funcional:
//   [steps]       lista de etiquetas de los pasos.
//   [currentStep] índice del paso activo (null = no controlado; arranca en 0).
//   [onStepTapped] callback con el índice del paso al tocarlo.
class Stepper extends StatefulWidget {
  final List<String> steps;
  final int? currentStep;
  final ValueChanged<int>? onStepTapped;

  // No es const: los getters dinámicos de Gx cambian con el tema.
  Stepper({
    super.key,
    required this.steps,
    this.currentStep,
    this.onStepTapped,
  });

  @override
  State<Stepper> createState() => _StepperState();
}

class _StepperState extends State<Stepper> {
  // Paso activo interno para modo no controlado (arranca en 0).
  int _internalStep = 0;

  // Paso efectivo: el externo (currentStep) tiene prioridad.
  int get _current => widget.currentStep ?? _internalStep;

  // Al tocar un paso: en modo no controlado navega a ese paso.
  void _tap(int index) {
    if (widget.currentStep == null) setState(() => _internalStep = index);
    widget.onStepTapped?.call(index);
  }

  @override
  // Fila de círculos de pasos con etiquetas + barra de progreso inferior.
  Widget build(BuildContext context) {
    final total = widget.steps.length;
    // Fracción de progreso: completados / (total-1). Mínimo 0, máximo 1.
    final progressFraction =
        total > 1 ? (_current / (total - 1)).clamp(0.0, 1.0) : 1.0;

    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        // Fila de círculos de pasos.
        Row(
          children: widget.steps.asMap().entries.map((e) {
            final completed = e.key < _current;
            final active = e.key == _current;
            // Color de estado: semántico interno de cada paso.
            final color = completed
                ? Gx.optimaCyan
                : active
                    ? Gx.transitionIndigo
                    : Gx.textBaseMuted;
            return Expanded(
              child: Column(children: [
                // Círculo del paso: clicable para navegar.
                GestureDetector(
                  onTap: () => _tap(e.key),
                  child: AnimatedContainer(
                    duration: const Duration(milliseconds: 200),
                    width: Gx.space24,
                    height: Gx.space24,
                    decoration: BoxDecoration(
                      shape: BoxShape.circle,
                      // Completado = relleno sólido; activo = tenue; pendiente = riel.
                      color: completed
                          ? Gx.optimaCyan
                          : active
                              ? Gx.transitionIndigo.withAlpha(60)
                              : Gx.gaugeTrack,
                      border:
                          Border.all(color: color, width: active ? 2 : 1),
                      boxShadow: (active || completed)
                          ? Gx.glow(color, blur: 10, opacity: 0.5)
                          : null,
                    ),
                    child: Center(
                      child: completed
                          // Ícono check en pasos completados; canvasBase sobre optimaCyan.
                          ? Icon(Gx.iconCheck,
                              size: 12, color: Gx.canvasBase)
                          // Número de paso en activo y pendiente.
                          : Text('${e.key + 1}',
                              style: Gx.dataMono(
                                  fontSize: 10, color: color)),
                    ),
                  ),
                ),
                SizedBox(height: Gx.space4),
                // Etiqueta del paso: token de texto según estado.
                Text(e.value,
                    style: Gx.uiSans(fontSize: 10, color: color)),
              ]),
            );
          }).toList(),
        ),
        SizedBox(height: Gx.space12),
        // Barra de progreso: riel gaugeTrack + relleno gradiente óptimo.
        Container(
          height: 3,
          decoration: BoxDecoration(
            color: Gx.gaugeTrack,
            // Radio de 2px: literal justificado por el grosor decorativo de 3px.
            borderRadius: BorderRadius.circular(2),
          ),
          child: FractionallySizedBox(
            alignment: Alignment.centerLeft,
            widthFactor: progressFraction,
            child: Container(
              decoration: BoxDecoration(
                gradient: Gx.linear(Gx.gradOptima),
                // Radio de 2px: literal justificado por el grosor decorativo de 3px.
                borderRadius: BorderRadius.circular(2),
                boxShadow: Gx.glow(Gx.optimaCyan, blur: 8, opacity: 0.5),
              ),
            ),
          ),
        ),
      ],
    );
  }
}
