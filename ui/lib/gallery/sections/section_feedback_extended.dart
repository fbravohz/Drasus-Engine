// Sección §9 Feedback extendido — notification-card, popconfirm, snackbar
// variantes, result/status-page, backdrop, stepper/wizard.
// Render-only con estado de UI local. Sin lógica de negocio ni FFI.

import 'package:flutter/material.dart';
import '../gallery_tokens.dart';
import '../gallery_fx.dart';

// ---------------------------------------------------------------------------
// Notification Card — tarjeta de notificación persistente (leída / no leída)
// ---------------------------------------------------------------------------

// Tarjeta de notificación con indicador de "no leída" (punto neón lateral).
// Al tocarla, el punto desaparece (simula marcar como leída).
class GlowNotificationCard extends StatefulWidget {
  const GlowNotificationCard({super.key});
  @override
  State<GlowNotificationCard> createState() => _GlowNotificationCardState();
}

class _GlowNotificationCardState extends State<GlowNotificationCard> {
  // Estado de lectura: true = no leída (punto neón visible).
  bool _unread = true;

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: () => setState(() => _unread = false),
      child: Container(
        padding: const EdgeInsets.all(12),
        decoration: BoxDecoration(
          gradient: Gx.linear([Gx.panelSolid, Gx.cardInner]),
          borderRadius: BorderRadius.circular(Gx.rPanel),
          border: Border.all(
              color: _unread ? Gx.transitionIndigo.withAlpha(80) : Gx.borderPanel),
          boxShadow: _unread
              ? Gx.glow(Gx.transitionIndigo, blur: 14, opacity: 0.15)
              : null,
        ),
        child: Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
          // Punto de "no leída".
          AnimatedContainer(
            duration: const Duration(milliseconds: 300),
            width: 8,
            height: 8,
            margin: const EdgeInsets.only(top: 4, right: 10),
            decoration: BoxDecoration(
              shape: BoxShape.circle,
              color: _unread ? Gx.transitionIndigo : Colors.transparent,
              boxShadow: _unread
                  ? Gx.glow(Gx.transitionIndigo, blur: 8, opacity: 0.7)
                  : null,
            ),
          ),
          Expanded(
            child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
              Text('node-07 entró en régimen óptimo',
                  style: Gx.uiSans(fontSize: 13, color: Gx.textPrimary,
                      weight: _unread ? FontWeight.w500 : FontWeight.w400)),
              const SizedBox(height: 2),
              Text('hace 3 min',
                  style: Gx.dataMono(fontSize: 11, color: Gx.textMuted)),
            ]),
          ),
        ]),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Popconfirm — confirmación inline con dos acciones
// ---------------------------------------------------------------------------

// Panel compacto de confirmación que aparece inline; tiene dos botones.
// Al pulsar Confirmar o Cancelar desaparece (simula el flujo de decisión).
class GlowPopconfirm extends StatefulWidget {
  const GlowPopconfirm({super.key});
  @override
  State<GlowPopconfirm> createState() => _GlowPopconfirmState();
}

