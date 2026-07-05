> ✅ **Implementado** 2026-06-27 · Orden de trabajo [STORY-024](../execution/STORY-024-sovereign-data-fetcher.md) · TTR-001 + TTR-002. Crate: `crates/features/data/sovereign-data-fetcher/`.

# Sovereign Data Fetcher

**Carpeta:** `./features/sovereign-data-fetcher/`
**Estado:** En Diseño
**Última actualización:** 2026-04-13
**Decisión Arquitectónica Asociada:** ADR-0034 (Ingesta Híbrida Soberana)

## ¿Qué es?

Es el componente encargado de saturar el ancho de banda para la obtención masiva de históricos. Resuelve el problema de la lentitud de las APIs REST (que son 100x más lentas) mediante una estrategia híbrida: descarga archivos comprimidos masivos (Bulk) y usa la API solo para el "Delta" final (datos recientes).

**Problema:** Descargar 5 años de datos por API REST puede tomar días y causar bloqueos por Rate Limit.
**Solución:** Descargar volcados mensuales de S3 en segundos y rellenar los últimos minutos vía API.

## Comportamientos Observables

- [ ] Usuario solicita histórico de BTC de 2020 a hoy.
  - El sistema identifica volcados en `data.binance.vision`.
  - Descarga archivos `.zip` concurrentemente usando todos los hilos disponibles.
  - Al terminar, conecta con la API REST para descargar las barras que faltan desde el último volcado hasta el "ahora".
- [ ] La interfaz muestra una barra de progreso indicando "Descargando Bulk (80%)" y luego "Sincronizando Delta (100%)".
- [ ] Si un archivo Bulk falla, el sistema intenta descargarlo de nuevo automáticamente.

## Restricciones

- NUNCA se usa la API REST para periodos que ya existen en volcados Bulk.
- NUNCA se inicia la ingesta si el espacio en disco es insuficiente para el tamaño estimado del Bulk.
- La descarga debe ser asíncrona y no bloquear el hilo principal de la aplicación.

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| CONCURRENT_DOWNLOADS | 5 | 1 - 20 | Cuántos archivos descargar simultáneamente | CONFIG |
| BULK_SOURCE_URL | Binance Vision | - | URL base para buscar volcados S3 | [FIJO] |
| DELTA_SYNC_RETRY | 3 | 1 - 10 | Reintentos para la sincronización REST | CONFIG |

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** Algoritmos de reconciliación de timestamps, detección de huecos (Gaps) y priorización de descargas en `fetcher_core.rs`.
- **Shell (Infraestructura):** Cliente HTTP asíncrono, descompresor de archivos y gestión de sistema de archivos local.
- **Frontera Pública:** Contrato `fetch_data(symbol, timeframe, range)`.

## Ciclo de Vida de la Feature — Sovereign Data Fetcher

### Entrada
- Símbolo (ej: BTCUSDT), Intervalo (1m), Rango de fechas.
- Credenciales de API (solo para Delta).

### Proceso
- Consulta el inventario de archivos Bulk en el servidor remoto.
- Descarga y descomprime archivos en segundo plano.
- Identifica el punto de corte (último timestamp del Bulk).
- Solicita formalmente el Delta a la API REST del broker.

### Salida
- Stream de datos crudos (CSV/JSON) listos para la capa de normalización.
- Reporte de éxito/fallo por cada bloque temporal.

### Contextos de Uso

**Contexto 1: Ingesta Inicial (Hydro-Ingest)**
- El sistema descarga años de historia para alimentar la generación de alfas (Ingest).

**Contexto 2: Reconexión Live**
- Si el sistema se apaga 2 horas, el fetcher usa la API Delta para rellenar el hueco sin intervención humana.

## Tareas (TTRs)

### **TTR-001: Descargador Asíncrono de Bulk (S3)**
- Implementa la lógica de descarga concurrente de archivos comprimidos optimizada para ancho de banda alto.

### **TTR-002: Reconciliador de Delta (REST)**
- Implementa la conexión con la API REST para descargar el segmento de datos faltante entre el Bulk y el presente.

