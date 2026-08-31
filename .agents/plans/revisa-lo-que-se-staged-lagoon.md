# Plan: `docs/business/PARTNERSHIP-PROGRAM.md`

**Estado: aprobado por el propietario ("adelante") — mismo plan, sin cambios de fondo tras la reestructuración de §3 por tipo de aporte.**

## Contexto

El propietario quiere abrir la infraestructura tecnológica de Drasus Engine a su círculo cercano (pareja, hermana, mamá, amigos con distinto perfil: Jr dev, científico de datos, vendedores) para que generen ingresos apalancados en el producto, sin regalar dinero ni ceder control. El objetivo es casi altruista — dar una ventana de oportunidad — pero con dos líneas rojas explícitas del propietario: **él siempre es el máximo accionista y máximo tomador de decisiones**, y **la relación personal debe poder terminar sin fricción** (entrada y salida fáciles).

Este documento no existía. No es arquitectura técnica — es estrategia societaria — así que no encaja en ninguna categoría del "Gate de Creación de Documentos" del skill Architect (`ADR`/`SAD`/`features`/`modules`/`moonshots`/`PRODUCT*`/`README`). Con el propietario ya se acordó:
- **Ubicación:** `docs/business/PARTNERSHIP-PROGRAM.md` (carpeta nueva, deja intacto el Gate de la raíz de `docs/`).
- **Concreción:** cada modelo trae rangos ilustrativos concretos (%, plazos), marcados explícitamente como punto de partida negociable, no como términos legales cerrados.

## Hallazgo que condiciona el diseño (investigación hecha en esta sesión)

Búsquedas reales hechas hoy (Slicing Pie, FAST advisor grid, profit split de prop firms tipo FTMO, phantom equity/profits interest, comisión de afiliados SaaS/fintech, y dos búsquedas de derecho mexicano):

- **La Ley General de Sociedades Mercantiles (México) prohíbe la aportación de trabajo puro a cambio de acciones** en sociedades de capital (S.A./S.A.P.I.) — no existe "socio industrial" formal viable en el vehículo que usaría Drasus. Esto descarta "le doy acciones por su chamba" como mecanismo simple y obliga el documento a apoyarse en alternativas legalmente limpias: **phantom equity** (bono contractual atado a valuación, sin acciones reales), **pool de opciones con precio de ejercicio nominal** (si algún día hay SAPI formal), o **profit-share contractual** vía honorarios variables.
- **Riesgo de simulación laboral:** pagar "honorarios" con horario fijo, supervisión directa y dependencia económica puede reclasificarse como relación laboral encubierta ante el SAT/autoridad laboral (multas, retroactivo de prestaciones). Afecta directamente el caso del "amigo Jr socio-empleado sin sueldo".
- **Prop-firm funded account (estilo FTMO):** split de ganancias típico 80/20 subiendo a 90/10 a favor del operador, sin exponerlo a pérdidas — modelo directamente aplicable al caso de la mamá vía `copy-trading-engine` + `verified-account-registry`.
- **Advisor equity grid (FAST, Founder Institute):** 0.15%–1.00% de equity según etapa/involucramiento, vesting corto (2 años, cliff 0–3 meses) — aplicable al científico de datos si su rol es asesoría, no ejecución.
- **Slicing Pie (equity dinámica por contribución):** convierte tiempo/dinero/ventas en "slices" proporcionales — útil para el side-project actual con el Jr antes de que exista vehículo legal formal, siempre con un techo agregado que protege el control del propietario.
- **Comisión de afiliados SaaS/fintech:** referencia de mercado 15%–30% (mediana ~20–23%), fintech más conservador (10–20%) por carga regulatoria — ancla el caso de los vendedores.

Estos datos alimentan los rangos ilustrativos del documento; el documento mismo dejará explícito que son puntos de partida, no asesoría legal cerrada, y que el caso de fondeo/copy-trading toca terreno regulado (asesoría de inversión no registrada) que debe pasar por el skill `general-counsel` antes de operar con dinero real de terceros.

## Segunda opinión (skill `partner`, rebote de diseño)