class _GlowPopconfirmState extends State<GlowPopconfirm> {
  // Visibilidad del panel de confirmación.
  bool _visible = true;
  // Resultado de la última acción.
  String _result = '';

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // Botón ancla que abre el popconfirm.
        GestureDetector(
          onTap: () => setState(() { _visible = true; _result = ''; }),
          child: Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
            decoration: BoxDecoration(
              color: Gx.glassFill,
              borderRadius: BorderRadius.circular(Gx.rButton),
              border: Border.all(color: Gx.borderPanel),
            ),
            child: Text('Retirar node-19',
                style: Gx.uiSans(fontSize: 13, color: Gx.criticalRed)),
          ),
        ),
        AnimatedSize(
          duration: const Duration(milliseconds: 200),
          child: _visible
              ? Padding(
                  padding: const EdgeInsets.only(top: 8),
                  child: frosted(
                    glow:
                        Gx.glow(Gx.criticalCrimson, blur: 14, opacity: 0.2),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        Text('¿Confirmar retiro?',
                            style: Gx.uiSans(
                                fontSize: 13, color: Gx.textPrimary,
                                weight: FontWeight.w500)),
                        const SizedBox(height: 4),
                        Text('Esta acción archivará la célula.',
                            style: Gx.uiSans(
                                fontSize: 12, color: Gx.textSecondary)),
                        const SizedBox(height: 10),
                        Row(children: [
                          GestureDetector(
                            onTap: () =>
                                setState(() { _visible = false; _result = 'Retirado'; }),
                            child: Container(
                              padding: const EdgeInsets.symmetric(
                                  horizontal: 12, vertical: 7),
                              decoration: BoxDecoration(
                                gradient: Gx.linear(Gx.gradCritical),
                                borderRadius: BorderRadius.circular(Gx.rButton),
                                boxShadow: Gx.glow(Gx.criticalCrimson,
                                    blur: 10, opacity: 0.5),
                              ),
                              child: Text('Retirar',
                                  style: Gx.uiSans(
                                      fontSize: 12, color: Gx.pureWhite)),
                            ),
                          ),
                          const SizedBox(width: 8),
                          GestureDetector(
                            onTap: () =>
                                setState(() { _visible = false; _result = 'Cancelado'; }),
                            child: Container(
                              padding: const EdgeInsets.symmetric(
                                  horizontal: 12, vertical: 7),
                              decoration: BoxDecoration(
                                color: Gx.glassFill,
                                borderRadius: BorderRadius.circular(Gx.rButton),
                                border: Border.all(color: Gx.borderPanel),
                              ),
                              child: Text('Cancelar',
                                  style: Gx.uiSans(
                                      fontSize: 12, color: Gx.textLabel)),
                            ),
                          ),
                        ]),
                      ],
                    ),
                  ),
                )
              : _result.isEmpty
                  ? const SizedBox.shrink()
                  : Padding(
                      padding: const EdgeInsets.only(top: 6),
                      child: Text(_result,
                          style: Gx.uiSans(
                              fontSize: 12,
                              color: _result == 'Retirado'
                                  ? Gx.criticalCrimson
                                  : Gx.textMuted)),
                    ),
        ),
      ],
    );
  }
}

// ---------------------------------------------------------------------------
// Snackbar variantes — éxito, alerta, error
// ---------------------------------------------------------------------------

// Muestra 3 variantes de snackbar/toast lado a lado con su color semántico.
Widget snackbarVariants() {
  return Column(
    mainAxisSize: MainAxisSize.min,
    children: [
      _snackbar(Gx.iconCheck, 'Backtest completado', Gx.optimaCyan, Gx.optimaChipBg),
      const SizedBox(height: 8),
      _snackbar(Gx.iconWarning, 'Drift detectado en SPX', Gx.alertAmber, Gx.alertChipBg),
      const SizedBox(height: 8),
      _snackbar(Gx.iconDanger, 'Slippage crítico — retiro', Gx.criticalCrimson,
          Gx.criticalChipBg),
    ],
  );
}

Widget _snackbar(IconData icon, String msg, Color c, Color bg) => frosted(
      glow: Gx.glow(c, blur: 12, opacity: 0.25),
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
      child: Row(mainAxisSize: MainAxisSize.min, children: [
        Icon(icon, size: 14, color: c, shadows: Gx.textGlow(c, 8)),
        const SizedBox(width: 10),
        Flexible(child: Text(msg, style: Gx.uiSans(fontSize: 12, color: Gx.textPrimary))),
      ]),
    );

// ---------------------------------------------------------------------------
// Result / Status Page — página de resultado (éxito / error / vacío)
// ---------------------------------------------------------------------------

