# Lecciones de Dart/Flutter — STORY-015: Panel Operativo Fundacional

> Story: [STORY-015 — Panel Operativo Fundacional](../../execution/STORY-015-panel-operativo-fundacional.md)
> Implementado: 2026-06-21
> Archivos producidos: `ui/lib/main.dart`, `ui/lib/panel_operativo.dart`, `ui/lib/tabs/clock_tab.dart`, `ui/lib/tabs/jobs_tab.dart`, `ui/lib/tabs/audit_tab.dart`, `ui/test/panel_smoke_test.dart`

---

## Conceptos

### `WidgetsFlutterBinding.ensureInitialized()`

**Qué es el problema que resuelve:** Flutter tiene dos capas — el motor (C++) y el framework (Dart). Cuando el proceso arranca, el motor no ha terminado de inicializarse cuando el código Dart comienza a ejecutarse. Si en ese estado intentas hacer una llamada de plataforma (como cargar una librería nativa via FFI), el framework no tiene a quién entregarle la respuesta y la app falla con una excepción difícil de leer.

**Qué hace `ensureInitialized()`:** fuerza la inicialización del binding motor↔framework de forma síncrona, antes de continuar con el resto de `main()`. Es el "espera a que el motor esté listo" obligatorio.

**Cuándo se necesita:** siempre que `main()` haga trabajo asíncrono *antes* de `runApp()`. Si vas directo a `runApp()` sin trabajo previo, Flutter lo inicializa solo.

**En STORY-015** (`ui/lib/main.dart`, línea 15):
```dart
WidgetsFlutterBinding.ensureInitialized();
await RustLib.init();  // sin ensureInitialized(), esta línea podría fallar
```

---

### `runApp()` — punto de adhesión del árbol de widgets a la pantalla

**Qué hace:** toma un widget y lo "adhiere" al canvas de la ventana. Desde ese momento, Flutter gestiona el ciclo completo: layout, pintura, gestos, animaciones. Sin `runApp()`, el código Dart corre pero no hay nada visible.

**Solo se llama una vez** en toda la app — es el punto de entrada del mundo UI, análogo a `main()` para el mundo Dart.

**En STORY-015** (`ui/lib/main.dart`, línea 21):
```dart
runApp(const DrasusApp());
```

---

### `StatelessWidget` vs `StatefulWidget`

**La pregunta clave:** ¿necesita el widget actualizar lo que muestra a partir de datos que cambian dentro de él mismo?

| Condición | Widget a usar |
|---|---|
| Los datos vienen de fuera (parámetros) y no cambian | `StatelessWidget` |
| El widget necesita actualizar su contenido por sí solo (timer, respuesta de red, tap interno) | `StatefulWidget` |

**Por qué importa:** `StatelessWidget` es más simple y el compilador puede optimizarlo mejor. Si el widget es `const`, Dart ni siquiera llama al constructor en reconstrucciones posteriores — reutiliza la instancia compilada.

**En STORY-015:**
- `DrasusApp` → `StatelessWidget`: solo configura tema y ruta, nunca cambia.
- `PanelOperativo` → `StatelessWidget`: `DefaultTabController` gestiona el estado de las pestañas, no el widget.
- `ClockTab`, `JobsTab`, `AuditTab` → `StatefulWidget`: necesitan actualizar contenido en respuesta a timers y futuras consultas.

---

### Ciclo de vida de `StatefulWidget`: `initState()` y `dispose()`

**Cómo funciona `StatefulWidget`:** Flutter crea dos objetos — el widget (efímero, puede recrearse en cualquier reconstrucción) y su `State` (persistente mientras el widget esté en el árbol). El `State` es donde viven las variables mutables.

**`initState()`** se llama exactamente una vez, justo después de crear el `State`. Es el lugar para arrancar recursos: timers, streams, suscripciones, conexiones. No debe hacer trabajo pesado síncrono (bloquea el hilo principal) ni llamar a `context` directamente (el widget aún no está completamente en el árbol).

**`dispose()`** se llama exactamente una vez, justo antes de destruir el `State`. Es el lugar para liberar todos los recursos iniciados en `initState()`. Si no se cancela un timer o se cierra un stream aquí, siguen consumiendo CPU/memoria indefinidamente aunque el widget ya no exista.

