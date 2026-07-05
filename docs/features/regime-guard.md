# Regime Guard

**Carpeta:** `./features/regime-guard/`
**Estado:** Especificación / Prioritario
**Última actualización:** 2026-06-11
**Decisiones Arquitectónicas Asociadas:** ADR-0108, ADR-0110

---

---

## ¿Qué es?

Componente de coherencia de mercado encargado de imponer la compatibilidad entre el modelo y el entorno. Su misión es la **Prevención de Mismatch**: asegura que una estrategia solo opere en los regímenes para los que tiene ventaja estadística probada.

**Veredicto de Coherencia como Acción del Genoma de Régimen y Filtro de Entorno (ADR-0108/ADR-0110):** el veredicto COMPATIBLE/INCOMPATIBLE de este componente es la materialización FIJA del Gen de Acción de máscara binaria Permitido/Prohibido del Dominio de Régimen y Filtro de Entorno. Cuando ese genoma está activo, el motor evolutivo puede condicionar este veredicto a combinaciones de sus cuatro Genes de Condición de Estado (exponente de Hurst, entropía de Shannon del volumen, pendientes multinivel de Hull MA, `regime_label` de [`hmm-regime-detection`](./hmm-regime-detection.md)), en lugar de — o además de — el "Hábitat Natural" fijo declarado en diseño.

---

## Comportamientos Observables

- [ ] Compara el régimen actual en tiempo real con el "Hábitat Natural" de la estrategia.
- [ ] **HMM Regime Detection Daily:** Recalcula el régimen global cada día y detecta transiciones estructurales.
- [ ] **Volatility Targeting:** Ajusta la exposición (size) inversamente a la volatilidad para mantener un Sharpe constante (PRD §5.3.4).
- [ ] Bloquea la operativa o sugiere rebalanceo si el mercado muta a un estado hostil.

---

## Ciclo de Vida de la Feature — Regime Guard

### Entrada
- Etiqueta de "Régimen Soportado" de la estrategia (pertenencia de diseño).
- Clasificación de régimen actual del módulo `ingest`.
- Nivel de confianza del modelo HMM.
- Genes de Condición de Estado del Dominio de Régimen y Filtro de Entorno (exponente de Hurst, entropía de Shannon del volumen, pendientes multinivel de Hull MA), cuando ese genoma está activo (ADR-0110).

### Proceso
- Evalúa la coherencia entre el diseño y la realidad en cada barra.
- Si hay mismatch (Ej: Estrategia Trend-Follower en mercado Mean-Reverting):
  1. Verifica si el cambio tiene confianza > umbral (ej: 80%).
  2. Verifica si el cambio ha persistido por N barras (Lookback de confirmación).

### Salida
- **Veredicto de Coherencia:** COMPATIBLE / INCOMPATIBLE.
- **Mismatch Alert:** Señal enviada al orquestador de retiro.

### Contextos de Uso
**Contexto 1: Retiro (Módulo Withdraw)**
- Causa raíz principal para pausar una estrategia que aún no ha perdido dinero pero cuyo entorno se ha vuelto hostil.
**Contexto 2: Ingesta (Módulo Ingest)**
- Valida que la clasificación de régimen sea útil para el consumo operativo.

---

## Tareas (TTRs) — Herencia de Módulo Retirar

### TTR-001: Validación de Mismatch de Régimen
*   **Descripción:** Lógica de comparación de etiquetas de régimen entre diseño y realidad operativa.

### TTR-002: Filtro de Confianza (Anti-Whipsaw)
*   **Descripción:** Asegura que solo se actúe ante cambios de régimen con confianza estadística > 80% (HMM).

### TTR-003: Recálculo Diario de Régimen (HMM Daily)
*   **Descripción:** Daemon que recalcula las probabilidades de estado HMM al cierre de cada sesión.
*   **Postcondición:** Emite evento `regime_transition_detected` si el estado dominante cambia.

### TTR-004: Ajuste por Volatility Targeting
*   **Descripción:** Calcula el `Volatility_Scaler` basado en el ATR histórico vs actual.
*   **Regla:** Si la volatilidad sube, el tamaño de la posición baja proporcionalmente para estabilizar el riesgo (ADR-0010).

### TTR-005: Veredicto de Coherencia como Gen de Acción del Genoma de Régimen y Filtro (ADR-0108/ADR-0110)
*   **Descripción:** Cuando el Genoma de Régimen y Filtro de Entorno está activo, el veredicto COMPATIBLE/INCOMPATIBLE deja de depender únicamente del "Hábitat Natural" fijo de la estrategia y se vuelve direccionable como un nodo `wildcard_group` del dominio, evaluado contra combinaciones de los cuatro Genes de Condición de Estado (Hurst, entropía de Shannon, pendientes Hull MA, `regime_label`).
*   **Postcondición:** Un Manifest sin ese genoma activo conserva el comportamiento actual (comparación contra el "Hábitat Natural" fijo).

---

## Gobernanza y Estándares (Fijos)
- **Local-First (ADR-0016):** 100% Local. El análisis de coherencia debe ser instantáneo.
- **Genomas Modulares por Dominio (ADR-0108/ADR-0110):** El veredicto COMPATIBLE/INCOMPATIBLE es el Gen de Acción de máscara del Dominio de Régimen y Filtro de Entorno. Ver Registro de Dominios Genómicos en [`SAD.md`](../SAD.md) §2.3.

## Persistencia (Inundación de Fundamentos — ADR-0020)

Toda evaluación de compatibilidad de régimen registra el set de relevancia técnica:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único de la validación |
| | `created_at` | Timestamp de comprobación |
| | `updated_at` | Timestamp de última modificación del registro |
| | `audit_hash` | Hash del estado del mercado (HMM) |
| | `audit_chain_hash` | Hash del historial de coherencia |
| | `event_sequence_id` | Secuencia del evento de validación |
| **II. Soberanía** | `owner_id` | Autor de la estrategia |
| | `manifest_id` | ID del contrato de diseño legal |
| | `access_token_id` | Token de autorización operativa |
| **III. Pesos/Arquitectura** | `logic_hash` | Hash de la matriz de compatibilidad |
| | `indicator_state_hash` | Snapshot del estado HMM detectado |
| | `version_node_id` | ID del modelo HMM de referencia |
| **IV. Hardware** | `node_id` | ID del hardware físico |
| | `process_id` | PID del protector de régimen |


---

## Estructura Interna (FCIS — ADR-0002)
- **Core (Lógica Pura):** Matriz de compatibilidad de regímenes en `guard_logic.rs`.
- **Shell (Infraestructura):** Integración con el clasificador de régimen HMM.
- **Frontera Pública:** Contrato `is_regime_compatible(strategy_id)`.

---

## Dependencias
**Consumido por:** `withdraw`.
**Depende de:** `hmm-regime-detection`.
