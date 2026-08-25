### AGENT-DEBT-007 · Las etapas de `tech-lead/SKILL.md` están mal numeradas

- **Severidad:** 🟠 Media
- **Origen:** señalado directamente por el propietario.
- **Descripción:** la secuencia de etapas del pipeline en `tech-lead/SKILL.md` rompe su propia numeración al menos una vez: entre "Etapa 6 — Validación Cuantitativa Post-Código" y "Etapa 7 — Retroalimentación" se intercala un bloque completo, "Barrido de Cierre Documental (OBLIGATORIO al cerrar cada iteración/Story)", sin número de etapa propio — queda fuera de la secuencia `0 / 0.3 / 0.4 / 0.5 / 1 / 2 / 3 / 4 / 5 / 6 / 7` aunque es un paso obligatorio con su propio trigger y checklist. El propietario indicó que el problema no se limita a este caso concreto y pidió una revisión completa del documento, no solo de este punto.
- **Impacto actual:** riesgo de confusión operativa — un agente que recorra las etapas por número puede tratar el bloque sin numerar como opcional o saltarlo, aunque el texto lo marca "OBLIGATORIO".
- **Causa raíz:** el documento creció por adiciones sucesivas de reglas del usuario sin una pasada de renumeración/reestructuración.
- **Disparador de pago:** próxima revisión dedicada de `tech-lead/SKILL.md` — candidato natural para `context-auditor` en Modo Documental.
- **Plan:** auditar la sección completa de etapas, corregir la numeración (asignar número de etapa al "Barrido de Cierre Documental" o declarar explícitamente por qué no lleva uno), y verificar que no haya más huecos de secuencia en el resto del documento.
- **Estado:** Abierta.
