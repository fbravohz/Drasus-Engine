# ui/ — Cáscara Flutter de Drasus Engine

## 1. Resumen

`ui/` es la interfaz de Drasus Engine: una cáscara Flutter/Impeller
(sin lógica de negocio) que se comunica con el núcleo Rust a través
de `flutter_rust_bridge`. Implementa el Panel Operativo (reloj,
trabajos, auditoría, galería de componentes) y la galería de diseño.

Relación con el bridge Rust:

```
Rust (crates/bridge) ──[flutter_rust_bridge codegen]──► ui/lib/src/rust/
                                ↓
                     ui/lib/ (Flutter/Dart)
```

El codegen lee `crates/bridge/src/api/` y genera los archivos
`ui/lib/src/rust/frb_generated*.dart`. Nunca se editan a mano.

---

## 2. Prerrequisitos / toolchain

Versiones verificadas en este repositorio:

| Herramienta | Versión |
|---|---|
| Flutter | 3.44.2 (channel stable) |
| Dart | 3.12.2 |
| DevTools | 2.57.0 |
| Rust / Cargo | 1.96.0 |
| flutter_rust_bridge_codegen | 2.12.0 |

Obtener versiones en tu máquina:

```bash
flutter --version
rustc --version
cargo --version
flutter_rust_bridge_codegen --version
```

---

## 3. Instalar dependencias

### Dependencias Dart

```bash
cd ui
flutter pub get
```

### Instalar el generador de bindings del bridge

```bash
cargo install flutter_rust_bridge_codegen --version 2.12.0
```

Si ya está instalado, verifica que sea la versión 2.12.0:

```bash
flutter_rust_bridge_codegen --version
```

---

## 4. Compilar el bridge Rust y generar bindings

El bridge produce una librería dinámica (`libbridge.so` en Linux) y
los bindings Dart que la app importa.

### Paso 1 — Compilar la librería Rust

Desde la raíz del repositorio (no desde `ui/`):

```bash
cargo build -p bridge
```

Esto genera `target/debug/libbridge.so`.

### Paso 2 — Generar los bindings Dart

Desde la raíz del repositorio, con el `flutter_rust_bridge.yaml`
como referencia:

```bash
flutter_rust_bridge_codegen generate
```

El archivo `flutter_rust_bridge.yaml` en la raíz declara:

```yaml
rust_root: "crates/bridge"
rust_input: "crate::api"
dart_output: "ui/lib/src/rust"
```

Los archivos resultantes (`frb_generated.dart`, `frb_generated.io.dart`)
se depositan en `ui/lib/src/rust/`. El archivo `frb_generated.web.dart`
es generado para el target web; contiene 2 errores de compilación
preexistentes del codegen que son ajenos — no tocar.

---

## 5. Ejecutar la app completa (con bridge)

Requiere haber completado el paso 4 (bridge compilado y bindings
generados).

```bash
cd ui
flutter run -d linux
```

Solo Linux está configurado como target de escritorio en este
repositorio (existe la carpeta `ui/linux/`). Los targets macOS,
Windows, Android, iOS y web no están habilitados aún (ver §6).

---

## 6. Dependencias del sistema por SO

### Linux (target configurado)

Flutter Desktop en Linux requiere estas librerías de sistema.
Verificadas en Ubuntu 24.04 (el entorno de desarrollo activo):

| Paquete | Versión instalada |
|---|---|
| `libgtk-3-dev` | 3.24.41-4ubuntu1.3 |
| `ninja-build` | 1.11.1-2 |
| `clang` | 18.0 |
| `cmake` | 3.28.3 |
| `pkg-config` | 1.8.1 |

Instalar en Ubuntu/Debian:

```bash
sudo apt-get install libgtk-3-dev ninja-build clang cmake pkg-config
```

### macOS — no configurado

La carpeta `ui/macos/` no existe. Para habilitar este target:

```bash
flutter create --platforms=macos .   # desde ui/
```

Requisitos adicionales: Xcode 14+, CocoaPods. ⚠️ No verificado.

### Windows — no configurado

La carpeta `ui/windows/` no existe. Para habilitar:

```bash
flutter create --platforms=windows .   # desde ui/
```

Requisito: Visual Studio 2022 con el workload "Desktop development
with C++". ⚠️ No verificado.

### Android — no configurado

La carpeta `ui/android/` no existe. Para habilitar:

```bash
flutter create --platforms=android .   # desde ui/
```

Requisitos: Android SDK, Android NDK (para compilar Rust con
`cargo-ndk`). ⚠️ No verificado.

### iOS — no configurado

La carpeta `ui/ios/` no existe. Para habilitar:

```bash
flutter create --platforms=ios .   # desde ui/
```

Requisito: macOS con Xcode 14+. ⚠️ No verificado.

