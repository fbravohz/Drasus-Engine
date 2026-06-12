# DRL Parameter Tuning — Optimización con Reinforcement Learning [MOONSHOT]

**Carpeta:** `./moonshots/drl-parameter-tuning/`
**Estado:** Moonshot (Prioridad Baja / I+D)
**Última actualización:** 2026-04-11

---

## ¿Qué es?

Es una feature de investigación avanzada que utiliza **Deep Reinforcement Learning (DRL)** para sintonizar dinámicamente los hiper-parámetros de las estrategias generadas durante la fase de evolución. 

A diferencia de la optimización estática (NSGA-II), el agente DRL aprende qué combinaciones de parámetros funcionan mejor en regímenes específicos, acelerando la convergencia térmica del módulo `generate`.

---

## Comportamientos Observables

- [ ] Durante el ciclo de generación, un agente DRL observa el rendimiento de la población y "sugiere" micro-ajustes de parámetros.
- [ ] El sistema reporta el "Delta de Eficiencia" (cuántas generaciones se ahorraron gracias al agente DRL).
- [ ] Permite visualizar la política de aprendizaje del agente (qué parámetros está priorizando).

---

## Restricciones

- **NO BLOQUEANTE:** Si el agente DRL no converge, el sistema debe caer (fallback) a la mutación genética estándar del NSGA2.
- **DETERMINISMO:** El agente debe usar una semilla fija para asegurar que el proceso de generación sea 100% reproducible.
- **AISLAMIENTO:** Debe correr en hilos/procesos separados para no penalizar la latencia del core.

---

## Parámetros Configurables

| Parámetro | Default | Qué hace |
|---|---|---|
| DRL_LEARNING_RATE | 0.001 | Tasa de aprendizaje del agente |
| TARGET_REWARD | "Sharpe Ratio" | Qué métrica intenta maximizar el agente |
| EPISODE_LENGTH | 100 | Cuántas generaciones dura un episodio de aprendizaje |

---

## Tareas (TTRs)

### TTR-001: Implementar Entorno de Entrenamiento (Gym-like)
Crear un wrapper para el proceso de generación que el agente DRL pueda observar como un entorno de refuerzo.

### TTR-002: Integración con NSGA-II
Implementar el puerto que permite al agente inyectar sugerencias de parámetros en la fase de "cross-over" y "mutación" de la feature `nsga2-optimizer`.

---

## Dependencias

**Consumido por:** `generate`
**Depende de:** `nsga2-optimizer`, `institutional-metrics`.
