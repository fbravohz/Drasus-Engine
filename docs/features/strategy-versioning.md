# Strategy Versioning — Versionamiento Git-like de Estrategias y Portafolios

**Carpeta:** `./features/strategy-versioning/`
**Estado:** Lista para implementar
**Última actualización:** 2026-04-09
**Decisión Arquitectónica Asociada:** ADR-0005 (Strategy-Portfolio Git-Like Versioning)

---

## ¿Qué es?

El Strategy Versioning implementa un sistema de historial completo similar a Git para estrategias y portafolios. Cada modificación a la configuración crea un nuevo nodo inmutable en un grafo acíclico dirigido (DAG). El sistema nunca sobreescribe versiones antiguas — todo el historial se preserva indefinidamente.

**Problema:** Si simplemente sobrescribes la configuración de una estrategia cada vez que la modificas, pierdes el historial. No puedes comparar versiones antiguas, no puedes revertir a configuraciones previas que funcionaban bien.

**Solución:** Cada cambio crea un nuevo nodo en el DAG. Cada nodo referencia su nodo padre (o es raíz si es la primera versión). El sistema permite múltiples ramas activas simultáneamente (ej: rama "main" y rama "experimental" en paralelo).

**Resultado observable:** Historial completo, reproducible e imputable de cada cambio a una estrategia o portafolio.

---

## Comportamientos Observables

- [ ] Usuario crea una estrategia nueva
  → Se crea el primer nodo (raíz) en el DAG: hash único, parent_hash=null, branch_name="main"
  → Se registra el snapshot de configuración en ese nodo
  → Se marca timestamp de creación

- [ ] Usuario modifica la configuración de la estrategia (ej: cambia período del indicador)
  → Se crea un nuevo nodo en el DAG como hijo del anterior
  → El nuevo nodo referencia al anterior como parent
  → La rama "main" ahora apunta al nuevo nodo
  → El nodo antiguo permanece en el DAG (no se sobreescribe)

- [ ] Usuario crea una rama experimental desde un nodo antiguo
  → Se crea un nuevo nodo que referencia al nodo antiguo como parent
  → Nueva rama se llama "experimental" (configurable)
  → La rama "main" sigue apuntando al nodo más reciente de main
  → Ambas ramas coexisten en paralelo

- [ ] Usuario compara dos versiones de la misma estrategia
  → Sistema recorre el DAG desde una versión a la otra
  → Muestra qué parámetros cambiaron, qué indicadores se agregaron/removieron
  → Muestra el orden exacto de cambios (porque el DAG es ordenado topológicamente)

- [ ] Un backtest se ejecuta en estrategia en rama "main"
  → Se registra el test_result (inmutable) con referencia al hash de la versión.
  → **Historial Acumulativo:** Si la versión V2 hereda de V1 y los parámetros del indicador X no cambiaron, V2 hereda y referencia los resultados de pruebas de V1 para ese indicador.
  → **Cumulative Test Results (ADR-0060):** El sistema utiliza el `params_hash` para identificar tests idénticos. Si v1 ya corrió WFA(252 barras), v2 hereda ese resultado y solo agrega nuevas ejecuciones (ej. WFA(504)) si los parámetros cambiaron, calculando únicamente el delta incremental.
  → El test_result se agrega al historial acumulativo de la estrategia (append-only) y se guarda en `test_results_array` dentro del registro de la versión.
  → **Esquema de Almacenamiento:** El archivo Parquet incluye las columnas: `strategy_id`, `version_hash`, `parent_hash`, `branch_name`, `parameters_json`, `test_results_array` (contiene listado de `{test_id, params_hash, metrics}`).

- [ ] Un usuario intenta crear un ciclo en el DAG (versión A → B → C → A)
  → Sistema rechaza: "Ciclo detectado. Grafo debe ser acíclico."
  → El cambio nunca se persiste

---

## Restricciones

