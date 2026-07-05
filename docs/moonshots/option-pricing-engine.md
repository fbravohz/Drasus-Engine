# Option Pricing Engine — Motor de Pricing de Opciones Financieras

**Carpeta:** `./moonshots/`
**Estado:** Moonshot (Post-MVP — Diferido por ADR-0140)
**Última actualización:** 2026-06-27
**Decisión Arquitectónica Asociada:** ADR-0140 (Opciones Financieras — Diferimiento al Post-MVP con Puerta Abierta)

---

## ¿Qué es?

Motor de valoración de opciones financieras que calcula el precio teórico (fair value) y las griegas (Delta, Gamma, Theta, Vega, Rho) de cualquier contrato de opción sobre los instrumentos soportados por Drasus Engine. Implementa tres modelos de pricing escalonados por complejidad: Black-Scholes-Merton (europeas, fórmula cerrada), Binomial Cox-Ross-Rubinstein (americanas, árbol recursivo) y Monte Carlo (exóticas, validación cruzada).

**Por qué es moonshot:** El pricing es la parte "fácil" de las opciones. El problema real es la **superficie de volatilidad implícita** (IV surface): interpolar la volatilidad correcta para cada strike × vencimiento requiere datos de mercado en tiempo real y es donde la mayoría de implementaciones retail fallan. Sin una IV surface precisa, el pricing produce valores teóricos que divergen del mercado real, invalidando cualquier estrategia basada en ellos.

**Condición de activación (ADR-0140):** los cinco prerrequisitos del ADR-0140 deben cumplirse antes de implementar.

---

## Comportamientos Observables

- [ ] El motor calcula el precio teórico de una opción europea (call/put) dado el precio del subyacente, strike, tiempo al vencimiento, tasa libre de riesgo y volatilidad implícita.
- [ ] El motor calcula las cinco griegas (Delta, Gamma, Theta, Vega, Rho) para cualquier contrato de opción.
- [ ] El modelo binomial price opciones americanas (ejercicio anticipado) con convergencia configurable por número de pasos del árbol.
- [ ] El modelo Monte Carlo valida los resultados de Black-Scholes y Binomial con un margen de error configurable.
- [ ] La IV surface se construye a partir de datos de mercado (bid/ask por strike × vencimiento) y se interpola para strikes y vencimientos no cotizados.

---

## Tareas (TTRs)

### **TTR-001: Motor Black-Scholes-Merton (Europeas)**
*   **¿Cuál es el problema?** Las opciones europeas son el caso base de pricing y la referencia para validar los modelos más complejos.
*   **¿Qué tiene que pasar?** Implementar la fórmula cerrada de Black-Scholes-Merton para calls y puts europeas, con cálculo analítico de las cinco griegas.
*   **¿Cómo sé que está hecho?**
    - [ ] El precio calculado coincide con la fórmula de referencia dentro de un margen de error de 10⁻⁶.
    - [ ] Las griegas calculadas analíticamente coinciden con las derivadas numéricas (finite differences) dentro de un margen configurable.

### **TTR-002: Motor Binomial Cox-Ross-Rubinstein (Americanas)**
*   **¿Cuál es el problema?** Las opciones americanas (la mayoría de las opciones sobre acciones en mercados US) permiten ejercicio anticipado, lo que invalida Black-Scholes.
*   **¿Qué tiene que pasar?** Implementar el árbol binomial con convergencia configurable (número de pasos) y detección óptima de ejercicio anticipado en cada nodo.
*   **¿Cómo sé que está hecho?**
    - [ ] Con un número alto de pasos, el precio converge al de Black-Scholes para opciones europeas (validación cruzada).
    - [ ] El precio de una opción americana es ≥ al de su equivalente europea (el ejercicio anticipado tiene valor).

### **TTR-003: Motor Monte Carlo (Validación y Exóticas)**
*   **¿Cuál es el problema?** Se necesita un método de validación independiente y la capacidad de pricear opciones exóticas (barrier, asian, lookback) si la demanda lo justifica.
*   **¿Qué tiene que pasar?** Implementar simulación Monte Carlo con reducción de varianza (antithetic variates, control variates) y comparación contra Black-Scholes/Binomial.
*   **¿Cómo sé que está hecho?**
    - [ ] El precio Monte Carlo converge al precio analítico dentro de un intervalo de confianza configurable (95% o 99%).

### **TTR-004: Construcción de Superficie de Volatilidad Implícita (IV Surface)**
*   **¿Cuál es el problema?** La volatilidad implícita varía por strike y vencimiento (smile/skew). Sin una superficie precisa, el pricing usa una volatilidad plana que no refleja el mercado real.
*   **¿Qué tiene que pasar?** Construir la IV surface a partir de datos de mercado (quotes bid/ask por strike × vencimiento), con interpolación (SVI, SABR o interpolación cúbica) para strikes y vencimientos no cotizados directamente.
*   **¿Cómo sé que está hecho?**
    - [ ] La surface se reconstruye a partir de quotes de mercado y produce volatilidades interpoladas coherentes (sin arbitraje de calendario ni butterfly).

---

## Gobernanza y Estándares (ADR-0020)
- Perfil IA / R&D: Identidad + Soberanía + Pesos/Arquitectura + Hardware. Registro del modelo de pricing activo, la IV surface utilizada y las griegas calculadas.

---

## Dependencias

**Depende de:**
- [`option-data-ingestor`](./option-data-ingestor.md) — para datos de cadenas de opciones y IV surface.
- [`option-chain-manager`](./option-chain-manager.md) — para la estructura de contratos disponibles.

**Bloquea:**
- [`greeks-monitor`](./greeks-monitor.md) — consume las griegas calculadas por este motor.
- [`option-strategy-builder`](./option-strategy-builder.md) — necesita pricing para evaluar estrategias multi-pata.
- [`exercise-assignment-handler`](./exercise-assignment-handler.md) — necesita pricing para determinar si una opción es ITM/OTM al vencimiento.
