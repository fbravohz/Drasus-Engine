# Veredicto Legal — Datos de Mercado de Terceros bajo la Nueva Doctrina "Cada Usuario Trae sus Datos"

> **Autor:** General Counsel · **Fecha:** 2026-07-18
> **Entrada:** doctrina de producto fijada por el propietario (Drasus no provee datos de mercado; el usuario los trae con sus propias credenciales) + tres frentes de revisión solicitados.
> **Relación con trabajo previo:** este veredicto es el insumo legal que [`plan-decision-fuentes-datos-mercado.md`](./plan-decision-fuentes-datos-mercado.md) (Architect, TASK-052, 2026-07-18) marca como **gate obligatorio y urgente antes que cualquier otro** (§4 de ese plan). Coincide de forma independiente con su hallazgo sobre Binance.US y lo confirma con cita textual (§Frente 2). Este documento no edita ADRs, features ni código — solo emite el veredicto y los requisitos.

**[knowledge consultado: `.agents/knowledge/base.md` — gobernanza obligatoria de arranque]**
**[knowledge consultado: `.agents/skills/general-counsel/SKILL.md` — mandato, gate de viabilidad, formato de veredicto]**
**[knowledge consultado: `.agents/knowledge/critical-domain-reasoning.md` — el Frente 1 sella una decisión de dominio legal (¿la anonimización rompe la cadena de titularidad?); las 7 comprobaciones se aplican en §0]**

---

## 0. Comprobaciones de razonamiento crítico (aplicadas al Frente 1)

