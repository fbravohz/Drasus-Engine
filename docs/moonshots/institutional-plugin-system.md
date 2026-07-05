# Institutional Plugin System

**Carpeta:** `./moonshots/`
**Estado:** Moonshot (Fase 5 - Experimental)
**Última actualización:** 2026-06-06
**Decisión Arquitectónica Asociada:** ADR-0103 (Filosofía Dual y Sandboxing en el Sistema de Plugins Institucionales)

---

## ¿Qué es?

Es una infraestructura de extensión modular que permite a desarrolladores y fondos de inversión integrar herramientas analíticas personalizadas, conectores a brokers propietarios e interfaces personalizadas en Drasus Engine. Incorpora un kit de desarrollo de software en Python (`drasus-sdk`), cifrado avanzado de extremo a extremo (Pro E2EE) para la protección del código de extensiones y un entorno de ejecución seguro (Sandbox) para validar la inocuidad de plugins desarrollados por terceros antes de su incorporación al sistema.

---

## Filosofía Dual (Client Zero first)

El sistema de plugins y su comercialización se rige de forma estricta por la **Filosofía Dual**:
1. **Fase Inicial Client Zero:** Foco absoluto en la operativa personal del fundador para alcanzar rentabilidad sostenida (sin comercialización B2C).
2. **Fase Comercial B2B/B2C:** Apertura de la infraestructura del Marketplace de plugins únicamente después de cumplir un hito de rentabilidad sostenida documentado de forma local (mínimo configurable de 6 a 12 meses de operaciones ganadoras reales).

---

## Comportamientos Observables

- [ ] **SDK de Integración en Python:** Desarrolladores importan el SDK nativo para consultar el catálogo de veredictos locales o inyectar señales en el bus de eventos de la plataforma.
- [ ] **Sandboxing de Terceros:** Al instalar un plugin de origen externo, el sistema lo ejecuta dentro de un entorno aislado restringiendo el acceso a internet, llaves API y archivos del disco del usuario no autorizados de forma explícita.
- [ ] **Cifrado E2EE de Plugins:** Los complementos de grado institucional se compilan y cifran en origen, de modo que el código de análisis propietario no es visible para el operador del sistema o intermediarios en tránsito.

---

## Restricciones

- **OBLIGATORIO:** Ejecutar los procesos de plugins externos en una máquina virtual de aislamiento WebAssembly (Wasmer/Wasmtime) o subprocesos desprovistos de privilegios de administrador.
- **NUNCA** permitir que un plugin no verificado acceda al módulo de ejecución en vivo (`execute`) o recupere credenciales de brokers.
- **FIJO:** Los puertos y APIs expuestas a través del SDK deben estar protegidos mediante autenticación JWT local con tokens de corta expiración.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
| :--- | :--- | :--- | :--- | :--- |
| MAX_PLUGIN_MEMORY_MB | 512 | 64 - 2048 | Límite máximo de memoria RAM asignable a un plugin individual en ejecución | CONFIG |
| ALLOW_NETWORK_ACCESS | false | true / false | Habilita o deshabilita la salida a internet de los plugins de sandbox | CONFIG |
| RETRY_CRASHED_PLUGINS | false | true / false | Intenta reiniciar complementos que fallaron en caliente de forma automática | CONFIG |

---

## Estructura Interna (FCIS)

- **Core (Lógica Pura):** Control de acceso por niveles y gestión de permisos del SDK.
- **Shell (Infraestructura):** Máquina virtual WebAssembly (Wasmer) y base de datos SQLite de firmas y validaciones de plugins autorizados.

---

## Ciclo de Vida de la Feature — Plugin System

### Entrada
- Archivo de plugin comprimido/cifrado (`.qfplugin`).
- Solicitudes de API entrantes desde el SDK de Python.

### Proceso
- Verificación de la firma digital de autenticidad del autor.
- Carga del binario WebAssembly en el entorno seguro de Wasmer.
- Enrutamiento restringido de comandos a través del bus de eventos locales.

### Salida
- Ejecución aislada de la analítica del plugin y telemetría de rendimiento hacia la interfaz gráfica.

---

## Tareas (TTRs)

### **TTR-001: Entorno de Aislamiento WebAssembly (Wasmer Sandbox)**
*   **¿Cuál es el problema?** Instalar extensiones de la comunidad puede introducir software malicioso (keyloggers, troyanos) que robe las credenciales de los exchanges o las estrategias del disco duro.
*   **¿Qué tiene que pasar?** Diseñar un motor de carga de plugins en Rust utilizando la máquina virtual Wasmer. Los plugins deben compilarse a WebAssembly (`wasm32-wasi`). La API del SDK solo expone funciones matemáticas e informativas puras, bloqueando llamadas al sistema para leer archivos del disco duro o hacer peticiones de red directas.
*   **¿Cómo sé que está hecho?**
    - [ ] Un plugin malicioso que intente ejecutar `std::fs::read` falla en tiempo de ejecución con error de violación de permisos de WASI.
    - [ ] Los plugins autorizados se ejecutan de manera concurrente en hilos de Rust sin acceso al entorno host del sistema.
*   **¿Qué no puede pasar?**
    - Los plugins no deben eludir el entorno de arena bajo ninguna condición de desbordamiento de memoria.

### **TTR-002: SDK Python de Ingesta y Consulta Analítica**
*   **¿Cuál es el problema?** Los analistas cuantitativos prefieren desarrollar y evaluar sus modelos utilizando el ecosistema de Python (Pandas, NumPy) y requieren interactuar de forma fluida con las bases de datos de veredictos de Drasus Engine.
*   **¿Qué tiene que pasar?** Desarrollar una librería en Python (`drasus-sdk`) que interactúe con el backend del monolito mediante llamadas gRPC locales protegidas. Permite a los scripts importar series temporales Parquet y enviar de vuelta reportes de validación formateados.
*   **¿Cómo sé que está hecho?**
    - [ ] Un script en Python importa la librería, se conecta al puerto local gRPC y lee un DataFrame de Polars en menos de 100ms.
    - [ ] El script envía una señal de validación y la UI de Flutter actualiza la visualización de la estrategia correspondiente.
*   **¿Qué no puede pasar?**
    - El SDK de Python no debe permitir el ruteo directo de órdenes de trading real sin pasar por el módulo de validación de 8 pasos (`pre-trade-validator`).

---

## Gobernanza y Estándares (ADR-0020)

### Perfil Ops / Auditoría
| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | UUID del plugin instanciado |
| | `created_at` | Timestamp de carga en la sesión |
| | `audit_hash` | Hash SHA-256 del binario WASM del plugin |
| **II. Soberanía** | `owner_id` | Identificador de firma del autor del plugin |
| **IV. Hardware** | `node_id` | ID único del procesador local |
| | `process_id` | PID de la instancia de Wasmer |
| | `execution_latency_ms` | Latencia consumida en la ejecución del ciclo de la extensión |
