# Multiplatform Execution Bridge — Puente de Ejecución Multiplataforma

**Carpeta:** `./features/multiplatform-execution-bridge/`
**Estado:** Lista para implementar
**Última actualización:** 2026-05-02
**Decisión Arquitectónica Asociada:** ADR-0078 (Autopilot Execution & Stealth Network Infrastructure)

---

## ¿Qué es?

El puente de ejecución multiplataforma es un desacoplador de órdenes y capa de abstracción diseñado para comunicar nuestro entorno de ejecución en vivo (Nautilus en el VPS) con múltiples plataformas externas (MetaTrader 4/5, NinjaTrader, Interactive Brokers, cTrader, etc.) sin exportar código complejo (evitando MQL4/MQL5, PineScript, EasyLanguage).

**Problema:** Operar directamente contra APIs propietarias o tener que exportar código a múltiples aplicaciones genera fragilidad y permite que brokers C-Book realicen Stop-Hunting sobre parámetros visibles (como Magic Numbers locales). El Multiplatform Execution Bridge mantiene toda la lógica de estrategia y stops en Rust y emite comandos JSON estandarizados hacia la plataforma externa actuante.

---

## Comportamientos Observables

- [ ] Execute genera una orden de mercado en el VPS blindado
  → Se emite un comando JSON genérico (`COMPRA`, `VENTA`, `MODIFICA`, `CIERRA`).
  → Se transmite via gRPC/WebSocket o REST API nativo hacia el receptor mudo (ej. terminal de MetaTrader).
  → El receptor mudo ejecuta la orden sin metadatos locales (ej. sin Magic Numbers ni stops visibles).

---

## Restricciones

- **NUNCA se exporta lógica de estrategia ni indicadores a las plataformas externas receptoras.**
- **Latencia máxima del bridge:** 5ms entre el VPS y la terminal receptora.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| BRIDGE_PORT | 8001 | 1024-65535 | Puerto local para gRPC/WebSocket o REST API | CONFIG |
| HEARTBEAT_TIMEOUT_MS | 1000 | 100-5000 | Tiempo máximo de espera para confirmar la conexión | CONFIG |

---

## Ciclo de Vida de la Feature

### Entrada
- Señal de orden autorizada por el módulo `execute`
- Formato estándar JSON de comando de ejecución

### Proceso
- Valida la integridad del mensaje y autenticación
- Transmite el comando vía gRPC/WebSocket / REST API hacia la plataforma destino
- El receptor ejecuta la orden nativa y devuelve la confirmación

### Salida
- Confirmación de recepción y fill por parte del broker real
- Reconciliación del fill mediante el `audit_hash` y actualización del ledger

---

## Tareas (TTRs)

### **TTR-001: Comunicación vía WebSockets/gRPC API**
*   **¿Cuál es el problema?** ZeroMQ requiere instalar librerías extras y DLLs complejas en MetaTrader. Queremos un protocolo estándar.
*   **¿Qué tiene que pasar?** El bridge transmite comandos JSON vía WebSockets/gRPC hacia la plataforma receptora.
*   **¿Cómo sé que está hecho?**
    - [ ] Puedo conectar una terminal de MetaTrader o NinjaTrader al socket y recibir comandos de compra/venta válidos.

### **TTR-002: Reconciliación por Hash sin Magic Numbers**
*   **¿Cuál es el problema?** Para ocultar la estrategia de cazas de stops, no se usan Magic Numbers tradicionales en el receptor.
*   **¿Qué tiene que pasar?** El VPS mantiene una tabla de correspondencia (`audit_hash` <-> `broker_order_id`) para reconciliar los fills devueltos.
*   **¿Cómo sé que está hecho?**
    - [ ] El ledger local se actualiza correctamente cuando el broker confirma el fill de la orden.

---

## Gobernanza y Estándares (Fijos)
## Persistencia (Inundación de Fundamentos — ADR-0020 V2)

Esta feature aplica el perfil de **Ops / Hot-Path**:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único de la orden/fill |
| | `created_at` | Timestamp de origen (nanosegundos) |
| | `audit_hash` | Hash de la transacción (Firma digital) |
| | `audit_chain_hash` | Hash de la secuencia de fills de la sesión |
| **II. Soberanía** | `owner_id` | Usuario responsable del capital real |
| | `compliance_status_id` | Veredicto del Pre-Trade Validator |
| **III. Hardware** | `node_id` | ID del hardware físico ejecutor |
| | `process_id` | PID del motor de ejecución real |
| | `execution_latency_ms` | Latencia señal-a-broker (Máximo 1ms) |

---

## Dependencias
**Depende de:**
- [`order-fsm`](../features/order-fsm.md) — para la máquina de estados.
- [`audit-log`](../features/audit-log.md) — para rastro inmutable.

**Consumido por:**
- [`execute`](../modules/execute.md) — para la ejecución blindada.
