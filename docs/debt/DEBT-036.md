# DEBT-036 · El puerto de salida de barras no puede cargar la referencia a la declaración sellada

| Campo | Valor |
|---|---|
| **ID** | DEBT-036 |
| **Severidad** | 🟠 Media |
| **Estado** | Abierta |
| **Abierta** | 2026-07-25 (hallazgo del QA-Engineer en el gate final de STORY-068) |
| **Feature** | [`data-import-wizard`](../features/data-import-wizard.md) |
| **Origen** | Veredicto APTO del QA con hueco de diseño señalado, [STORY-068](../execution/STORY-068-mt5-bar-reader.md) |

## Qué queda debiendo

El puerto por el que salen las barras ya canónicas transporta las filas, pero **no el identificador de la declaración de mapeo que las originó**. ADR-0156 restricción 8 exige que un resultado cualquiera se pueda rastrear hacia atrás hasta la declaración que lo produjo: resultado → conjunto de datos → corrida de transformación → fila del ledger de mapeo.

Hoy esa cadena está documentada honestamente como "por convención de aplicación, aún no física" — porque el consumidor real (`hybrid-data-transformer`) quedó deliberadamente fuera del alcance de STORY-068.

## Impacto actual

**Ninguno vivo.** Mientras el destino de las barras sea un consumidor en memoria, no hay nada que rastrear ni nada que se pierda. El ledger existe, es inmutable y se sella antes de que haya dato — la evidencia está completa; lo que falta es el hilo que la ata al resultado.

## Por qué importa cuándo llegue

El día que ese puerto se conecte con la persistencia real, su forma actual **no admite** pasar la referencia. Habrá que romper el contrato del puerto en ese momento — más caro que anticiparlo, y con el riesgo de que se resuelva duplicando la declaración dentro de cada fila de mercado, que es exactamente lo que ADR-0156 restricción 8 prohíbe (la trazabilidad va por referencia estable, nunca por copia).

Sin ese hilo, la evidencia inmutable prueba qué declaró el usuario, pero no prueba que **este** resultado salió de **esa** declaración — que es la mitad que sostiene la defensa de LDR-0008.

## Disparador de pago

**La Story que conecte la salida de barras con la persistencia real** (`hybrid-data-transformer` escribiendo Parquet). En esa Story, la forma del puerto se extiende para cargar la referencia a la declaración, y la corrida de transformación la registra.

No antes: hoy no hay destino que rastrear. No después: en cuanto haya persistencia real sin la referencia, cada partición escrita nace huérfana de su declaración.

## Relacionadas

- [[DEBT-035]] — la otra deuda abierta por el cierre de STORY-068 (pantalla de la vía de mapeo manual).
- ADR-0156 restricción 8 — trazabilidad por referencia estable, nunca duplicando la declaración.
- [`LDR-0008`](../legal/LDR-0008.md) condición técnica 4 — la trazabilidad hasta el resultado es requisito, no preferencia.
