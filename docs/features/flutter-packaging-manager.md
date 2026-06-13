# Flutter FFI Packaging Manager

**Carpeta:** `./features/flutter-packaging-manager.md`
**Estado:** En Diseño
**Última actualización:** 2026-05-01
**Decisión Arquitectónica Asociada:** ADR-0001 (Monolito Modular), ADR-0029 (Patrón Todo en Uno)

## ¿Qué es esta feature?

El **Manejador de Empaquetado de Flutter FFI** orquesta el ciclo de vida del binario congelado de Rust (backend + assets frontend) utilizando Flutter FFI para renderizar la interfaz de usuario. Permite que Drasus Engine se distribuya como una única aplicación ejecutable sin requerir que el usuario instale Rust, CUDA o dependencias de sistema manualmente, con un instalador profesional por sistema operativo.

## Comportamientos Observables

- [ ] Al abrir Drasus Engine.exe/.app/AppImage, Flutter FFI inicia el loop del backend y el visor UI.
- [ ] Detección automática de GPU al arranque: fallback automático a CPU.
- [ ] Búsqueda de puerto libre dinámicamente para evitar colisiones en el arranque.
- [ ] En Windows: El instalador despliega automáticamente el Microsoft WebView2 Evergreen Bootstrapper si no está instalado.
- [ ] Al cerrar la ventana de la aplicación, el proceso de Rust se cierra limpia y automáticamente.

## Restricciones

- **NUNCA** exponer el puerto a redes externas (enlace obligatorio a `127.0.0.1`).
- **OBLIGATORIO:** El core de Rust debe estar compilado nativamente (a través de Cargo) en una librería de enlace dinámico o estático (`.dll`/`.so`/`.dylib`) para proteger la IP y optimizar el enlace directo vía FFI (`flutter_rust_bridge`).

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
| :--- | :--- | :--- | :--- | :--- |
| API_PORT | 8000 | 1024 - 65535 | Puerto base dinámico para la comunicación gRPC/WebSockets local | CONFIG |
| COMPILATION_PROFILE | release | release / debug | Perfil de compilación de Cargo para optimizaciones del compilador | [FIJO] |
| GPU_AUTO_DETECT | True | True/False | Si detecta hardware al inicio | [FIJO] |

## Estructura Interna (FCIS)

- **Core (Lógica Pura):** Módulo de detección de hardware GPU, selección dinámica de puerto libre y orquestación del ciclo de vida de la aplicación.
- **Shell (Infraestructura):** Generación de instaladores nativos (Inno Setup/NSIS para Windows, DMG packager para macOS, y AppImage Tool para Linux).

## Ciclo de Vida de la Feature — Packaging Manager

### Entrada
- Evento de inicio del binario nativo empaquetado.

### Proceso
1. **Dynamic Port Range Scanning:** Busca un puerto TCP disponible.
2. **Environment Setup:** Configura variables de entorno locales de ejecución.
3. **App Initialization:** Inicia el runtime del core Rust.
4. **Flutter FFI Startup:** Inicializa los canales binarios de comunicación en memoria compartida a través de `flutter_rust_bridge`.

### Salida
- Aplicación de escritorio operativa con el frontend renderizado sobre Flutter FFI.

## Tareas (TTRs)

### **TTR-001: Pipeline de Compilación Nativa Multiplataforma (Cargo workspaces)**
* **¿Cuál es el problema?** Unificar todas las dependencias críticas de Rust (Polars, DuckDB embebido, `ndarray`/`candle` — sin libtorch, ADR-0112) en librerías dinámicas compatibles con la distribución multiplataforma de Flutter sin requerir dependencias del compilador en el destino.
* **¿Qué tiene que pasar?** El sistema de integración continua y scripts de compilación locales deben generar librerías optimizadas en Rust (`.dll`/`.dylib`/`.so`) con todas las librerías estáticas enlazadas, garantizando que el usuario no necesite dependencias previas de desarrollo.
* **¿Cómo sé que está hecho?**
    - [ ] El instalador resultante se ejecuta en un "Client Zero" (máquina limpia) con éxito.
    - [ ] El sistema detecta y carga dinámicamente CUDA/Metal si está disponible (vía `candle`, opcional), o corre en CPU `ndarray`/Rayon por defecto sin GPU (ADR-0112).
* **¿Qué no puede pasar?** PROHIBIDO requerir compiladores o herramientas de desarrollo en la máquina del usuario final durante la ejecución.

### **TTR-002: Orquestador de Salud de la Aplicación (Flutter FFI)**
* **¿Cuál es el problema?** Si la ventana se cierra o el proceso del backend se detiene de forma inesperada, el estado local puede corromperse.
* **¿Qué tiene que pasar?** Implementar un supervisor que cierre de forma ordenada los hilos de Orquestador Rust y libere los recursos del sistema al cerrar la ventana.
* **¿Cómo sé que está hecho?**
    - [ ] Al cerrar la aplicación, no quedan procesos de Rust huérfanos.
* **¿Qué no puede pasar?** NUNCA dejar el puerto local expuesto a conexiones que no provengan de `127.0.0.1`.

## Persistencia (Filtro de Relevancia Hardware — ADR-0020 V2)

Cada instancia y log de ejecución de la aplicación debe portar el set filtrado de metadatos para trazabilidad de infraestructura local:

| Campo | Tipo | Categoría | Descripción |
| :--- | :--- | :--- | :--- |
| `id` | UUID | Identidad | Identificador único de la instancia |
| `created_at` | INT64 | Identidad | Timestamp de inicio del binario |
| `updated_at` | INT64 | Identidad | Último registro de salud del proceso |
| `audit_hash` | VARCHAR | Identidad | Verificación forense de la integridad del binario |
| `audit_chain_hash` | VARCHAR | Identidad | Hash encadenado de la sesión de ejecución |
| `event_sequence_id` | INT64 | Identidad | Secuencia ordinal de eventos de salud del proceso |
| `owner_id` | UUID | Soberanía | Usuario local de la aplicación |
| `institutional_tag` | VARCHAR | Soberanía | Etiqueta de cumplimiento organizacional |
| `manifest_id` | UUID | Soberanía | ID de la configuración de despliegue |
| `access_token_id` | UUID | Soberanía | Token de comunicación FFI/gRPC interna |
| `process_id` | INT32 | Hardware | OS PID del proceso de Rust |
| `session_id` | UUID | Hardware | Sesión única de la aplicación |
| `node_id` | VARCHAR | Hardware | Hash único del hardware local (CPU/GPU) |

## Gobernanza y Estándares (Fijos)
- **Local-First (ADR-0016):** 100% Local.
- **Inundación de Fundaciones (ADR-0020 V2):** El log de inicio registra el `manifest_id` y `hardware_hash`.

