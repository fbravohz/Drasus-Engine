# Efficiency & Incubation Dashboard

**Carpeta:** `./features/efficiency-incubation-dashboard/`
**Estado:** Lista para implementar
**Última actualización:** 2026-06-06
**Decisión Arquitectónica Asociada:** ADR-0106 (Paradigma de Interfaz de Usuario y Dashboards Visuales de Alta Precisión), ADR-0088 (Protocolo de Incubación & Cono de Silencio)

---

## ¿Qué es esta feature?

El `Efficiency & Incubation Dashboard` es la interfaz de visualización y control del período de incubación (cuarentena) de las estrategias de trading. Permite monitorear si el rendimiento de una estrategia en paper trading (In-Sample en vivo) se desvía estadísticamente de las expectativas simuladas en el backtest histórico, aplicando de manera estricta el concepto de "Cono de Silencio" (bandas de confianza estadísticas basadas en simulaciones de Monte Carlo).

---

## Comportamientos Observables

- [ ] El usuario visualiza la curva de equidad (equity curve) de la estrategia incubada en tiempo real superpuesta sobre las bandas de confianza de Monte Carlo generadas en el backtest.
- [ ] Si la equidad en vivo de la estrategia sale por debajo del límite inferior de las bandas del Cono de Silencio, la UI cambia el estado visual de la estrategia a "Desviación Crítica" (alerta roja).
- [ ] El dashboard muestra los KPIs de incubación:
  - Ratio de consistencia Pardo (relación entre el Sharpe del backtest vs Sharpe en vivo).
  - Máxima desviación de MAE/MFE observada en vivo.
- [ ] Permite al usuario forzar la finalización de la incubación o verter la estrategia al retiro (Withdraw).

---

## Restricciones

- **NUNCA** recalcular las bandas de confianza de Monte Carlo en el frontend; se leen como objetos inmutables calculados en Rust.
- **NUNCA** permitir la promoción automática de una estrategia que se encuentre fuera del Cono de Silencio en el momento de la evaluación.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| CONE_CONFIDENCE_LEVEL | 95% | 90% - 99% | Nivel de confianza estadística para las bandas del cono | CONFIG |
| MIN_INCUBATION_DAYS | 21 | 7 - 90 | Duración mínima obligatoria en cuarentena | CONFIG |
| MAX_SHARPE_DRIFT | 30% | 10% - 50% | Porcentaje máximo admisible de caída de Sharpe en vivo vs backtest | CONFIG |

---

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** Algoritmos de cálculo de bandas de desviación del cono y consistencia Sharpe.
- **Shell (Infraestructura):** Repositorios SQLite para persistir el estado de incubación y feeds de ticks de paper trading.
- **Frontera Pública:** Puertos para consultar el estado de la cuarentena de una estrategia y emitir eventos de violación del cono.

---

## Ciclo de Vida de la Feature — Efficiency & Incubation Dashboard

### Entrada
- Datos de backtest histórico (simulación Monte Carlo).
- Registro diario de equidad en vivo (paper trading).

### Proceso
- Superpone la equidad real sobre la matriz de distribución Monte Carlo.
- Evalúa los KPIs de consistencia diarios.

### Salida
- Bandas de visualización y marcas de estado de la cuarentena.
- Veredicto de incubación: `EN_LIMITES` / `DESVIADO` / `RECHAZADO`.

---

## Tareas (TTRs)

### **TTR-001: Widget del Cono de Silencio**
*   **¿Cuál es el problema?** El operador necesita ver rápidamente si el desempeño actual en vivo diverge del rango estadístico aceptable de la simulación.
*   **¿Qué tiene que pasar?** Graficar mediante Impeller la curva en vivo superpuesta a las bandas de percentiles Monte Carlo históricas de la estrategia.
*   **¿Cómo sé que está hecho?**
    - [ ] La UI renderiza las bandas sombreadas y la línea de equidad en vivo sin retrasos notables.

### **TTR-002: Monitor de KPIs de Consistencia**
*   **¿Cuál es el problema?** Evaluar la salud de la incubación requiere comparar métricas clásicas acumuladas en caliente.
*   **¿Qué tiene que pasar?** Rust calcula diariamente el Sharpe Drift y la desviación MAE/MFE y los envía al dashboard.
*   **¿Cómo sé que está hecho?**
    - [ ] El panel muestra los KPIs de consistencia actualizados y colorea en rojo si superan los límites de deriva configurados.

---

## Gobernanza y Estándares (Fijos)
- **Local-First (ADR-0016):** 100% Local.
- **Inundación de Fundaciones (ADR-0020): Perfil B (IA / R&D)** — guarda la configuración del dashboard de eficiencia (II + III subset + IV).

  | Categoría | Campo | Descripción |
  | :--- | :--- | :--- |
  | **I. Identidad** | `id` | Identificador único de la config del dashboard |
  | | `created_at` | Timestamp de creación |
  | | `updated_at` | Timestamp de última modificación del registro |
  | | `audit_hash` | Hash de integridad de la configuración |
  | | `audit_chain_hash` | Hash encadenado del historial de cambios |
  | | `event_sequence_id` | Secuencia de recuperación |
  | **II. Soberanía** | `owner_id` | Dueño del dashboard |
  | | `manifest_id` | Estrategia/incubación observada |
  | **III. Pesos/Arquitectura** | `version_node_id` | Versión de la vista del dashboard |
  | **IV. Hardware** | `node_id` | ID del hardware físico |
  | | `process_id` | PID del proceso de render |
- **Rastro de Evidencia:** Emite el estado de consistencia del cono al módulo de `feedback`.

---

## Dependencias
- **Depende de:** `/features/incubation-manager.md`, `/features/monte-carlo-simulator.md`
- **Bloquea:** `/modules/incubate.md`
