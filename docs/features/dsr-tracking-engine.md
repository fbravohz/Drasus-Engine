# DSR Tracking Engine

**Carpeta:** `./features/dsr-tracking-engine/`
**Estado:** En Diseño
**Última actualización:** 2026-04-30
**Decisión Arquitectónica Asociada:** ADR-0067 (Capa de Inferencia Estadística (EBTA))

## ¿Qué es esta feature?

El DSR Tracking Engine es el encargado de registrar el volumen de intentos y la varianza de los resultados durante la fase de minería genética. Estos datos son críticos para que el Deflated Sharpe Ratio (DSR) pueda calcular correctamente la probabilidad de sobreajuste por selección.

**Problema que resuelve:** Sin un registro exacto de cuántas combinaciones probó el sistema antes de encontrar una estrategia "ganadora", el Sharpe Ratio es una métrica mentirosa. Esta feature proporciona el denominador $N$ necesario para la deflación institucional.

## Comportamientos Observables

- [ ] El motor inicializa un contador atómico al inicio de cada `SessionID` de generación.
- [ ] Los workers distribuidos incrementan el contador global en lotes (*batches*) para minimizar la contención de base de datos.
- [ ] El sistema calcula la varianza de los Sharpe Ratios de todos los candidatos probados en la sesión.
- [ ] Al finalizar la sesión, los valores finales ($N$ y $\sigma^2$) se guardan en el metadato de la sesión y se vinculan a cada estrategia producida.

## Restricciones

- NUNCA perder el conteo de intentos si un worker falla (resiliencia vía lotes confirmados).
- EL registro debe ser "Zero-Docker" y "Local-First", utilizando SQLite WAL para la coordinación atómica.
- EL overhead de registro no debe degradar el throughput de backtesting en más de un 2%.

## Estructura Interna (FCIS)

- **Core (Lógica Pura):** Lógica de agregación de estadísticas de sesión (media y varianza incremental).
- **Shell (Infraestructura):** Repositorio SQLite que gestiona los contadores atómicos por `SessionID`.
- **Frontera Pública:** Interfaz para que los workers reporten lotes de intentos y para que el módulo `validate` consulte el historial de una sesión.

## Ciclo de Vida de la Feature — DSR Tracking Engine

### Entrada
- Señales de "Backtest Completado" desde los workers.
- Sharpe Ratio de cada intento (incluso los fallidos).

### Proceso
- Incrementa el contador global de intentos ($N$).
- Actualiza la varianza acumulada de los Sharpe Ratios de la sesión.

### Salida
- Metadatos consolidados: `trials_count` y `sharpe_variance`.

## Tareas (TTRs)

### **TTR-001: Contador Atómico por Sesión en SQLite**
Implementar la tabla de seguimiento de sesiones y la lógica de incremento atómico optimizada para alta concurrencia.

### **TTR-002: Reportero de Lotes para Workers**
Desarrollar el mecanismo de acumulación local en el worker y envío por lotes al orquestador central.

### **TTR-003: Calculador de Varianza Incremental**
Implementar el algoritmo de Welford para calcular media y varianza en un solo paso sin almacenar todos los resultados en memoria.

## Gobernanza y Estándares

- **Local-First (ADR-0016):** 100% Local (SQLite WAL).
- **Inundación de Fundaciones (ADR-0020 V2):** 
    - **Perfil Datos / Ingest:** Foco en Identidad de Sesión + Linaje de Pruebas.
- **Contrato de Persistencia:** Tabla `mining_sessions` (session_id, trials_count, sharpe_variance, start_time, end_time).
- **Rastro de Evidencia:** Proporciona los parámetros $N$ y $\sigma^2$ para el módulo `validate`.
