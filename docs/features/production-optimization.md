# Optimización y Producción

**Carpeta:** `./features/optimizacion-produccion/`
**Estado:** Pendiente (condicionado a resultados de benchmarks)
**Última actualización:** 2026-04-06

---

## ¿Qué es?

Después de que todos los módulos están implementados y benchmarked **individualmente**, esta fase se ejecuta **solo si hay cuellos de botella identificados**. Si cada módulo cumple su SLA (latencia < 10ms, throughput adecuado) y el benchmark e2e es medible y demostrablemente más rápido que MT5/SQX/QuantConnect en igual hardware (ADR-0114; sin KPI absoluto de barras/seg), esta feature no tiene trabajo (está COMPLETA).

Si hay incumplimientos SLA, aquí se optimizan las partes lentas sin romper correctitud. Esto incluye: vectorización con Rust SIMD/Rayon, compilación AOT, mejoras de acceso a datos, paralelización donde es seguro.

También incluye preparación final para producción: documentación de arranque/parada, kill switches robustos, alertas opcionales.

---

## Comportamientos Observables

**Si TODOS los módulos cumplen SLA:**
- [ ] Benchmark e2e demuestra ventaja competitiva sobre MT5/SQX/QuantConnect ✓
- [ ] Cada módulo está dentro de su latencia esperada
- [ ] Feature se marca COMPLETA (no hay trabajo por hacer)

**Si HAY cuellos de botella:**
- [ ] Benchmark identifica qué módulo es lento (ej: "MOD-04 es 3x lento")
- [ ] Después de optimización, ese módulo pasa SLA
- [ ] El benchmark e2e se re-ejecuta y confirma mejora
- [ ] Sistema puede correr de manera continua sin intervención manual

---

## Restricciones

- La optimización SOLO ocurre si el benchmark lo identifica como necesario
- La optimización no debe romper la correctitud del sistema (rendimiento no vale más que exactitud)
- No se optimiza "por si acaso" (datos-driven, no especulación)
- El criterio de throughput es **relativo** (ventaja medible sobre MT5/SQX/QuantConnect, ADR-0114); cualquier cifra absoluta es solo una referencia interna configurable, no un KPI
- Las alertas de producción son opcionales pero recomendadas

---

## Parámetros Configurables

| Parámetro | Default | Qué hace |
|---|---|---|
| COMPETITIVE_BASELINE | MT5 / SQX / QuantConnect | Plataformas de referencia a superar en igual hardware (criterio relativo, ADR-0114) |
| TARGET_THROUGHPUT_REF | configurable (referencia interna) | Cifra opcional de barras/seg solo para seguimiento interno; NO es un KPI de salida |
| TARGET_ORDER_LATENCY | < 5ms | Latencia máxima del hot path de ejecución |

---

## Tareas (TTRs)

### TTR-001: Benchmark e2e integrado (validar SLA global)

* **¿Cuál es el problema?**
  Una vez que cada módulo tiene su propio benchmark y pasa SLA individual, necesitamos saber si funcionan JUNTOS conservando la ventaja competitiva sobre MT5/SQX/QuantConnect. A veces módulos rápidos se ralentizan al acoplarse.

* **¿Qué tiene que pasar?**
  Hay un benchmark que corre el pipeline COMPLETO (ingest → generar → validar → incubar → gestionar → ejecutar → retirar → retroalimentar) sobre un dataset de referencia. Reporta:
  - Throughput total comparado contra MT5/SQX/QuantConnect en igual hardware
  - Latencia por etapa (¿dónde se pierde tiempo?)
  - Identificación automática de cuello de botella (si existe)

* **¿Cómo sé que está hecho?**
  - [ ] Corro el benchmark e2e y obtengo reporte con números por módulo
  - [ ] Si todo cumple: reporte dice "✓ SISTEMA LISTO (ventaja competitiva sobre MT5/SQX/QuantConnect confirmada)"
  - [ ] Si hay cuello de botella: reporte dice "✗ MOD-04 es el bottleneck (22ms, target <10ms)"
  - [ ] Reporte incluye hardware y datos usados (reproducibilidad)

* **¿Qué no puede pasar?**
  - No puede el benchmark modificar datos de producción
  - No puede reportarse rendimiento sin condiciones (hardware, datasets)
  - No puede omitirse un módulo (reportar si no se pudo medir)

* **Bloqueantes:** Todos los módulos deben tener TTR de benchmark individual completo (cada uno pasó su SLA local)

---

### TTR-002: Optimización del módulo identificado como cuello de botella

* **¿Cuál es el problema?**
  Si el benchmark e2e muestra que un módulo incumple SLA (ej: MOD-04 tarda 22ms cuando debe ser <10ms), necesitamos optimizarlo sin romper correctitud.

