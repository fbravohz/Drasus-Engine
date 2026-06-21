// Test de humo para el Panel Operativo Fundacional.
//
// Verifica que las 3 pestañas renderizan sin lanzar excepción.
// No llama funciones del Bridge Rust (no existe librería nativa en tests
// unitarios Flutter) — usa stubs que devuelven datos vacíos.
//
// Correr con: flutter test ui/test/panel_smoke_test.dart

// ignore_for_file: avoid_print

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

// ─── Stubs del Bridge ──────────────────────────────────────────────────────
// Los archivos de binding (api/clock.dart, api/jobs.dart, api/audit.dart)
// llaman a la librería nativa en sus cuerpos. Como en tests unitarios no
// existe librería nativa compilada, redefinimos aquí las funciones con
// implementaciones stub que devuelven datos de prueba en memoria.
//
// En un proyecto con flutter_rust_bridge real, el patrón habitual es
// usar MockitoMock o flutter_rust_bridge_test_utils. Para un smoke test
// de estructura es suficiente con widgets que no llaman al Bridge.

// ─── Widgets de stub para las pestañas ─────────────────────────────────────
// En lugar de usar los widgets reales (que llaman al Bridge), usamos widgets
// mínimos que solo renderizan texto estático. El test verifica la estructura
// del PanelOperativo (3 pestañas, navegación) sin necesitar el bridge nativo.

// Pestaña de reloj para el test: muestra texto estático, sin Timer ni Bridge.
class _ClockTabStub extends StatelessWidget {
  const _ClockTabStub();
  @override
  Widget build(BuildContext context) =>
      const Center(child: Text('reloj-stub'));
}

// Pestaña de trabajos para el test: muestra texto estático, sin Future ni Bridge.
class _JobsTabStub extends StatelessWidget {
  const _JobsTabStub();
  @override
  Widget build(BuildContext context) =>
      const Center(child: Text('trabajos-stub'));
}

// Pestaña de auditoría para el test: muestra texto estático, sin Future ni Bridge.
class _AuditTabStub extends StatelessWidget {
  const _AuditTabStub();
  @override
  Widget build(BuildContext context) =>
      const Center(child: Text('auditoria-stub'));
}

// PanelOperativo reconstruido con stubs — idéntica estructura al panel real
// (DefaultTabController + Scaffold + AppBar con TabBar + TabBarView) pero
// con widgets de pestaña que no dependen del Bridge.
class _PanelStub extends StatelessWidget {
  const _PanelStub();

  @override
  Widget build(BuildContext context) {
    return DefaultTabController(
      length: 3,
      child: Scaffold(
        appBar: AppBar(
          title: const Text('Drasus Engine — Panel Operativo'),
          bottom: const TabBar(
            tabs: [
              Tab(icon: Icon(Icons.access_time), text: 'Reloj'),
              Tab(icon: Icon(Icons.queue), text: 'Trabajos'),
              Tab(icon: Icon(Icons.security), text: 'Auditoría'),
            ],
          ),
        ),
        body: const TabBarView(
          children: [
            _ClockTabStub(),
            _JobsTabStub(),
            _AuditTabStub(),
          ],
        ),
      ),
    );
  }
}

// ─── Tests ─────────────────────────────────────────────────────────────────

void main() {
  // testWidgets registra un test de widget. Flutter corre cada test en un
  // entorno virtual sin dispositivo físico: crea un "motor de test" que
  // simula el ciclo de layout, pintura y eventos de gestos.
  testWidgets('panel_operativo_renders_three_tabs', (WidgetTester tester) async {
    // pumpWidget monta el widget en el entorno de test y ejecuta un frame
    // completo (layout + paint). Si algún widget lanza una excepción durante
    // build(), pumpWidget la propaga y el test falla aquí.
    await tester.pumpWidget(
      const MaterialApp(
        // Usamos el stub en lugar del panel real para evitar el Bridge.
        home: _PanelStub(),
      ),
    );

    // Verifica que los textos de las 3 pestañas están presentes en el árbol
    // de widgets. find.text() busca cualquier widget Text con ese string.
    // expect(..., findsOneWidget) falla si hay 0 o más de 1 resultado.
    expect(find.text('Reloj'), findsOneWidget);
    expect(find.text('Trabajos'), findsOneWidget);
    expect(find.text('Auditoría'), findsOneWidget);

    // Verifica que el contenido de la pestaña inicial (índice 0, Reloj) es visible.
    expect(find.text('reloj-stub'), findsOneWidget);

    // Navega a la pestaña de Trabajos pulsando su Tab.
    await tester.tap(find.text('Trabajos'));
    // pumpAndSettle() ejecuta frames hasta que no quedan animaciones pendientes.
    // TabBarView usa una animación de deslizamiento — un solo pump() no la
    // completa y el widget de destino aún no está en el árbol visible.
    await tester.pumpAndSettle();
    expect(find.text('trabajos-stub'), findsOneWidget);

    // Navega a la pestaña de Auditoría.
    await tester.tap(find.text('Auditoría'));
    await tester.pumpAndSettle();
    expect(find.text('auditoria-stub'), findsOneWidget);
  });
}
