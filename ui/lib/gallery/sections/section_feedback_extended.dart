// Sección §9 Feedback extendido — notification-card, popconfirm, snackbar
// variantes, result/status-page, backdrop, stepper/wizard, accordion.
// Render-only con estado de UI local. Sin lógica de negocio ni FFI.
// Tokens: superficies via wrappers frosted()/glassEnhanced()/cardSurface(),
// texto via Gx.textBase*, bordes via Gx.borderBase/accentDynamic.

import 'package:flutter/material.dart';
import '../gallery_tokens.dart';
import '../gallery_fx.dart';

// ---------------------------------------------------------------------------
// GlowNotificationCard — tarjeta de notificación persistente (leída / no leída)
// Parámetros: ninguno (estado local _unread).
// Tokens de chrome: glassEnhanced (superficie), Gx.borderBase (borde reposo),
//   Gx.textBase/textBaseSecondary/textBaseMuted (texto).
// Color de estado: Gx.transitionIndigo (señal "no leída" — se conserva).
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
  // Renderiza la tarjeta con borde semántico en estado no leído y borde estructural
  // global en estado leído; texto con tokens dinámicos para paper/bunker.
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: () => setState(() => _unread = false),
      child: glassEnhanced(
        semanticColor: _unread ? Gx.transitionIndigo : Gx.accentDynamic,
        padding: const EdgeInsets.all(Gx.space12),
        glow: _unread
            ? Gx.glow(Gx.transitionIndigo, blur: 14, opacity: 0.15)
            : null,
        child: Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
          // Punto indicador "no leída": semántico (transitionIndigo) — señalización interna.
          AnimatedContainer(
            duration: const Duration(milliseconds: 300),
            width: Gx.space8,
            height: Gx.space8,
            margin: EdgeInsets.only(
                top: Gx.space4, right: Gx.space8 + Gx.space4),
            decoration: BoxDecoration(
              shape: BoxShape.circle,
              // Colors.transparent es el valor invisible del punto; no es chrome.
              color: _unread ? Gx.transitionIndigo : Colors.transparent,
              boxShadow: _unread
                  ? Gx.glow(Gx.transitionIndigo, blur: 8, opacity: 0.7)
                  : null,
            ),
          ),
          Expanded(
            child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
              // Texto principal con token dinámico — legible en paper y bunker.
              Text('node-07 entró en régimen óptimo',
                  style: Gx.uiSans(
                      fontSize: 13,
                      color: Gx.textBase,
                      weight: _unread ? FontWeight.w500 : FontWeight.w400)),
              SizedBox(height: Gx.space4 / 2),
              // Timestamp con token muted dinámico.
              Text('hace 3 min',
                  style: Gx.dataMono(fontSize: 11, color: Gx.textBaseMuted)),
            ]),
          ),
        ]),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// GlowPopconfirm — confirmación inline con dos acciones