* **¿Qué tiene que pasar?**
  **SOLO SI TTR-001 identifica un cuello de botella:** Se optimiza ese módulo específico. Pueden incluir: vectorización con Rust SIMD/Rayon, compilación AOT, mejora de queries SQL, paralelización segura, o cambios en estructura de datos. Después, se verifica que los resultados siguen siendo idénticos.

* **¿Cómo sé que está hecho?**
  - [ ] El módulo optimizado ahora cumple su SLA (benchmarks antes/después documentados)
  - [ ] Los resultados del sistema siguen siendo exactamente iguales (bit-perfect)
  - [ ] Se documenta qué cambió, por qué, y cuál fue la mejora (ej: "Cambié lista a Polars → 5x más rápido")

* **¿Qué no puede pasar?**
  - No puede sacrificarse exactitud por velocidad (reproducibilidad primero)
  - No puede optimizarse algo que no fue identificado por el benchmark como cuello de botella
  - No puede dejarse código de optimización experimental sin limpiar

* **Bloqueantes:** TTR-001 (y solo si TTR-001 identifica un cuello de botella)

---

### TTR-003: Preparación para operación continua y documentación de deployment

* **¿Cuál es el problema?**
  El sistema necesita correr días o semanas sin intervención manual. Necesitamos documentación clara de cómo iniciarlo, detenerlo, monitorear problemas, y recuperarse de fallos.

* **¿Qué tiene que pasar?**
  Existe documentación clara de:
  - Cómo iniciar el sistema (comando único, pre-requisitos claros)
  - Cómo detenerlo correctamente (graceful shutdown)
  - Kill switch robusto que funciona bajo estrés
  - Alertas opcionales (email, Slack, etc.) si hay condiciones críticas (DD excedido, módulo caído, datos malformados)
  - Logs de producción en formato estructurado (JSON, fácil de procesar)

* **¿Cómo sé que está hecho?**
  - [ ] Hay un archivo README.md en raíz con instrucciones de startup/shutdown
  - [ ] Puedo ejecutar el sistema en loop 24h sin intervención
  - [ ] El kill switch responde en < 100ms incluso si hay carga
  - [ ] Si hay condición crítica, obtengo notificación clara
  - [ ] No hay claves API, contraseñas o datos sensibles en documentación pública

* **¿Qué no puede pasar?**
  - No puede el sistema correr sin kill switch funcional
  - No puede haber credenciales reales en repositorio o documentación
  - No pueden quedar logs en formato print() (deben ser estructurados)
  - No puede haber estado ambiguo (ej: "¿el sistema está corriendo o no?")

* **Bloqueantes:** TTR-001 (y opcionalmente TTR-002 si hubo optimizaciones)

---

## Gobernanza y Estándares (Fijos)

- **Inundación de Fundaciones (ADR-0020 V2):** 
    - Toda ejecución de benchmark y reporte de optimización registra el set completo de **25 campos mandatorios** (ver ADR-0020 V2 V2).
    - Metadatos de infraestructura: `node_id`, `session_id`, `execution_latency_ms` (Benchmark results), `audit_chain_hash`.
    - Integridad de Lógica: `logic_hash` (Code footprint), `process_id`.
    - Soberanía: `owner_id`, `manifest_id`.


---

## Dependencias

**Depende de:** 
- setup-infraestructura (TTR-009 y TTR-010: fixtures y framework de benchmarking)
- Cada módulo debe tener su propio benchmark y pasar SLA local ANTES de integración

**Bloquea:** Nada — es la fase final

**Nota de Ejecución:**
- TTR-001 (benchmark e2e) se ejecuta tan pronto como todos los módulos estén listos
- Si TTR-001 reporta "✓ SISTEMA LISTO", entonces TTR-002 NO se ejecuta (no hay cuello de botella)
- Si TTR-001 reporta cuello de botella, ENTONCES se ejecuta TTR-002 para ese módulo específico
- TTR-003 se ejecuta después de TTR-001 y opcionalmente TTR-002 (preparación final)

## Bloqueantes Pendientes

- ❓ **Alertas:** ¿Canal de notificación preferido para condiciones críticas? (email, Slack, Telegram, otro)
- ❓ **Umbrales de alerta:** ¿Qué exactamente dispara alerta? (DD > -30%, módulo offline > 5min, data gap > 1 min)
- ❓ **SLA configurables:** el criterio de throughput es relativo (superar a MT5/SQX/QuantConnect, ADR-0114); la latencia < 5ms del hot path sí es un objetivo absoluto. ¿Dependen del hardware/requerimientos del usuario?
