# Operational Safety Monitor (Pardo Profile & SSL)

**Carpeta:** `./features/operational-safety-monitor/`
**Estado:** Lista para implementar
**Última actualización:** 2026-04-30
**Decisión Arquitectónica Asociada:** ADR-0070 (Monitoreo de Seguridad Operativa)

## ¿Qué es esta feature?

El **Operational Safety Monitor** es el guardián de la integridad del capital en tiempo real. Combina el monitoreo de deriva estadística (**Pardo Profile**) con un interruptor de emergencia mandatorio (**Strategy Stop-Loss**). Actúa como la última línea de defensa ante el colapso del modelo o cambios estructurales del mercado.

**Problema que resuelve:** Las estrategias "mueren" por obsolescencia. Seguir operando una estrategia que ha perdido su ventaja estadística (drift) o que está en un drawdown fuera de lo normal es una negligencia operativa. Esta feature automatiza la desconexión antes de que el daño sea irreversible.

## Comportamientos Observables

- [ ] **Pardo Profile Monitor:** El sistema compara continuamente las métricas en vivo (Win%, Avg Trade) con el perfil histórico. Si la desviación supera el 50%, se emite una alerta y se suspende la estrategia.
- [ ] **Strategy Stop-Loss (SSL):** Si el drawdown en vivo supera el `HistMaxDD * Safety Factor`, el sistema cierra todas las posiciones de esa estrategia inmediatamente.
- [ ] **Dashboard de Salud:** El usuario ve un indicador tipo "Semáforo" (Verde/Amarillo/Rojo) basado en el drift estadístico actual.
- [ ] **Auto-Kill por Desviación de Confianza Monte Carlo:** El sistema superpone la curva de capital en vivo sobre el cono de confianza generado en las pruebas de estrés (Spaghetti Chart). Si el Drawdown en vivo cruza la línea del Nivel de Confianza (ej. 95%) calculado en el backtest, significa que el mercado actual es más hostil que el peor universo simulado: se activa el Kill Switch (cierra posiciones, pausa el bot) y se envía alerta urgente.

## Restricciones

- **SSL es un Hard Limit (ADR-0010):** Se ejecuta automáticamente y sin latencia humana.
- **NUNCA** se permite la operativa de una estrategia que no tenga un perfil histórico base cargado.
- **FIJO:** El cálculo de DD para el SSL debe ser tick-a-tick (fidelidad máxima).

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| PARDO_DRIFT_THRESHOLD | 0.50 | 0.10 - 0.80 | Máxima desviación permitida en métricas | CONFIG |
| SSL_SAFETY_FACTOR | 1.5 | 1.1 - 2.5 | Multiplicador del MaxDD histórico | CONFIG |
| MIN_TRADES_TO_MONITOR | 10 | 5 - 50 | Trades mínimos antes de activar Pardo Monitor | CONFIG |
| AUTO_LIQUIDATE_ON_SSL | true | true / false | Cierra posiciones al tocar el SSL | [FIJO] |

## Estructura Interna (FCIS)

- **Core (Lógica Pura):** Algoritmos de comparación de perfiles (Z-Score de métricas) y lógica de detección de ruptura de límites de drawdown.
- **Shell (Infraestructura):** Integración con el `Shadow Watchdog` (ADR-0026) para ejecución de kill-switches y persistencia de veredictos de salud.
- **Frontera Pública:** Interfaz para consultar el estado de salud y resetear alertas (bajo responsabilidad del usuario).

## Ciclo de Vida de la Feature — Operational Safety Monitor

### Entrada
- Trades ejecutados en vivo.
- Perfil histórico de la estrategia (métricas de backtest/validación).
- Configuración de límites (SSL, Drift).

### Proceso
- Recalcula métricas en vivo con cada nuevo trade.
- Compara `Live Metrics` vs `Historical Profile`.
- Verifica si `Current DD` > `Threshold`.
- Determina el estado de salud (HEALTHY / WARNING / CRITICAL).

### Salida
- Veredicto de salud.
- Acciones de ejecución (Kill Signal si CRITICAL).
- Telemetría de drift para el dashboard.

## Tareas (TTRs)