// Parámetros: ninguno (estado local _visible, _result).
// Tokens de chrome: glassEnhanced (panel del confirm), Gx.borderBase (botón ancla),
//   Gx.textBase*/textBaseLabel (texto), gradCritical + pureWhite (botón destructor).
// Color de estado: Gx.criticalCrimson (señaliza acción destructiva — se conserva).
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
  // Renderiza el botón ancla y el panel de confirmación animado.
  // El panel usa glassEnhanced con color crítico como énfasis semántico interno.
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // Botón ancla que abre el popconfirm.
        GestureDetector(
          onTap: () => setState(() {
            _visible = true;
            _result = '';
          }),
          child: Container(
            padding: const EdgeInsets.symmetric(
                horizontal: Gx.space12, vertical: Gx.space8),
            decoration: BoxDecoration(
              color: Gx.surfaceFill,
              borderRadius: BorderRadius.circular(Gx.rButton),
              // Borde estructural global dinámico para botón secundario.
              border: Border.all(color: Gx.borderBase),
            ),
            // El color criticalRed es señalización interna de acción destructiva.
            child: Text('Retirar node-19',
                style: Gx.uiSans(fontSize: 13, color: Gx.criticalRed)),
          ),
        ),
        AnimatedSize(
          duration: const Duration(milliseconds: 200),
          child: _visible
              ? Padding(
                  padding: EdgeInsets.only(top: Gx.space8),
                  child: glassEnhanced(
                    semanticColor: Gx.criticalCrimson,
                    padding: const EdgeInsets.all(Gx.space12),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        // Pregunta con token dinámico base.
                        Text('¿Confirmar retiro?',
                            style: Gx.uiSans(
                                fontSize: 13,
                                color: Gx.textBase,
                                weight: FontWeight.w500)),
                        SizedBox(height: Gx.space4),
                        // Descripción con token dinámico secundario.
                        Text('Esta acción archivará la célula.',
                            style: Gx.uiSans(
                                fontSize: 12,
                                color: Gx.textBaseSecondary)),
                        SizedBox(height: Gx.space8 + Gx.space4),
                        Row(children: [
                          // Botón destructor: gradiente semántico de la familia crítica.
                          GestureDetector(
                            onTap: () => setState(
                                () {
                                  _visible = false;
                                  _result = 'Retirado';
                                }),
                            child: Container(
                              padding: const EdgeInsets.symmetric(
                                  horizontal: Gx.space12, vertical: 7),
                              decoration: BoxDecoration(
                                gradient: Gx.linear(Gx.gradCritical),
                                borderRadius:
                                    BorderRadius.circular(Gx.rButton),
                                boxShadow: Gx.glow(Gx.criticalCrimson,
                                    blur: 10, opacity: 0.5),
                              ),
                              // pureWhite: texto sobre gradiente oscuro saturado (legibilidad).
                              child: Text('Retirar',
                                  style: Gx.uiSans(
                                      fontSize: 12, color: Gx.pureWhite)),
                            ),
                          ),
                          SizedBox(width: Gx.space8),
                          // Botón secundario con borde estructural global.
                          GestureDetector(
                            onTap: () => setState(
                                () {
                                  _visible = false;
                                  _result = 'Cancelado';
                                }),
                            child: Container(
                              padding: const EdgeInsets.symmetric(
                                  horizontal: Gx.space12, vertical: 7),
                              decoration: BoxDecoration(
                                color: Gx.surfaceFill,
                                borderRadius:
                                    BorderRadius.circular(Gx.rButton),
                                border:
                                    Border.all(color: Gx.borderBase),
                              ),
                              // textBaseLabel: etiqueta discreta en acción secundaria.
                              child: Text('Cancelar',
                                  style: Gx.uiSans(
                                      fontSize: 12,
                                      color: Gx.textBaseLabel)),
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
                      padding: EdgeInsets.only(top: Gx.space4 + Gx.space4 / 2),
                      child: Text(_result,
                          style: Gx.uiSans(
                              fontSize: 12,
                              // Color de resultado: semántico interno del estado final.
                              color: _result == 'Retirado'
                                  ? Gx.criticalCrimson
                                  : Gx.textBaseMuted)),
                    ),
        ),
      ],
    );
  }
}

// ---------------------------------------------------------------------------
// snackbarVariants() — 3 variantes de snackbar/toast (éxito, alerta, error)
// Tokens de chrome: glassEnhanced por variante (superficie), Gx.textBase (texto).
// Colores de dato: optimaCyan/alertAmber/criticalCrimson (señalizan tipo de mensaje).
// ---------------------------------------------------------------------------

// Muestra 3 variantes de snackbar/toast apiladas con su color semántico.
Widget snackbarVariants() {
  return Column(
    mainAxisSize: MainAxisSize.min,
    children: [
      _snackbar(Gx.iconCheck, 'Backtest completado', Gx.optimaCyan,
          Gx.optimaChipBg),
      SizedBox(height: Gx.space8),
      _snackbar(Gx.iconWarning, 'Drift detectado en SPX', Gx.alertAmber,
          Gx.alertChipBg),
      SizedBox(height: Gx.space8),
      _snackbar(Gx.iconDanger, 'Slippage crítico — retiro',
          Gx.criticalCrimson, Gx.criticalChipBg),
    ],
  );
}

// Snackbar individual: glassEnhanced con el color semántico del evento.
// El ícono y el texto usan el color semántico (señalización interna del tipo de evento).
Widget _snackbar(IconData icon, String msg, Color c, Color bg) =>
    glassEnhanced(
      semanticColor: c,
      padding: const EdgeInsets.symmetric(
          horizontal: Gx.space12, vertical: Gx.space8 + Gx.space4),
      child: Row(mainAxisSize: MainAxisSize.min, children: [
        Icon(icon, size: 14, color: c, shadows: Gx.textGlow(c, 8)),
        SizedBox(width: Gx.space8 + Gx.space4),
        // Texto del mensaje con token base dinámico.
        Flexible(
            child: Text(msg,
                style: Gx.uiSans(fontSize: 12, color: Gx.textBase))),
      ]),
    );

// ---------------------------------------------------------------------------
// resultPage() — página de resultado (éxito / error)
// Parámetros: [success] bool con default true.
// Tokens de chrome: glassEnhanced (superficie), Gx.textBaseSecondary (cuerpo).
// Colores de dato: optimaCyan/criticalCrimson (señalizan resultado — se conservan).
// pureWhite: obligatorio para que ShaderMask coloree el texto del título correctamente.
// ---------------------------------------------------------------------------

// Renderiza una variante de página de resultado (éxito o error).
// [success] determina si se muestra la variante de backtest exitoso (true)
// o fallo sistémico (false).
Widget resultPage({bool success = true}) {
  final c = success ? Gx.optimaCyan : Gx.criticalCrimson;
  final grad = success ? Gx.gradOptima : Gx.gradCritical;
  final title = success ? 'Backtest exitoso' : 'Fallo sistémico';
  final body = success
      ? 'La estrategia node-07 superó el filtro de calidad.'
      : 'Slippage letal detectado. La célula fue archivada.';

  return glassEnhanced(
    semanticColor: c,
    padding: const EdgeInsets.all(Gx.space16),
    glow: Gx.glow(c, blur: 20, opacity: 0.15),
    child: Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        // Icono de estado con halo radial semántico.
        Container(
          width: 40,
          height: 40,
          decoration: BoxDecoration(
            shape: BoxShape.circle,
            gradient: RadialGradient(
                colors: [c.withAlpha(80), Colors.transparent]),
          ),
          child: Icon(success ? Gx.iconCheck : Gx.iconDanger,
              size: 20,
              color: c,
              shadows: Gx.textGlow(c, 12)),
        ),
        SizedBox(height: Gx.space8 + Gx.space4),
        // Título con ShaderMask del gradiente semántico.
        // pureWhite necesario para que el ShaderMask pinte el gradiente correctamente.
        ShaderMask(
          shaderCallback: (r) =>
              LinearGradient(colors: grad).createShader(r),
          child: Text(title,
              style: Gx.displayGrotesque(
                  fontSize: 18,
                  color: Gx.pureWhite,
                  weight: FontWeight.w500)),
        ),
        SizedBox(height: Gx.space4 + Gx.space4 / 2),
        // Cuerpo con token secundario dinámico — legible en paper/bunker.
        Text(body,
            textAlign: TextAlign.center,
            style:
                Gx.uiSans(fontSize: 12, color: Gx.textBaseSecondary)),
      ],
    ),
  );
}

