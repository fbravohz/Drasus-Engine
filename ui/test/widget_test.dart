// Smoke test mínimo del arranque de widgets.
// Reemplaza el test plantilla de Flutter (contador) que referenciaba `MyApp`
// —clase que nunca existió; el widget raíz real es AppRoot, que requiere
// ThemeState + RustLib—. La cobertura real de UI vive en gallery_smoke_test.dart
// y gallery_golden_test.dart.

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  testWidgets('MaterialApp monta y renderiza su home', (tester) async {
    await tester.pumpWidget(
      const MaterialApp(home: Scaffold(body: Text('OK'))),
    );
    expect(find.text('OK'), findsOneWidget);
  });
}