---

## 7. Tests

### Test de humo de la galería

Verifica que el árbol completo de componentes de diseño construye
sin excepciones. No requiere el bridge Rust.

```bash
cd ui
flutter test test/gallery_smoke_test.dart
```

### Golden tests (regresión visual)

Compara la galería renderizada contra imágenes de referencia PNG.
Las fuentes reales (Space Grotesk, Inter, JetBrains Mono) se cargan
con `FontLoader` en el `setUpAll`, de modo que el texto aparece como
glifos legibles y no como cajas.

```bash
# Ejecutar comparación contra los goldens existentes:
cd ui
flutter test test/gallery_golden_test.dart

# Regenerar los PNG de referencia (tras cambios visuales intencionados):
cd ui
flutter test --update-goldens test/gallery_golden_test.dart
```

Los PNG viven en `ui/test/goldens/` y están versionados en git:

- `gallery_full_scroll.png` — viewport 1440×5000 (galería completa)
- `gallery_fundamentos.png` — viewport 1200×900 (sección superior)

### Análisis estático

```bash
cd ui
flutter analyze
```

**Errores preexistentes ajenos** (no introducidos por este proyecto,
no tocar):

- `lib/src/rust/frb_generated.web.dart:111` — `undefined_class 'RustLibWasmModule'`
- `lib/src/rust/frb_generated.web.dart:113` — `experiment_not_enabled`
- `test/widget_test.dart:16` — `creation_with_non_type 'MyApp'`

El total de 84 avisos es el baseline conocido. Los 3 errores son del
código generado automáticamente y del test placeholder de Flutter.

### Panel de smoke (opcional)

```bash
cd ui
flutter test test/panel_smoke_test.dart
```

---

## 8. Galería de componentes (preview de diseño)

La galería es un catálogo visual con datos hardcodeados. No llama al
bridge Rust y puede correrse de forma completamente aislada.

```bash
cd ui
flutter run -d linux -t lib/gallery/gallery_preview_main.dart
```

Esto lanza solo la galería, sin el Panel Operativo ni el bridge.
Útil para iterar sobre diseño sin compilar Rust.

Las imágenes de referencia se encuentran en `ui/test/goldens/`.

---

## 9. Compilar release

Solo el target Linux está configurado:

```bash
cd ui
flutter build linux
```

El binario se genera en `build/linux/x64/release/bundle/drasus_ui`.

Los targets macOS (`flutter build macos`), Windows (`flutter build
windows`), Android (`flutter build apk`) e iOS (`flutter build ios`)
requieren habilitar primero la plataforma (ver §6). ⚠️ No verificados.

---

## 10. Estructura de carpetas

```
ui/
├── assets/
│   └── fonts/                     # Fuentes embebidas (offline-first)
│       ├── SpaceGrotesk-Medium.ttf    68 KB — títulos w500
│       ├── Inter-Regular.ttf         318 KB — UI w400
│       ├── Inter-Medium.ttf          318 KB — UI w500
│       ├── JetBrainsMono-Regular.ttf 110 KB — monoespaciado w400
│       └── JetBrainsMono-Medium.ttf  2.4 MB — monoespaciado w500 (NerdFont;
│                                              reemplazar con la versión limpia
│                                              de fonts.google.com/specimen/
│                                              JetBrains+Mono para reducir a ~110 KB)
├── lib/
│   ├── main.dart                  # Punto de entrada principal (con bridge)
│   ├── operational_panel.dart     # Shell del Panel: tabs clock/jobs/audit/gallery
│   ├── tabs/                      # Una pestaña por archivo (clock, jobs, audit)
│   ├── src/
│   │   └── rust/                  # Bindings generados por flutter_rust_bridge (NO editar)
│   │       ├── frb_generated.dart
│   │       ├── frb_generated.io.dart
│   │       └── frb_generated.web.dart
│   └── gallery/                   # Galería de diseño (render-only, sin bridge)
│       ├── gallery_preview_main.dart   # Punto de entrada aislado de la galería
│       ├── gallery_tab.dart            # Widget raíz de la galería
│       ├── gallery_tokens.dart         # Tokens de diseño: todos los hex viven aquí
│       ├── gallery_fx.dart             # Widgets de efecto (glow, frosted, hover)
│       ├── gallery_painters.dart       # CustomPainters (telón, DAG, gráficos)
│       └── sections/                   # Una sección por archivo
│           ├── section_nav.dart        # §5 Navegación (pill, breadcrumbs, scrollspy…)
│           ├── section_inputs_extended.dart
│           ├── section_buttons_extended.dart
│           ├── section_data_display_extended.dart
│           ├── section_feedback_extended.dart
│           ├── section_dataviz_extended.dart
│           ├── section_drasus_core_extended.dart
│           └── section_std_missing.dart  # Piezas STD (cascader, dropzone, etc.)
├── linux/                         # Configuración de build Linux (único target activo)
├── test/
│   ├── gallery_smoke_test.dart    # Smoke test: galería construye sin excepciones
│   ├── gallery_golden_test.dart   # Golden tests con tipografía real
│   ├── panel_smoke_test.dart      # Smoke test del Panel Operativo
│   ├── widget_test.dart           # Placeholder de Flutter (3 errores preexistentes)
│   └── goldens/                   # PNG de referencia para los golden tests
│       ├── gallery_full_scroll.png
│       └── gallery_fundamentos.png
├── pubspec.yaml                   # Dependencias y declaración de fuentes
└── analysis_options.yaml          # Reglas de análisis estático
```

