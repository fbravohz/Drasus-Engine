# Integración con NautilusTrader

> 🟡 **Parcial** 2026-06-21 · Orden de trabajo [STORY-014](../execution/STORY-014-nautilus-smoke-test.md)
> SPIKE-001 cerrado: `nautilus-model =0.58.0` compila en el workspace, capa anticorrupción stub verificada. Implementación real (TTR-001 a TTR-004) → EPIC-2/5.

**Carpeta:** `./features/`
**Estado:** En Diseño (Pilar Central)
**Última actualización:** 2026-06-21
**Decisión Arquitectónica Asociada:** ADR-0013, ADR-0107

---

## ¿Qué es?
Es la capa de adaptación que permite a Drasus Engine usar NautilusTrader como motor de ejecución y backtesting sin quedar acoplados permanentemente a su API. Actúa como el puente entre el Monolito Modular de Drasus Engine y el loop de eventos institucional de Nautilus. Conforme al ADR-0107, NautilusTrader se consume mediante los **crates Rust nativos de su núcleo v2** como dependencias Cargo (versiones fijadas y vendorizadas), dentro del mismo proceso Rust del Core: no existe fork del repositorio, ni sidecar, ni intérprete Python en ninguna ruta de ejecución.

## Ciclo de Vida
- **Entrada:** Configuraciones de estrategia (AST), Datos de mercado (Parquet/Arrow), Estado del Portafolio.
- **Proceso:** Traduce la lógica funcional de Drasus Engine a `NautilusTrader` Actors, Resolvers y DataBundles. Implementa el **Core Simulation Loop**: `Ingesta → Warm-up → Indicadores → Sincronización → Señal → Risk Check → Order Server → Matching Engine → Account/Ledger Update`.
- **Salida:** Flujo de eventos tipados (Orders, Fills, Latencies, Signals) con soporte para **Settlement vs Histórico (Davey)**.

## Comportamientos Observables
- **Abstracción de Broker:** Permite cambiar el backend de ejecución (Binance, IBKR, Oanda) sin modificar la lógica de la estrategia.
- **Determinismo Bit-a-Bit:** Garantiza que el mismo input de datos y semillas genere el mismo resultado exacto de ejecución en backtesting.
- **Finitud de Estados:** Sincroniza el `order-fsm` interno con los estados reales de las órdenes en el broker.

## Restricciones
- La lógica de negocio NO debe depender de clases internas de Nautilus; el adaptador debe mapear tipos NT a tipos Drasus Engine.
- El throughput de datos no debe penalizar la latencia de ejecución (uso de Arrow/Zero-Copy).
- Los fallos de conexión en el adaptador deben ser reportados al Watchdog en menos de 5 segundos.
- **Versionado Congelado (ADR-0107):** Las versiones de los crates de NT se fijan exactas y se vendorizan. NUNCA se actualiza el upstream sin pasar la suite de paridad bit-a-bit del puente.
- **Cumplimiento LGPL-3.0 (ADR-0107):** PROHIBIDO modificar el código de los crates vendorizados; el empaquetado comercial debe permitir el reenlazado de la porción LGPL.
- **Brechas de Adaptadores (ADR-0107):** Los brokers sin adaptador estable en el núcleo v2 se cubren con adaptadores propios contra los traits públicos de NT, en crates independientes; NUNCA parchando el núcleo.
- **Cobertura de Activos (ADR-0107):** El mapeo de instrumentos debe soportar acciones, forex, futuros, ETFs y CFDs como ciudadanos de primera clase; las opciones financieras se difieren a la última fase del roadmap.

## Tareas (TTRs)

### TTR-001: Adaptador de Datos Arrow/Nautilus
*   **Descripción:** Implementa la conversión eficiente de estructuras Polars/Arrow a `NautilusTrader` DataBundle.
*   **Criterio de Éxito:** Conversión zero-copy que permite cargar 1 millón de barras en sub-segundos.

