# Detección de Bypass del Provider — Patrón Arquitectónico

Lección transversal detectada al refactorizar los helpers tipográficos de `Gx` (sesión 2026-06-26). No pertenece a una Story concreta sino a un patrón recurrente que cualquier agente debe capturar.

## Concepto

### El patrón de bypass

El código crea una **capa intermedia** (helpers, constantes, funciones) que define valores de diseño (fontSize, color, spacing) con literales, en paralelo al `ThemeData`/provider. Esa capa no hereda del tema y no reacciona a cambios del mismo.

**Síntoma:** cambiar el tema (paleta, acento, fuente) no afecta a ciertos textos o componentes.

**Causa raíz:** la capa intermedia se creó antes de que el provider existiera y nunca se migró. Queda como deuda técnica invisible.

### Cómo detectarlo

1. Buscar `TextStyle` con `fontSize:` o `color:` literales fuera del archivo del tema (`drasus_theme.dart`, `gallery_tokens.dart`)
   - `grep 'fontSize:' --include='*.dart' ui/lib/` y revisar cada match
   - Si el fontSize no viene de `Theme.of(context).textTheme` o de un getter que delegue al tema → violación
2. Buscar `Colors.*` o `Color(0xFF` literales en `TextStyle` o en widgets
   - `grep 'Colors\.(grey|red|amber|blue|green|white|black)' --include='*.dart' ui/lib/`
   - Si no es un color semántico de estado (crítico, alerta, óptimo) que necesite ser fijo → violación
3. Buscar helpers estáticos que devuelvan `TextStyle` con valores fijos
   - Si el helper no lee de `DrasusThemeState` → violación
4. Probar con paleta clara (`paper`) + modo sólido: lo que no se aclara/oscurece está bypassando

### Ejemplo real (lo corregido en esta sesión)

**Antes:** `gallery_tokens.dart` línea 269:
```dart
static TextStyle get microLabel =>
    uiSans(fontSize: 13, height: 1.3, color: textBaseLabel);
```
`fontSize: 13`, `height: 1.3` literales → no venían del theme provider.

**Después:** delega a `DrasusThemeState.globalMicroLabel`, que se sincroniza desde `_buildTextTheme()` en el provider:
```dart
static TextStyle get microLabel => DrasusThemeState.globalMicroLabel;
```
El fontSize ahora viene de `titleSmall` del TextTheme.

### Regla general

> Todo valor de diseño que pueda cambiar con el tema (fontSize, color, fontFamily, weight, height, spacing) DEBE definirse en el provider y consumirse por delegación, nunca por literal en el callsite.

## Trucos de Senior

- El patrón de **espejos estáticos** (`_globalTextColor`, `_globalMicroLabel`) es la solución cuando necesitas valores del tema en código sin `BuildContext` (helpers estáticos, `CustomPainter`). Se sincronizan en `load()` y en cada `set*()`.
- Para detectar bypass automáticamente: ejecuta `grep -r 'fontFamily:\|fontSize:\|Colors\.' ui/lib/ --include='*.dart' | grep -v 'gallery_tokens.dart\|drasus_theme.dart\|gallery_registry.dart'` y revisa los resultados — si un TextStyle no referencia `Theme.of(context).textTheme`, `Gx.*` o `DrasusThemeState.*`, es sospechoso.
- El Gx helper raw (`uiSans`, `displayGrotesque`, `dataMono`) no es el problema: es una utility que construye TextStyles con la fuente correcta. El problema son los helpers **nombrados** que añaden fontSize/color fijos. Los raw reciben todo por parámetro.
