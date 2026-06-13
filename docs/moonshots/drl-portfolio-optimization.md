# DRL Portfolio Optimization — Ajuste Dinámico de Pesos con Reinforcement Learning

**Carpeta:** `./moonshots/`
**Estado:** Moonshot (Fase 4 - Experimental)
**Última actualización:** 2026-04-13

---

## ¿Qué es?

Utiliza agentes de **Aprendizaje por Refuerzo Profundo (DRL)** para gestionar la asignación de capital del portafolio. A diferencia de las reglas estáticas, el agente aprende de la recompensa (Sharpe/Retorno) y ajusta los pesos dinámicamente según el estado del mercado.

---

## Comportamientos Observables

- [ ] **Agent Training:** Entrenamiento de agentes PPO / DDPG sobre entornos de mercado simulados (Gym/PettingZoo).
- [ ] **Dynamic Weighting:** El agente emite un vector de pesos `[w1, w2, ..., wn]` en cada ventana de rebalanceo.
- [ ] **Reward Function:** Optimización para maximizar el retorno ajustado por riesgo y minimizar los costos de transacción.

---

## Tareas (TTRs)

### **TTR-001: Implementación del Entorno de Trading (Gym-Ready)**
*   **Descripción:** Crea el wrapper para que NautilusTrader funcione como un entorno de entrenamiento para el framework DL nativo Rust `burn`/`candle` (ADR-0112; `tch-rs` erradicado).

### **TTR-002: Entrenamiento y Validación de Políticas (Policy Training)**
*   **Descripción:** Loop de entrenamiento de agentes sobre datos históricos sanitizados.

---

## Gobernanza y Estándares (ADR-0020 V2)
- Cada sesión de entrenamiento y decisión del agente registra el **Grupo I (universal) + Perfil IA/R&D** (ADR-0020 V2).
- Metadatos: `logic_hash` (Model architecture), `data_snapshot_id` (Environment states).