### **TTR-003: Alternative Data Webhook Listener**
- **Qué tiene que pasar:** Implementar un receptor local de HTTP Webhooks en Rust que exponga un puerto seguro. Permite a plataformas como n8n y Zapier inyectar datos alternativos (ej: puntajes de sentimiento, feeds de noticias de impacto) como señales en tiempo real para el generador y motor de ejecución.

### **TTR-004: Alternative Time-Series Converter (Backtestable Data)**
- **¿Cuál es el problema?** Los datos alternativos asíncronos (sentimiento de mercado, noticias fundamentales, análisis macroeconómicos) no son útiles para investigar si no se estructuran históricamente, impidiendo su backtesting.
- **¿Qué tiene que pasar?** Implementar un alineador en Rust que normalice, indexe y asocie eventos asíncronos alternativos a las marcas de tiempo Point-in-Time (PIT) de las velas de mercado en los archivos Parquet locales (Hive-Style), haciéndolos completamente backtesteables sin sesgo de look-ahead.
- **¿Cómo sé que está hecho?**
  - [ ] Puedo correr un backtest que cargue columnas de sentimiento histórico y verificar que el motor reaccione con precisión determinista a eventos pasados.

---

## Puertos de Integración

> *(ADR-0137)* El Sovereign Data Fetcher es un **nodo fuente** del pipeline: no recibe datos de otro nodo del canvas — su "entrada" es una solicitud de configuración del usuario (símbolo, intervalo, rango), no un tipo de dato cableable. Produce los datos crudos que alimentan a la capa de normalización/sanitización.

| Puerto | ID de tipo | Dirección | Cardinalidad | Descripción |
|---|---|---|---|---|
| `ticks_out` | `Tick` | Output | 0..N | Transacciones crudas Bid/Ask/Last descargadas (volcados de trades + Delta REST) |
| `bars_out` | `Bars` | Output | 0..N | Barras OHLCV crudas cuando la fuente entrega volcados de klines directamente |

> El catálogo de tipos de ADR-0137 declara a `sovereign-data-fetcher` como feature productora canónica de `Tick`. Las dos salidas son mutuamente excluyentes por solicitud: una descarga de trades produce `Tick`; una descarga de klines produce `Bars`.

## Contrato de Integración UI (ADR-0117 / ADR-0136)

**Superficie propia: Inspector Panel** (nodo fuente del canvas). Como produce tipos de dominio (`Tick`/`Bars`) y recibe configuración del usuario, NO es plomería: al hacer clic en su nodo del Forge se abre un inspector panel lateral (ADR-0136) con su UI de configuración:

- Selector de **broker/exchange** (ej. Binance Vision).
- Selector de **símbolo** (ej. BTCUSDT).
- **Rango de fechas** (desde / hasta).
- **Timeframe/intervalo** (1m, 5m, 1h, 1D, …) y tipo de salida (trades → `Tick` / klines → `Bars`).
- Disparador de descarga + estimación de tamaño y verificación de espacio en disco.

UX complementaria provista por features hermanas del módulo `ingest` (NO por esta feature):
- **Progreso de descarga** (barras "Bulk %/Delta %") → [`background-download-manager`](background-download-manager.md) (`ui_progress_updates`).
- **Exploración visual del dataset** descargado → [`canvas-navigation`](canvas-navigation.md).

**Estado de entrega:** el MOTOR de descarga (TTR-001 Bulk + TTR-002 Delta) se implementó en STORY-024 (backend, QA APTO). El **inspector panel de configuración es una entrega de UI pendiente** (Story de UI futura: UI-Designer escribe la Cáscara Visual ADR-0135 → Bridge → Flutter) — aún no construida.

## Cáscara Visual (Thin Shell)

> Autoridad: ADR-0106 · ADR-0136 · ADR-0117 · `docs/DESIGN.md` · `docs/DESIGN.md §"Catálogo de Componentes"`