- **NUNCA un nodo en el DAG se modifica después de ser creado.** Nodos inmutables.
- **NUNCA el DAG forma un ciclo.** Siempre debe ser acíclico.
- **NUNCA se borra un nodo del DAG.** Historial completo, infinito.
- **NUNCA se pierden test_results.** Cada test se agrega al historial de la versión (append-only).
- **NUNCA se regenera test_analysis.** Se genera UNA SOLA VEZ post-Validate y nunca se modifica.
- **NUNCA un portafolio referencia una versión de estrategia que no existe.** Las referencias son locked (verificadas al momento de creación).

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace |
|---|---|---|---|
| BRANCH_LIMIT | 10 | 1-100 | Cuántas ramas distintas puede tener una estrategia antes de obligar a limpiar/merge |
| DEPTH_LIMIT | 1000 | 100-10000 | Cuántos nodos máximo en el DAG antes de advertir (archivado sugerido) |
| SNAPSHOT_COMPRESSION | false | true / false | Si true, comprime snapshots viejos para ahorrar espacio |

---

## Ciclo de Vida de la Feature

### Entrada
- **Quién llama:** Generate (crea nuevas versiones al cambiar configuración), Validate (agrega test_results), Manage (agrupa estrategias en portafolios versionados), Incubate (ejecuta versiones específicas)
- **Qué recibe:** Un cambio de configuración, o un nuevo test_result, o una nueva rama

### Proceso
1. **Crear nuevo nodo:** Se calcula hash único basado en contenido (snapshot de configuración)
2. **Validar grafo:** Se verifica que crear este nodo no forma ciclos
3. **Establecer parent:** Se referencia el nodo actual como parent del nuevo
4. **Actualizar rama:** La rama activa ahora apunta al nuevo nodo
5. **Registrar snapshot:** Se persiste el snapshot de configuración en el nuevo nodo

### Salida
- **Qué produce:** 
  - Un nuevo nodo en el DAG (inmutable desde ahora)
  - Un nuevo version_hash que identifica esta configuración de forma única
  - El DAG permanece acíclico garantizado

### Contextos de Uso
- **Generate:** Cada generación de nuevos candidatos crea versiones nuevas
- **Validate:** Validate agrega test_results al historial de una versión
- **Manage:** Los portafolios referencian versiones locked de estrategias
- **Incubate:** Paper trading ejecuta una versión específica (no cambia mid-session)

---

## Tareas (TTRs)

### **TTR-001: Crear Nodo de Versión Inmutable (DAG)**
*   **Descripción:** Registra un nuevo snapshot de configuración como un nodo inmutable en el grafo (DAG).
*   **Reglas de Negocio:**
    * El `version_hash` DEBE ser determinista basado en el contenido (Content-Addressed) (ADR-0005).
    * Toda versión nueva DEBE incluir el `process_id` del generador/usuario (ADR-0020 V2).
*   **Entrada:** `config_payload` (JSON), `parent_hash`, `branch_name`.
*   **Salida:** `version_hash` (Sha256), `node_id`.
*   **Precondición:** Hash del padre verificado y existente en el DAG.
*   **Postcondición:** Nodo persistido en `strategy_nodes` con `audit_hash` encadenado.

### **TTR-002: Persistencia de Historial de Test (Incremental & Acumulativo)**
*   **Descripción:** Agrega un `TestResult` al rastro inmutable. Si el sistema detecta que la versión actual es un cambio menor (ej: solo visualización), permite heredar los resultados de robustez (Monte Carlo) de la versión padre para ahorrar cómputo (ADR-0005).
*   **Reglas de Negocio:**
    * NUNCA permitir el borrado de un test_result una vez persistido.
    * Todo resultado de test debe incluir el `hardware_fingerprint` del ejecutor (ADR-0020 V2).
*   **Entrada:** `version_hash`, `test_payload`, `test_type`, `inherit_from_parent` (bool).
*   **Postcondición:** El rastro de evidencia en `audit-log` confirma la integridad y el linaje de los resultados.

