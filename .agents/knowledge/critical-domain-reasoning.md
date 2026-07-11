# Razonamiento Crítico de Dominio — asociar lo que una IA normal no asocia

> **Qué es:** documento de knowledge **específico y de lectura bajo demanda** — NO forma parte de `base.md` (que es gobernanza general para todos). Lo leen los roles que **sellan decisiones** (Architect, Tech-Lead, ingenieros especialistas) **cuando el concepto en curso toca un dominio experto**. Al consultarlo, declara al invocador que lo hiciste (Gate de Reportaje de Conocimiento, `README.md`). No es una skill invocable.
>
> **Por qué existe (caso raíz, 2026-07-11):** el modelo de aplicación del Deflated Sharpe Ratio (DSR) se estaba codificando como *"N = universo total de exploraciones de toda la historia"*. En el límite eso hace crecer el Sharpe umbral sin cota y **condena a toda estrategia futura al fracaso permanente** — destruye el propósito del producto (descubrimiento continuo de alfa). Ni el Tech-Lead ni el Architect lo detectaron; **lo atrapó el propietario**. El objetivo de este documento es que ese tipo de error lo atrape el experto, no el dueño.

Antes de sellar cualquier decisión en un dominio cuantitativo, estadístico, financiero, criptográfico, de microestructura, fiscal o legal, corre estas **7 comprobaciones** y deja constancia en el ADR/doc. Si alguna falla, **DETENTE y escala al especialista**.

1. **Prueba de límite degenerado (reductio) — OBLIGATORIA.** Empuja cada parámetro al extremo (N→∞, T→0, corridas→∞, usuarios→∞, tiempo→∞, saldo→0). Si el resultado es absurdo (condena universal, coste infinito, bloqueo total, división por cero conceptual), el modelo está **incompleto**, no "aproximadamente bien". Escribe la prueba y su resultado.

2. **Gate de especialista de dominio.** El Architect y el Tech-Lead son **generalistas**. Toda decisión dentro de un dominio con especialista (Quant-engineer = estadística/finanzas cuantitativas; Bridge/cripto; microestructura; fiscal/legal) **DEBE pasar por ese skill ANTES** de codificarse como canon. Un framing "que suena riguroso" de un generalista **no es fuente válida** en dominio especialista.

3. **Desconfía del rigor impresionante.** Cuanto más impresionante y *load-bearing* suene una afirmación (p. ej. "sistema de registro del presupuesto de pruebas"), **más adversarial** debe ser su revisión, no menos. El brillo retórico es señal de **autoridad no escrutada**, no de solidez.

4. **Re-deriva los supuestos; no cites la conclusión.** Al adoptar un concepto externo (un teorema, una métrica, un patrón), enumera sus **supuestos** y verifica uno por uno si se cumplen en NUESTRO contexto. *(El DSR asume ensayos independientes; en Drasus las estrategias comparten datos → están correlacionadas → se usa N efectivo vía ρ, no N crudo.)* Cita el supuesto en el doc, no solo el resultado.

5. **Separa el primitivo de la política.** Almacenar datos (un ledger) **≠** la política que los interpreta (cómo se acota, agrupa o pondera). No colapses una política matizada en un primitivo de almacenamiento. *(El `expedition-ledger` guarda trials; CÓMO se computa N — por bloque de decisión, por familia, ajustado por correlación, con reset — es una política aparte.)*

6. **Cierre transitivo del concepto.** Un concepto transversal rara vez toca una sola feature. Enumera **TODOS** los puntos donde ocurre el mismo evento. *(El sesgo de selección ocurre en CADA operación de selección: minería, filtro top-X%, optimización de parámetros, construcción de portafolio, selección de cluster, ranking por fitness compuesta.)*

7. **Distingue tipos; no promedies.** Cuando algo "se aplica a varios casos", pregunta si se aplica **igual** a cada tipo. *(La corrección por pruebas múltiples aplica a la SELECCIÓN, no a la validación OOS ni a Montecarlo; y el WFO re-optimiza → sesgo por ventana, a diferencia del WFA puro OOS. Tratarlos igual es el error.)*

**Regla de cierre:** el propietario **no** debe ser el último filtro contra errores que destruyen el producto. Si una decisión de dominio llega a canon sin estas 7 comprobaciones documentadas y sin el visto bueno del especialista, es un defecto de proceso, no un descuido aislado.

Relacionado: [[roles-explicitos-y-subagentes]], [[politica-de-pruebas-y-validacion]].
