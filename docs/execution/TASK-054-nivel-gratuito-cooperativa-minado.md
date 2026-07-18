# TASK-054 · El nivel gratuito como cooperativa de minado

| Campo | Valor |
|---|---|
| **ID** | TASK-054 |
| **Tipo** | Task (diseño, sin código todavía) |
| **Responsable** | **Architect**, que invoca al **General Counsel** para colaborar (así lo pidió el propietario) |
| **Estado** | 🟡 Abierta — pendiente de despacho |
| **Creada** | 2026-07-18 |
| **Origen** | Riesgo detectado por el General Counsel + decisión del propietario |

## El problema que resuelve

El General Counsel identificó que **la cláusula de mayor riesgo de todo el corpus no está en los datos**: está en ADR-0143, donde el trabajo del usuario del nivel gratuito fluye hacia el proveedor y es suyo por contrato — abriendo la puerta a que la estrategia de un usuario acabe operada con capital del propietario. En mercados con protección fuerte al consumidor, una cláusula así se anula por **sorpresiva** cuando vive enterrada en el clausulado.

## La solución aprobada por el propietario

**Convertir el intercambio en el producto, no en letra pequeña.** El nivel gratuito se anuncia como lo que es: una cooperativa de minado de estrategias donde el usuario **aporta cómputo y exploración** y **recibe algo a cambio**.

Dos condiciones para que funcione, no negociables:
1. El intercambio es **visible en el momento de registrarse**, no en la página catorce.
2. Es un intercambio **real con valor palpable**, no una formalidad.

**Qué recibe el usuario queda deliberadamente abierto** por decisión del propietario: acceso preferente, participación en lo encontrado, créditos de cómputo, o lo que se decida. El diseño debe soportar cualquiera de esas variantes sin rehacerse.

**Lo que NO cambia:** el propietario confirmó que necesita **la estrategia completa y los resultados** — el mapa de dónde vive el edge. Los agregados anónimos no le sirven. Así que la protección es la franqueza del intercambio, **no** tomar menos dato. Las dos vías son alternativas, no capas que se sumen.

## Punto de partida obligatorio

**No inventar un marco nuevo.** El proyecto ya tiene diseñada la minería descentralizada de estrategias (ADR-0086, "La Colmena", y su moonshot). El encargo es **evaluar si ese diseño sirve como marco del nivel gratuito** y qué le falta, antes de escribir nada desde cero.

## Qué debe entregar

- **Architect:** el marco de arquitectura — qué se aporta, qué se devuelve, cómo se mide la contribución, y qué hay que construir en producto (consentimiento explícito y separado, visibilidad de lo aportado por el usuario, y el bloqueo por jurisdicción que el propietario aprobó). Enmienda de ADR-0143 si procede.
- **General Counsel, invocado por el Architect:** si el marco resiste el escrutinio de consumo, cómo debe anunciarse para no ser sorpresivo, y qué queda fuera del alcance de un agente de IA.

## Relación con otras tareas

Depende de la jurisdicción elegida (TASK-053): qué es oponible cambia según dónde se constituya la empresa.
