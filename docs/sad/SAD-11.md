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
* **Implementación:** Conversión de decimal a exacto ocurre solo en acceso datos y capas externas. Lógica pura siempre usa exactos.
* **Beneficio:** Reproducibilidad absoluta; cálculos de ganancias/pérdidas sin errores.

### Sin Sorpresas de Tiempo en Lógica Pura (Transversal)
* **Regla:** Nunca obtener la hora actual dentro de la lógica pura. Recibir el tiempo como parámetro de entrada.
* **Por qué:** Reproducibilidad y testeabilidad. Una prueba puede decir "es 2024-01-01 09:30:00" y forzar ese tiempo.
* **Implementación:** El tiempo es un parámetro que se pasa (inyección de dependencia).
* **Beneficio:** Simulaciones históricas reproducibles; debugging sin sorpresas.

---

