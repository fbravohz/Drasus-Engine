## 16. Grafo de Dependencias Técnicas Entre Módulos

El sistema está estructurado en capas de dependencia que definen el orden lógico requerido:

### Capa Base: Ingesta de Datos
**MOD-01 (ingest)** — Fuente de datos inmutable de verdad.
* Sin dependencias de otros módulos.
* Todos los módulos posteriores dependen directa o indirectamente de su salida.

### Capas de Descubrimiento y Validación
**MOD-02 (generate)** depende de MOD-01.
* Lee datos históricos de ingest.
* Genera candidatas de estrategias.

**MOD-03 (validate)** depende de MOD-02.
* Valida candidatas generadas.
* Produce veredictos sobre robustez.

### Capa de Forward Testing
**MOD-04 (incubate)** depende de MOD-03.
* Requiere estrategias aprobadas en validación.
* Prueba en tiempo forward (perfil de incubación configurable: 7/21 días o 3-6 meses — ADR-0088).
* Filtra overfitting y cambios de régimen.

### Capa de Orquestación
**MOD-05 (manage)** depende de MOD-04.
* Requiere estrategias promovidas desde incubación.
* Ensambla portafolios optimizados.
* Define reglas de portafolio que guían ejecución.

### Capas Vivas (Ejecución y Monitoreo — Paralelo Continuo)
Estas capas operan en paralelo mientras el portafolio está activo. No tienen relación secuencial entre sí, pero todas dependen de MOD-05:

**MOD-06 (execute)** depende de MOD-05.
* Recibe portafolio optimizado y reglas.
* Ejecuta en tiempo real.
* Registra todas las decisiones automáticas en audit trail.

**MOD-07 (feedback)** — Analista. Observa TODOS los módulos en paralelo.
* Recolecta datos de ejecución (MOD-06), validación histórica (MOD-03) y P&L del portafolio.
* Detecta degradación y anomalías transversales; diagnostica la causa (Alpha muerto vs Beta/régimen).
* Emite el veredicto de continuidad/retiro y retroalimenta constraints a MOD-02 (cierre de bucle, ADR-0015).

**MOD-08 (withdraw)** — Actuador. Ejecuta el veredicto de retiro emitido por MOD-07 (no monitorea).
* Gestiona la transición del FSM: Ejecutando → En Pausa (ventana de veto) → Retirado/Archivo.
* Archiva métricas terminales y notifica a MOD-05 (manage) para rebalanceo.

---

