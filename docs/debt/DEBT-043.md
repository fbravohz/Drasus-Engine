# DEBT-043 · Los mensajes de rechazo de la importación conservan jerga de programador donde el ADR exige lenguaje del usuario

| Campo | Valor |
|---|---|
| **ID** | DEBT-043 |
| **Severidad** | 🟡 Baja |
| **Estado** | Abierta |
| **Abierta** | 2026-07-31 (gate de QA de STORY-073) |
| **Feature** | [`data-import-wizard`](../features/data-import-wizard.md) |
| **Origen** | ADR-0156 restricción 1 (FIJO) |

## Qué queda debiendo

ADR-0156 restricción 1 exige que un archivo rechazado lo sea "con un mensaje **en lenguaje del usuario** que explica por qué — nunca un error de parseo genérico". Los mensajes de la feature ya cumplen la mayor parte: están en español, son específicos y dicen qué hacer. Quedan tres tipos de fuga:

- **Verbos de programador:** "error al **parsear** la línea de dato número 5", "no cumple el formato", "al mapear la línea al **esquema canónico**".
- **Depuración de Rust filtrada al texto:** los marcadores `{field:?}` y `{raw:?}` imprimen la representación de depuración del valor, con comillas y escapes de Rust, no el nombre que el usuario ve en su archivo.
- **Vocabulario interno:** "sellar el ledger", "persistencia aguas abajo" — precisos para nosotros, opacos para quien acaba de arrastrar un CSV.

## Impacto actual

**Bajo.** Ningún mensaje es un volcado crudo del analizador ni un código de error: todos identifican la fila y el campo concretos, que es lo que permite corregir. El usuario puede actuar sobre cualquiera de ellos. El incumplimiento es de registro, no de utilidad.

## Por qué no se arregló en el momento

Es un barrido sobre **todas** las cadenas de error de la feature, no un arreglo puntual, y ninguna está rota: cambiarlas a mitad de STORY-073 habría mezclado un pulido de redacción con los arreglos de corrección que sí eran bloqueantes, y habría obligado a re-verificar cada prueba que asierta sobre el texto exacto.

## Disparador de pago

**El primer trabajo que toque la redacción de errores de cara al usuario en cualquier feature** — porque entonces conviene fijar de una vez la convención (qué se dice y qué no se dice en un mensaje de rechazo) y aplicarla, en vez de negociarla otra vez en cada Story.

No antes: no hay usuario bloqueado. No después del primer usuario externo: el mensaje de rechazo es el momento en que Drasus se explica solo, sin nadie al lado.

## Cómo se verifica que quedó pagada

Ningún mensaje de error que pueda llegar a la pantalla contiene verbos de programador ("parsear", "mapear al esquema"), marcadores de depuración de Rust (`{:?}`) ni vocabulario interno de la arquitectura ("ledger", "aguas abajo"). El nombre de columna que aparece en el mensaje es **el que el usuario ve en la cabecera de su archivo**.

## Relacionadas

- [`ADR-0156`](../adr/ADR-0156.md) restricción 1 — la exigencia que esto incumple.
- [`LDR-0008`](../legal/LDR-0008.md) — el rechazo explicado es parte de la cadena de consentimiento informado.
