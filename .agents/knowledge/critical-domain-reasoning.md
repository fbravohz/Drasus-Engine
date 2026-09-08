# Razonamiento Crítico de Dominio

Filtro obligatorio antes de sellar una decisión de dominio cuantitativo, estadístico, financiero, criptográfico, de microestructura, fiscal o legal — 7 comprobaciones que atrapan el modelo que se rompe en el límite o el framing de generalista que un especialista nunca validó. Documento de lectura bajo demanda (no forma parte de `governance.md`); al consultarlo, declara al invocador que lo hiciste.

Slug ID: `CRITICAL-DOMAIN-REASONING`.

Tamaño: `5676 caracteres (~1419 tokens aprox., ratio 4:1)`.

---

## CRITICAL-DOMAIN-REASONING-FV — Filtro de Verificación

**Aplica a:** Architect, Tech-Lead, ingenieros especialistas, como último paso antes de sellar una decisión de dominio experto.

* **CRITICAL-DOMAIN-REASONING-FV-1:** ¿corrí las 7 comprobaciones y dejé constancia de cada una en el ADR/doc, deteniéndome a escalar al especialista si alguna falló (`CRITICAL-DOMAIN-REASONING-1`)?
* **CRITICAL-DOMAIN-REASONING-FV-2:** ¿la decisión pasó por el especialista de dominio antes de sellarse como canon, sin dejar que el propietario fuera el último filtro (`CRITICAL-DOMAIN-REASONING-2`)?

De ser satisfactoria cada validación de la lista, declara `CRITICAL-DOMAIN-REASONING - FILTRO DE VERIFICACION APROBADO`.

---

## CRITICAL-DOMAIN-REASONING-1 — Las 7 Comprobaciones

Antes de sellar cualquier decisión en un dominio cuantitativo, estadístico, financiero, criptográfico, de microestructura, fiscal o legal, corre estas 7 comprobaciones y deja constancia en el ADR/doc. Si alguna falla, DETENTE y escala al especialista.

**Aplica a:** Architect, Tech-Lead, ingenieros especialistas, al sellar una decisión de dominio experto.

* **CRITICAL-DOMAIN-REASONING-1.1 — Prueba de Límite Degenerado (Reductio):**

  obligatoria. Empuja cada parámetro al extremo (N→∞, T→0, corridas→∞, usuarios→∞, tiempo→∞, saldo→0). Si el resultado es absurdo (condena universal, coste infinito, bloqueo total, división por cero conceptual), el modelo está incompleto, no "aproximadamente bien". Escribe la prueba y su resultado.

* **CRITICAL-DOMAIN-REASONING-1.2 — Filtro de Especialista de Dominio:**

  el Architect y el Tech-Lead son generalistas. Toda decisión dentro de un dominio con especialista (Quant-engineer = estadística/finanzas cuantitativas; Bridge/cripto; microestructura; fiscal/legal) DEBE pasar por ese skill ANTES de codificarse como canon. Un framing "que suena riguroso" de un generalista no es fuente válida en dominio especialista.

* **CRITICAL-DOMAIN-REASONING-1.3 — Desconfía del Rigor Impresionante:**

  cuanto más impresionante y *load-bearing* suene una afirmación (p. ej. "sistema de registro del presupuesto de pruebas"), más adversarial debe ser su revisión, no menos. El brillo retórico es señal de autoridad no escrutada, no de solidez.

* **CRITICAL-DOMAIN-REASONING-1.4 — Re-Deriva los Supuestos, No Cites la Conclusión:**

  al adoptar un concepto externo (un teorema, una métrica, un patrón), enumera sus supuestos y verifica uno por uno si se cumplen en nuestro contexto. Cita el supuesto en el doc, no solo el resultado.
  * ❌ *Incorrecto:* aplicar el Deflated Sharpe Ratio (DSR) asumiendo ensayos independientes sin verificarlo.
  * ✅ *Correcto:* el DSR asume ensayos independientes; en Drasus las estrategias comparten datos → están correlacionadas → se usa N efectivo vía ρ, no N crudo.

* **CRITICAL-DOMAIN-REASONING-1.5 — Separa el Primitivo de la Política:**

  almacenar datos (un ledger) no es lo mismo que la política que los interpreta (cómo se acota, agrupa o pondera). No colapses una política matizada en un primitivo de almacenamiento.
  * ❌ *Incorrecto:* asumir que el `expedition-ledger` (que guarda trials) ya define cómo se computa N.
  * ✅ *Correcto:* el `expedition-ledger` guarda trials; cómo se computa N — por bloque de decisión, por familia, ajustado por correlación, con reset — es una política aparte, documentada por separado.

* **CRITICAL-DOMAIN-REASONING-1.6 — Cierre Transitivo del Concepto:**

  un concepto transversal rara vez toca una sola feature. Enumera TODOS los puntos donde ocurre el mismo evento.
  * ✅ *Correcto:* el sesgo de selección ocurre en CADA operación de selección: minería, filtro top-X%, optimización de parámetros, construcción de portafolio, selección de cluster, ranking por fitness compuesta.

* **CRITICAL-DOMAIN-REASONING-1.7 — Distingue Tipos, No Promedies:**

  cuando algo "se aplica a varios casos", pregunta si se aplica igual a cada tipo.
  * ❌ *Incorrecto:* tratar la corrección por pruebas múltiples como si aplicara igual a la validación OOS y a Montecarlo.
  * ✅ *Correcto:* la corrección por pruebas múltiples aplica a la SELECCIÓN, no a la validación OOS ni a Montecarlo; el WFO re-optimiza → sesgo por ventana, a diferencia del WFA puro OOS — tratarlos igual es el error.

**Filtro de verificación:** `CRITICAL-DOMAIN-REASONING-FV-1`.

---

## CRITICAL-DOMAIN-REASONING-2 — Regla de Cierre

El propietario no debe ser el último filtro contra errores que destruyen el producto.

* **CRITICAL-DOMAIN-REASONING-2.1 — Defecto de Proceso, No Descuido Aislado:** si una decisión de dominio llega a canon sin las 7 comprobaciones de `CRITICAL-DOMAIN-REASONING-1` documentadas y sin el visto bueno del especialista, es un defecto de proceso, no un descuido aislado.

**Filtro de verificación:** `CRITICAL-DOMAIN-REASONING-FV-2`.

**Lectura bajo demanda:** [[.agents/memory/roles-explicitos-y-subagentes.md]] — para el detalle de qué rol es generalista y cuál especialista de dominio.

**Lectura bajo demanda:** [[.agents/memory/politica-de-pruebas-y-validacion.md]] — para el detalle de cómo verificar una decisión de dominio con pruebas reales.
