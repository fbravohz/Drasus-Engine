# Validador de Datos (Data Validator)

**Carpeta:** `./features/data-validator/`
**Estado:** Especificación / Prioritario
**Última actualización:** 2026-04-12

---

## ¿Qué es?

Es el componente encargado de la integridad estructural de los datos de mercado. Su misión es detectar anomalías técnicas (precios negativos, saltos imposibles, huecos de tiempo) y normalizar los datos a un formato interno estándar (enteros `int64`).

---

## Comportamientos Observables

- [ ] Valida que **High >= Low** y que **Open/Close** estén dentro de ese rango.
- [ ] Detecta y rechaza precios o volúmenes negativos.
- [ ] **Normalización a Enteros:** Convierte precios decimales a `int64` (centavos o ticks) para eliminar errores de coma flotante.
- [ ] Detecta huecos (gaps) en las series temporales comparando con el calendario del mercado.

---

## Restricciones

- **CERO REDONDEO:** La conversión a enteros debe ser exacta según la precisión del instrumento.
- **INMUTABILIDAD:** Una vez validado y normalizado, el dato se considera "Golden Source".

---

## Parámetros Configurables

| Parámetro | Default | Qué hace |
|---|---|---|
| TICK_SIZE | 0.01 | Resolución mínima para la conversión a enteros |
| MAX_GAP_ALLOWED | 5 | Máximo de barras faltantes antes de marcar "Data Incomplete" |

---

---

## Tareas (TTRs)

### **TTR-001: Motor de Validación OHLCV Structural**
*   **Descripción:** Detecta anomalías técnicas (High < Low, precios negativos, etc.) antes de cualquier proceso.
*   **Reglas de Negocio:**
    * Si CUALQUIER precio es <= 0, la barra es `REJECTED` (excepto en activos específicos debidamente marcados).
    * Toda barra rechazada DEBE registrar el `error_code` y el `audit_hash` del stream original.
*   **Entrada:** `raw_bar_data` (Dict/Row).
*   **Salida:** `is_valid` (bool), `error_code`.
*   **Precondición:** Datos de entrada mapeados a campos estándar.
*   **Postcondición:** Registro de la anomalía en `data_quality_logs` con `process_id` (ADR-0020 V2).

### **TTR-002: Conversor Decimal-to-Integer (Tick-Safe)**
*   **Descripción:** Convierte precios decimales a `int64` (centavos/ticks) para eliminar errores de coma flotante (ADR-0002).
*   **Reglas de Negocio:**
    * El multiplicador (precision) debe ser inmutable para cada símbolo.
    * La conversión DEBE ser reversible sin pérdida de precisión.
*   **Entrada:** `decimal_price`, `tick_size`.
*   **Salida:** `integer_price` (int64).
*   **Precondición:** `tick_size` verificado en los metadatos del instrumento.
*   **Postcondición:** El rastro de auditoría incluye el `scaling_factor` utilizado (ADR-0020 V2).

---

## Persistencia (Inundación de Fundamentos — ADR-0020 V2)

Cada reporte de validación y limpieza registra el set de relevancia técnica para ingesta:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único del reporte |
| | `created_at` | Timestamp de validación |
| | `audit_hash` | Hash del dato validado |
| | `audit_chain_hash` | Hash de la secuencia de limpieza |
| **II. Linaje** | `data_snapshot_id` | Ref al snapshot original del broker |
| | `transformation_id` | ID del paso de limpieza (Raw vs Cleaned) |
| | `logic_hash` | Hash del motor de validación |
| **III. Hardware** | `node_id` | ID del hardware físico |
| | `process_id` | PID del validador |

## Gobernanza y Estándares (Fijos)
- **Decisión Arquitectónica Asociada:**
    - ADR-0002: Arregrística entera para precios (int64).
    - ADR-0013: Stack Tecnológico (Nautilus ready).
    - ADR-0020 V2: Inundación de Fundaciones.

---

## Dependencias
**Depende de:**
- [`clock`](../features/clock.md) — para timestamps de validación.

**Consumido por:**
- [`ingest`](../modules/ingest.md) — para la limpieza inicial de datos.
- [`feedback`](../modules/feedback.md) — para el análisis de degradación de datos del broker.
