// section_feedback_extended.dart — Funciones de galería de feedback no migradas.
// Render-only. Sin lógica de negocio ni FFI.
// Tokens: superficies via wrappers panelSurface(); texto via Gx.textBase*.
//
// MIGRADO a ui/lib/components/ — Batch 3, STORY-025 (2026-06-30):
//   GlowNotificationCard → ui.NotificationCard (notification_card.dart)
//   GlowPopconfirm       → ui.Popconfirm       (popconfirm.dart)
//   GlowStepper          → ui.Stepper           (stepper.dart)
//   GlowAccordion        → ui.Accordion         (accordion.dart)
//
// Conservado: snackbarVariants(), resultPage(), backdropExample()
// (funciones helper de galería; no son componentes con contrato funcional).

import 'package:flutter/material.dart';
import '../gallery_tokens.dart';
import '../gallery_fx.dart';

// ---------------------------------------------------------------------------
// snackbarVariants() — 3 variantes de snackbar/toast (éxito, alerta, error)
// Tokens de chrome: panelSurface por variante, Gx.textBase (texto).
// Colores de dato: optimaCyan/alertAmber/criticalCrimson — señalizan tipo.
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

// Snackbar individual: superficie dinámica estándar con borde semántico izquierdo.
Widget _snackbar(IconData icon, String msg, Color c, Color bg) =>
    panelSurface(
      glow: Gx.glow(c, blur: 14, opacity: 0.15),
      padding: const EdgeInsets.symmetric(
          horizontal: Gx.space12, vertical: Gx.space8 + Gx.space4),
      child: Container(
        decoration: BoxDecoration(
          border: Border(left: BorderSide(color: c, width: 3)),
        ),
        child: Row(mainAxisSize: MainAxisSize.min, children: [
          Icon(icon, size: 14, color: c, shadows: Gx.textGlow(c, 8)),
          SizedBox(width: Gx.space8 + Gx.space4),
          Flexible(
              child: Text(msg,
                  style: Gx.uiSans(fontSize: 12, color: Gx.textBase))),
        ]),
      ),
    );

// ---------------------------------------------------------------------------
// resultPage() — página de resultado (éxito / error)
// [success] bool con default true (éxito = optimaCyan; error = criticalCrimson).
// pureWhite necesario para que ShaderMask coloree el texto correctamente.
// ---------------------------------------------------------------------------

// Renderiza la variante de backtest exitoso (success=true) o fallo sistémico (false).
Widget resultPage({bool success = true}) {
  final c = success ? Gx.optimaCyan : Gx.criticalCrimson;
  final grad = success ? Gx.gradOptima : Gx.gradCritical;
  final title = success ? 'Backtest exitoso' : 'Fallo sistémico';
  final body = success
      ? 'La estrategia node-07 superó el filtro de calidad.'
      : 'Slippage letal detectado. La célula fue archivada.';

  return panelSurface(
    padding: const EdgeInsets.all(Gx.space16),
    glow: Gx.glow(c, blur: 20, opacity: 0.15),
    child: Container(
      decoration: BoxDecoration(
        border: Border(left: BorderSide(color: c, width: 3)),
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          // Ícono de estado con halo radial semántico.
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
              style: Gx.uiSans(fontSize: 12, color: Gx.textBaseSecondary)),
        ],
      ),
    ),
  );
}

// ---------------------------------------------------------------------------
// backdropExample() — velo de fondo oscuro con panel flotante encima.
// canvasBase con alpha 200: scrim que permite ver el contenido detrás.
// ---------------------------------------------------------------------------

// Muestra el velo canvasBase semitransparente con un panel de vidrio encima.
// canvasBase es el token del lienzo base del ZUI — su uso como velo es correcto.
Widget backdropExample() {
  return Stack(
    children: [
      // Velo de fondo: canvasBase con alpha parcial para simular el scrim.
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