> **Alcance STORY-024 — Opción B ratificada:** se diseñan las manifestaciones (1) SVF en el Banco de Verificación y (2) Dashboard widget. La manifestación (3) nodo Canvas DAG queda como **deuda de integración visual** — la infraestructura Canvas (drag-drop, puertos tipados, bezier S-curve) no existe en EPIC-1. El Tech Lead debe registrarla en `PROGRESS.md` y resolverla al construir la infra Canvas en EPIC-8, conforme ADR-0117 / ADR-0136 (enmienda 2026-06-28).

---

### Contexto de superficie (ADR-0136)

| # | Manifestación | Contexto (ADR-0136) | Descripción en una frase |
|---|---|---|---|
| 1 | SVF — sección del Banco de Verificación | Sección seleccionable en el Banco de Verificación (panel-solid denso, tab único del shell operacional) | El usuario configura broker, símbolo, rango, timeframe y tipo de salida, dispara el job real vía FFI y ve el resultado del job más el historial persistido de `sovereign_download_records`. |
| 2 | Dashboard widget | **Dashboard widget** — read-only en el centro de monitoreo | Tarjeta compacta que muestra la última descarga: símbolo, timestamp, bytes totales y estado del job. |
| 3 · deuda | Nodo Canvas DAG + Inspector Panel lateral | **Inspector Panel** (definitivo — diferido a EPIC-8) | Al hacer clic en el nodo del Canvas [Forge/Reactor], abre el inspector panel lateral con la UI de configuración completa — no construida hasta que exista la infra Canvas. |

---

### Superficie y Densidad

**SVF (Banco de Verificación):**
- Superficie principal: `panel-solid` — `panelSurface(child: ...)` leyendo `Gx.surfacePanel`. Radio 11px, borde `Gx.borderBase` (1px hairline).
- Densidad: **densa** — contexto de verificación operacional. Padding perimetral `Gx.space16`. Gaps entre zonas `Gx.space8`.
- Lienzo de fondo del Banco: `deepSpace` (`Gx.canvasBase`) + telón cósmico tenue (`starField #E6ECF8` @ 2–5%).

**Dashboard widget:**
- Superficie: `panel-solid` compacto — `panelSurface(child: ...)`. Radio 11px.
- Densidad: **máxima** (bento grid). Padding `Gx.space12`.

---

### Componentes

#### Zona A — Panel de Control (SVF, controles de entrada)

| ID de catálogo (DESIGN.md §4–§11) | Rol en esta feature | Tokens clave | Estados semánticos |
|---|---|---|---|
| `panel-solid` | Contenedor de los controles de entrada | `Gx.surfacePanel`, `Gx.borderBase`, radio 11px | — |
| `select / dropdown` (`GlowDropdown`) | Selector de Broker/Exchange (Binance Vision, …) | `glassFill` superficie; foco: borde 1.5px + `glow(transitionIndigo, blur 18)`; chevron rota 180° al abrir | default, focus, selected, disabled |
| `text-field / input` (`GlowInput`) | Campo de símbolo (ej. BTCUSDT) — texto libre, uppercase | `glassFill`; foco: borde 1.5px `transitionIndigo` + `glow(transitionIndigo, blur 18)`; error: borde `criticalCrimson` | default, focus, error (`criticalCrimson` si vacío al disparar) |
| `date-picker` (`GlowDatePicker`) × 2 | Fecha Desde / Fecha Hasta — dos instancias lado a lado | `glassFill`; día seleccionado: anillo `optimaCyan`; día actual: anillo `transitionIndigo` | default, focus, error (`criticalCrimson` si Desde ≥ Hasta) |
| `select / dropdown` (`GlowDropdown`) | Selector de Timeframe/Intervalo (1m, 5m, 15m, 1h, 4h, 1D, 1W) | mismos tokens que selector de broker | default, focus, selected |
| `segmented-control` (`GlowSegmented`) | Tipo de salida: **Trades → Tick** / **Klines → Bars** | Segmento activo: `glassFill` + borde neón `transitionIndigo` + texto `Gx.textBase`; inactivo: texto `Gx.textBaseMuted` | selected |
| `button-primary` (`GlowButton`) | Botón "Descargar" — dispara `submitDownloadJob(…)` vía FFI | Relleno `gradReactor`, texto `deepSpace #080A18`, `glowStrong(reactorGreen)`; al hover glow se intensifica; al press escala 0.96 + propagación de luz ~460ms; en loading glow pulsante `transitionIndigo` | default, hover, pressed, loading, disabled (cuando hay job activo) |

