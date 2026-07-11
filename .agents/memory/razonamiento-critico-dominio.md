---
name: razonamiento-critico-dominio
description: Cómo TL/Architect deben aplicar ojo crítico experto antes de sellar decisiones de dominio (cuant/estadística/cripto/legal); nació del casi-desastre del DSR.
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 7afc36c0-aab0-4801-8ee7-bd8897008d98
---

El propietario detectó un error que TL y Architect NO vieron: el modelo de aplicación del Deflated Sharpe Ratio se estaba codificando como "N = universo total de exploraciones de toda la historia", lo que en el límite condena a TODA estrategia futura al fracaso permanente (destruye el propósito de Drasus: descubrimiento continuo de alfa). El dueño NO quiere ser el último filtro contra errores que destruyen el producto — "para eso contrata expertos".

**Why:** un framing plausible de un generalista (TL/Architect) se convirtió en canon en un dominio especialista (estadística cuantitativa) sin escrutinio adversarial ni revisión del especialista. El Quant-engineer **existía y nunca se consultó** — falla de proceso, no descuido aislado.

**How to apply:** protocolo canónico en [[critical-domain-reasoning]] (`.agents/knowledge/critical-domain-reasoning.md`) — 7 comprobaciones obligatorias antes de sellar decisión de dominio: (1) prueba de límite degenerado/reductio, (2) gate de especialista, (3) desconfiar del rigor impresionante, (4) re-derivar supuestos no citar conclusión, (5) separar primitivo de política, (6) cierre transitivo del concepto, (7) distinguir tipos no promediar. Regla dura: decisión en dominio con especialista (Quant/Bridge-cripto/microestructura/fiscal-legal) DEBE pasar por ese skill ANTES de canonizarse. Correcciones técnicas concretas del caso DSR: usar N efectivo por correlación ρ (no N crudo); DSR solo en puntos de SELECCIÓN; WFO re-optimiza → sesgo por ventana (≠ WFA puro OOS → PSR); Montecarlo no incurre sesgo de selección; reset de N por data fresca (T nuevo) y por familia/espacio de búsqueda. Relacionado con [[roles-explicitos-y-subagentes]].