### **TTR-001: Implementación del Pardo Profile Monitor**
*   **¿Cuál es el problema?** El trader no sabe cuándo una estrategia ha dejado de funcionar hasta que es demasiado tarde.
*   **¿Qué tiene que pasar?** El sistema crea un "Thumbprint" (huella digital) de la estrategia tras la validación. En vivo, calcula el drift y suspende si la probabilidad de que la estrategia siga siendo la misma cae por debajo de un umbral.
*   **¿Cómo sé que está hecho?**
    - [ ] El sistema suspende una estrategia si su Win% cae de 60% (histórico) a 20% (vivo) tras 10 trades.

### **TTR-002: Lógica de Strategy Stop-Loss (SSL)**
*   **¿Cuál es el problema?** El drawdown puede exceder lo previsto y quemar la cuenta.
*   **¿Qué tiene que pasar?** El orquestador de ejecución inyecta un check en cada tick que compara el equity actual con el High-Water-Mark histórico escalado.
*   **¿Cómo sé que está hecho?**
    - [ ] Prueba de estrés: forzar un drawdown artificial y verificar que el sistema cierra la posición automáticamente.

### **TTR-003: Real-Time Auditor (Análisis de Drift con WRC/KS)**
*   **¿Cuál es el problema?** Las métricas de rentabilidad en vivo de una estrategia pueden degradarse lentamente de forma indetectable para filtros simples de drawdown rígidos.
*   **¿Qué tiene que pasar?** Implementar un auditor de rendimiento en tiempo real asíncrono. Compara la distribución de retornos en caliente vs los retornos esperados del backtest usando la Prueba de Realidad de White (WRC) y el Test de Kolmogorov-Smirnov (KS Test). Si las métricas divergen sistemáticamente (drift detectado), dispara una alerta visual.
*   **¿Cómo sé que está hecho?**
    - [ ] El motor asíncrono recalcula periódicamente los p-valores del KS Test y WRC en base a las últimas posiciones cerradas.
    - [ ] Si las métricas de drift exceden los límites configurables, la UI de Flutter muestra una alerta de advertencia y se notifica al usuario.
*   **¿Qué no puede pasar?** PROHIBIDO ejecutar los cálculos pesados del KS Test y WRC síncronamente en el hilo principal de NautilusTrader para no interferir con la latencia crítica de órdenes.

### **TTR-004: Auto-Kill por Cruce de Banda de Confianza Monte Carlo**
*   **¿Cuál es el problema?** En la herramienta competidora, el nivel de confianza estadístico (ej. 95% Drawdown < 20%) se queda en un PDF; no protege el capital en vivo. Hay que conectar el laboratorio con el mundo real.
*   **¿Qué tiene que pasar?** El monitor superpone la curva de capital en vivo sobre el cono/percentiles Monte Carlo (provistos por el preparador de visualización dual de `validate`). Si el Drawdown en vivo cruza el percentil del Nivel de Confianza configurado, dispara el Kill Switch (cierra posiciones, pausa la estrategia) y emite alerta urgente con opción de reentrenar o apagar.
*   **¿Cómo sé que está hecho?**
    - [ ] La UI muestra la curva en vivo sobre las bandas de confianza MC.
    - [ ] Al cruzar la banda del nivel configurado, el bot se pausa automáticamente y recibo la alerta.
    - [ ] El nivel de confianza y el percentil de corte son configurables.
*   **¿Qué no puede pasar?**
    - El corte NUNCA puede ejecutarse sin registrar el evento de desviación estructural en el log de auditoría.
    - El cálculo de cruce NUNCA debe bloquear el hilo crítico de órdenes.
*   **Dependencia:** consume el `visual_mc_payload` (cono/percentiles) preparado por `monte-carlo-simulator` en el módulo `validate`.

- **Local-First (ADR-0016):** 100% Local.
- **Fidelidad (ADR-0017):** Alta (Tick-by-Tick logic).
- **Inundación de Fundaciones (ADR-0020 V2):** Perfil Ops / Auditoría.
- **Rastro de Evidencia:** Emite `safety_verdict` y `drift_metrics` para `feedback`.
