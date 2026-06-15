## 12. Lifecycle de Estrategia (FSM Completo)

```
                    ┌─────────────────────────────────────┐
                    │  EN PRUEBA                          │
                    │  (generación crea candidatos)       │
                    └──────────┬──────────────────────────┘
                               │ validación: aprobado
                               ▼
                    ┌─────────────────────────────────────┐
                    │  EN INCUBACIÓN                      │
                    │  (simulada; perfil config. ADR-0088)│
                    └──────────┬──────────────────────────┘
                               │ incubación: pasa validación
                               ▼
                    ┌─────────────────────────────────────┐
                    │  EJECUTANDO                         │
                    │  (ejecución viva en portafolio)     │
                    └──────────┬──────────────────────────┘
                               │ retiro detecta degradación
                               ▼
                    ┌─────────────────────────────────────┐
                    │  EN PAUSA                           │
                    │  (período para reconsiderar: 1 día) │
                    └─┬──────────────────────────────────┬┘
       usuario reactiva│                                 │ usuario decide
              (vuelve a EJECUTANDO)                       │ retiro permanente
                                                          ▼
                                              ┌─────────────────────────────────────┐
                                              │  RETIRADO                           │
                                              │  (archivado; no se reactiva solo)   │
                                              └─────────────────────────────────────┘
```

**Reglas de Transición:**
- EN PRUEBA → EN INCUBACIÓN: Después de validar (aprobado).
- EN INCUBACIÓN → EJECUTANDO: Después de incubar (pasa validación).
- EJECUTANDO → EN PAUSA: Retiro detecta degradación (rendimiento cae >30% OR pérdidas máximas >150% de lo esperado).
- EN PAUSA → EJECUTANDO: Usuario decide reactivar dentro del período.
- EN PAUSA → RETIRADO: Usuario decide retiro permanente tras el período.
- EJECUTANDO → RETIRADO: Usuario fuerza retiro inmediato (sin pasar por EN PAUSA).
- RETIRADO → X: Sin retorno automático (requiere acción manual).

### 12.1 El Ciclo de Refinamiento (Refine Cycle)

El sistema no es un pipeline lineal unidireccional. Permite bucles de realimentación (loops) donde una estrategia o portafolio puede retroceder a fases anteriores para ajustes sin perder su identidad ni su historial en el DAG de versiones:

1.  **Operación → Validación:** Si el módulo de ejecución detecta una debilidad o cambio de comportamiento leve, la estrategia puede ser enviada de vuelta a validación robusta para re-evaluar sus métricas bajo el nuevo régimen de mercado.
2.  **Gestión → Validación:** Si el optimizador de portafolio detecta que una combinación de estrategias es subóptima, puede forzar una re-validación de las piezas individuales antes de rebalancear.
3.  **Generación Continua:** El módulo de Feedback detecta anomalías persistentes y dispara un nuevo ciclo en Generación, inyectando restricciones (constraints) que eviten repetir errores pasados.

---