Se invocó al skill `partner` para auditar el diseño antes de cerrar el plan. Hallazgos incorporados abajo (marcados `[partner]` en cada sección afectada):
- Faltaba cláusula de no cesión/no transferencia, confidencialidad/IP y árbitro de la fórmula de recompra en las salvaguardas.
- El científico de datos quedaba con dos mecanismos diferidos (suena a vaporware para un perfil senior); el Jr no tenía umbral objetivo de "graduación"; la hermana se confundía con el mecanismo de la mamá pese a pedir mínimo esfuerzo; la novia entraba en instrumentos con sabor a equity sin advertir el riesgo de mezclar dinero con la relación de pareja (régimen de bienes conyugales).
- Faltaba techo numérico agregado real (no solo cualitativo) y un ledger tipo cap-table informal llevado por el propietario desde HOY, no solo cuando exista SAPI; faltaba cláusula anti-coalición y cláusula de no-punto-único-de-fallo.
- Modelo disruptivo nuevo a sumar al catálogo: **royalty por estrategia aportada** (no equity) — pago de % del fee de performance que genere una estrategia específica del científico de datos mientras esté viva, desacoplado del cap table.
- La secuencia de próximos pasos necesitaba un paso 0 antes de `general-counsel`: el propietario define él mismo los números duros por escrito (el abogado valida legalidad, no decide montos); y paralelizar con un contador para ISR/IVA de comisiones y profit-share (fuera del alcance de `general-counsel`).

## Estructura del documento

Tono y formato: prosa clara en español, tablas donde ayuden a comparar, sin pseudocódigo ni jerga sin explicar — mismo registro que `docs/PRODUCT.md` (líneas 1-45 ya leídas como referencia de estilo), pero sin el aparato `CAP-NN` (no aplica aquí). Sin cabecera de Slug ID/Tamaño — esa disciplina es para documentos normativos de `.agents/` que un agente ejecuta; este es un documento humano de estrategia, mismo régimen que `PRODUCT.md`.

**0. Principios rectores (no negociables)**
Control máximo del propietario siempre — nadie iguala o supera su % de equity ni su voto; secretos (credenciales, IPs live) nunca se comparten con nadie fuera del propietario, alineado con `ADR-0093`/`ADR-0143`; toda relación empieza reversible y de bajo compromiso, y solo escala a compromiso mayor con evidencia de aporte sostenido; la amistad es un activo a proteger — cláusulas de salida siempre definidas por adelantado, nunca improvisadas en el momento de la ruptura.

**1. El menú de mecanismos** (el catálogo reutilizable, cada uno con: qué es, cuándo usarlo, rango ilustrativo, riesgo legal a vigilar, cómo se sale sin fricción)
1.1 Comisión pura por adquisición de cliente (afiliado/vendedor) — sin equity, rango 15-30% ilustrativo sobre lo que trae, salida inmediata.
1.2 Cuenta fondeada estilo prop-firm (usuario que solo opera) — capital del propietario o de una fondeadora externa real, split de ganancias ilustrativo 70/30 a 90/10 a favor del operador, protegido por los límites de riesgo ya existentes en el producto (`copy-trading-engine`, `verified-account-registry`, auto-auditoría de portafolios vivos); nota expresa de que fondeo de terceros con dinero real cae en terreno regulado y requiere paso por `general-counsel`.
1.3 Socio-empleado sin sueldo con profit-share contractual (honorarios variables) — para quien trabaja regular pero el proyecto aún no factura; advertencia expresa de riesgo de simulación laboral y cómo evitarlo (sin horario fijo, sin subordinación, entregables por objetivo).
1.4 Equity dinámica por contribución (tipo Slicing Pie) — para la fase actual de side-project sin vehículo legal formal, con techo agregado que nunca acerca a nadie al % del propietario.
1.5 Phantom equity / bono atado a valuación — la vía "limpia" para dar upside real sin ceder acciones ni exponerse a la restricción de la LGSM sobre aportación de trabajo.
1.6 Equity real diferida vía pool de opciones (cuando exista SAPI formal) — vesting ilustrativo 4 años/cliff 1 año, precio de ejercicio nominal para sortear la prohibición de aportación de trabajo.
1.7 Advisor equity grid (tipo FAST) — para experticia puntual de alto valor sin ejecución diaria, rango ilustrativo 0.15%-1.00%, vesting corto.
1.8 `[partner]` Royalty por estrategia/modelo aportado — no es equity: se paga un % del fee de performance que genere una estrategia o modelo específico del socio mientras esté viva en producción. Desacopla el incentivo del cap table por completo, es medible por resultado real y encaja mejor que un grid de asesor para un perfil quant que sí construye artefactos concretos.