### TTR-002: Orquestador de Backtest Determinista
*   **Descripción:** Crea el entorno de ejecución simétrica donde el motor de Nautilus opera bajo semillas PRNG fijas.
*   **Criterio de Éxito:** Reproducibilidad total de equity curves sobre el mismo dataset.

### TTR-003: Puente de Feedback y Auditoría
*   **Descripción:** Intercepta los eventos de Nautilus (PositionClosed, OrderFilled) y los inyecta en el Event Store de Drasus Engine.
*   **Criterio de Éxito:** Trazabilidad 1 a 1 entre lo que Nautilus reporta y lo que la auditoría de Feedback registra.

### TTR-004: Adaptadores Propios para Brokers No Cubiertos
*   **Descripción:** Implementa adaptadores de datos y ejecución (prioridad: Interactive Brokers y Oanda) contra los traits públicos de cliente del núcleo v2 de NT, en crates independientes candidatos a contribución upstream.
*   **Criterio de Éxito:** El operador conecta un broker no soportado por el upstream sin que exista ningún parche dentro del código vendorizado de NT, y el adaptador supera la misma suite de paridad sim/live que los adaptadores oficiales.

---

## Persistencia (Inundación de Fundamentos — ADR-0020 V2)

Aplica el Grupo I (universal) + solo los campos de su Perfil Técnico (Filtro de Relevancia, ADR-0020 V2), detallados en la tabla — NO el catálogo completo de 25 campos:

| Campo | Tipo | Descripción |
| :--- | :--- | :--- |
| `id` | UUID | Identificador único del evento de puente |
| `created_at` | INT64 | Timestamp del evento en Nautilus |
| `updated_at` | INT64 | Timestamp de registro en Drasus Engine |
| `audit_chain_hash` | VARCHAR | Hash de la secuencia de eventos de ejecución |
| `owner_id` | UUID | Dueño de la sesión de trading |
| `institutional_tag` | VARCHAR | Etiqueta de entorno (Live/Paper/Backtest) |
| `manifest_id` | UUID | ID del diseño evaluado |
| `access_token_id` | UUID | Token de autorización |
| `logic_hash` | VARCHAR | Hash de la versión del puente adaptador |
| `data_snapshot_id` | UUID | Puntero a los datos de mercado inyectados |
| `transformation_id` | UUID | ID del mapeo de datos NT → QF |
| `indicator_state_hash` | VARCHAR | Snapshot del estado del Actor en Nautilus |
| `process_id` | INT32 | PID del proceso anfitrión del puente (el Core; NT corre in-process per ADR-0107) |
| `session_id` | UUID | Sesión operativa global |
| `node_id` | VARCHAR | ID del hardware físico |
| `event_sequence_id` | INT64 | Orden secuencial (NTP-synchronized) |
| `parent_id` | UUID | ID del Job padre |
| `compliance_status_id` | INT32 | Veredicto del ruteador de órdenes |
| `risk_audit_id` | UUID | Referencia al log de ejecución en NT |
| `signature_hash` | VARCHAR | Firma de integridad del mensaje de puente |
| `execution_latency_ms` | DOUBLE | Latencia del puente (NT Event → QF Event) |
| `source_signal_id` | UUID | ID de la señal original |
| `audit_hash` | VARCHAR | Verificación forense final |
| `version_node_id` | UUID | Versión de la rama Git-like activa |

## Gobernanza y Estándares (Fijos)
- **Decisión Arquitectónica Asociada:**
    - ADR-0013: Stack Tecnológico (NautilusTrader).
    - ADR-0107: Integración Nativa con NautilusTrader v2 (Crates Rust, Sin Python, Sin Fork).
    - ADR-0020 V2: Inundación de Fundaciones.

## Dependencias y Bloqueantes
- **Depende de:** `data-validator`, `order-fsm`, `audit-log`.
- **Bloquea:** Implementación final de los módulos `validate` y `execute`.
