# Volatility Stabilization (Target Vol Certification)

**Carpeta:** `./features/volatility-stabilization/`
**Estado:** Lista para implementar
**Última actualización:** 2026-04-30
**Decisión Arquitectónica Asociada:** ADR-0068 (Certificación de Estabilización de Volatilidad)

## ¿Qué es esta feature?

El motor de **Volatility Stabilization** garantiza que las estrategias operen bajo un perfil de riesgo constante (Target Vol) y sean certificadas como estables antes de su aprobación. Su función es escalar dinámicamente el tamaño de la posición o bloquear la operativa si el mercado entra en regímenes de volatilidad extrema no soportados por el modelo.

**Problema que resuelve:** Las estrategias suelen romperse o generar drawdowns catastróficos ante picos de volatilidad. El Target Vol normaliza la "presión" estadística sobre el capital, asegurando que 1 trade en baja volatilidad tenga el mismo impacto esperado que 1 trade en alta volatilidad (ajustado por lotaje).

## Comportamientos Observables

- [ ] **Certificación Pre-Aprobación:** Durante la validación, el sistema evalúa si la volatilidad realizada se desvía > 30% del target. Si no es estable, la estrategia es rechazada.
- [ ] **Escalado Dinámico:** En tiempo real, el sistema recalcula la volatilidad (ATR/StdDev) y ajusta el lotaje para mantener el Target Vol configurado.
- [ ] **Bloqueo por Régimen:** Si la volatilidad del mercado supera el límite máximo certificado, el sistema desautoriza nuevas entradas (Veto Preventivo).

## Restricciones

- **NUNCA** se aprueba una estrategia que no haya pasado la certificación de estabilidad en 3 regímenes: Calmo, Normal y Volátil.
- **NUNCA** se permite el escalado de posición que viole el riesgo máximo por trade absoluto.
- **FIJO:** El cálculo de volatilidad debe usar una ventana de lookback coherente con el time-frame de la estrategia.

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| TARGET_VOL_ANNUALIZED | 0.10 | 0.01 - 0.50 | Volatilidad objetivo (ej. 10% anual) | CONFIG |
| VOL_STABILITY_THRESHOLD | 0.30 | 0.10 - 0.50 | Máxima desviación permitida para certificar | CONFIG |
| VOL_MAX_LIMIT | 3.5 | 2.0 - 10.0 | Multiplicador de vol base para bloqueo total | CONFIG |
| LOOKBACK_PERIOD | 20 | 5 - 200 | Período para el cálculo de volatilidad realizada | CONFIG |

## Estructura Interna (FCIS)

- **Core (Lógica Pura):** Algoritmos de cálculo de volatilidad realizada (anualizada) y fórmulas de escalado de posición inversamente proporcionales.
- **Shell (Infraestructura):** Integración con el motor de ejecución para ajustar órdenes antes de su envío y persistencia del "Sello de Certificación" en la DB.
- **Frontera Pública:** Interfaz para solicitar el lotaje ajustado por vol y verificar el estado de certificación.

## Ciclo de Vida de la Feature — Volatility Stabilization

### Entrada
- Precio de mercado (bars/ticks) para cálculo de vol.
- Target Vol configurado.
- Tamaño de cuenta y riesgo base.

### Proceso
- Calcula la volatilidad realizada actual.
- Compara la vol actual con el target.
- Determina el factor de escalado (Target / Actual).
- Valida si el régimen es apto para operar.

### Salida
- Factor de lotaje ajustado (multiplicador).
- Veredicto de régimen (OPERATIVO / BLOQUEADO).
- Rastro de vol para auditoría.

## Tareas (TTRs)

### **TTR-001: Implementación del Motor de Cálculo de Volatilidad Anualizada**
*   **¿Cuál es el problema?** Necesitamos una forma determinista de medir la volatilidad que sea comparable entre diferentes activos y temporalidades.
*   **¿Qué tiene que pasar?** El sistema calcula la volatilidad diaria logarítmica y la anualiza ($\sigma \times \sqrt{252}$). El resultado debe ser bit-a-bit idéntico en backtest y real.
*   **¿Cómo sé que está hecho?**
    - [ ] El cálculo en Polars (batch) coincide con el cálculo en Nautilus (online).
    - [ ] Se puede visualizar la vol anualizada en el dashboard.

### **TTR-002: Lógica de Certificación en Guantelete de Validación**
*   **¿Cuál es el problema?** Estrategias que ganan mucho por "suerte" en alta vol suelen ser frágiles.
*   **¿Qué tiene que pasar?** El módulo `validate` inyecta un test de estabilidad que rechaza la estrategia si la varianza del rendimiento por unidad de volatilidad es excesiva.
*   **¿Cómo sé que está hecho?**
    - [ ] Estrategias inestables reciben veredicto "RECHAZADA (Vol Instability)".

## Gobernanza y Estándares

- **Local-First (ADR-0016):** 100% Local.
- **Fidelidad (ADR-0017):** Institucional (Slippage Dinámico afectado por vol).
- **Inundación de Fundaciones (ADR-0020 V2):** Perfil Ops / Hot-Path.
- **Rastro de Evidencia:** Emite `realized_vol` y `scaling_factor` al módulo de `feedback`.