// Renderiza 2 variantes de página de resultado: éxito y error.
Widget resultPage({bool success = true}) {
  final c = success ? Gx.optimaCyan : Gx.criticalCrimson;
  final grad = success ? Gx.gradOptima : Gx.gradCritical;
  final title = success ? 'Backtest exitoso' : 'Fallo sistémico';
  final body = success
      ? 'La estrategia node-07 superó el filtro de calidad.'
      : 'Slippage letal detectado. La célula fue archivada.';

  return Container(
    padding: const EdgeInsets.all(16),
    decoration: BoxDecoration(
      gradient: Gx.linear([Gx.panelSolid, Gx.deepSpace]),
      borderRadius: BorderRadius.circular(Gx.rChrome),
      border: Border.all(color: c.withAlpha(80)),
      boxShadow: Gx.glow(c, blur: 20, opacity: 0.15),
    ),
    child: Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        // Icono de estado con glow.
        Container(
          width: 40,
          height: 40,
          decoration: BoxDecoration(
            shape: BoxShape.circle,
            gradient: RadialGradient(colors: [c.withAlpha(80), Colors.transparent]),
          ),
          child: Icon(success ? Gx.iconCheck : Gx.iconDanger, size: 20, color: c,
              shadows: Gx.textGlow(c, 12)),
        ),
        const SizedBox(height: 10),
        ShaderMask(
          shaderCallback: (r) => LinearGradient(colors: grad).createShader(r),
          child: Text(title,
              style: Gx.displayGrotesque(
                  fontSize: 18, color: Colors.white, weight: FontWeight.w500)),
        ),
        const SizedBox(height: 6),
        Text(body,
            textAlign: TextAlign.center,
            style: Gx.uiSans(fontSize: 12, color: Gx.textSecondary)),
      ],
    ),
  );
}

// ---------------------------------------------------------------------------
// Backdrop / Scrim — velo de fondo oscuro con panel encima
// ---------------------------------------------------------------------------

// Muestra el velo deepSpace semitransparente con un panel de vidrio encima.
Widget backdropExample() {
  return Stack(
    children: [
      // Velo de fondo.
      Container(
        height: 80,
        decoration: BoxDecoration(
          color: Gx.deepSpace.withAlpha(200),
          borderRadius: BorderRadius.circular(Gx.rPanel),
        ),
      ),
      // Panel flotante centrado sobre el velo.
      Positioned(
        left: 20,
        right: 20,
        top: 16,
        child: frosted(
          padding: const EdgeInsets.all(12),
          glow: Gx.glow(Gx.transitionIndigo, blur: 14, opacity: 0.2),
          child: Text('Modal sobre backdrop',
              style: Gx.uiSans(fontSize: 13, color: Gx.textPrimary)),
        ),
      ),
    ],
  );
}

// ---------------------------------------------------------------------------
// Stepper / Wizard — pasos secuenciales con estados completo/actual/pendiente
// ---------------------------------------------------------------------------

// Muestra un stepper de 4 pasos: algunos completados, uno activo, el resto pendiente.
class GlowStepper extends StatefulWidget {
  const GlowStepper({super.key});
  @override
  State<GlowStepper> createState() => _GlowStepperState();
}

class _GlowStepperState extends State<GlowStepper> {
  // Índice del paso actualmente activo.
  int _current = 1;