// ---------------------------------------------------------------------------
// backdropExample() — velo de fondo oscuro con panel encima
// Tokens de chrome: Gx.deepSpace (velo estructural — lienzo base del ZUI),
//   glassEnhanced (panel flotante), Gx.textBase (texto del modal).
// ---------------------------------------------------------------------------

// Muestra el velo deepSpace semitransparente con un panel de vidrio encima.
// deepSpace es el token canónico del lienzo base — su uso como velo es correcto.
Widget backdropExample() {
  return Stack(
    children: [
      // Velo de fondo: deepSpace es el lienzo base del ZUI, apropiado para scrim.
      Container(
        height: 80,
        decoration: BoxDecoration(
          color: Gx.canvasBase.withAlpha(200),
          borderRadius: BorderRadius.circular(Gx.rPanel),
        ),
      ),
      // Panel flotante centrado sobre el velo.
      Positioned(
        left: Gx.space16 + Gx.space4,
        right: Gx.space16 + Gx.space4,
        top: Gx.space16,
        child: panelSurface(
          padding: const EdgeInsets.all(Gx.space12),
          child: Text('Modal sobre backdrop',
              style: Gx.uiSans(fontSize: 13, color: Gx.textBase)),
        ),
      ),
    ],
  );
}

// ---------------------------------------------------------------------------
// GlowStepper — stepper de 4 pasos con estados completo/actual/pendiente
// Parámetros: ninguno (estado local _current).
// Tokens de chrome: Gx.gaugeTrack (riel de progreso), Gx.textBase* (texto).
// Colores de estado: optimaCyan (completo), transitionIndigo (activo),
//   textMuted (pendiente) — señalización interna del paso actual.
// ---------------------------------------------------------------------------

