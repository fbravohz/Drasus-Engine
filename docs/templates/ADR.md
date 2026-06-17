# Plantilla: ADR (Decisión Arquitectónica — Dura Años)

**¿Cuándo usar?** Cuando decides algo que NO va a cambiar en 6 meses. Ej: "Usamos Rust", "Estructura de 8 módulos", "Precios siempre enteros".

## Formato

**Título:** [Decisión corta]

* **¿Qué decidimos?** (1 línea clara)
* **¿Por qué lo decidimos?** (El problema que resuelve)
* **¿Qué restricciones tiene?** (Lo que NUNCA puede pasar)
* **¿Cómo se vería en el sistema?** (Efecto observable, SIN código)
* **¿Qué cuesta?** (Trade-off real)
* **Trazabilidad:** [Nombre de las Features que implementan este ADR]

---

## Ejemplo (CORRECTO)

**ADR-0007: Estrategias pueden pausarse antes de retirarse**

* **Decisión:** Entre "está operando" y "está retirada", siempre hay un estado "pausada" donde el usuario puede cambiar de idea.

* **Problema que resuelve:** Si una estrategia tiene una mala semana, queremos poder pausarla 1-2 días sin borrarla para siempre. Después el usuario decide si la reactiva o la retira.

* **Restricciones:**
  - No puedes retirar una estrategia directamente (siempre pasa por PAUSED primero)
  - La ventana de veto (tiempo para cambiar idea) es configurable pero NO infinita

* **Efecto observable:**
  - Usuario ve opción "Pausar por 1 día"
  - Después de ese día, puede reactivarla o retirarla
  - Si no decide nada, se retira automáticamente

* **Costo:** Complejidad extra en la máquina de estados (más transiciones). Beneficio: flexibilidad operacional sin perder datos históricos.

---

Ver reglas transversales (Lo Prohibido, Regla de Oro, Checklist) en [`TEMPLATES.md`](./TEMPLATES.md).