  static const _steps = ['Datos', 'Backtest', 'Validar', 'Incubar'];

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        // Fila de pasos.
        Row(
          children: _steps.asMap().entries.map((e) {
            final completed = e.key < _current;
            final active = e.key == _current;
            final color = completed
                ? Gx.optimaCyan
                : active
                    ? Gx.transitionIndigo
                    : Gx.textMuted;
            return Expanded(
              child: Column(children: [
                // Círculo del paso.
                GestureDetector(
                  onTap: () => setState(() => _current = e.key),
                  child: AnimatedContainer(
                    duration: const Duration(milliseconds: 200),
                    width: 24,
                    height: 24,
                    decoration: BoxDecoration(
                      shape: BoxShape.circle,
                      color: completed
                          ? Gx.optimaCyan
                          : active
                              ? Gx.transitionIndigo.withAlpha(60)
                              : Gx.gaugeTrack,
                      border: Border.all(color: color, width: active ? 2 : 1),
                      boxShadow: (active || completed)
                          ? Gx.glow(color, blur: 10, opacity: 0.5)
                          : null,
                    ),
                    child: Center(
                      child: completed
                          ? Icon(Gx.iconCheck, size: 12, color: Gx.deepSpace)
                          : Text('${e.key + 1}',
                              style:
                                  Gx.dataMono(fontSize: 10, color: color)),
                    ),
                  ),
                ),
                const SizedBox(height: 4),
                Text(e.value,
                    style: Gx.uiSans(fontSize: 10, color: color)),
              ]),
            );
          }).toList(),
        ),
        const SizedBox(height: 12),
        // Barra de progreso.
        Container(
          height: 3,
          decoration: BoxDecoration(
            color: Gx.gaugeTrack,
            borderRadius: BorderRadius.circular(2),
          ),
          child: FractionallySizedBox(
            alignment: Alignment.centerLeft,
            widthFactor: _current / (_steps.length - 1),
            child: Container(
              decoration: BoxDecoration(
                gradient: Gx.linear(Gx.gradOptima),
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

// ---------------------------------------------------------------------------
// Accordion / Collapse — secciones plegables
// ---------------------------------------------------------------------------

// Lista de secciones plegables; cada sección se abre/cierra con animación.
class GlowAccordion extends StatefulWidget {
  const GlowAccordion({super.key});
  @override
  State<GlowAccordion> createState() => _GlowAccordionState();
}

class _GlowAccordionState extends State<GlowAccordion> {
  // Índice de la sección actualmente abierta (-1 = ninguna).
  int _open = 0;

  static const _sections = [
    ('Parámetros del backtest', 'Ventana 252 días · Capital 1M · Comisión 0.1bps'),
    ('Filtros de régimen', 'HMM 4 estados · Umbral volatilidad 0.22'),
    ('Criterios de retiro', 'Drawdown máx. 8% · Slippage letal >15bps'),
  ];

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: _sections.asMap().entries.map((e) {
        final isOpen = e.key == _open;
        return Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            // Cabecera de la sección.
            GestureDetector(
              onTap: () =>
                  setState(() => _open = isOpen ? -1 : e.key),
              child: Container(
                padding: const EdgeInsets.symmetric(
                    horizontal: 12, vertical: 10),
                decoration: BoxDecoration(
                  color: isOpen ? Gx.surfaceRaised : Gx.panelSolid,
                  border: Border(
                      bottom: BorderSide(
                          color: isOpen ? Gx.transitionIndigo : Gx.divider,
                          width: isOpen ? 1.5 : 1)),
                ),
                child: Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    Expanded(
                      child: Text(e.value.$1,
                          overflow: TextOverflow.ellipsis,
                          style: Gx.uiSans(
                              fontSize: 13,
                              color: isOpen ? Gx.textPrimary : Gx.textSecondary,
                              weight: isOpen ? FontWeight.w500 : FontWeight.w400)),
                    ),
                    AnimatedRotation(
                      turns: isOpen ? 0.5 : 0,
                      duration: const Duration(milliseconds: 200),
                      child: Icon(Gx.iconChevronDown,
                          size: 14, color: Gx.textSecondary),
                    ),
                  ],
                ),
              ),
            ),
            // Contenido expandido.
            AnimatedSize(
              duration: const Duration(milliseconds: 220),
              curve: Curves.easeOut,
              child: isOpen
                  ? Container(
                      padding: const EdgeInsets.symmetric(
                          horizontal: 12, vertical: 10),
                      color: Gx.cardInner,
                      child: Text(e.value.$2,
                          style: Gx.uiSans(
                              fontSize: 12, color: Gx.textSecondary)),
                    )
                  : const SizedBox.shrink(),
            ),
          ],
        );
      }).toList(),
    );
  }
}
