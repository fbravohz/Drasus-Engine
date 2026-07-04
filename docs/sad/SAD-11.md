## 11. Restricciones de Negocio (Invariantes del Sistema)

### Validación de Datos (ingest)
* **Regla:** Ningún dato sin validación puede entrar en un módulo.
* **Por qué:** Datos malos contaminan toda simulación histórica → estrategias falsas → pérdida de dinero.
* **Implementación:** Antes de cualquier lectura externa (gRPC/WebSocket, archivo), validar; rechazar si no pasa.
* **Consecuencia:** Anomalía registrada en observabilidad; no procesar ese dato.

### Regímenes de Mercado Incompletos (ingest)
* **Regla:** "Régimen desconocido" es válido pero explícito; módulos posteriores saben que no hay clasificación.
* **Por qué:** Evitar que generar/validar asuman régimen cuando no hay suficiente historial de volatilidad.
* **Implementación:** Precio con régimen desconocido se guarda; generar puede usarla pero debe registrar advertencia.

### Inmutabilidad de Veredictos de Validación (validar)
* **Regla:** Una vez que se genera un análisis, es inmutable. Nuevas pruebas se agregan, pero el veredicto original no cambia.
* **Por qué:** Auditoría regulatoria; reproducibilidad histórica. Si el veredicto cambiara, se pierden registros.
* **Implementación:** Marcar análisis como bloqueado después de primera generación; rechazar recomputaciones.
* **Consecuencia:** Historial completo rastreable + reproducibilidad total.

### Herencia de Resultados (validar - Optimización del Historial)
* **Regla:** Si la prueba es idéntica a una versión anterior, heredar resultado sin re-ejecutar.
* **Por qué:** Pruebas A/B sin costo extra; evitar recalcular lo ya validado (ahorro >80% en iteraciones rápidas).
* **Implementación:** El **[`incremental-test-engine`](../features/incremental-test-engine.md)** gestiona el hashing de parámetros y la búsqueda de evidencia previa.
* **Beneficio:** Pruebas transversales (WFA, MC, Stress) más rápidas y consistentes.


### Baseline Congelado en Comparativas (incubar)
* **Regla:** La comparativa entre ejecución simulada y viva usa el baseline original, no un recálculo nuevo.
* **Por qué:** Si el baseline cambia, la comparativa pierde validez estadística → alertas falsas de degradación.
* **Implementación:** Guardar baseline cuando se aprueba la estrategia; usarlo siempre igual.

### Portafolio tiene prioridad sobre Estrategia Individual (gestionar / ejecutar)
* **Regla:** Si hay conflicto entre regla de portafolio y regla de estrategia individual, portafolio gana.
* **Por qué:** El portafolio gestiona riesgo global; una estrategia no puede violar límites del conjunto.
* **Implementación:** Al ejecutar, validar contra reglas de portafolio ANTES que reglas de estrategia.

### Decisiones Automáticas Críticas Revertibles (ejecutar)
* **Regla:** Toda decisión automática crítica (cierre de posición, reducción de peso) puede deshacerse en un plazo configurable.
* **Por qué:** Control del usuario: el sistema actúa pero el dueño mantiene poder de decisión final.
* **Implementación:** Marcar decisión como reversible, registrar cuándo ocurrió, permitir ventana de tiempo (ej: 5 minutos). Usuario puede deshacer.

### Retiro con Período de Espera (retirar)
* **Regla:** Entre ejecutando y retirado siempre hay pausa con período configurable (ej: 1 día) donde se puede revertir.
* **Por qué:** Evitar retiros accidentales por anomalías temporales; poder cambiar de opinión.
* **Implementación:** Máquina de estados: Ejecutando → En Pausa → Retirado. En pausa, usuario puede reactivar.

### Precios en Lógica Pura son Números Exactos (Transversal)
* **Regla:** En la lógica pura, precios siempre son números exactos (centavos/ticks), no decimales.
* **Por qué:** Evitar errores acumulados de decimales en operaciones financieras.
* **Implementación:** Conversión de decimal a exacto ocurre solo en acceso datos y capas externas. Lógica pura siempre usa exactos. En la capa de persistencia, la escala canónica es × 10⁸ (8 decimales) en columnas `INTEGER NOT NULL` (ADR-0141). `REAL` prohibido para precios y volumen en SQLite y Parquet.
* **Beneficio:** Reproducibilidad absoluta; cálculos de ganancias/pérdidas sin errores.

### Sin Sorpresas de Tiempo en Lógica Pura (Transversal)
* **Regla:** Nunca obtener la hora actual dentro de la lógica pura. Recibir el tiempo como parámetro de entrada.
* **Por qué:** Reproducibilidad y testeabilidad. Una prueba puede decir "es 2024-01-01 09:30:00" y forzar ese tiempo.
* **Implementación:** El tiempo es un parámetro que se pasa (inyección de dependencia).
* **Beneficio:** Simulaciones históricas reproducibles; debugging sin sorpresas.

