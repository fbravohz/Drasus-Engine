# Performance Monitor

**Carpeta:** `./features/performance-monitor/`
**Estado:** Especificación / Prioritario
**Última actualización:** 2026-04-12

---

---

## ¿Qué es?

Componente de vigilancia encargado de detectar la degradación del rendimiento en vivo. Su misión es la **Prevención de Quiebra (Bankruptcy Prevention)**: asegura que ninguna estrategia opere fuera de sus intervalos de confianza estadísticos establecidos en la validación.

---

## Narrativa de Producto

**Capacidad:** `CAP-08` — El ciclo que se cierra solo

**Historia de usuario:** Como trader con dinero real en juego, quiero que el sistema vigile continuamente si mi estrategia sigue comportándose como en la validación, para enterarme de la degradación antes de que se traduzca en una pérdida grande.

**El problema (técnico):** una estrategia aprobada trae consigo una expectativa estadística —un Sharpe, un drawdown máximo, un rango de confianza— establecida en la validación. Esa expectativa no es una garantía permanente: el régimen de mercado que la sostenía puede desvanecerse sin que ninguna señal visible lo anuncie, hasta que la racha de pérdidas ya es dolorosa. El error común es descubrir la degradación solo cuando la cuenta ya sintió el golpe.

**La solución (técnica):** un vigilante que recalcula los indicadores clave de la estrategia después de cada operación cerrada, contrasta el Sharpe y el drawdown actuales contra la línea base de validación, y aplica una prueba estadística sobre la racha reciente de resultados para distinguir una mala racha normal de una desviación real. Cuando la desviación cruza el umbral, emite una alerta de degradación que dispara el protocolo de retiro, con la evidencia histórica que explica por qué.

**Explicado sin tecnicismos:** piensa en el examen médico anual. No espera a que sientas dolor para avisarte que algo cambió; mide tus números contra tu propio historial y te avisa en cuanto se salen del rango normal, mucho antes de que el síntoma aparezca.

Esta pieza hace eso mismo con cada estrategia que opera dinero real: compara constantemente cómo se está comportando ahora contra cómo se comportó en su examen de validación. En cuanto la diferencia deja de ser ruido normal y empieza a verse como una tendencia real, suena la alarma — antes de que la cuenta pague la factura completa.

---

## Comportamientos Observables

- [ ] Compara el Sharpe actual vs el Baseline de Validación en tiempo real.
- [ ] Detecta brechas de DrawDown que superen los límites históricos permitidos.
- [ ] Emite señales de alerta ante desviaciones de 2 sigmas en la curva de equidad.

---

## Ciclo de Vida de la Feature — Performance Monitor

### Entrada
- Serie de trades en vivo (Live/Paper).
- Metadatos de la estrategia (Baseline Stats: Sharpe, Max DD).
- Umbrales de tolerancia dinámicos.

### Proceso
- Recalcula los KPIs de la estrategia tras cada cierre de posición.
- Realiza un test de **Z-Score** sobre la racha actual de pérdidas/ganancias.
- Valida si la eficiencia (Profit Factor) se mantiene dentro de los límites esperados.

### Salida
- **Salud de Estrategia:** % de degradación acumulado.
- **Alertas de Drift:** Eventos enviados al orquestador.

### Contextos de Uso
**Contexto 1: Retiro (Módulo Withdraw)**
- Activa el protocolo de pausa/retiro si la degradación es crítica.
**Contexto 2: Retroalimentación (Módulo Feedback)**
- Proporciona la evidencia histórica de la degradación para el análisis de causas raíz.

---

## Tareas (TTRs) — Herencia de Módulo Retirar

### TTR-001: Detección Automática de Drift
*   **Descripción:** Monitor de KPIs que evalúa la caída de calidad.
*   **Criterio de Éxito:** Alerta activa si el Sharpe cae > 30% del original.

### TTR-002: Análisis de Bandas de Confianza
*   **Descripción:** Compara la equidad contra el túnel de probabilidad del backtest.

---

## Gobernanza y Estándares (Fijos)
- **Local-First (ADR-0016):** 100% Local. Los datos de rendimiento propio son confidenciales.
## Persistencia (Inundación de Fundamentos — ADR-0020)

Toda evaluación de performance y detección de drift registra el set de relevancia técnica:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único del reporte |
| | `created_at` | Timestamp de evaluación |
| | `updated_at` | Timestamp de última modificación del registro |
| | `audit_hash` | Hash del P&L Snapshot T-0 |
| | `audit_chain_hash` | Hash de la sesión de monitoreo |
| | `event_sequence_id` | Secuencia de recuperación (event-sourcing) |
| **II. Soberanía** | `owner_id` | Dueño responsable de la estrategia |
| | `institutional_tag` | Tag de cumplimiento institucional |
| **III. Pesos/Arquitectura** | `logic_hash` | Hash de la fórmula de KPI (Sharpe/DD) |
| | `indicator_state_hash` | Snapshot del porcentaje de degradación |
| | `version_node_id` | ID de la versión evaluada en el DAG |
| **IV. Hardware** | `node_id` | ID del hardware físico supervisor |
| | `process_id` | PID del daemon de vigilancia |


---

## Estructura Interna (FCIS — ADR-0002)
- **Core (Lógica Pura):** Algoritmos de cálculo de drift en `monitor_logic.rs`.
- **Shell (Infraestructura):** Listener de eventos de ejecución (fills).
- **Frontera Pública:** Contrato `get_health_status(strategy_id)`.

---

## Dependencias
**Consumido por:** `withdraw`, `feedback`.
**Depende de:** `institutional-metrics`, `audit-log`.