---

## 11. Fuentes y golden tests, explicados sin tecnicismos

Esta sección desarma dos temas que suenan complicados pero son simples.

### 11.1 ¿Qué es un archivo `.ttf` y por qué los "embebemos"?

Un `.ttf` (*TrueType Font*) es, literalmente, **el archivo de una tipografía**:
contiene el dibujo de cada letra y número. La app usa tres:

| Familia | Archivo | Para qué se usa |
|---|---|---|
| **Space Grotesk** | `SpaceGrotesk-Medium.ttf` | Títulos grandes (sabor técnico) |
| **Inter** | `Inter-Regular.ttf` / `Inter-Medium.ttf` | Texto normal de la interfaz |
| **JetBrains Mono** | `JetBrainsMono-Regular.ttf` / `-Medium.ttf` | Números e IDs (monoespaciada) |

**Embeber** = guardar esos `.ttf` dentro del proyecto (`ui/assets/fonts/`) y
declararlos en `pubspec.yaml`. Antes la app los **descargaba de internet** la
primera vez (con el paquete `google_fonts`). El problema: sin internet, no había
tipografía correcta. Ahora viajan dentro de la app → **funciona 100% offline**
y siempre se ve igual.

### 11.2 El pendiente del JetBrains Mono pesado

Hay un detalle a corregir. El archivo `JetBrainsMono-Medium.ttf` pesa **2.4 MB**
cuando debería pesar **~110 KB**. La razón: se copió por error la variante
*Nerd Font*, que trae miles de iconos extra que no usamos. **No rompe nada**, solo
infla el tamaño de la app. Para arreglarlo (cuando tengas internet):

1. Entra a `fonts.google.com/specimen/JetBrains+Mono`.
2. Descarga la familia y saca el archivo **`JetBrainsMono-Medium.ttf`** (el normal,
   *sin* "Nerd Font" en el nombre).
3. Reemplaza el archivo en `ui/assets/fonts/` con ese.
4. `cd ui && flutter pub get` y listo. No hay que tocar código: `pubspec.yaml` ya
   declara la familia con sus dos pesos (w400 y w500).

### 11.3 ¿Qué es un "golden test"?

Imagina que tomas una **foto de referencia** de cómo debe verse la pantalla y la
guardas. Un *golden test* hace justo eso: cada vez que corres los tests, la
herramienta **vuelve a renderizar la galería, le saca una foto nueva y la compara
píxel por píxel** contra la foto guardada. Si algo cambió sin querer (un color, un
espacio, un tamaño), el test **falla** y te avisa. Es un detector de cambios
visuales accidentales.

Las "fotos de referencia" son los PNG en `ui/test/goldens/`:
- `gallery_fundamentos.png` — la parte de arriba de la galería.
- `gallery_full_scroll.png` — la galería completa de arriba abajo.

Dos comandos, dos intenciones distintas:

```bash
# "¿Cambió algo sin que yo quisiera?" — compara contra las fotos guardadas:
flutter test test/gallery_golden_test.dart

# "Cambié el diseño A PROPÓSITO, actualiza las fotos de referencia":
flutter test --update-goldens test/gallery_golden_test.dart
```

> Regla práctica: corre el primero normalmente. Usa `--update-goldens` **solo**
> cuando hiciste un cambio visual intencional y quieres que esa nueva apariencia
> pase a ser la referencia.

### 11.4 Por qué antes el texto salía como "cajitas"

Detalle técnico que causó confusión: en los tests, Flutter **no carga las fuentes
automáticamente**, así que dibujaba cada letra como un rectángulo gris (una fuente
de relleno llamada *Ahem*). Por eso los primeros goldens parecían tener "cajitas"
en vez de texto.

Se arregló cargando los `.ttf` reales **dentro del propio test** (con un
`FontLoader` en el `setUpAll` de `gallery_golden_test.dart`). Resultado: los
goldens ahora muestran la **tipografía real legible**, igual que se vería la app.