// Stepper de 4 pasos: algunos completados, uno activo, el resto pendiente.
// Al tocar un círculo de paso se navega a ese paso (demo de interacción).
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
  // Renderiza la fila de pasos y la barra de progreso; el paso activo recibe glow.
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        // Fila de círculos de pasos.
        Row(
          children: _steps.asMap().entries.map((e) {
            final completed = e.key < _current;
            final active = e.key == _current;
            // Colores de estado: semánticos internos del paso.
            final color = completed
                ? Gx.optimaCyan
                : active
                    ? Gx.transitionIndigo
                    : Gx.textBaseMuted;
            return Expanded(
              child: Column(children: [
                // Círculo del paso con glow en completado/activo.
                GestureDetector(
                  onTap: () => setState(() => _current = e.key),
                  child: AnimatedContainer(
                    duration: const Duration(milliseconds: 200),
                    width: Gx.space24,
                    height: Gx.space24,
                    decoration: BoxDecoration(
                      shape: BoxShape.circle,
                      color: completed
                          ? Gx.optimaCyan
                          : active
                              ? Gx.transitionIndigo.withAlpha(60)
                              : Gx.gaugeTrack,
                      border: Border.all(
                          color: color, width: active ? 2 : 1),
                      boxShadow: (active || completed)
                          ? Gx.glow(color, blur: 10, opacity: 0.5)
                          : null,
                    ),
                    child: Center(
                      child: completed
                          ? Icon(Gx.iconCheck,
                              size: 12, color: Gx.deepSpace)
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
        // Barra de progreso con riel de gauge y relleno gradiente óptimo.
        Container(
          height: 3,
          decoration: BoxDecoration(
            color: Gx.gaugeTrack,
            // barra del stepper (3px alto): radio decorativo
            borderRadius: BorderRadius.circular(2),
          ),
          child: FractionallySizedBox(
            alignment: Alignment.centerLeft,
            widthFactor: _current / (_steps.length - 1),
            child: Container(
              decoration: BoxDecoration(
                gradient: Gx.linear(Gx.gradOptima),
                // barra del stepper (3px alto): radio decorativo
                borderRadius: BorderRadius.circular(2),
                boxShadow:
                    Gx.glow(Gx.optimaCyan, blur: 8, opacity: 0.5),
              ),
            ),
          ),
        ),
      ],
    );
  }
}

// ---------------------------------------------------------------------------
// GlowAccordion — secciones plegables con animación
// Parámetros: ninguno (estado local _open).
// Tokens de chrome: Gx.surfaceRaised (fondo de cabecera activa — hover de fila),
//   cardSurface() (cuerpo de sección expandida), Gx.textBase*/borderBase (texto/borde).
// Color de estado: Gx.transitionIndigo (borde de sección activa — señal interna).
// ---------------------------------------------------------------------------

// Lista de secciones plegables; cada sección se abre/cierra con animación.
// Al tocar la cabecera la sección alterna entre abierta y cerrada.
class GlowAccordion extends StatefulWidget {
  const GlowAccordion({super.key});
  @override
  State<GlowAccordion> createState() => _GlowAccordionState();
}

class _GlowAccordionState extends State<GlowAccordion> {
  // Índice de la sección actualmente abierta (-1 = ninguna).
  int _open = 0;

  static const _sections = [
    ('Parámetros del backtest',
        'Ventana 252 días · Capital 1M · Comisión 0.1bps'),
    ('Filtros de régimen',
        'HMM 4 estados · Umbral volatilidad 0.22'),
    ('Criterios de retiro',
        'Drawdown máx. 8% · Slippage letal >15bps'),
  ];

  @override
  // Renderiza cada sección con cabecera clicable y contenido expandible.
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: _sections.asMap().entries.map((e) {
        final isOpen = e.key == _open;
        return Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            // Cabecera de la sección: fondo surfaceRaised en activa.
            GestureDetector(
              onTap: () =>
                  setState(() => _open = isOpen ? -1 : e.key),
              child: Container(
                padding: const EdgeInsets.symmetric(
                    horizontal: Gx.space12, vertical: Gx.space8 + Gx.space4),
                decoration: BoxDecoration(
                  // surfaceRaised es el token de hover de fila — correcto para cabecera activa.
                  color: isOpen ? Gx.surfaceRaisedDynamic : Gx.surfacePanel,
                  border: Border(
                      bottom: BorderSide(
                          // Borde inferior: semántico interno (activa) vs divider (inactiva).
                          color: isOpen
                              ? Gx.transitionIndigo
                              : Gx.divider,
                          width: isOpen ? Gx.borderFocus : Gx.borderHairline)),
                ),
                child: Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    Expanded(
                      child: Text(e.value.$1,
                          overflow: TextOverflow.ellipsis,
                          style: Gx.uiSans(
                              fontSize: 13,
                              // Texto: base en activa, secundario en inactiva.
                              color: isOpen
                                  ? Gx.textBase
                                  : Gx.textBaseSecondary,
                              weight: isOpen
                                  ? FontWeight.w500
                                  : FontWeight.w400)),
                    ),
                    AnimatedRotation(
                      turns: isOpen ? 0.5 : 0,
                      duration: const Duration(milliseconds: 200),
                      child: Icon(Gx.iconChevronDown,
                          size: 14, color: Gx.textBaseSecondary),
                    ),
                  ],
                ),
              ),
            ),
            // Cuerpo expandido con cardSurface() — reacciona a los modos.
            AnimatedSize(
              duration: const Duration(milliseconds: 220),
              curve: Curves.easeOut,
              child: isOpen
                  ? cardSurface(
                      padding: const EdgeInsets.symmetric(
                          horizontal: Gx.space12,
                          vertical: Gx.space8 + Gx.space4),
                      child: Text(e.value.$2,
                          style: Gx.uiSans(
                              fontSize: 12,
                              color: Gx.textBaseSecondary)),
                    )
                  : const SizedBox.shrink(),
            ),
          ],
        );
      }).toList(),
    );
  }
}
