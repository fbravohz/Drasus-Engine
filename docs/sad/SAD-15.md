## 15. Riesgos y Mitigaciones

| Riesgo | Impacto | Mitigación |
|---|---|---|
| Dos cambios simultáneos de estado | Estado confuso, pérdida de dinero | Transacción atómica en base de datos. |
| Lógica pura toca base de datos | Rompe compilación automática, lento | Pruebas + verificación automática que prohibe acceso a datos en lógica pura. |
| Módulo A consulta tabla de módulo B | Acoplamiento (cambios rompen todo) | Regla de diseño; revisión de código. |
| Control de cambios confunde tablas | Inconsistencia de base de datos | Una sola fuente de verdad para cambios (archivo centralizado). |
| Librerías vectoriales se quedan sin memoria | Crash en producción | Procesar en lotes pequeños; pruebas de volumen. |

---

