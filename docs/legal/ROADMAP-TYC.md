# Hoja de Ruta hacia los Términos y Condiciones — Estado del Corpus Legal

> **Propósito:** cerrar el corpus legal indicando, cláusula por cláusula, qué ya tiene una decisión registrada que la sostiene, qué falta por decidir o redactar, y qué exige un abogado colegiado de una jurisdicción concreta antes de publicarse. No es el texto final de los T&C — es el mapa de qué falta para llegar a él.
> **Autor:** General Counsel · **Fecha:** 2026-07-18

---

## Límite de alcance de este corpus (léase antes que cualquier tabla)

Quien redacta este documento y los `LDR-XXXX` es un **agente de IA operando bajo el rol de General Counsel de Drasus**, no un abogado colegiado en ninguna jurisdicción. Su valor es: detectar exposición temprano, evitar construir sobre un supuesto legal falso, y dejar el trabajo de análisis y borrador ya hecho. Su límite, sin ambigüedad:

- **Ninguna cita de norma en este corpus (GDPR, CCPA, LFPDPPP, MiFID, doctrina "hot news") sustituye la opinión de un abogado colegiado.** Se citan como fundamento razonado, con honestidad sobre qué está verificado (fetch directo a la fuente) y qué no (resumen de búsqueda, marcado explícitamente).
- **Ningún texto de cláusula de este corpus es publicable tal cual.** Es borrador de trabajo en español, pensado para fijar la intención y el alcance — la versión que se publique necesita traducción, adaptación y validación por abogado colegiado en cada jurisdicción donde Drasus opere.
- **Este corpus no determina si Drasus necesita registrarse como asesor de inversión, gestor regulado, o entidad de money-transmitter** — solo señala dónde esa línea está en riesgo de cruzarse. La determinación final es de un abogado de valores/regulación financiera colegiado.
- Fase del proyecto: **greenfield**. Nada de esto bloquea el desarrollo. Bloquea el **primer lanzamiento distribuido y el primer usuario de pago real** — ese es el disparador que convierte cada fila "falta" de abajo en urgente.

---

## Tabla 1 — Cláusulas ya sostenidas por una decisión registrada

