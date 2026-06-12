# Generador Genómico de Ejecución y Enrutamiento (Quinto Dominio Candidato — Excluido del Registro)

**Carpeta:** `./moonshots/`
**Estado:** Moonshot (Dominio Candidato — Evaluado y Excluido del Registro de Dominios Genómicos)
**Última actualización:** 2026-06-11
**Decisiones Arquitectónicas Asociadas:** ADR-0108 (exclusión formal), ADR-0100 (mismo principio de no-viabilidad de datos)

---

## ¿Qué es?

Exploración de un quinto dominio para el Registro de Dominios Genómicos (ADR-0108): un **Generador Genómico de Ejecución y Enrutamiento** que evolucionaría, vía NSGA-II, *cómo* se materializa una orden ya decidida por el Genoma de Señal — en lugar de *cuándo entrar*, *cuánto arriesgar* o *con qué portafolio convive* (dominios ya admitidos en ADR-0109/ADR-0110/ADR-0111).

Conceptualmente, este dominio evolucionaría:
- **Genes de Condición de Estado:** lecturas del estado de la microestructura del mercado en el momento de enviar la orden (profundidad del libro más allá de Nivel 1, posición en cola de ejecución, latencia real observada hacia cada broker/venue conectado).
- **Genes de Acción:** mutaciones sobre la forma de envío de la orden ya decidida — tipo de orden (mercado/límite/iceberg), fraccionamiento temporal de la ejecución (slicing), retraso de entrada dentro de la barra, y selección del broker/venue de destino cuando existe más de uno disponible para el mismo instrumento.

---

## ¿Por qué es moonshot? (Exclusión Formal, ADR-0108)

ADR-0108 evaluó este dominio contra los **Criterios de Admisión al Registro** y lo **reprobó en el criterio (a)**: un Gen de Condición de Estado debe ser observable y reproducible determinísticamente sobre datos históricos almacenados. La profundidad de libro más allá de Nivel 1 (L2/L3) y la latencia real de enrutamiento hacia múltiples venues/brokers **no están disponibles de forma consistente** para el operador retail/solopreneur objetivo de este sistema.

Este es el **mismo principio que ADR-0100** (relegación de Microestructura L3, ver [`microestructura-l3.md`](./microestructura-l3.md)): los feeds L2/L3 tienen costos prohibitivos ($5K-$20K/mes por instrumento) y volúmenes de almacenamiento incompatibles con hardware de consumo (10-50 GB/día por símbolo). Sin esos datos, los Genes de Condición de este dominio no podrían evaluarse de forma determinista en backtest, violando la **Reproducibilidad Bit-a-Bit (ADR-0107)** que el resto del Registro de Dominios Genómicos exige.

---

## Condición de Re-evaluación (Camino a la Admisión)

Este dominio queda archivado, no descartado. Podría volver a evaluarse contra los Criterios de Admisión de ADR-0108 si, en el futuro:

1. La expansión de SaaS institucional descrita en ADR-0100 hace viable el acceso a datos L2/L3 históricos consistentes (ver [`microestructura-l3.md`](./microestructura-l3.md)).
2. El [`multiplatform-execution-bridge`](../features/multiplatform-execution-bridge.md) y/o un futuro enrutador multi-broker registran telemetría de latencia real por venue de forma suficientemente consistente para construir un dataset histórico reproducible de latencias de enrutamiento.

Sin ambas condiciones, los Genes de Condición de este dominio seguirían siendo no-reproducibles y el dominio permanece fuera del Registro.

---

## Tareas (TTRs) — Exploratorias, Condicionadas a Re-evaluación

### **TTR-001: Genes de Condición de Microestructura de Enrutamiento (Bloqueado por Disponibilidad de Datos)**
*   **¿Cuál es el problema?** Si este dominio fuera admitido, su motor evolutivo necesitaría leer profundidad de libro L2/L3 y latencia real por venue como Genes de Condición de Estado, reproducibles determinísticamente entre backtest y operativa en vivo.
*   **¿Qué tiene que pasar?** Depende íntegramente de que [`microestructura-l3.md`](./microestructura-l3.md) deje de ser un moonshot y de que exista un dataset histórico de latencias de enrutamiento por venue con la misma garantía de reproducibilidad que el resto de Genes de Condición del Registro.
*   **¿Cómo sé que está hecho?**
    - [ ] Ambas fuentes de datos (L2/L3 histórico y latencia de enrutamiento histórica) cumplen el Criterio de Admisión (a) de ADR-0108 de forma demostrable.
*   **¿Qué no puede pasar?** Este dominio no puede activarse en `ACTIVE_GENOME_DOMAINS` mientras sus Genes de Condición no sean reproducibles bit-a-bit (ADR-0107).

### **TTR-002: Genes de Acción de Enrutamiento Evolutivo (Diseño Conceptual)**
*   **¿Cuál es el problema?** Aun con datos disponibles, las primitivas de acción de este dominio (mutación de tipo de orden, slicing temporal, retraso de entrada, selección de venue) deben mapearse a comportamientos ya configurables en [`broker-connector`](../features/broker-connector.md), [`order-priority-queue`](../features/order-priority-queue.md) y [`multiplatform-execution-bridge`](../features/multiplatform-execution-bridge.md) — el Registro de Dominios Genómicos prohíbe construir motores nuevos paralelos.
*   **¿Qué tiene que pasar?** Si TTR-001 se resuelve y el dominio es admitido, este TTR mapearía cada primitiva de acción candidata a un parámetro ya existente (o a extender) en las features mencionadas, siguiendo el mismo patrón de "Primitivas de Acción" usado por ADR-0109/ADR-0110/ADR-0111.
*   **¿Cómo sé que está hecho?**
    - [ ] Cada Gen de Acción propuesto tiene una feature existente que lo materializa, sin motores de enrutamiento paralelos.
*   **¿Qué no puede pasar?** Este TTR no se ejecuta mientras TTR-001 permanezca bloqueado.

---

## Gobernanza y Estándares (Fijos)

- **Registro de Dominios Genómicos (ADR-0108):** este documento es el archivo formal del quinto dominio candidato evaluado y excluido. Ver Registro de Dominios Genómicos en [`SAD.md`](../SAD.md) §2.3.
- **Relegación de Microestructura (ADR-0100):** la condición de re-evaluación de este dominio depende directamente del estado de [`microestructura-l3.md`](./microestructura-l3.md).