**En STORY-015** (`ui/lib/tabs/clock_tab.dart`):
```dart
@override
void initState() {
  super.initState();
  _actualizarReloj();          // primera lectura inmediata
  _timer = Timer.periodic(const Duration(seconds: 1), (_) {
    _actualizarReloj();        // lectura cada segundo
  });
}

@override
void dispose() {
  _timer?.cancel();            // CRÍTICO: sin esto, el timer sigue vivo tras cerrar la pestaña
  super.dispose();
}
```

---

### `setState()` — aviso de cambio de estado

**Por qué existe:** Flutter no detecta automáticamente cuando cambia una variable en el `State`. `setState()` es el aviso explícito al framework: "algo cambió, por favor llama a `build()` de nuevo y actualiza la pantalla".

**Qué hace internamente:** marca el widget como "sucio" (necesita reconstruirse) y encola una llamada a `build()` para el próximo frame. Solo reconstruye el subárbol del widget que llamó a `setState()` — no toda la app.

**Error frecuente:** cambiar una variable sin `setState()` → el valor cambia en memoria pero la pantalla no se actualiza.

**En STORY-015** (`ui/lib/tabs/clock_tab.dart`):
```dart
void _actualizarReloj() {
  final ns = getClockTimestampNs();
  setState(() {
    _timestampNs = ns;  // sin esto, el Text del build() nunca cambia
  });
}
```

---

### `DefaultTabController`, `TabBar` y `TabBarView`

**El problema:** `TabBar` (las etiquetas) y `TabBarView` (el contenido) son dos widgets independientes que necesitan saber cuál pestaña está activa. Sin coordinación, pulsar una etiqueta no cambiaría el contenido.

**`DefaultTabController`:** es un `InheritedWidget` que propaga el estado de "pestaña activa" hacia abajo por el árbol. Cualquier `TabBar` o `TabBarView` descendiente se registra automáticamente y sincroniza su comportamiento. `length` define cuántas pestañas son válidas — debe coincidir exactamente con el número de `Tab` en `TabBar` y de hijos en `TabBarView`.

**Sincronización posicional:** el `Tab` en índice 0 corresponde al hijo en índice 0 del `TabBarView`. La correspondencia es por posición, no por nombre.

**En STORY-015** (`ui/lib/panel_operativo.dart`):
```dart
return DefaultTabController(
  length: 3,                   // 3 pestañas: índices 0, 1, 2
  child: Scaffold(
    appBar: AppBar(
      bottom: const TabBar(
        tabs: [
          Tab(icon: Icon(Icons.access_time), text: 'Reloj'),   // índice 0
          Tab(icon: Icon(Icons.queue), text: 'Trabajos'),       // índice 1
          Tab(icon: Icon(Icons.security), text: 'Auditoría'),   // índice 2
        ],
      ),
    ),
    body: const TabBarView(
      children: [
        ClockTab(),   // se muestra cuando índice activo = 0
        JobsTab(),    // se muestra cuando índice activo = 1
        AuditTab(),   // se muestra cuando índice activo = 2
      ],
    ),
  ),
);
```

---

### `Scaffold` — el esqueleto de una pantalla Material

**Qué es:** un widget que provee las zonas predefinidas de una pantalla según el diseño Material: `appBar` (barra superior), `body` (contenido principal), `floatingActionButton`, `drawer`, `bottomNavigationBar`, etc.

**Por qué usarlo:** sin `Scaffold`, tendrías que calcular manualmente el posicionamiento relativo al `AppBar`, al notch de cámara, a la barra de navegación del sistema. `Scaffold` lo resuelve automáticamente para todos los sistemas operativos soportados.

---

### `const` en widgets — optimización de reconstrucción

**Por qué importa:** Flutter puede reconstruir el árbol de widgets muchas veces por segundo. Si un widget es `const`, Dart crea la instancia en tiempo de compilación y la reutiliza en cada reconstrucción — sin llamar al constructor, sin allocar memoria. Para widgets estáticos (etiquetas, íconos, estructuras que no cambian), `const` es una optimización gratuita.