#### Zona B — Panel de Resultados (SVF, job activo)

| ID de catálogo (DESIGN.md §4–§11) | Rol en esta feature | Tokens clave | Estados semánticos |
|---|---|---|---|
| `panel-solid` | Contenedor del estado del job activo o último job | `Gx.surfacePanel`, `Gx.borderBase`, radio 11px | — |
| `key-value-row` (`_keyValue`) × 4 | Job ID · Estado · Archivos descargados · Bytes totales | Label: `Gx.textBaseLabel` (textLabel @55%) alineado izq; valor: `dataMono 13px` alineado der; separador `divider`; color del valor = color del estado | según estado del job (ver §Estados Semánticos) |
| `badge / tag / chip / pill` (`_chip`) | Estado del job activo en el key-value de Estado | Texto, fondo, borde y radio según §Estados Semánticos | Completado / En progreso / En cola / Reintentando / Fallido |
| `spinner / loader` (`_scanRing`) | Indicador de descarga activa — solo visible mientras estado = En progreso | `scanRing(transitionIndigo)` ritmo medio 2–3s | solo cuando estado = running |

#### Zona C — Historial de Descargas (SVF, registros persistidos de la DB)

| ID de catálogo (DESIGN.md §4–§11) | Rol en esta feature | Tokens clave | Estados semánticos |
|---|---|---|---|
| `panel-solid` | Contenedor del historial `sovereign_download_records` | `Gx.surfacePanel`, `Gx.borderBase`, radio 11px | — |
| `table / data-grid` (`GlowTable`) | Tabla de registros: id · created_at · símbolo · bytes · estado · source_endpoint | Cabecera 11px `Gx.textBaseLabel`; celdas `dataMono 13px`; hover fila `Gx.surfaceRaisedDynamic`; separador `divider`; números bytes alineados derecha | completado (`optimaCyan`) / fallido (`criticalCrimson`) / en progreso (`transitionIndigo`) |
| `badge / tag / chip / pill` (`_chip`) | Chip de estado en cada fila del historial | mismos tokens que §Estados Semánticos | todos los estados |
| `tooltip` (`GlowTooltip`) | Valor completo de `source_endpoint` al hover sobre celda truncada | `glassFill`, `glassRim`, radio 12px | — |
| `empty-state` (`GlowEmpty`) | Sin registros aún | Texto `Gx.textBaseMuted`; ícono cristal latente | — |

#### Manifestación 2 — Dashboard widget (read-only)

| ID de catálogo (DESIGN.md §4–§11) | Rol en esta feature | Tokens clave | Estados semánticos |
|---|---|---|---|
| `panel-solid` | Contenedor del widget en el bento grid | `Gx.surfacePanel`, radio 11px, padding `Gx.space12` | — |
| `stat / metric` (`_kpi`) | Bytes totales de la última descarga — número-héroe | `dataMono 28px`; color = estado del último job; `textGlow(estadoColor)` | óptimo (`optimaCyan`) / fallido (`criticalCrimson`) / en progreso (`transitionIndigo`) |
| `key-value-row` (`_keyValue`) × 2 | Símbolo descargado · Timestamp `created_at` | Label `Gx.textBaseLabel`; valor `dataMono 13px` | — |
| `badge / tag / chip / pill` (`_chip`) | Estado del último job registrado | mismos tokens que §Estados Semánticos | todos los estados |

---

### Estados Semánticos (Espectro de Vitalidad)

Mapeados al campo `status` del job de descarga y de `sovereign_download_records`:

