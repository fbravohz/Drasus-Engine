// Test de widget del Banco de Pruebas: distinción visual rojo/ámbar.
//
// Salda DEBT-022. Regla FIJO de ADR-0152: un input mal formado para el
// puente FFI (rojo) y un input válido cuya operación de backend falló
// (ámbar) son dos estados distintos, con dos tratamientos visuales
// distintos — nunca deben colapsar al mismo color. STORY-050 encontró este
// bug a mano (ambos pintaban rojo); este test lo caza de forma automática.
//
// Cómo se evita el puente FFI real: GenericVerificationSection expone el
// seam `verifyOverride` (ver generic_verification_section.dart) — un doble
// que devuelve un VerificationOutcome fabricado, sin cruzar a Rust. El
// widget no sabe que es un doble: solo presenta lo que recibe (Cáscara
// Delgada, cero lógica de negocio en Dart).
//
// Correr con: flutter test test/verification_bench_status_test.dart

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:drasus_ui/components/components.dart' as custom_ui;
import 'package:drasus_ui/src/rust/api/verification.dart';
import 'package:drasus_ui/tabs/verification_bank/generic_verification_section.dart';

// Monta la sección con un doble de verifyFeature ya resuelto, dispara el
// envío y deja que el árbol se asiente antes de inspeccionar el resultado.
Future<void> _pumpAndSend(
  WidgetTester tester, {
  required VerificationOutcome outcome,
}) async {
  // Viewport amplio (mismo criterio que gallery_smoke_test.dart): el layout
  // de tres zonas (input/acción/respuesta) de GenericVerificationSection
  // desborda con el tamaño de test por defecto (800×600) porque el editor
  // de input reserva 26 líneas. No es un bug del widget — el Banco de
  // Pruebas siempre corre en una ventana de escritorio real.
  tester.view.physicalSize = const Size(1280, 900);
  tester.view.devicePixelRatio = 1.0;
  addTearDown(tester.view.resetPhysicalSize);
  addTearDown(tester.view.resetDevicePixelRatio);

  await tester.pumpWidget(
    MaterialApp(
      home: Scaffold(
        body: GenericVerificationSection(
          featureId: 'feature-de-prueba',
          title: 'Feature de prueba',
          icon: Icons.science,
          defaultInputJson: '{"a": 1}',
          // Doble inyectado: nunca cruza el puente FFI real (no ejecutable
          // en un widget test). Solo devuelve el outcome fabricado por caso.
          verifyOverride: ({required featureId, required inputJson}) async =>
              outcome,
        ),
      ),
    ),
  );

  // Dispara el envío tocando el botón "Enviar" (Zona central).
  await tester.tap(find.text('Enviar'));
  // pumpAndSettle: el doble resuelve el Future de inmediato, pero el
  // rebuild tras el `setState` de _enviar() todavía necesita un frame.
  await tester.pumpAndSettle();
}

void main() {
  testWidgets(
    'input inválido pinta banner ROJO (error) y chip crítico — NO ámbar',
    (WidgetTester tester) async {
      await _pumpAndSend(
        tester,
        outcome: const VerificationOutcome(
          inputStatus: InputStatus.invalid(reason: 'falta el campo "a"'),
          ok: false,
          outputJson: '',
        ),
      );

      // Banner: debe existir el rojo (error) y NO debe existir el ámbar (warning).
      final banners = tester
          .widgetList<custom_ui.Banner>(find.byType(custom_ui.Banner))
          .toList();
      expect(banners.length, 1);
      expect(banners.single.type, custom_ui.BannerType.error);
      expect(
        find.byWidgetPredicate(
          (w) => w is custom_ui.Banner && w.type == custom_ui.BannerType.warning,
        ),
        findsNothing,
      );

      // Chip: el chip de estado debe ser crítico (rojo), no alerta (ámbar).
      final statusChip = tester
          .widgetList<custom_ui.Chip>(find.byType(custom_ui.Chip))
          .firstWhere((c) => c.label == 'Input inválido');
      expect(statusChip.status, custom_ui.ChipStatus.critical);
    },
  );

  testWidgets(
    'error de backend con input válido pinta banner ÁMBAR (warning) — NO rojo',
    (WidgetTester tester) async {
      await _pumpAndSend(
        tester,
        outcome: const VerificationOutcome(
          inputStatus: InputStatus.valid(),
          ok: false,
          outputJson: '',
          error: 'la operación fue rechazada por el dominio',
        ),
      );

      // Banner: debe existir el ámbar (warning) y NO debe existir el rojo (error).
      final banners = tester
          .widgetList<custom_ui.Banner>(find.byType(custom_ui.Banner))
          .toList();
      expect(banners.length, 1);
      expect(banners.single.type, custom_ui.BannerType.warning);
      expect(
        find.byWidgetPredicate(
          (w) => w is custom_ui.Banner && w.type == custom_ui.BannerType.error,
        ),
        findsNothing,
      );

      // Chip: el chip "Backend: error" debe ser alerta (ámbar), no crítico (rojo).
      final backendChip = tester
          .widgetList<custom_ui.Chip>(find.byType(custom_ui.Chip))
          .firstWhere((c) => c.label == 'Backend: error');
      expect(backendChip.status, custom_ui.ChipStatus.alert);
    },
  );

  testWidgets(
    'éxito no pinta ningún banner de error ni de advertencia',
    (WidgetTester tester) async {
      await _pumpAndSend(
        tester,
        outcome: const VerificationOutcome(
          inputStatus: InputStatus.valid(),
          ok: true,
          outputJson: '{"resultado": "ok"}',
        ),
      );

      // Ni rojo ni ámbar: el éxito se comunica solo con chips ópticos.
      expect(find.byType(custom_ui.Banner), findsNothing);

      final chips = tester
          .widgetList<custom_ui.Chip>(find.byType(custom_ui.Chip))
          .toList();
      expect(
        chips.where((c) => c.label == 'Backend: OK').single.status,
        custom_ui.ChipStatus.optima,
      );
    },
  );
}