### Timestamps son INTEGER Nanosegundos UTC (Transversal — ADR-0141)
* **Regla:** Todo timestamp en SQLite es `INTEGER NOT NULL` en nanosegundos Unix UTC desde epoch. NUNCA `TEXT` (ISO-8601) ni `REAL` (segundos fraccionarios).
* **Por qué:** Precisión de nanosegundos para datos de mercado de alta frecuencia; operaciones aritméticas directas sin conversión; univocidad de representación.
* **Implementación:** UTC estricto en toda la pila (Rust → Parquet → DuckDB). La conversión a zona horaria local ocurre solo en Flutter. Los campos `created_at` (tiempo de persistencia) y `event_timestamp_ns` / `bar_timestamp_ns` (tiempo del evento de mercado) tienen nombres distintos y no son intercambiables.
* **Consecuencia:** La confusión de `created_at` con el timestamp del evento introduce look-ahead bias en backtests.

### Enums son TEXT con CHECK Constraint (Transversal — ADR-0141)
* **Regla:** Todo campo de estado o tipo con valores discretos es `TEXT NOT NULL` con un `CHECK (col IN (...))` que declara explícitamente los valores válidos. Doble validación: CHECK en SQL + validación en Rust.
* **Por qué:** Sin el CHECK, SQLite acepta cualquier string en una columna `TEXT`; el schema no documenta los valores posibles y los bugs de estado inválido son silenciosos.
* **Implementación:** Para nuevas migraciones, obligatorio. Para tablas existentes (`jobs.state`), corregir en la primera migración que toque esa tabla.

### FKs con ON DELETE RESTRICT; CASCADE Prohibido (Transversal — ADR-0141)
* **Regla:** Toda FK se declara con `ON DELETE RESTRICT` explícito. `ON DELETE CASCADE` está **PROHIBIDO** en todo el sistema.
* **Por qué:** La inmutabilidad de los event-stores es un invariante físico. Un CASCADE borraría registros de auditoría en cascada, violándola irreversiblemente.
* **Implementación:** `PRAGMA foreign_keys = ON` activo en pool.rs. Toda FK en toda migración, presente y futura, incluye `ON DELETE RESTRICT`.

### Cambio de Estado y Registro en Audit-Log son Atómicos (Transversal — ADR-0141)
* **Regla:** Todo cambio de estado de una entidad de dominio y su registro en `audit_events` se realizan en una única transacción `BEGIN IMMEDIATE`. Si cualquiera falla, todo hace rollback.
* **Por qué:** El invariante "todo cambio de estado genera un registro en el audit-log" declarado en SAD-08 no tiene garantía técnica si los dos pasos ocurren en operaciones separadas. Un fallo entre ambos dejaría el sistema en estado inconsistente sin traza.
* **Implementación:** En la Shell de la feature. No como trigger (la lógica de `audit_hash` requiere computación Rust).
* **Consecuencia:** PROHIBIDO insertar en `audit_events` fuera de la transacción del cambio de estado.

### Soberanía Condicionada por Tier y Telemetría Obligatoria (Transversal — ADR-0143)
* **Regla:** El cómputo corre siempre en hardware del usuario; el proveedor nunca ejecuta el motor. Pero toda instancia mantiene un canal de control obligatorio hacia la Cabina de Mando Central (Zero-Telemetry derogado). Qué datos de trabajo se envían depende del tier: gratis = todo el trabajo (dueño: el proveedor, por ToS); pago al corriente = supresión en origen; pago vencido = emisión reactivada (sin borrar el entorno).
* **Por qué:** Habilitar cualquier modelo de monetización (licencias, datos agregados, cuotas) sin perder la ventaja de margen del cómputo local ni la privacidad vendible al cliente de pago.
* **Implementación:** El emisor de telemetría de artefactos se gobierna por el estado de licencia, evaluado localmente con licencia cacheada y período de gracia offline. Los secretos (credenciales de bróker, IPs live) nunca se exfiltran, en ningún tier (ADR-0093).
* **Consecuencia:** Lo que no se envía (tier de pago) no se puede filtrar ni requerir judicialmente. El consentimiento/ToS versionado es prerrequisito legal del firehose gratuito.

### Permisos del Agente Graduados por Riesgo de Pipeline (Transversal — ADR-0123)
* **Regla:** Un agente LLM conectado vía MCP nace con permiso total sobre ingestar/generar/validar/incubar/retroalimentar; sobre gestionar queda condicionado al `institutional_tag` (Demo concede, Live exige el interruptor de producción); sobre ejecutar/retirar nace sin permiso alguno.
* **Por qué:** Permite delegar el trabajo tedioso de descubrimiento sin abrir por defecto una puerta sin control hacia el capital real.
* **Implementación:** El interruptor de producción está apagado y oculto por defecto; el propietario lo activa de forma explícita y reversible. En modo SaaS, antes no existe la opción de activarlo sin que el usuario acepte expresamente los términos de riesgo.
* **Consecuencia:** Toda llamada del agente queda auditada con su procedencia; cerrar el canal MCP no degrada la operación manual desde la interfaz.

---