1. **Límite degenerado:** si "anonimizar al usuario contribuyente limpia cualquier derivado" fuera cierto sin matices, cualquier dato de terceros —por restrictivo que sea su contrato— se volvería vendible con solo pasarlo por ruido gaussiano y sumar 5 contribuyentes. Eso es absurdo: convertiría la privacidad diferencial en una lavadora universal de restricciones contractuales, algo que ningún régimen legal reconoce. El modelo mental de partida ("anonimizar = protegido") está **incompleto**: confunde dos preguntas distintas.
2. **Gate de especialista:** este documento ES el gate de especialista (dominio legal). No se salta.
3. **Desconfianza del rigor impresionante:** la premisa "el usuario descargó el dato con sus propias credenciales, luego Drasus está protegido" suena rigurosa (desplaza la responsabilidad contractual al usuario) pero es la clase exacta de afirmación que hay que escrutar más, no menos — se analiza en detalle en el Frente 2/3.
4. **Re-derivación de supuestos:** la privacidad diferencial y el k-anonimato son herramientas diseñadas para un problema específico — impedir la re-identificación de una PERSONA dentro de un conjunto de datos agregado (protección de **datos personales**, GDPR/CCPA). Su supuesto de diseño es "el sujeto a proteger es el contribuyente individual". **No fueron diseñadas** para resolver restricciones contractuales/de propiedad sobre la FUENTE del dato (un bróker o un mercado). Ese es un cuerpo de derecho distinto (contratos, derecho de bases de datos, en EE. UU. potencialmente *misappropriation*). Aplicar la herramienta correcta al problema equivocado es el error central que este veredicto corrige.
5. **Separar el primitivo de la política:** el primitivo (`aggregate_index` en `crates/shared/src/domain/data_aggregation.rs`) es agnóstico de la fuente — recibe un `i64` genérico. La política (¿de qué fuente viene ese `i64`, y esa fuente permite agregación externa?) vive en el adaptador que llena `AggregationEventInput.metric_e8`, hoy sin especificar. No se debe colapsar "el motor de agregación es legal" con "cualquier cosa que se le dé de comer es legal" — son capas distintas.
6. **Cierre transitivo:** la pregunta de titularidad del dato no toca solo `data-aggregation` (#9). Toca también: `sovereign-data-fetcher` (la ingesta), la exportación de `data-portability` (#13 — exportar a un usuario un dato que vino de un tercero es también una forma de redistribución), y cualquier reporte institucional (#7) que cite cifras derivadas de una fuente restringida.
7. **Distinguir tipos, no promediar:** exactamente el pedido del propietario. Los cuatro índices vendibles (`Sentiment`, `Regime`, `BrokerFriction`, `Correlation`) **no** tienen la misma exposición porque no tienen la misma fuente de dato. Se tratan por separado en §1.2.

---

## FRENTE 1 — Agregación de Datos (cimiento #9)

### Veredicto: **APTO-CON-CONDICIONES**

### 1.1. La pregunta central, respondida directamente

**No.** Que el usuario haya descargado el dato con sus propias credenciales **no** protege a Drasus al monetizar el derivado agregado, **y la anonimización tampoco lo hace por sí sola** — son dos mecanismos que resuelven problemas distintos:

- **Privacidad diferencial + k-anonimato (`apply_differential_privacy`, `meets_k_anonymity`)** protegen al **usuario contribuyente** frente a re-identificación. Responden a GDPR/CCPA. Están bien implementadas (ruido sembrado y determinista, supresión estricta en el borde exacto de cohorte, guardarraíl de que ningún dato crudo sobrevive — verificado en el código y sus pruebas). **Esto SÍ está resuelto.**
- **La restricción contractual de la FUENTE del dato** (lo que el bróker/mercado permite hacer con SU dato) es un problema distinto: de contrato, no de privacidad. Anonimizar a los 5 usuarios que contribuyeron a un índice de "Fricción del Bróker X" no cambia que el índice sigue siendo, en sustancia, **un producto derivado de los datos de mercado del Bróker X, vendido sin su consentimiento**. El bróker no está preocupado por la privacidad de tus usuarios — está preocupado por que alguien construya un negocio de datos sobre su infraestructura sin pagarle. Ese interés **sobrevive intacto** a la anonimización, porque la anonimización nunca lo tocó.

Quién descargó el dato (Drasus de forma anónima, hoy; el usuario con sus credenciales, mañana) determina **quién es la contraparte contractual directa** del bróker, pero no determina si el **uso posterior** (agregar y vender externamente) está permitido por ese contrato. Un usuario puede tener pleno derecho a bajar su propio historial de operaciones y aun así no tener el derecho de **cederle a un tercero (Drasus) el permiso de reempaquetar y vender** ese dato — porque ese permiso, si el contrato del bróker lo prohíbe, el usuario nunca lo tuvo para dar. **Nadie puede transferir un derecho que no posee.**

### 1.2. La frontera exacta, por tipo de índice

El Core (`aggregate_index`) es agnóstico de fuente — el riesgo no vive ahí, vive en qué alimenta `metric_e8`. La feature doc lista cuatro índices; su exposición es muy distinta:

| Índice | Sustancia real | Fuente | Exposición |
|---|---|---|---|
| **Regime** | Estado del mercado (volatilidad/tendencia) — inherentemente derivado de series de precios de MUCHOS instrumentos | Datos de mercado de terceros (precios/velas) | 🔴 **Alta.** Es, en sustancia, un producto de datos de mercado repackaged. Cae directo en la clase de cláusula verificada en Binance.US: "data feeding... that make use of any market data" (§Frente 2). |
| **Correlation** | Correlación cruzada entre instrumentos — requiere series de precios de múltiples fuentes/brókers | Datos de mercado de terceros | 🔴 **Alta.** Mismo razonamiento que Regime; además cruza MÚLTIPLES fuentes, multiplicando el número de contratos potencialmente violados por cada agregado publicado. |
| **BrokerFriction** | Slippage/tiempo de llenado del bróker X frente al motor propio de Drasus | Confirmaciones de ejecución del propio usuario (su "estado de cuenta") + un precio de referencia | 🟡 **Media-baja, la más defendible.** La sustancia es "cómo se comportó tu propia orden", análogo a un extracto bancario — el usuario tiene un derecho más claro sobre ESE dato (ver Frente 3 §3.3). Pero: (a) el índice **nombra explícitamente al bróker** — no es solo "usuarios anónimos", es "análisis de rendimiento de la infraestructura del Bróker X"; (b) el precio de referencia contra el que se mide el slippage típicamente viene de un feed de mercado, que puede estar restringido. Necesita revisión por bróker, pero parte de mejor posición que Regime/Correlation. |
| **Sentiment** | Sin definir en el código — puede ser (a) momentum de precio (derivado de mercado, alta exposición) o (b) posicionamiento/flujo de órdenes de los propios usuarios (comportamiento propio, baja exposición) | **Ambiguo — no especificado** | ⚠️ **Indeterminada hasta que se fije la fuente.** No puede evaluarse en abstracto. |

**Conclusión de la distinción pedida:** la salida viable SÍ existe, y es exactamente la que el propietario intuyó — pero con un matiz importante: la frontera no es "dato de mercado vs. comportamiento del motor" en abstracto, sino **si la sustancia del índice, una vez despojada de la identidad del usuario, sigue siendo reconocible como "los datos del bróker/mercado X"** (Regime, Correlation) **o si se ha convertido en información sobre la relación del usuario con ese bróker** (BrokerFriction — más cerca de "tu propio historial de cuenta"). Esa frontera es de sustancia económica, no de mecanismo técnico; un abogado del bróker mirando el índice publicado preguntaría "¿esto compite con nuestro propio negocio de datos de mercado?" — la respuesta es probablemente sí para Regime/Correlation, probablemente no (o mucho más débil) para BrokerFriction.

### 1.3. Fallos en la especificación

- El puerto `aggregate_out` / `AggregatedIndex` no tiene ningún campo que registre la **fuente de datos subyacente** ni una marca de "permitido agregar/vender externamente por esa fuente". Hoy es posible, técnicamente, encender `EXTERNAL_SALE_ENABLED=true` y vender un índice `Regime` construido sobre datos de un bróker cuyo ToS lo prohíbe expresamente, sin que ningún guardarraíl del código lo impida — el guardarraíl actual (`ADR-0093/0102`, "datos crudos nunca en la salida") protege identidad de usuario, no titularidad de fuente.
- Esto coincide con — y refuerza — el hallazgo independiente del Architect en `plan-decision-fuentes-datos-mercado.md` §4 y §7.5: "el primitivo es la partición almacenada; la política es qué se puede hacer con ella... propongo una marca de 'agregable' separada del identificador de linaje." **Concuerdo formalmente con esa recomendación** desde el lado legal.

### 1.4. Requisitos (criterios de aceptación para trabajo futuro)

1. **Marca de agregabilidad por fuente, obligatoria antes de cualquier venta externa.** Cada partición de dato (identificada por `data_snapshot_id`/proveedor, ver Frente 2/3) lleva una marca binaria "agregable externamente: sí/no", poblada por dictamen de General Counsel por proveedor — nunca por default `true`, nunca decidida por el ingeniero. `run_aggregation` debe rechazar (no solo advertir) un `channel=EXTERNAL` cuyos eventos contribuyentes incluyan una fuente sin esa marca.
2. **Fijar la fuente de `Sentiment` antes de construir su adaptador** — Tech-Lead/Quant-Engineer deciden si es momentum de precio o comportamiento propio; ese dato determina su exposición y no puede quedar implícito.
3. **`BrokerFriction` puede avanzar primero** hacia el canal externo (una vez resuelto el Frente 3 sobre datos del propio usuario) porque es el índice con mejor postura legal de los cuatro — pero solo tras confirmar que el precio de referencia usado para calcular el slippage no arrastra, él mismo, una fuente restringida.
4. **`Regime` y `Correlation` quedan bloqueados para el canal `EXTERNAL`** hasta que exista, para cada fuente de mercado que los alimente, o bien (a) consentimiento escrito del proveedor de datos, o (b) migración a un proveedor de datos con derechos de redistribución explícitos y documentados (el plan del Architect ya identifica a Databento como el único candidato con esa política publicada — útil como referencia de qué se ve un contrato limpio).
5. El canal `INTERNAL` (uso propio del motor, sin venta) **no** tiene esta restricción con la misma fuerza — sigue siendo prudente documentar la fuente, pero el uso interno de datos de mercado para operar/analizar (no para revender) es una práctica estándar de la industria financiera y no es el foco de las cláusulas verificadas (que hablan de "cobrar por" o "servicios que se alimentan de" el dato hacia terceros).

---

## FRENTE 2 — El Adaptador de Binance

### Veredicto: **NO-APTO** en su forma actual (descarga anónima a escala de compañía, para un producto de pago) · **APTO-CON-CONDICIONES** la ruta de migración

### 2.1. Exposición concreta, verificada

Verifiqué directamente (fetch a la fuente primaria, no solo snippet de búsqueda) el texto de los Términos de Uso de Binance.US:

> *"Without written consent from Binance.US, the following commercial uses of Binance.US data are prohibited: Trading services that make use of Binance.US quotes or market bulletin board information; Data feeding or streaming services that make use of any market data of Binance.US; or Any other websites/apps/services that charge for or otherwise profit from (including through advertising or referral fees) market data obtained from Binance.US."*
> — [Binance.US Terms of Use](https://www.binance.us/terms-of-use) / [confirmación en la página de datos históricos](https://www.binance.us/institutions/market-history)

Esto **confirma textualmente** el hallazgo que ya tenía el paquete de escalamiento (TASK-052). El código actual en `crates/features/data/sovereign-data-fetcher/src/schemas.rs:29` (`bulk_source_url: "https://data.binance.vision"`) y `public_interface.rs:338` (`"https://api.binance.com"`) descarga **de forma anónima, a nombre de Drasus como compañía, sin ninguna cuenta de usuario**, y ese dato alimenta un motor que es la base de un **producto de pago**. Ese es, casi literalmente, el patrón que la cláusula describe ("data feeding... that make use of... market data", "services that... profit from... market data obtained from Binance").

**Binance global (binance.com):** intenté verificar el texto exacto en dos fuentes (`binance.com/en/terms` y `developers.binance.com/.../PROD-TERMS-OF-USE`) y **no pude obtener el texto sustantivo** — ambas páginas redirigen a documentos que mi herramienta no pudo renderizar completos. Los resúmenes de búsqueda (no verbatim, no confiables por sí solos) sugieren una cláusula equivalente ("resale or commercial purposes... unless expressly agreed by Binance in writing"), pero **esto queda marcado como NO VERIFICADO** — no lo afirmo como hecho. Dado que Binance.US y Binance global comparten linaje corporativo y patrón de redacción, la prudencia razonable es asumir una exposición **igual o mayor**, no menor, hasta verificación directa (recomiendo que alguien con acceso humano al sitio, o el propio equipo, confirme `dev.binance.vision/tos` — mi fetch automatizado no devolvió el contenido).

### 2.2. ¿La migración al modelo de credenciales del usuario resuelve esto?

**Parcialmente — reduce la exposición más clara, no la elimina.**

- **Sí resuelve:** el patrón más literal y más fácil de perseguir hoy —Drasus, como compañía, operando una descarga masiva anónima y sistemática que alimenta un producto de pago— desaparece. Un usuario individual conectándose con su propia cuenta para ver/analizar SU actividad es un patrón ampliamente tolerado en la industria (terminales de trading de terceros, bots, rastreadores de portafolio que se conectan vía API-key del propio usuario son un modelo comercial extendido y generalmente aceptado).
- **No resuelve — dos huecos reales:**
  1. **El texto de la cláusula no se activa por "quién tiene la llave", se activa por "si el servicio cobra por / se alimenta de datos de mercado de Binance".** Drasus sigue siendo un producto de pago cuyo motor computa sobre esos datos. La lectura más favorable a Drasus es que la cláusula apunta a competidores directos del negocio de datos de Binance (revendedores de feed), no a herramientas que ayudan a un usuario a analizar su propia cuenta — y esa lectura es razonable y es como se aplican estas cláusulas en la práctica (el ecosistema de bots/terminales de trading conectados por API existe y no ha sido barrido). Pero es una lectura de **riesgo de aplicación**, no una certeza contractual — el texto literal es más amplio. No debe confundirse "bajo riesgo de que te persigan" con "permitido".
  2. **Muchos exchanges exigen que la propia APLICACIÓN (no solo el usuario final) se registre como desarrollador y acepte términos de API separados**, que atan al software independientemente de con qué cuenta opera. No pude verificar el contenido de `dev.binance.vision/tos` (Frente 2.1). **Esto es un hueco de verificación real y debe cerrarse antes de declarar la migración "segura"** — no basta con cambiar de quién es la credencial si el software mismo queda atado a un acuerdo de desarrollador con sus propias restricciones.
  3. Independientemente de lo anterior, el Frente 1 (venta externa de índices derivados de mercado) **sigue expuesto** aunque la ingesta pase a ser por credenciales del usuario — anonimizar al usuario no libera al dato de las restricciones de su fuente (§1.1).

**Nota sobre ADR-0143 y la propiedad del dato del tier gratuito:** el ADR dice que el trabajo del usuario gratuito "fluye al proveedor y es suyo por ToS". Esto gobierna la relación **Drasus↔usuario**, y es válida en ese plano. Pero es **jurídicamente irrelevante frente al bróker**: el usuario no puede cederle a Drasus, vía la aceptación de un ToS con Drasus, un derecho sobre el dato de Binance que el propio usuario nunca tuvo frente a Binance. Este matiz **no está capturado hoy en ADR-0143** y debería añadirse como aclaración cuando se ejecute la doctrina de "usuario trae sus datos" (ver Frente 3, requisito 3).

### 2.3. La muestra congelada de 20 operaciones (2 KB)

**De minimis — riesgo real pero bajo, condicionado.** Razones:
- Es dato público (mismo endpoint que cualquiera puede consultar sin autenticación), en volumen trivial (20 trades — muy por debajo de cualquier umbral razonable de "parte sustancial" bajo doctrina de derecho de bases de datos tipo UE, que exige inversión sustancial extraída, no una muestra ínfima con fines de ingeniería).
- Uso puramente interno de prueba (fixture de test), sin explotación comercial del archivo en sí, sin distribución a terceros como producto.
- **Condición:** esto deja de ser de minimis si el repositorio se vuelve público o si el archivo se empaqueta dentro de un binario distribuido (deja de ser "uso interno de ingeniería" y pasa a ser "redistribución del dato de Binance a cualquiera que descargue el software"). No verifiqué el estado de visibilidad del repositorio — márquese como pendiente de confirmar antes de cualquier release público.
- Recomendación de bajo costo: sustituir por datos sintéticos generados (mismo formato, sin depender de que la muestra real siga siendo "pequeña" para siempre) antes de que el repo pase a ser distribuido — barato de hacer ahora, evita tener que recordar la condición después.

### 2.4. Requisitos

1. **Detener/no ampliar la descarga anónima a escala de compañía en producción** hasta que se resuelva (a) o (b): (a) consentimiento escrito de Binance, (b) migración completa al modelo de credenciales del usuario **y** verificación de los términos de desarrollador de API (`dev.binance.vision/tos` u homólogo).
2. **Verificar el texto exacto de Binance.com global** antes de asumir que es más permisivo que Binance.US — hoy no hay evidencia de que lo sea.
3. **Verificar los términos de desarrollador/API** (distintos de los términos de usuario final) antes de declarar que el modelo de credenciales del usuario resuelve la exposición del lado del software.
4. **Aplicar el mismo veredicto, proveedor por proveedor**, a cada fuente Forex/futuros candidata del plan del Architect — ya lo marca ese documento (§4) y coincido: Dukascopy y HistData.com tienen riesgo igual o mayor (sin términos publicados en absoluto, lo cual es peor que un término restrictivo explícito, porque "ausencia de permiso ≠ permiso").
5. **Sustituir la muestra congelada por datos sintéticos** antes de cualquier release público del repositorio (bajo costo, cierra la condición de forma permanente).

---

## FRENTE 3 — El Modelo "Cada Usuario Trae sus Datos"

### Veredicto: **APTO-CON-CONDICIONES**

### 3.1. Responsabilidad sobre calidad del dato importado y resultados derivados

Exposición baja si se documenta correctamente; exposición real si no. Drasus no puede verificar la integridad/exactitud del dato que el usuario trae desde su propio bróker o exporta a mano — eso es responsabilidad del usuario y de su fuente. Pero si Drasus presenta resultados de backtest/análisis como si fueran fiables sin ese disclaimer, se acerca a una representación implícita de calidad que no puede sostener. Se conecta directamente con la línea regulatoria central de esta skill (herramienta vs. asesoría de inversión, SKILL.md §2): un backtest corrido sobre datos de calidad desconocida, presentado sin matices, es doblemente peligroso — no solo por la exactitud del dato, sino porque refuerza la lectura de "consejo financiero fiable" que Drasus necesita evitar activamente.

**Requisito:** cláusula explícita de T&C/EULA: (a) Drasus no verifica ni garantiza la exactitud, integridad o actualidad de ningún dato importado por el usuario desde fuentes de terceros; (b) los resultados (backtests, indicadores, reportes) son tan buenos como el dato de entrada y se ofrecen "tal cual", sin garantía; (c) reiterar la postura de "herramienta, no asesor" en el mismo lugar.

### 3.2. Almacenamiento de credenciales de bróker en el equipo del usuario

La arquitectura ya está alineada con la postura de menor riesgo posible: ADR-0093 (AES-256-GCM, Master Key nunca en servidor) + ADR-0143 (secretos jamás salen, en ningún tier, invariante FIJO) significan que Drasus **nunca custodia** la credencial en un servidor propio — elimina el escenario de mayor exposición típico de este tipo de producto (una brecha del lado del proveedor que filtra credenciales de bróker de miles de usuarios a la vez). Esto es una ventaja legal real y vendible, no solo técnica.

Dos residuales que sí importan:
- **Responsabilidad por defecto de diseño de seguridad:** aunque la credencial nunca salga de la máquina del usuario, si el mecanismo de cifrado local tiene un defecto (mal manejo de la Master Key, cifrado débil), Drasus como fabricante del software puede ser responsable por diseño de seguridad negligente — la "soberanía" arquitectónica reduce el radio de explosión de una brecha, no elimina la responsabilidad de construirla bien.
- **Veracidad de la afirmación de mercadeo:** si Drasus promete "nunca tenemos acceso a tus credenciales", esa afirmación debe ser literalmente cierta de punta a punta (incluyendo herramientas de soporte/depuración) — una sola excepción no documentada convierte la promesa en una representación engañosa frente al consumidor.

**Requisito:** ninguno bloqueante — la arquitectura ya es correcta. Añadir a T&C/política de privacidad la descripción exacta y verificable del mecanismo (qué se cifra, con qué, quién tiene la llave), y una auditoría técnica puntual (fuera de mi mandato, del Rust/Refactoring Engineer) que confirme que no existe ninguna ruta de excepción antes de publicitar la promesa de soberanía como garantía absoluta.

### 3.3. ¿Conectarse con las credenciales del usuario para descargar en su nombre nos convierte en algo distinto ante los términos del bróker?

Cubierto en profundidad en el Frente 2.2. Resumen aplicado aquí: **sí cambia el patrón de hecho** (de "Drasus revendedor de datos" a "herramienta que el usuario dirige con su propia llave", un modelo comercial tolerado) pero **no es una defensa categórica** frente a cláusulas que restringen el USO del dato (no solo el acceso) ni frente a posibles términos de desarrollador de API que atan al software independientemente de la cuenta usada. Es una mejora real, no una inmunidad.

**Matiz adicional relevante para este frente:** hay una analogía útil pero **débil y no transferible sin más** con el "open banking"/agregación financiera (en EE. UU., Sección 1033 de Dodd-Frank; en la UE, PSD2) — la idea de que un usuario tiene derecho a autorizar a un tercero a acceder a sus propios datos financieros pese a que el proveedor original lo restrinja. **Verificado:** la norma de la CFPB bajo la Sección 1033 está hoy **suspendida judicialmente** (medida cautelar del Tribunal de Distrito de Kentucky, octubre 2025) mientras la CFPB rehace la reglamentación — no es un piso legal sólido en EE. UU. hoy. Además, 1033 apunta a **cuentas financieras de consumo** (bancarias/pago); su aplicación a cuentas de bróker/exchange de cripto **no está resuelta**. PSD2 en la UE tiene el mismo problema de alcance (cuentas de pago, no brokerage). **Conclusión: no existe hoy un piso regulatorio firme de "derecho a tus propios datos de trading" en ninguna jurisdicción principal que Drasus pueda invocar como escudo.** La postura de "usuario trae sus datos" es defendible por buenas prácticas de producto y por reducir la exposición de Drasus como intermediario — no porque exista un derecho estatutario que la respalde.

### 3.4. Dato en vivo vs. dato histórico

**Confirmado: sí hay una distinción real en la industria, y aplica aquí con matices.**

- **Patrón general de la industria de datos de mercado** (bolsas tradicionales: modelos de licenciamiento de NYSE/Nasdaq separan explícitamente tiempo real vs. retrasado 15 minutos vs. fin de día, con tarifas y restricciones de redistribución mucho más estrictas para tiempo real) — esto es conocimiento general de la industria, no verificado específicamente para cripto/Binance, pero es un patrón consistente y bien documentado del sector.
- **Doctrina de "hot news" (EE. UU.):** *INS v. AP* (1918), refinada por *NBA v. Motorola* (1997) — protege información generada a costo, sensible al tiempo, contra el aprovechamiento (free-riding) de un competidor que erosiona el incentivo de quien la genera. Aplica con más fuerza a dato en vivo que a dato histórico ya "enfriado". Es una doctrina real pero angosta, y **no verifiqué que se haya aplicado a datos de mercado cripto en un caso concreto** — se cita como respaldo direccional, no como precedente resuelto.
- **Aplicación concreta al código:** `sovereign-data-fetcher` tiene dos canales con perfiles de riesgo distintos que hoy se tratan igual. El canal **Bulk** (`data.binance.vision`) es un archivo histórico que la propia Binance publica activamente para descarga pública masiva ("Binance Data Collection... offers easy access for anyone to download") — el más cercano a una licencia implícita, aunque no escrita, para ESE uso específico. El canal **Delta/REST** (`api.binance.com/klines`, casi en tiempo real) está mucho más cerca del uso de producción de la API de trading, donde vive la cláusula restrictiva verificada del Frente 2 ("Data feeding or streaming services"). **Distinción a preservar:** el riesgo del canal Delta es mayor que el del canal Bulk, y cualquier revisión legal futura por proveedor debería evaluarlos por separado, no como una sola fuente.

### 3.5. Requisitos consolidados del Frente 3

1. Cláusula de T&C sobre calidad de dato de terceros y resultados derivados (§3.1) — no bloqueante, documental.
2. Confirmar (fuera de mi mandato — Rust/Refactoring Engineer) que no existe ninguna ruta de excepción a "las credenciales nunca salen de la máquina" antes de publicitarlo como garantía absoluta (§3.2).
3. Aclarar en ADR-0143 (o en su enmienda futura) que "el trabajo del usuario gratuito es del proveedor por ToS" **no** transfiere derechos que el usuario nunca tuvo sobre datos de un tercero (bróker/mercado) — evita que un futuro lector interprete el ADR como si resolviera el Frente 1/2 (§2.2).
4. Verificar los términos de desarrollador/API de cada bróker/exchange soportado (no solo los términos de usuario final) antes de dar por buena la migración a credenciales del usuario como solución completa (§2.2, §3.3).
5. Tratar el canal Bulk/histórico y el canal Delta/tiempo real como perfiles de riesgo separados en cualquier matriz legal por proveedor (§3.4) — recomiendo incorporar esta distinción a la "marca de agregable por partición" que ya propone el plan del Architect (Frente 1, requisito 1).

---

## Resumen ejecutivo (para el propietario)

| Frente | Veredicto | Por qué |
|---|---|---|
| **1. Agregación de datos** | APTO-CON-CONDICIONES | El motor de agregación en sí es legalmente neutro — el riesgo vive en qué fuente alimenta cada índice, no en el código de anonimización (que sí cumple su propio objetivo: proteger al usuario). `Regime` y `Correlation` no pueden venderse externamente sin permiso del proveedor de datos; `BrokerFriction` tiene mejor postura; `Sentiment` necesita definirse primero. |
| **2. Adaptador de Binance** | NO-APTO como está (descarga anónima a escala de compañía para un producto de pago) | Verificado con cita textual: Binance.US prohíbe expresamente lo que el sistema hace hoy. Migrar a credenciales del usuario reduce el riesgo pero no lo elimina — persiste mientras Drasus venda o cobre por algo construido con ese dato, y mientras no se verifiquen los términos de desarrollador de API. La muestra de 2 KB es de minimis, no la exposición real. |
| **3. Modelo "cada usuario trae sus datos"** | APTO-CON-CONDICIONES | Reduce exposición de forma genuina (elimina el escenario de brecha centralizada de credenciales, aleja a Drasus del rol de "revendedor de datos"), pero no es un escudo legal automático — no existe hoy un derecho estatutario firme al que Drasus pueda aferrarse, y persisten los términos de desarrollador de API sin verificar. El dato en vivo es más riesgoso que el histórico; ya hoy el código trata ambos igual y debería distinguirlos.

**La idea que hay que corregir en la doctrina del propietario:** "el usuario lo descargó, luego es su dato, luego Drasus está protegido al venderlo" mezcla dos preguntas legales distintas (privacidad del contribuyente vs. titularidad contractual de la fuente). La primera SÍ está resuelta por el diseño actual. La segunda **no se resuelve con anonimización ni con el cambio de credenciales** — se resuelve fuente por fuente, con permiso escrito o con proveedores de datos que ya venden ese permiso.
