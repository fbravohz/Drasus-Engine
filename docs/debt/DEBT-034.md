# DEBT-034 · El descargador soberano nunca pasó el gate de mutación

| Campo | Valor |
|---|---|
| **ID** | DEBT-034 |
| **Severidad** | 🟠 Media |
| **Estado** | Abierta |
| **Abierta** | 2026-07-19 |
| **Origen** | Gate de mutación corrido por el Tech-Lead al cerrar el retrofit de DEBT-032 |
| **Dominio** | Ingesta — `sovereign-data-fetcher` |

## Qué se encontró

Al correr `scripts/mutation-gate.sh sovereign-data-fetcher` para verificar el retrofit del ledger:

| Resultado | Conteo |
|---|---|
| Cazados | 64 |
| **Supervivientes** | **49** |
| Timeouts (cuentan como captura) | 2 |
| Inviables | 19 |

**Veredicto: NO APTO.**

Distribución: `orchestrator.rs` 21 · `public_interface.rs` 18 · `domain.rs` 7 · `persistence.rs` 3.

## Lo que SÍ quedó cerrado

**Cero supervivientes en `is_transient_write_conflict`.** El clasificador —el objeto exacto de DEBT-032, que antes podía mutarse a "devolver siempre transitorio" sin que nada fallara— ahora está completamente cazado. **El retrofit funcionó y esta deuda no lo invalida.**

## Causa raíz: no es una regresión, es una crate pre-gate

`sovereign-data-fetcher` se cerró como **QA APTO el 2026-06-27** (STORY-024), con dos rondas de gate serias — la primera detectó que la concurrencia era decorativa. Pero **la mutación no era gate formal todavía**: se adoptó como capa 8 de ADR-0133 más tarde, y esta crate nunca se midió contra ese estándar.

Los 49 supervivientes son **el precio de haber sido construida antes de que existiera la vara**, no un deterioro. Ninguna prueba dejó de funcionar y ningún comportamiento cambió.

**Es el mismo patrón que la auditoría retroactiva de EPIC-0:** el substrato construido bajo el estándar nuevo salió sano; los hallazgos se concentraron en lo anterior al estándar. Aquí se repite a escala de una crate.

## Impacto actual

**Ninguno vivo.** El código funciona y sus pruebas pasan. Es un hueco de **detección**: 49 formas de romper esta crate que nada avisaría.

Concentración relevante: `orchestrator.rs` (21) es donde vive la concurrencia real de descarga — precisamente lo que el QA de 2026-06-27 tuvo que rechazar una vez por ser teatro. Que sea la zona con más supervivientes no es tranquilizador.

## Qué hay que hacer

1. Clasificar los 49 en **matables** frente a **equivalentes**. Un equivalente solo se acepta con demostración escrita de por qué ningún test puede matarlo; "es trivial" no vale (`.agents/knowledge/mutation-gate.md`).
2. Priorizar `orchestrator.rs`: es la lógica de concurrencia y ya falló una vez por pruebas que pasaban trivialmente.
3. Revisar el bloque de mutaciones aritméticas repetidas en `public_interface.rs` (líneas 272/284/373) — el mismo patrón multiplicado sugiere una conversión de unidades sin prueba de valor concreto, no 18 defectos distintos.
4. Escribir las pruebas que faltan y volver a correr el gate hasta 0 supervivientes.

## Disparador de pago

**Antes de que `sovereign-data-fetcher` reciba su siguiente cambio funcional** — y con prioridad si ese cambio toca `orchestrator.rs`. No antes: hoy no hay defecto vivo. No después: modificar código sin red de detección sobre una crate con 49 huecos conocidos es exactamente cómo se cuela una regresión silenciosa.

## Nota de método (Tech-Lead)

Corrí el script canalizando su salida a `tail`, lo que **enmascaró el código de salida**: la tubería devolvió 0 (éxito de `tail`) mientras el gate devolvía 1 (NO APTO). Estuve a un paso de reportar "gate superado" leyendo solo el código de salida.

Lo detecté al leer la salida real. **Regla:** el veredicto del gate se lee en su texto, nunca en el código de salida de una tubería. Si se canaliza, hay que usar el estado de la tubería, no el del último comando.
