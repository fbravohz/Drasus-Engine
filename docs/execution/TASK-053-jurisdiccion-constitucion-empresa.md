# TASK-053 · Dónde constituir Drasus — análisis de jurisdicción

| Campo | Valor |
|---|---|
| **ID** | TASK-053 |
| **Tipo** | Task (investigación, sin código) |
| **Responsable** | General Counsel |
| **Estado** | 🟡 Abierta — pendiente de despacho |
| **Creada** | 2026-07-18 |
| **Origen** | Instrucción directa del propietario |

## Encargo

Determinar **dónde debería establecerse Drasus Engine** para obtener las mayores ventajas legales con las menores desventajas. El propietario aún no ha decidido entre México y Estados Unidos, y pide expresamente que **no sea un análisis básico**: la elección condiciona todo el corpus legal, así que se evalúa a fondo o no sirve.

## Qué debe cubrir

- **Candidatas:** México y Estados Unidos como punto de partida; el abogado añade y justifica cualquier otra que compita de verdad (no un catálogo de paraísos por inercia).
- **Régimen sobre el producto:** ¿un motor de generación y simulación de estrategias cae bajo alguna licencia de asesoría financiera, gestión de activos o proveedor de tecnología regulada? Es la pregunta que más puede doler.
- **Protección al consumidor:** hasta dónde son oponibles las cláusulas de exención y las de jurisdicción. Es donde el corpus legal se juega su eficacia real.
- **Fiscalidad** de un producto de software vendido internacionalmente: retenciones, obligaciones de recaudación de impuestos indirectos, y nexo fiscal por tener usuarios en un país.
- **Datos personales:** qué régimen aplica y cómo se compone con la arquitectura ya construida (los secretos nunca salen de la máquina del usuario, ADR-0093).
- **El pilar de cuentas verificadas:** publicar historiales de rentabilidad es la superficie más regulada del producto. Evaluar por jurisdicción si se acerca a captación de inversores.
- **Bloqueo por jurisdicción:** el propietario aprobó poder excluir países problemáticos. Qué mercados conviene excluir y qué se pierde con cada exclusión.
- **Coste y fricción real** de constituir y mantener en cada opción.

## Contexto que el abogado debe tener presente

El propietario declaró: *"Drasus Engine es para mí; si yo y mi familia, amigos y seres queridos la usamos prósperamente, el objetivo está resuelto."* La expansión masiva no es el objetivo declarado — eso cambia el peso relativo entre alcance de mercado y simplicidad regulatoria, y el análisis debe reflejarlo en vez de optimizar para escala global por defecto.

## Salida

Documento en el corpus legal (`docs/legal/`, prefijo `LDR`) con recomendación razonada, el contrario de cada opción, y qué queda fuera del alcance de un agente de IA y exige abogado colegiado.