| Estado de negocio | Color token | Tratamiento visual completo |
|---|---|---|
| Completado | `optimaCyan #54E8D0` | chip: texto `#54E8D0`, fondo `#08251F`, borde 1px `#1E5E4F`, radio 8px · `glow(optimaCyan)` · valor key-value: texto `#54E8D0` |
| En progreso | `transitionIndigo #9A8CFF` | chip: texto `#9A8CFF`, fondo `#130F2A`, borde 1px `#3A2E6E`, radio **999px** (estado vivo) · `glow(transitionIndigo)` pulsante · `scanRing(transitionIndigo)` en panel activo · valor key-value: texto `#9A8CFF` |
| En cola | `transitionBlue #56A8FF` | chip: texto `#56A8FF`, fondo `#0A1526`, borde 1px `#1A3A6E`, radio 8px · sin glow activo |
| Reintentando | `alertAmber #FFC94D` | chip: texto `#FFC94D`, fondo `#241900`, borde 1px `#5C3D00`, radio **999px** (estado vivo) · `glow(alertAmber)` pulsante |
| Fallido | `criticalCrimson #F0413F` | chip: texto `#F0413F`, fondo `#2A0C0C`, borde 1px `#7A2A28`, radio 8px · `glow(criticalCrimson)` parpadeante |

---

### Layout

**SVF — Banco de Verificación:**

```
┌──────────────────────────────────────────────────────────────────┐
│  Banco de Verificación  (tab único del shell operacional)        │
│  ┌─────────────┬────────────────────────────────────────────┐   │
│  │  Menú lat.  │  Sección: Sovereign Data Fetcher           │   │
│  │  (navRail)  │                                            │   │
│  │  · Reloj    │  ┌─── Zona A: Panel de Control ─────────┐ │   │
│  │  · Trabajos │  │  Broker [Dropdown]  Símbolo [Input]   │ │   │
│  │  · Auditoría│  │  Desde [DatePicker] Hasta [DatePicker]│ │   │
│  │  · Datos ←  │  │  Timeframe [Drop.]  [Trades|Klines]   │ │   │
│  │    Soberanos│  │              [  Descargar  ]           │ │   │
│  └─────────────┤  └───────────────────────────────────────┘ │   │
│                │  (Gx.space8)                                │   │
│                │  ┌─── Zona B: Job Activo ─────────────────┐│   │
│                │  │ Job ID: [mono]      Estado: [chip]      ││   │
│                │  │ Archivos: [mono]    Bytes: [mono]       ││   │
│                │  └────────────────────────────────────────-┘│   │
│                │  (Gx.space8)                                │   │
│                │  ┌─── Zona C: Historial ──────────────────┐│   │
│                │  │ id │ created_at │ símbolo │ bytes │ est ││   │
│                │  │ ── │ ────────── │ ─────── │ ───── │ ── ││   │
│                │  │ [fila hover → surfaceRaised]            ││   │
│                │  │ ...                    [scroll vertical]││   │
│                │  └────────────────────────────────────────-┘│   │
│                └────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────────┘
```

- **Menú lateral del Banco:** `navRail #0B1022`, entradas con ícono Iconsax + label `Gx.textBase 13px`. Entrada activa: borde izq 2px `transitionIndigo` + `glow(transitionIndigo)`. Sigue el patrón de `ui/lib/gallery/` (registro centralizado, construcción bajo demanda).
- **Zona A — controles:** `Column` con tres filas de inputs. Cada fila es `Row([Expanded, SizedBox(Gx.space12), Expanded])`. Botón "Descargar" en `Row` centrado al final. Padding `Gx.space16` perimetral. Gap entre filas de controles: `Gx.space12`.
- **Zona B — job activo:** `GridView.count(crossAxisCount: 2, childAspectRatio: 4.0)` con 4 `_keyValue`. Padding `Gx.space12`. Solo visible cuando `_jobId != null`. Separación de la Zona A: `Gx.space8`.
- **Zona C — historial:** `GlowTable` dentro de `Expanded` que toma el resto del alto disponible. Scroll vertical. Columnas — proporciones flex: `id` (1, truncado 8 chars), `created_at` (2, ISO 8601 mono), `símbolo` (1), `bytes` (1, alineado der), `estado` (1, chip), `source_endpoint` (2, truncado + tooltip). Separación de la Zona B: `Gx.space8`.