**2. Salvaguardas transversales de control y salida**
Techo duro de equity total repartible, llevado como un **ledger informal tipo cap-table desde hoy** `[partner]` — no se pospone hasta que exista SAPI formal; sin voto o voto no decisorio para los socios minoritarios; cláusula de recompra a fórmula (no a negociación ad-hoc) al momento de salida, con el propietario como **árbitro designado de la valuación** en caso de desacuerdo sobre el aporte `[partner]`; período de prueba antes de cualquier compromiso mayor; plantilla de conversación de salida ("cierre de cuentas, no cierre de relación"); **cláusula de no cesión/transferencia** — ningún instrumento (equity dinámica, phantom equity, profit-share) es transferible a un tercero sin consentimiento del propietario `[partner]`; **cláusula de confidencialidad y propiedad intelectual** explícita para cualquiera que toque lógica/algoritmos (Jr, científico de datos) `[partner]`; **cláusula anti-coalición** — los minoritarios no pueden actuar en bloque ni agregar peso cediéndose instrumentos entre sí `[partner]`; **cláusula de no-punto-único-de-fallo** — todo rol operativamente crítico (ej. el científico de datos) lleva documentación/transferencia de conocimiento obligatoria, para que nadie se vuelva indispensable ni gane apalancamiento informal sobre el propietario `[partner]`.

**3. Casos de uso, organizados por tipo de aporte — NUNCA por perfil de persona** `[corrección del propietario, rechazó el primer intento]`

Regla rectora de esta sección: ningún caso está reservado a una profesión, parentesco o cercanía. Lo que determina el mecanismo es el **tipo de aporte real** que la persona trae hoy, no su título. La misma persona puede calificar para un caso genérico en un momento de su vida y para uno particular más adelante (o al revés): el científico de datos puede querer entrar solo como explorador sin compromiso (3.1), y la mamá o el Jr podrían con el tiempo aportar una estrategia propia y calificar para 3.2. El documento aplica aquí el mismo principio que `docs/PRODUCT.md` usa para no bifurcar por gremio: **etiqueta por conducta/aporte, no por oficio**.

**3.1 Casos genéricos** (abiertos a cualquier persona del círculo, sin curva técnica exigida — el mecanismo depende de lo que la persona *hace*, no de quién es)
- Solo usa la plataforma con capital fondeado, como explorador o copier pasivo → mecanismo 1.2. Sin exigencia de conocimiento técnico.
- Solo trae clientes / hace networking → mecanismo 1.1. Sin exigencia de involucramiento operativo.
- Aporta tiempo operativo regular sin sueldo (soporte, contenido, atención a usuarios, código, lo que sea sostenido) → mecanismo 1.3 + 1.4, con umbral objetivo explícito de "graduación" a sueldo/equity mayor (monto de facturación o fecha, nunca a discreción del momento) `[partner]`.

**3.2 Casos particulares de involucramiento profundo** (definidos por el tipo de aporte específico, abiertos a cualquiera que efectivamente lo traiga — no reservados a un perfil técnico)
- Aporta una estrategia, modelo o pieza de investigación cuantitativa propia y verificable → 1.5 (phantom equity, ya operante hoy) + 1.8 (royalty por estrategia aportada, medible desde el primer resultado); 1.6/1.7 (equity real diferida / advisor grid) como destino a futuro cuando exista SAPI, nunca como punto de entrada — prometer solo mecanismos diferidos a una entidad legal que no existe todavía suena a promesa vacía.
- Aporta experticia puntual de asesoría de alto valor sin ejecución diaria → 1.7.
- Aporta compromiso operativo crítico y sostenido de largo plazo, con trayectoria demostrada → 1.6, el techo superior del catálogo, solo alcanzable por trayectoria, nunca de entrada.

