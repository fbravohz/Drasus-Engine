# Robustness Score Aggregator (Scoring Ponderado)

**Carpeta:** `./features/robustness-score-aggregator/`
**Estado:** En Diseño
**Última actualización:** 2026-04-29
**Decisión Arquitectónica Asociada:** ADR-0058 (Política de Scoring Ponderado de Robustez)

---

## ¿Qué es?

El agregador de score de robustez es el motor de consolidación que reemplaza el viejo enfoque binario de "Muerte Súbita". Toma los 5 resultados individuales del guantelete de robustez y los consolida en un único score ponderado de 0 a 100.

**Problema que resuelve:** Descartar estrategias por fallar un solo test genera parálisis por análisis. Una estrategia con WFA excelente pero Monte Carlo mediocre no debería ser rechazada automáticamente — debería obtener un score que refleje su perfil completo de robustez.

---

## Comportamientos Observables

- [ ] El sistema recibe 5 scores individuales (WFA, MC Trades, MC Tóxico, CPCV/PBO, Ockham), uno por cada test del guantelete de robustez.
- [ ] Aplica pesos configurables a cada test y calcula el score ponderado final: `(WFA × Peso_WFA) + (MC_Trades × Peso_MC_Trades) + (MC_Tóxico × Peso_MC_Tóxico) + (CPCV_PBO × Peso_CPCV_PBO) + (Ockham × Peso_Ockham)`.
- [ ] La suma de los 5 pesos siempre debe ser exactamente 100%. Si no lo es, el sistema rechaza la configuración con error explícito.
- [ ] El score final se trunca al rango 0-100.
- [ ] Estrategia con score > 75 es marcada como "Aprobable".

---

## Restricciones

- **FIJO:** La suma de los 5 pesos debe ser exactamente 100%. Configuraciones inválidas son rechazadas en el arranque.
- **MANDATORIO (Fail-Fast):** Si cualquier test marcado como **Fase 0 (Gatekeeper)** falla su umbral mínimo, el agregador emite un veredicto de RECHAZO inmediato y aborta la ejecución de tests de fases superiores (Short-Circuit), ahorrando ciclos de cómputo.
- **FIJO:** El score final es inmutable una vez calculado para una versión de estrategia.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| WFA_WEIGHT | 30% | 10% - 50% | Peso del test Walk-Forward en el score final. | CONFIG |
| MC_TRADES_WEIGHT | 25% | 10% - 40% | Peso del Monte Carlo de permutación de trades. | CONFIG |
| MC_TOXIC_WEIGHT | 20% | 10% - 40% | Peso del Monte Carlo de eventos letales (Prop Firms). | CONFIG |
| CPCV_PBO_WEIGHT | 15% | 5% - 30% | Peso del test CPCV/PBO (probabilidad de sobreajuste). | CONFIG |
| OCKHAM_WEIGHT | 10% | 5% - 30% | Peso de la penalización por complejidad (Ockham). | CONFIG |
| APPROVAL_THRESHOLD | 75 | 50 - 95 | Score mínimo para considerar una estrategia "Aprobable". | CONFIG |

---

## Ciclo de Vida de la Feature — Robustness Score Aggregator

### Entrada
- 5 scores individuales normalizados (0-100), uno por cada test del guantelete:
  - WFA score (resultado del walk-forward-analyzer)
  - MC Trades score (resultado del monte-carlo-simulator, modo Trades)
  - MC Tóxico score (resultado del monte-carlo-simulator, modo Tóxico)
  - CPCV/PBO score (resultado del walk-forward-analyzer, métrica PBO)
  - Ockham score (resultado del complexity-penalization)
- Matriz de pesos configurables (deben sumar 100%).

### Proceso
- Valida que los 5 pesos sumen exactamente 100%.
- Multiplica cada score individual por su peso correspondiente.
- Suma los 5 productos para obtener el score ponderado final.
- Trunca al rango 0-100.
- Compara contra el umbral de aprobación.

### Salida
- `final_robustness_score` (0-100).
- `approval_status` (APPROVABLE / BELOW_THRESHOLD).
- `score_breakdown`: Desglose de cada contribución individual al score final.

### Contextos de Uso

**Contexto 1: Consolidación del Guantelete de Robustez (Validate)**
- Al finalizar los 5 tests, el módulo de validación invoca al agregador para obtener el score final.
- El score determina el veredicto de aprobación y se transmite al Verdict Engine para explicación en lenguaje natural.
- El score se almacena en los metadatos de la estrategia y se transmite al módulo de ejecución para dimensionamiento de posición.

---

## Tareas (TTRs)

### **TTR-001: Motor de Cálculo del Score Ponderado**
* **¿Cuál es el problema?** Necesitamos consolidar 5 métricas de distinta naturaleza en un solo número accionable.
* **¿Qué tiene que pasar?** El sistema recibe los 5 scores individuales, aplica los pesos configurados y calcula el score ponderado final. Valida que la suma de pesos sea 100% antes de calcular.
* **¿Cómo sé que está hecho?**
  - [ ] Con pesos default y todos los scores en 80, el score final es 80.
  - [ ] Si cambio WFA_WEIGHT a 50% (ajustando los demás), el score refleja la nueva distribución.
  - [ ] Si la suma de pesos no es 100%, el sistema lanza error de configuración.
* **¿Qué no puede pasar?** No puede calcularse si falta alguno de los 5 scores individuales. No puede exceder 100 ni ser menor que 0.

### **TTR-002: Desglose de Contribuciones y Transmisión a Sizing**
* **¿Cuál es el problema?** El usuario y el módulo de ejecución necesitan saber no solo el score final sino qué componentes contribuyeron.
* **¿Qué tiene que pasar?** El sistema genera un desglose detallado que muestra la contribución de cada test al score final. Este desglose se empaqueta para transmisión al módulo de ejecución.
* **¿Cómo sé que está hecho?**
  - [ ] El reporte muestra: "WFA: 24/30, MC Trades: 20/25, MC Tóxico: 18/20, CPCV/PBO: 12/15, Ockham: 8/10 → Total: 82/100".
  - [ ] El módulo de ejecución recibe el score como parámetro de entrada para dimensionamiento de posición.

---

## Gobernanza y Estándares (Fijos)

- **Inundación de Fundaciones (ADR-0020 V2):**
  - Perfil: AI / R&D.
  - **I. Identidad & Integridad:** `id`, `created_at`, `audit_hash`, `event_sequence_id`.
  - **II. Soberanía & Propiedad:** `owner_id`, `institutional_tag`, `manifest_id`.
  - **III. Linaje Alpha & Datos:** `version_node_id`, `logic_hash`, `data_snapshot_id`.
  - **IV. Infraestructura & Ops:** `process_id`, `node_id`.
- **Rastro de Evidencia:** Emite `final_robustness_score` y `score_breakdown` para consumo del módulo de feedback (veredicto de calidad Pardo) y del módulo de ejecución (dimensionamiento de posición).

---

## Dependencias

**Consumido por:** `validate`, `execute`, `feedback`.
**Depende de:** `monte-carlo-simulator`, `walk-forward-analyzer`, `complexity-penalization`.