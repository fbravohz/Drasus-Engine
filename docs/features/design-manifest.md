# Design Manifest — Quality Gate & Robustness Score

**Carpeta:** `./features/design-manifest/`
**Estado:** Especificación / Crítica (Fase 2)
**Última actualización:** 2026-04-30

---

## ¿Qué es?

El Design Manifest es el "Vigilante de la Puerta" (Filtro de Calidad) final antes de que una estrategia sea promovida a incubación o ejecución en vivo. Emite un veredicto automático basado en un **Puntaje de Robustez (0-100)** agregado de múltiples métricas de estrés y robustez.

Además de actuar como filtro, el Design Manifest opera como un **Envoltorio de Despliegue** y **Contrato de Metas SMART**. El manifiesto y sus restricciones viven fuera de la estrategia puramente matemática, envolviendo el Árbol de Sintaxis Abstracta (AST) base. Esto asocia el AST algorítmico a una capa de riesgo y objetivos de portafolio externa previo a cualquier proceso de minado u optimización.

---

## Comportamientos Observables

- [ ] **AI Robustness Verdict Panel:** Calcula métricas de WFE, DSR, PBO, probabilidad de ruina e inversión lógica.
- [ ] **Veredicto Automático:** 
  - **APROBADO:** Puntaje > 75 (O el umbral específico del manifiesto). 
  - **REVISAR:** Puntaje 50-75 (Requiere intervención humana).
  - **RECHAZADO:** Puntaje < 50 (Archivado automático).
- [ ] **Contrato de Ejecución (Metas SMART):** Toda estrategia se somete a un contrato extensible y configurable mediante un esquema de validación sin parámetros duros (ej. Sharpe mínimo, Drawdown máximo permitido, número mínimo de transacciones).
- [ ] **Envoltorio de Despliegue:** El contrato de diseño envuelve el AST y es inmutable; la lógica algorítmica es evaluada estrictamente contra la capa de riesgo externa del envoltorio.
- [ ] **Bloqueo Automático:** Si tras pasar la validación (OOS, WFA, Monte Carlo), la estrategia no cumple milimétricamente las metas SMART, el sistema la rechaza automáticamente y bloquea su promoción.
- [ ] **Validation Gate:** Bloqueo automático de aprobación si el Puntaje de Robustez final no cumple estrictamente con el umbral definido en el contrato inicial.

---

## Restricciones

- **NUNCA permitir el paso a la Fase 3** de una estrategia que no cumpla milimétricamente el Design Manifest.
- **NUNCA utilizar valores fijos (hardcoding)** en los umbrales de las metas SMART; todos deben ser configurables por el usuario.
- **El umbral del Robustness Score es obligatorio** para la validación final del gatekeeper.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| APPROVAL_THRESHOLD | 75 | 50-90 | Puntaje mínimo de robustez para pasar el gatekeeper | CONFIG |
| MIN_SHARPE_RATIO | 1.5 | 0.5-3.0 | Sharpe Ratio mínimo aceptable | CONFIG |
| MAX_DRAWDOWN_PCT | 15.0 | 5.0-40.0 | Máximo Drawdown tolerado | CONFIG |
| MIN_TRADES | 100 | 20-500 | Cantidad mínima de transacciones | CONFIG |

---

## Ciclo de Vida de la Feature — Design Manifest

### Entrada
- Estrategia validada y certificada (AST, track-record, métricas de robustez).
- Parámetros de metas SMART configurables.

### Proceso
- Evalúa el cumplimiento milimétrico del contrato de metas SMART.
- Compara el Robustness Score contra el umbral del gatekeeper.
- Emite un veredicto definitivo.

### Salida
- Veredicto: APROBADO / RECHAZADO.
- Bloqueo de promoción si falla alguna restricción.

### Contextos de Uso

**Contexto 1: Generación de Estrategias (Módulo Generate)**
- Define las condiciones de fitness necesarias para avanzar.

**Contexto 2: Validación (Módulo Validate)**
- Actúa como el filtro final ineludible de aprobación para el paso a la Fase 3.

---

## Tareas (TTRs)

### **TTR-001: Motor de Cálculo del Puntaje de Robustez**
*   **Descripción:** Agregador ponderado de métricas de robustez.
*   **Métricas Core:** WFE (20%), DSR (20%), PBO (15%), ProbaRuina (25%), LogicInversion (20%).
*   **Salida:** `float` (0-100).

### **TTR-002: Panel de Veredicto (Controlador del Filtro de Calidad)**
*   **Descripción:** Compara el Puntaje final contra los umbrales configurados por el usuario.
*   **Postcondición:** Emite evento de promoción si el veredicto es APROBADO.

### **TTR-003: Validador de Umbrales Duros (Límites Duros & Metas SMART)**
*   **Descripción:** Filtro binario previo al puntaje. Si los objetivos contractuales (`max_dd`, `min_sharpe`) no se cumplen en la muestra, rechaza inmediatamente sin calcular el puntaje avanzado.

### **TTR-004: Envoltorio de Despliegue (Wrapper)**
*   **Descripción:** Lógica que asocia y envuelve el AST generado con los parámetros de riesgo del Design Manifest, desacoplando la matemática pura de la evaluación de cartera.

---

## Persistencia (Inundación de Fundamentos — ADR-0020 V2)

Cada veredicto de calidad registra el set de relevancia técnica para AI/R&D:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único del veredicto |
| | `created_at` | Timestamp de evaluación final |
| | `audit_hash` | Hash del certificado de robustez |
| **II. Soberanía** | `owner_id` | Autor que promueve la estrategia |
| | `manifest_id` | ID del contrato legal de la estrategia |
| | `institutional_tag` | Tag de cumplimiento (Firmado) |
| **III. Pesos/Arquitectura** | `logic_hash` | Hash de la lógica verificada |
| | `indicator_state_hash` | Puntaje de Robustez (WFE, DSR, PBO) |
| | `version_node_id` | ID de la versión aprobada en el DAG |
| **IV. Hardware** | `node_id` | ID del hardware físico (Gatekeeper Node) |
| | `process_id` | PID del motor de veredicto |

---

## Dependencias
- [`walk-forward-analyzer`](../features/walk-forward-analyzer.md) — provee WFE.
- [`monte-carlo-simulator`](../features/monte-carlo-simulator.md) — provee Probabilidad de Ruina.
- [`institutional-metrics`](../features/institutional-metrics.md) — provee DSR/PBO.