**3.3 Notas de riesgo transversal por tipo de vínculo, no por caso de uso** (aplican sobre cualquier caso de 3.1/3.2, según quién sea la persona, sin dictar qué mecanismo le toca)
- Vínculo de pareja: advertir siempre el riesgo de régimen de bienes conyugales si la relación escala — evitar instrumentos con sabor a equity (1.4/1.5/1.6) mientras el vínculo sea de pareja, sin importar qué tan involucrada quiera estar.
- Cualquier vínculo con dependencia económica real (alguien con problemas económicos): priorizar mecanismos que no dependan de curva de aprendizaje ni de resultados inciertos, y dejar la puerta abierta a que migre a un caso de 3.2 si con el tiempo aporta algo más.
- Cualquiera que toque lógica/algoritmos (sin importar el caso): cláusula de confidencialidad y propiedad intelectual obligatoria (ver §2).

**4. Matriz resumen** (tabla: caso de uso [3.1/3.2] → mecanismo(s) → rango ilustrativo → riesgo legal/relacional a vigilar → cómo se sale — sin columna de "persona", la matriz es de casos, no de personas)

**5. Próximos pasos recomendados**
Ningún acuerdo de esta lista se firma tal cual — es el menú de opciones, no el contrato. Antes de mover dinero o equity real con cualquier persona: (0) `[partner]` **el propietario define él mismo, por escrito, los números duros** — % exacto por caso de uso (3.1/3.2) y el techo agregado — porque un abogado valida legalidad, no decide montos; ese ledger de cap-table informal es la verdadera línea de defensa, no el catálogo; (a) pasar el caso concreto por el skill `general-counsel` para blindaje fiscal/laboral/regulatorio específico de México; (b) `[partner]` **en paralelo**, no después, validar con un contador el tratamiento de ISR/IVA de comisiones y profit-share — fuera del alcance de `general-counsel`; (c) si el caso toca equity real o fondeo de terceros, considerar activar el skill `partner` para la mecánica de cap table / decisiones estratégicas del vehículo legal cuando exista.

## Cambios de archivos a ejecutar (tras aprobación, fuera de plan mode)

1. **Crear** `docs/business/PARTNERSHIP-PROGRAM.md` con la estructura de arriba.
2. **Ampliar el Gate de Creación de Documentos** en `.agents/skills/architect/SKILL.md` (sección "🚫 RESTRICCIONES DOCUMENTALES" → "Flujo Permitido"): añadir `docs/business/*.md` como categoría permitida (estrategia societaria/negocio, fuera del pipeline técnico ADR/SAD/Feature). Edición quirúrgica de una línea, no reescritura.
3. **Enlazar** el nuevo documento desde `docs/README.md` — una entrada nueva junto a `PRODUCT.md`/`ROADMAP.md` en el índice de navegación, con una línea que aclare que es estrategia de negocio, no arquitectura.
4. **No toca** ADRs, SAD, features, ni ROADMAP — este documento no es una decisión arquitectónica ni cambia lo que el producto promete al usuario final; no dispara el Protocolo de Definición de Producto (`CAP-NN`) porque no crea ni reasigna una Feature.

## Verificación

- Revisión de lectura: el documento se puede leer de corrido en menos de 15 minutos; cada caso de uso de §3.1/§3.2 tiene su fila en la matriz resumen §4, y ninguna fila ni ningún caso menciona una profesión, parentesco o perfil como condición de entrada — solo el tipo de aporte.
- Chequeo cruzado: ningún mecanismo del §1 permite que un tercero iguale o supere el % de equity/voto del propietario (principio §0 verificable línea por línea).
- Chequeo de no-perfilado: releer §3 buscando cualquier frase que ate un mecanismo a una profesión/parentesco específico en vez de a un tipo de aporte — si aparece, es un defecto de diseño que hay que corregir antes de publicar.
- Chequeo de coherencia con producto real: los casos que usan cuenta fondeada/copy-trading citan las features reales (`copy-trading-engine`, `verified-account-registry`) en vez de inventar mecánica nueva.
- `grep -rn "docs/business" docs/README.md` confirma el enlace nuevo; `grep -n "docs/business" .agents/skills/architect/SKILL.md` confirma la ampliación del Gate.