**Regla práctica:** si todos los parámetros del constructor son valores conocidos en compilación (literales, otras constantes), añade `const`. El compilador Dart te lo pide si lo omites.

---

### `FutureBuilder` — gestión declarativa de operaciones asíncronas

**El problema:** `getJobsSummary()` y `getRecentAuditEvents()` son asíncronas — hacen I/O a SQLite. Si las llamas en `build()` directamente, crearías un nuevo `Future` en cada frame, lanzando consultas en cascada.

**`FutureBuilder`:** toma un `Future` existente y reconstruye su `builder` cada vez que el Future cambia de estado. El `Future` se guarda en el `State` (`_futureJobs`, `_futureEventos`) — se crea una sola vez (o cuando el usuario pulsa "Refrescar"), no en cada llamada a `build()`.

**Los tres estados que debes manejar:**

```dart
FutureBuilder<List<JobSummary>>(
  future: _futureJobs,
  builder: (context, snapshot) {
    if (snapshot.connectionState == ConnectionState.waiting) {
      return const CircularProgressIndicator();  // cargando
    }
    if (snapshot.hasError) {
      return Text('Error: ${snapshot.error}');   // error
    }
    final jobs = snapshot.data ?? [];            // datos listos
    return ListView.builder(...);
  },
)
```

**En STORY-015** este patrón aparece en `jobs_tab.dart` y `audit_tab.dart`.

---

### `ListView.builder` — renderizado diferido de listas

**Diferencia con `ListView(children: [...])`:**

- `ListView(children: [...])`: construye todos los hijos inmediatamente, estén visibles o no. Con 1000 ítems → 1000 widgets en memoria.
- `ListView.builder(itemBuilder: ...)`: construye solo los widgets visibles más un buffer. El widget del ítem 500 no existe hasta que el usuario hace scroll hasta él.

**En STORY-015** la lista tiene 20 ítems (jobs) o 50 (eventos de auditoría) — la diferencia de rendimiento es imperceptible a esta escala. El patrón de `builder` es el correcto porque la lista puede crecer.

---

### Memoria vs. disco: por qué los datos de SQLite persisten

**Memoria:** las variables del `State` (como `_timestampNs` en `ClockTab`) solo existen mientras el proceso está vivo. Al cerrar la app, se pierden.

**Disco (SQLite):** la base de datos `drasus.db` es un archivo en el sistema de archivos. Cuando la app cierra, el archivo permanece. Cuando reabre y el `FutureBuilder` lanza una nueva consulta al Bridge, Rust lee el mismo archivo y devuelve los mismos datos más los que se hayan añadido. Este es el principio "Local-First" de Drasus: el estado vive en disco, no en memoria, y la UI solo lo lee.

---

### Hash de cadena en auditoría — verificación visual con 8 caracteres

**Qué es el hash de cadena:** la bitácora de Drasus es una cadena de eventos enlazados por hashes. Cada evento almacena el hash SHA-256 del evento anterior. Si alguien modifica o borra un evento pasado, el hash almacenado en el siguiente evento ya no coincide con el hash real del modificado — la manipulación es detectable.

**Por qué 8 caracteres son suficientes para verificación visual:** un hash SHA-256 completo tiene 64 chars en hexadecimal. Para el uso visual del Panel — "¿este evento pertenece a la misma cadena que vi ayer?" — los últimos 8 son suficientes: la probabilidad de que dos hashes diferentes coincidan en sus últimos 8 chars es 1 en 2^32 (≈4 mil millones). No es un sustituto de la verificación criptográfica completa, es un sello rápido de integridad.

**El evento génesis** (el primero de la cadena, sin predecesor) tiene `auditChainHash` vacío en la BD. El binding Dart lo recibe como `String` vacío (Rust hace `unwrap_or_default()`). En la UI se muestra como `"——genesis"` para distinguirlo visualmente de un error.

**En STORY-015** (`ui/lib/tabs/audit_tab.dart`):
```dart
String _hashCorto(String hash) {
  if (hash.isEmpty) return '——genesis';
  return hash.length > 8 ? hash.substring(hash.length - 8) : hash;
}
```

---

### `BigInt` vs `int` en Dart para tipos enteros de Rust

