# Adversarial Noise Agent (Red Team AI)

**Carpeta:** `./features/adversarial-noise-agent/`
**Estado:** En Diseño
**Última actualización:** 2026-04-30
**Decisión Arquitectónica Asociada:** ADR-0020

---

## ¿Qué es?

El "Adversarial Noise Agent" o Red Team AI es el gran villano del auditor de robustez. Es un componente que ejecuta **Data Perturbation** inteligente. En lugar de simular ruido estático y aburrido, este agente ataca deliberadamente a la estrategia.

**Problema que resuelve:** Las estrategias con Stop Loss "milimétricos" a veces sobreviven por pura suerte de que el precio no los tocó por un pip. El agente IA se da cuenta de los puntos ciegos (zonas de baja liquidez) e inyecta "Slippage Agresivo" y perturbaciones de volatilidad (ATR) en los altos y bajos históricos. Si la estrategia se rompe al cambiarle 1-2 pips de spread en los peores momentos, no sirve.

---

## Narrativa de Producto

**Capacidad:** `CAP-04` — El guantelete de robustez

**Historia de usuario:** Como investigador de estrategias, quiero que un atacante deliberado busque los puntos ciegos de mi estrategia e intente romperla a propósito, para descubrir su punto de quiebre antes de que el mercado real lo encuentre por mí.

**El problema (técnico):** una estrategia con órdenes de protección muy ajustadas puede sobrevivir en el backtest por pura casualidad de que el precio nunca tocó ese límite por un margen mínimo. El ruido genérico y aleatorio no encuentra necesariamente esos puntos ciegos porque no busca activamente; el peligro real está concentrado en zonas específicas —momentos de baja liquidez, mechas de vela cercanas a un stop— que un atacante inteligente sí sabe encontrar.

**La solución (técnica):** un agente que examina el historial de operaciones y busca deliberadamente las victorias más ajustadas —las que sobrevivieron por poco margen— y ahí concentra el ataque: ensancha las mechas de las velas según la volatilidad real de ese momento, fuerza el peor deslizamiento y el spread más ancho jamás registrado en esa franja horaria, y recalcula el resultado bajo esas condiciones hostiles. Las perturbaciones nunca inventan precios imposibles: quedan acotadas a la volatilidad histórica real del instrumento.

**Explicado sin tecnicismos:** piensa en un auto que pasa la inspección técnica sin problema, pero nadie lo llevó nunca a un camino con baches de verdad. Un mecánico desconfiado no se conforma con la carretera lisa de la prueba oficial: lo lleva directo a los baches más profundos que existen en esa ciudad, buscando dónde se va a romper primero.

Esta pieza es ese mecánico desconfiado, aplicado a tu estrategia. No le aplica ruido genérico y parejo: busca tus victorias más apretadas —las que ganaron por un pelo— y ahí concentra el peor escenario posible que el mercado real ya ha producido alguna vez. Si tu estrategia sobrevive a ese ataque dirigido, es porque de verdad tiene margen. Si se rompe, ese margen nunca existió: solo tuviste la fortuna de que el mercado no te probó tan duro cuando importaba.

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

- **Inundación de Fundaciones (ADR-0020):** 
    - **Perfil B (IA / R&D):** red-team AI / perturbación adversarial = R&D, no auditoría.
    - **I. Identidad & Integridad:** `id`, `created_at`, `updated_at`, `audit_hash`, `audit_chain_hash`, `event_sequence_id`.
    - **II. Soberanía & Propiedad:** `owner_id`, `institutional_tag`, `manifest_id`.
    - **III. Pesos/Arquitectura (subset):** `logic_hash`, `data_snapshot_id`, `version_node_id`.
    - **IV. Infraestructura & Ops:** `process_id`, `node_id`.