| Cláusula de T&C | Sostenida por | Estado del texto |
|---|---|---|
| Responsabilidad sobre calidad/licencia del dato importado por el usuario | [LDR-0004](./LDR-0004.md) | Borrador redactado, con alcance de oponibilidad explícito |
| Frontera de agregación (qué datos se pueden agregar y vender) | [LDR-0001](./LDR-0001.md), [LDR-0006](./LDR-0006.md) | Regla fijada; falta traducir a cláusula pública dirigida al usuario y a cláusula de licencia dirigida al comprador institucional del índice |
| Grabación de flujo en vivo — consentimiento y alcance | [LDR-0005](./LDR-0005.md) | Alcance legal fijado; falta el texto de consentimiento que el usuario ve al activar la grabación |
| Consentimiento y opt-outs (mecanismo general) | [LDR-0007](./LDR-0007.md) §1, ADR-0144 (#5) | Mecanismo cimentado; falta el texto de cada opt-in concreto (firehose gratuito, publicación de track record, pool de inteligencia colectiva) |
| Portabilidad y derecho al olvido | [LDR-0007](./LDR-0007.md) §2, ADR-0148 | Mecanismo cimentado (catálogo + registro); falta el texto de la cláusula y el SLA de respuesta |
| Roles de operador y responsabilidad delegada | [LDR-0007](./LDR-0007.md) §3, ADR-0149 | Mecanismo cimentado; falta el texto de "acciones del operador = acciones del titular de la cuenta" |
| Veracidad de cuentas verificadas y su publicación | [LDR-0007](./LDR-0007.md) §4, ADR-0145 | Mecanismo cimentado (atestado vs. reportado, LIVE vs. virtual); falta el texto público que acompaña cada track record publicado |
| Soberanía de datos condicionada por tier (qué es del usuario, qué es del proveedor) | [LDR-0007](./LDR-0007.md) §5, ADR-0143 | Mecanismo cimentado; falta el texto — es la cláusula de mayor visibilidad y mayor exposición de todo el corpus (ver Tabla 3) |
| Descarga/migración de datos de bróker — postura general | [LDR-0002](./LDR-0002.md), [LDR-0003](./LDR-0003.md) | Postura y condiciones fijadas; **bloqueada para publicación** hasta cerrar verificaciones pendientes (ver Tabla 2) |
| Transformación de archivos con mapeo declarado por el usuario — responsabilidad sobre el resultado | [LDR-0008](./LDR-0008.md) | Regla fijada (mapeo declarado protege bajo 4 condiciones; plantilla de plataforma no); falta el texto público que acompaña la pantalla de vista previa/aceptación |
| Promesa "nunca vemos tus credenciales" | ADR-0093, [LDR-0003](./LDR-0003.md) §2 | Arquitectura correcta; texto solo publicable como garantía absoluta tras auditoría técnica (ver Tabla 2) |

---

## Tabla 2 — Falta por resolver antes de redactar el texto final (trabajo interno, no requiere abogado externo todavía)

| Pendiente | Qué falta | Dueño | Bloquea |
|---|---|---|---|
| Texto exacto de Binance.com global | Verificación directa del ToS (dos fetch automatizados fallaron) | General Counsel / humano con acceso al sitio | Publicar cualquier cláusula sobre el adaptador de Binance |
| Términos de desarrollador de API de cada bróker soportado | Verificar `dev.binance.vision/tos` y equivalentes — distinto de los términos de usuario final | General Counsel, por bróker | Declarar "segura" la migración a credenciales del usuario |
| Auditoría de "cero ruta de excepción" al cifrado de credenciales | Confirmación técnica de que ninguna herramienta de soporte/depuración accede a la credencial | Rust/Refactoring Engineer | Publicitar "nunca vemos tus credenciales" como garantía absoluta |
| Fuente de datos del índice `Sentiment` | Decisión de si es momentum de precio o comportamiento propio del usuario | Tech-Lead / Quant-Engineer | Clasificar su exposición legal (LDR-0001) |
| Columna histórico/vivo en la tabla de capacidades de ADR-0154 | Construcción del campo (hoy es un solo booleano implícito) | Tech-Lead (DEBT-026) | Activar cualquier venta externa de índice |
| Sustitución de la muestra de 20 trades por datos sintéticos | Bajo costo, ya recomendado | Rust Engineer | Cualquier release público del repositorio |
| Estado de visibilidad del repositorio (público/privado) | No verificado en este corpus | Tech-Lead | Determina si la muestra congelada sigue siendo de minimis |

---

## Tabla 3 — Exige abogado colegiado de una jurisdicción concreta antes de publicarse

| Cláusula | Por qué no basta con este corpus | Jurisdicción(es) prioritaria(s) |
|---|---|---|
| **Cesión de propiedad del trabajo del usuario gratuito, incluida la recreación de estrategias con capital del propietario del proyecto** (ADR-0143) | Es la cláusula más agresiva y más atípica de todo el corpus — un usuario gratuito puede ver su estrategia tomada y operada con capital ajeno. El riesgo de que un regulador de protección al consumidor la declare abusiva/desproporcionada es real y no lo resuelve un disclaimer bien escrito; necesita opinión de abogado de consumo antes de publicarse en cualquier mercado con protección al consumidor fuerte (UE, Reino Unido, California). | UE (unfair contract terms directive), Reino Unido, California (CCPA/CPRA + Unfair Competition Law) |
| **Determinación de si Drasus cruza la línea de "asesoría de inversión regulada"** | Es la exposición #1 del mandato de este skill (`SKILL.md §2`) y es, por naturaleza, una pregunta de derecho de valores que un generalista no puede cerrar — depende de matices de jurisdicción y de la redacción exacta de cada feature que rankee, recomiende o certifique. | EE. UU. (SEC/CFTC/NFA), UE (MiFID II/MiFIR), Reino Unido (FCA) |
| **Cláusula de ley aplicable, jurisdicción y resolución de disputas** | No existe todavía decisión sobre dónde se constituye la entidad Drasus ni bajo qué ley se rigen los T&C — es una decisión corporativa previa, no solo legal. | Depende de la jurisdicción de incorporación (aún no decidida) |
| **Cumplimiento GDPR Art. 15/17/20 en su mecánica real** (SLA de exportación/olvido) | El generador de export está diferido por diseño (ADR-0148); antes del primer usuario UE, un abogado de protección de datos debe validar que el SLA prometido en el texto público sea el que el sistema puede cumplir en la práctica. | UE (GDPR), Reino Unido (UK GDPR) |
| **Verificación final de la interpretación de la cláusula de Binance como "riesgo de aplicación bajo" y no "prohibición literal"** | La lectura favorable a Drasus (LDR-0002) es razonable pero es una opinión de riesgo, no un hecho contractual. Antes de escalar comercialmente el adaptador, conviene una opinión externa o consentimiento escrito de Binance. | Jurisdicción del contrato de Binance (revisar sus términos de elección de foro) |
| **Sanciones/control de exportación (OFAC y equivalentes) para un producto cripto con alcance potencialmente global** | Mencionado en el mandato de este skill, no auditado en este corpus — ningún ADR revisado lo cubre todavía. | EE. UU. (OFAC), UE |
| **Límites de responsabilidad (topes en USD) e indemnización** | Cláusulas estándar de SaaS que dependen de jurisprudencia local sobre qué tope es "razonable" y ejecutable — no auditadas aquí. | Todas las jurisdicciones de publicación |
| **Exigibilidad de la aceptación "clickwrap" del mapeo de transformación** ([LDR-0008](./LDR-0008.md)) | Si un clic sobre la pantalla de vista previa/aceptación cuenta como consentimiento informado real depende de jurisprudencia local de exigibilidad de clickwrap y de términos abusivos de consumo — mismo tipo de riesgo que la fila de cesión de propiedad, caso propio. | UE (unfair contract terms directive), Reino Unido, California (CCPA/CPRA + Unfair Competition Law), EE. UU. (línea de casos clickwrap vs. browsewrap) |

---

## Orden recomendado (no bloquea desarrollo, sí bloquea distribución)

1. Cerrar Tabla 2 (trabajo interno, sin costo de abogado) — puede avanzar ya, en paralelo al desarrollo.
2. Enviar Tabla 3 a abogado colegiado **antes** del primer release distribuido (disparador: fin de fase greenfield, CLAUDE.md §1) — priorizar la fila de cesión de propiedad del trabajo gratuito y la determinación de asesoría regulada, por ser las de mayor severidad.
3. Publicar el texto final de T&C solo después de que la Tabla 3 tenga sign-off de abogado colegiado en al menos la jurisdicción de lanzamiento inicial.
4. El portal público (`../drasus-portal/`) tendrá su propia política de privacidad/cookies — diferida al mismo disparador que su construcción ("primer cobro real, tras EPIC-5", ADR-0143/ADR-0145) — no se aborda en este corpus.

---

**Trazabilidad:** consolida [LDR-0001](./LDR-0001.md) a [LDR-0007](./LDR-0007.md). Índice del corpus en [`docs/LEGAL.md`](../LEGAL.md).