**Dashboard widget:**
- `Column(children: [header, heroStat, keyValueRows, chipEstado])`.
- Header: ícono Iconsax `cloud_download` 14px `Gx.textBaseLabel` + label `"Datos Soberanos"` en `displayGrotesque 12px 500` `Gx.textBaseLabel`.
- Número-héroe: bytes totales en `dataMono 28px` con color y `textGlow(estadoColor)`.
- Dos `key-value-row`: símbolo + `created_at`.
- Chip de estado al pie.
- Alto mínimo: 110px. Sin scroll.

---

### Animaciones Aplicables

Seleccionadas de `docs/DESIGN.md §Motion Philosophy`:

- [x] **Clic de botón "Descargar":** hundimiento (escala 0.96) + propagación de luz (~460ms) — comportamiento estándar de `GlowButton`
- [x] **Hover en controles** (inputs, dropdowns, botón): `glowStrong` intensificado + leve escala (~160–220ms)
- [x] **Foco de input** (símbolo, fechas): borde 1.5px `transitionIndigo` + glow limpio (~200ms), sin aberración RGB
- [x] **Dropdown** (broker, timeframe): `AnimatedSize` + rotación del chevron 180°
- [x] **Chip pulsante / parpadeante:** glow pulsante `transitionIndigo` (En progreso) / `alertAmber` (Reintentando) / parpadeo `criticalCrimson` (Fallido)
- [x] **Loader activo:** `scanRing(transitionIndigo)` ritmo medio 2–3s mientras job corre
- [ ] Zoom canvas: no aplica (sin Canvas en EPIC-1)
- [ ] Switch: no aplica
- [ ] Slider: no aplica

---

### Notas de implementación para el Flutter Engineer

**1. Patrón de widget y polling:**
`SovereignDataFetcherSection` como `StatefulWidget`. Usar `Timer.periodic(const Duration(seconds: 2), _actualizarEstadoJob)` mientras `_jobId != null && !_jobTerminado`. Cancelar en `dispose()`. Patrón idéntico a `ui/lib/tabs/clock_tab.dart`.

**2. `GlowDatePicker` × 2 (no `GlowDateRangePicker`):**
`GlowDateRangePicker` está en `gallery/sections/section_std_missing.dart` — pendiente de implementación. Usar dos instancias de `GlowDatePicker({initial, onSelected})` lado a lado. Validar `_fechaDesde.isBefore(_fechaHasta)` en cada callback; si falla, pintar el borde de la segunda instancia en `criticalCrimson` y deshabilitar "Descargar".

**3. `GlowSegmented` para tipo de salida:**
```dart
// Conmutador Trades/Klines — mapeo interno al enum de Rust
GlowSegmented(
  options: const ['Trades (Tick)', 'Klines (Bars)'],
  selected: _outputTypeIndex,
  onChanged: (i) => setState(() => _outputTypeIndex = i),
)
// 0 → OutputType.ticks,  1 → OutputType.bars
```

**4. Superficie dinámica — obligatorio:**
Todo contenedor usa `panelSurface(child: ...)`. Prohibido `Container(color: Gx.panelSolid)` raw. Colores de texto: `Gx.textBase`, `Gx.textBaseLabel`, `Gx.textBaseMuted`. Prohibido `const` en cualquier widget de superficie (congela colores al cambiar el modo global).

**5. `GlowTable` — columna `source_endpoint`:**
Truncar a 40 caracteres con `"…"`. Envolver en `GlowTooltip({message: record.sourceEndpoint, child: Text(truncated)})`.

**6. Sidebar del Banco de Verificación:**
Añadir entrada al menú lateral del Banco con `IconsaxPlusLinear.cloud_download`, label `"Datos Soberanos"`. Entrada seleccionada: borde izq 2px `transitionIndigo` + `glow(transitionIndigo)`. Seguir el patrón de registro existente en `ui/lib/gallery/`.

