# Adversarial Noise Agent (Red Team AI)

**Carpeta:** `./features/adversarial-noise-agent/`
**Estado:** En Diseño
**Última actualización:** 2026-04-30
**Decisión Arquitectónica Asociada:** ADR-0020 V2

---

## ¿Qué es?

El "Adversarial Noise Agent" o Red Team AI es el gran villano del auditor de robustez. Es un componente que ejecuta **Data Perturbation** inteligente. En lugar de simular ruido estático y aburrido, este agente ataca deliberadamente a la estrategia.

**Problema que resuelve:** Las estrategias con Stop Loss "milimétricos" a veces sobreviven por pura suerte de que el precio no los tocó por un pip. El agente IA se da cuenta de los puntos ciegos (zonas de baja liquidez) e inyecta "Slippage Agresivo" y perturbaciones de volatilidad (ATR) en los altos y bajos históricos. Si la estrategia se rompe al cambiarle 1-2 pips de spread en los peores momentos, no sirve.

---

## Comportamientos Observables

- [ ] **Ataque de Volatilidad (ATR Noise):** Altera sutilmente los High y Low de las barras históricas basándose en la volatilidad dinámica (ATR).
- [ ] **Ataque de Cisne Negro:** El agente Red Team busca activamente los trades ganadores donde la densidad de volumen fue bajísima y simula las peores condiciones de ejecución posibles.
- [ ] **Inyección de Slippage Agresivo:** Obliga a los peores fills posibles para estresar los límites de riesgo de la estrategia.
- [ ] Si el PnL colapsa estrepitosamente bajo el ataque, la estrategia es rechazada.

---

## Restricciones

- **FIJO:** Las inyecciones de ruido no pueden inventar precios ridículos; están acotadas matemáticamente a desviaciones estándar de la volatilidad histórica registrada en ese momento.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| RED_TEAM_AGGRESSION | 1.0 | 0.5 - 3.0 | Multiplicador del ruido (1.0 = ruido normal de mercado, 3.0 = terror extremo). | CONFIG |
| SLIPPAGE_INJECTION_RATE | 0.20 | 0.05 - 0.50 | Porcentaje de operaciones en las que se inyecta el peor slippage posible. | CONFIG |

---

## Ciclo de Vida de la Feature — Adversarial Noise Agent

### Entrada
- Datos históricos.
- Fills (órdenes ejecutadas) de la simulación.
- Perfil de Volatilidad (ATR).

### Proceso
- El agente lee el track-record original.
- Altera artificialmente la serie de tiempo (ensanchamiento de mechas) en momentos críticos.
- Castiga la ejecución obligando al motor a tragar slippage en puntos de baja liquidez probada.
- Re-calcula la curva de capital.

### Salida
- `adversarial_equity_curve`.
- `fragility_score_under_attack`.
- Veredicto de Resiliencia (RESILIENT / BROKEN).

### Contextos de Uso
**Contexto 1: Simulación Anti-Cisne Negro (Validate)**
- Actúa como la prueba de fuego final antes de mandar la estrategia a pelear con dinero real. Evita sorpresas por spreads nocturnos y flash crashes menores.

---

## Tareas (TTRs)

### **TTR-001: Motor Dinámico de Perturbación ATR (Data Perturbation)**
*   **¿Cuál es el problema?** El mercado real no tiene velas perfectas; siempre hay fluctuaciones milimétricas que pueden saltar un Stop ajustado.
*   **¿Qué tiene que pasar?** El sistema altera High/Low usando ruido gaussiano escalado por el ATR histórico para simular un "broker en condiciones de alto estrés".
*   **¿Cómo sé que está hecho?**
    - [ ] Hay un test de "Robustez con Ruido", y se loguea "Aplicando ruido del 1.2x ATR".
    - [ ] Estrategias con stops de 3 pips fracasan espectacularmente.

### **TTR-002: Ejecutor de Slippage Malicioso (Red Team)**
*   **¿Cuál es el problema?** Algunos backtests asumen llenados óptimos.
*   **¿Qué tiene que pasar?** El agente identifica los trades ganadores más ajustados y los fuerza a abrir con el spread más amplio registrado en esa franja horaria.
*   **¿Cómo sé que está hecho?**
    - [ ] La equidad final se reduce drásticamente (ej: -30%), pero la estrategia no llega a la quiebra (aprobando así el test).

---

## Gobernanza y Estándares (Fijos)

- **Inundación de Fundaciones (ADR-0020 V2):** 
    - Perfil: Ops / Auditoría.
    - **I. Identidad & Integridad:** `id`, `created_at`, `audit_hash`.
    - **II. Soberanía & Propiedad:** `owner_id`, `institutional_tag`, `manifest_id`.
    - **IV. Infraestructura & Ops:** `process_id`, `node_id`.
    - **V. Forense & Ejecución:** `compliance_status_id`, `risk_audit_id`.