**El problema teórico:** `u64` de Rust (0 a 18.446.744.073.709.551.615) es más grande que el `int` nativo de Dart (con signo, máximo ≈9.2 × 10^18). Un `u64` con valor > 2^63 − 1 no cabe en un `int` de Dart y causaría desbordamiento silencioso.

**La solución sería `BigInt`:** Dart tiene el tipo `BigInt` para enteros arbitrariamente grandes. `flutter_rust_bridge 2.x` convierte automáticamente `u64` de Rust a `BigInt` de Dart cuando lo detecta en la firma.

**Por qué no fue necesario en STORY-015:** el Bridge-Engineer eligió `i64` (con signo) para `limit` y para los campos de timestamp. `i64` cabe perfectamente en un `int` de Dart. Los bindings tienen `required int limit` — no `required BigInt limit`. Esta decisión simplifica el lado Dart a cambio de un rango máximo teórico menor (que en la práctica nunca se alcanzará: nadie tiene 9 × 10^18 eventos de auditoría).

---

### Tests Flutter con `testWidgets` y `pumpWidget`

**Por qué no se necesita dispositivo real:** Flutter separa el motor de renderizado (C++/GPU) del framework de widgets (Dart). Los tests de widget usan un motor simulado en memoria que puede hacer layout, pintar y simular gestos sin GPU real, sin sistema operativo, sin pantalla.

**`testWidgets((WidgetTester tester) async { ... })`:** registra un test de widget. `WidgetTester` es el objeto que controla el motor simulado. El test es `async` porque las operaciones como `pumpWidget` y `tap` pueden necesitar esperar frames.

**`pumpWidget(widget)`:** monta el widget en el motor simulado y ejecuta un frame completo. Si cualquier widget en el árbol lanza una excepción en `build()`, `pumpWidget` la propaga y el test falla con el stack trace exacto. Es la forma de verificar "¿se rompe algo al renderizar?".

**`pump()`:** ejecuta un frame de animación/evento pendiente. Necesario después de `tester.tap()` para que Flutter procese el tap y complete la transición antes de hacer assertions.

**Por qué stubs en lugar de widgets reales:** los widgets reales de STORY-015 llaman al Bridge en `initState()`. En el entorno de test, la librería nativa no está cargada (`RustLib.init()` no se ha llamado). Los widgets stub replican la estructura (mismas pestañas, mismos textos) sin tocar FFI.

**En STORY-015** (`ui/test/panel_smoke_test.dart`):
```dart
testWidgets('panel_operativo_renders_three_tabs', (tester) async {
  await tester.pumpWidget(const MaterialApp(home: _PanelStub()));
  expect(find.text('Reloj'), findsOneWidget);
  expect(find.text('Trabajos'), findsOneWidget);
  await tester.tap(find.text('Trabajos'));
  await tester.pump();
  expect(find.text('trabajos-stub'), findsOneWidget);
});
```

---

## Trucos de Senior

**`Timer.periodic` + `dispose()` — patrón de ciclo de vida seguro:**
```dart
// En initState():
_timer = Timer.periodic(const Duration(seconds: 1), (_) => _actualizar());

// En dispose():
_timer?.cancel();  // El operador ?. evita NPE si initState() falló antes de asignar el timer
super.dispose();   // SIEMPRE llama al super al final de dispose(), nunca al principio
```

**`snapshot.data ?? []` — lista vacía como fallback seguro:**
`snapshot.data` es `null` antes de que el Future resuelva. El operador `??` evita un NPE: si es null, usamos lista vacía. El `ListView.builder` con `itemCount: 0` renderiza un widget vacío sin errores.

**Timestamps en nanosegundos → `DateTime` de Dart:**
```dart
// Dart no tiene fromNanosecondsSinceEpoch — usa microsegundos con división entera
DateTime.fromMicrosecondsSinceEpoch(timestampNs ~/ 1000).toUtc()
// ~/ es división entera en Dart (equivalente a (int)(a / b) en C)
```

**`const` en `TabBar` y sus hijos:** `Tab`, `Icon`, `Text` con literales son todos `const`. Marcarlos ahorra allocaciones en cada reconstrucción del `AppBar`. El árbol entero del `TabBar` se compila a constantes — Flutter ni lo toca en reconstrucciones.