**7. Registro en `dashboard_registry.dart`:**
Añadir con `available: true`:
```dart
// Widget de última descarga de históricos soberanos
DashboardWidgetMeta(
  id: 'sovereign-data-fetcher',
  name: 'Datos Soberanos',
  description: 'Estado de la última descarga: símbolo, bytes y estado del job.',
  icon: IconsaxPlusLinear.cloud_download,
  available: true,
)
```
El widget implementado se llama `SovereignDataFetcherDashboardWidget` — read-only, sin callbacks.

**8. Estado vacío del historial:**
Si `listDownloadRecords()` retorna lista vacía, mostrar `GlowEmpty({message: 'Sin descargas aún.', icon: IconsaxPlusLinear.cloud_download})` en la Zona C.

**9. Techo Fijo (ADR-0117):**
Sin manejo elaborado de errores (un `GlowBanner(type: error)` en Zona B es suficiente para fallos FFI). Sin paginación de la tabla en EPIC-1. Sin validación inline en tiempo real salvo la restricción Desde < Hasta. El botón "Descargar" se deshabilita únicamente mientras haya un job en estado `running` o `queued`.

---

### Despacho al Bridge Engineer — Métodos FFI necesarios

Antes de que el Flutter Engineer implemente esta sección, el Bridge Engineer debe exponer:

| Método FFI | Firma sugerida | Descripción |
|---|---|---|
| `submitDownloadJob` | `({String symbol, String broker, DateTime startDate, DateTime endDate, String timeframe, OutputType outputType}) → String` | Dispara el job y retorna el `job_id` asignado por el Core. |
| `getJobStatus` | `({String jobId}) → JobStatus` | Retorna el estado actual: `state` (enum queued/running/retrying/completed/failed), `files_downloaded`, `bytes_total`. |
| `listDownloadRecords` | `() → List<DownloadRecord>` | Lee `sovereign_download_records` de la DB; lista completa (sin paginación en EPIC-1). |

`JobStatus.state` es el enum Rust que mapea a los cinco estados semánticos de §Estados Semánticos.

---

### Correcciones de violaciones (pre-diseño)

Ninguna violación detectada. El documento usa correctamente la terminología de ADR-0136 (Inspector Panel), no los términos descontinuados MACRO/MESO/MICRO. No hay menciones de WebGL, cálculo en frontend ni tecnologías rechazadas.

---

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016):** 100% Local (los datos se descargan y procesan en el disco del usuario).
- **Fidelidad (ADR-0017):** Alta (maneja Ticks y Barras de 1M).
## Persistencia (Inundación de Fundamentos — ADR-0020)

Cada descarga registra el set de relevancia técnica para Datos:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único del job de descarga |
| | `created_at` | Timestamp de inicio |
| | `updated_at` | Timestamp de última modificación del registro |
| | `audit_hash` | Hash de integridad del archivo comprimido |
| | `audit_chain_hash` | Hash de la secuencia de descarga |
| | `event_sequence_id` | Secuencia de recuperación (event-sourcing) |
| **III. Linaje** | `data_snapshot_id` | Referencia al snapshot/volcado del broker que originó el segmento descargado |
| | `logic_hash` | Hash del driver del fetcher que produjo el registro |
| **IV. Hardware** | `node_id` | ID del hardware físico donde se ejecutó la descarga |
| | `process_id` | PID del worker de descarga |
| **Campo propio (fuera del catálogo de 25)** | `source_endpoint` | URL/endpoint de la fuente Bulk o REST de la que provino el dato — provenance obligatorio para la soberanía de datos |

> **Nota de perfil (ADR-0020):** esta tabla es Perfil A (Datos) — usa Grupo I (universal) + Grupo III (Linaje) + Grupo IV (Hardware). El tiempo total de descarga **no** se persiste aquí (sería un campo de Grupo V, ajeno al Perfil A): la duración y el progreso del trabajo de descarga los lleva el registro del trabajo asíncrono (tipo `Job` de `async-job-executor`) y la telemetría del `background-download-manager`. `source_endpoint` es un campo propio de dominio (provenance), fuera del catálogo de 25, justificado por la soberanía de datos.