### **TTR-003: Versionado de Portafolio Git-Like (Composición Dinámica)**
*   **Descripción:** Persiste y ramifica composiciones de portafolios de forma inmutable.
*   **Reglas de Negocio:**
    * Los cambios generan un nuevo nodo y rama. No se sobreescribe el estado previo.
    * Almacena datos en `portfolios.parquet`.
*   **Campos:** `portfolio_id`, `version_hash`, `parent_hash`, `branch_name`, `strategy_composition_json`, `weights_json`, `test_results_array`.

### **TTR-004: Diff Visual y Reversión entre Versiones**
*   **¿Cuál es el problema?**
    Cuando un algoritmo de optimización (ej. Walk-Forward) o la IA modifican una estrategia, el humano necesita ver exactamente qué cambió la máquina frente a su versión original, y poder deshacerlo si la máquina "se pasó de la mano".
*   **¿Qué tiene que pasar?**
    El usuario selecciona dos nodos del DAG (ej. `v1.0 Manual` y `v1.1 Optimizada`) y ve una comparación visual lado a lado que resalta cada diferencia (parámetros, reglas, indicadores añadidos/quitados). Un botón "Revertir a v1.0" restaura la versión anterior creando un nuevo nodo (sin borrar el historial).
*   **¿Cómo sé que está hecho?**
    - [ ] Comparo dos versiones y veo resaltadas las diferencias línea por línea.
    - [ ] Pulso "Revertir" y la rama vuelve a apuntar a la versión elegida mediante un nuevo nodo.
    - [ ] El nodo descartado sigue existiendo en el DAG (nada se sobrescribe).
*   **¿Qué no puede pasar?**
    - No puede perderse ninguna versión al revertir; la reversión nunca borra nodos.
    - No puede mostrarse un diff que omita un campo que sí cambió.
*   **Slice Visual (Flutter/Impeller/FFI):** Vista de comparación dual renderizada con Impeller; el cálculo del diff vive en el Core Rust y cruza por FFI.

---

## Persistencia (Inundación de Fundamentos — ADR-0020 V2)

Cada nodo del Grafo de Versiones (DAG) registra el set de relevancia técnica para AI/R&D:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único del nodo de versión |
| | `created_at` | Timestamp de creación inmutable |
| | `audit_hash` | Hash del snapshot de configuración |
| | `audit_chain_hash` | Hash de integridad del timeline del DAG |
| **II. Soberanía** | `owner_id` | Autor de la versión |
| | `manifest_id` | ID del contrato de diseño |
| | `access_token_id` | Token de autorización de cambios |
| **III. Pesos/Arquitectura** | `logic_hash` | Huella digital de la configuración lógica |
| | `parent_id` | ID del nodo padre (DAG Link) |
| | `event_sequence_id` | Posición en la secuencia topológica |
| **IV. Hardware** | `node_id` | ID del hardware físico |
| | `process_id` | PID del proceso de versionamiento |

- **Decisión Arquitectónica Asociada:**
    - ADR-0005: Versionamiento Reproducible (DAG).
    - ADR-0016: Local-First (Soberanía de propiedad intelectual).
    - ADR-0020 V2: Inundación de Fundaciones.

---

## Dependencias
**Depende de:**
- [`clock`](../features/clock.md) — para timestamps inmutables de creación.
- [`audit-log`](../features/audit-log.md) — para registro de transiciones del grafo.

**Consumido por:**
- [`generate`](../modules/generate.md) — para la creación de linajes de candidatos.
- [`validate`](../modules/validate.md) — para la certificación de resultados de pruebas.
- [`manage`](../modules/manage.md) — para orquestación de portafolios versionados.
- [`incubate`](../modules/incubate.md) — para ejecutar versiones específicas.
- [`withdraw`](../modules/withdraw.md) — para rastrear cambios en el historial.
