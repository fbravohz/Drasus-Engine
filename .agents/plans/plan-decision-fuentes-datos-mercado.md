# Plan de decisión — Fuentes de datos de mercado (TASK-052)

> ⛔ **PARCIALMENTE OBSOLETO — 2026-07-18, por la doctrina del propietario canonizada en [ADR-0154](../../docs/adr/ADR-0154.md).**
>
> **Muere toda la mitad de "elegir proveedor para Drasus":** §2 Q1 y Q2 (la pregunta rectora ya no existe — no hay fuente masiva de Drasus), §3.1 y §3.3 como catálogo de adopción, §6.1 (Fase A: Dukascopy vs. HistData), §6.2 (Fase B: TrueFX) y la decisión #8 de §8. Drasus **no provee datos de mercado**: no adopta proveedor, no compra feed y no aporta lago. El usuario trae los suyos desde sus propios brókers.
>
> **Sobrevive, y sube de rango:** §3.2 (vía MT5 → **vía 3 canónica** de ADR-0154, ya no un extra de Fase C), §2 Q3 (compra/venta separadas → generalizado a "se captura todo lo que la fuente entregue"; el esquema concreto lo produce el Quant-Engineer), §2 Q4 (el canal no siempre es HTTP), §2 Q5 (derivar el minuto del tick), §2 Q7 (linaje por proveedor y método → obligatorio), §4 (riesgo legal, con el agravante nuevo del tier gratuito de ADR-0143), §5 (encaje con el motor ya construido) y §7 (las siete comprobaciones críticas).
>
> **Cambia de sentido:** §6.4 (futuros) — ya no se elige proveedor; se construye el conector para el feed que el usuario contrate. Databento y CQG/Rithmic/NinjaTrader pasan a ser filas de la tabla de capacidades, no candidatos a adoptar.
>
> **Estado:** 🟡 Borrador para aprobación del propietario.
> **Autor:** Architect · **Fecha:** 2026-07-18 · **Entrada:** [`docs/execution/TASK-052-architect-escalation-market-data-sources.md`](../../docs/execution/TASK-052-architect-escalation-market-data-sources.md)
> **Este documento NO sella ningún proveedor.** No se ha editado ningún ADR, feature ni el ROADMAP. Al aprobarse, el Architect ejecuta el pipeline documental (enmienda a ADR-0034 + features + ROADMAP).

**[knowledge consultado: `base.md` — gobernanza obligatoria de arranque]**
**[knowledge consultado: `critical-domain-reasoning.md` — la elección de fuente y granularidad es decisión de dominio cuantitativo (fricción, spread, microestructura); las 7 comprobaciones están en §7]**

---

## 0. Resumen para el propietario (nivel decisión)

**Lo que hay que decidir:** de dónde saca Drasus los años de histórico de precios que necesita para que un backtest signifique algo.

**Los tres hallazgos que cambian el planteamiento:**

- **Tus cuatro brókers no sirven como fuente de histórico masivo, y está verificado.** IC Markets declara que no guarda tick data y su histórico en cuenta real arranca en 2021 (con reportes de recorte a 2024 en adelante). Pepperstone cerró su descarga propia y hoy remite a un tercero (TrueFX). De Swissquote no hay volcado masivo público. La única excepción real es Darwinex, que sí publica tick con compra y venta desde octubre de 2017, pero exige cuenta real y descarga por FTP.
- **La premisa del paquete de escalamiento es correcta**, y la refuerzo: la fuente masiva tiene que ser un proveedor de referencia. Forzar "el dato del bróker desde el inicio" hace que la fase nunca cierre (demostración en §7.1).
- **Hay una decisión gratis hoy que es cara mañana:** el tipo `Tick` del código todavía está vacío, sin campos. Definirlo ahora con precio de compra y venta separados no cuesta nada; añadirlo después obliga a rebajar todo el lago de datos y todos los backtests hechos hasta entonces.

**Sobre tu aporte de MT5 (§3.2):** verificado y **es mejor de lo que parece a primera vista.** La exportación del terminal entrega precio de compra y venta reales de tu bróker, gratis y con cualquiera de los cuatro. No sirve como lago —arranca hacia 2015-2016 según el bróker, tiene huecos reconocidos y exige que un humano lo exporte a mano— pero **es la mejor forma de medir cuánto se aparta el proveedor de referencia de tu bróker real**, que es la duda de fondo de todo este plan. Lo incorporé como un tercer canal de ingesta, con rol de calibración.

**Lo que te pido decidir:** siete preguntas en §2, el camino de adopción en §6, y una elección entre dos arranques posibles (§6.1) — uno más fiel y más caro de programar, otro más rápido y algo menos fiel.

**Coste de la ruta recomendada:** cero en dinero para cerrar la fase actual. El único gasto potencial aparece en futuros, y va diferido.

---

## 1. Auditoría bidireccional de las premisas

Contrasté los tres huecos del paquete contra el código, el ROADMAP y ADR-0034, en ambas direcciones — no di por buena la versión del Tech-Lead.

### 1.1. H1 (fuente Forex nunca construida) — CONFIRMADO, pero el defecto no es el que parece

- **Verificado:** `crates/features/data/sovereign-data-fetcher/src/schemas.rs:29` fija `https://data.binance.vision`; `public_interface.rs:338` fija la API de Binance para el delta. No existe adaptador Forex.
- **Contraste contra el ROADMAP:** `docs/ROADMAP.md:165` exige "prioridad de clase de activo: una Forex/CFD primero; una cripto como segunda fuente". El código invirtió el orden.
- **Mi lectura, contraria a la del paquete:** el orden del ROADMAP era una preferencia de producto, no un invariante técnico, y **el código hizo lo correcto**. Binance Vision es HTTP público, sin cuenta, sin credenciales, con archivos `.zip` bien formados — la fuente ideal para estrenar el primer crate hexagonal de dominio y validar el motor sin mezclar dos problemas (arquitectura nueva + fuente hostil). Ninguna fuente Forex gratuita reúne esas cuatro propiedades (§3.1).
- **Consecuencia:** el defecto real es que la fuente Forex **nunca se eligió**, no que se construyera cripto primero. Al aprobarse este plan, la redacción del ROADMAP §EPIC-1 debe corregirse para reflejar "dos clases de activo al cierre", sin imponer un orden que la realidad de los proveedores desaconseja.

### 1.2. H2 (ADR-0034 no cubre Forex ni futuros) — CONFIRMADO, y el ADR está incompleto en el mecanismo, no equivocado en la decisión

- **Verificado en `docs/adr/ADR-0034.md`:** la decisión Bulk + Delta es sólida y sigue siendo correcta. Pero el mecanismo declarado —"descarga concurrente HTTP de archivos `.zip` o `.csv.gz`"— **describe el mundo cripto y no el mundo Forex/futuros**:
  - Dukascopy sirve binario comprimido con LZMA, no `.zip`/`.gz`.
  - Darwinex sirve por **FTP con credenciales**, no HTTP anónimo.
  - Databento sirve un binario propietario (DBN) vía API de lotes.
- **Veredicto:** ADR-0034 necesita **enmienda**, no reemplazo. El patrón dual sobrevive; lo que cae es la suposición de que el canal masivo es siempre HTTP anónimo de `.zip`.
- **Segundo defecto detectado:** ADR-0034 delega el Delta en "adaptadores de NautilusTrader (Binance, IBKR, Oanda v20)". Oanda e IBKR no están en la lista de brókers del propietario (IC Markets, Darwinex, Pepperstone, Swissquote), y ninguno de esos cuatro tiene adaptador Nautilus verificado. La línea de Delta del ADR está desalineada con el plan de negocio real.

### 1.3. H3 (los brókers no son proveedores de histórico) — CONFIRMADO Y REFORZADO por evidencia externa

| Bróker | ¿Publica volcado histórico masivo? | Evidencia |
|---|---|---|
| **IC Markets** | **No.** Declara no almacenar tick data y remite a terceros. Histórico en cuenta real desde 2021-06-29 en EURUSD; reportes de recorte a 2024+ | Foros MQL5 (verificado como reporte de usuarios, no como declaración oficial) |
| **Pepperstone** | **Ya no.** Descontinuó su acceso FTP; hoy **recomienda TrueFX** como fuente de histórico a sus propios clientes | Referencias a `pepperstone.com/en/client-resources/historical-tick-data`; página no accesible en verificación directa (404) — **parcialmente verificado** |
| **Swissquote** | **No verificado.** Ofrece API REST/FIX institucional para datos de mercado; no hay evidencia de volcado masivo descargable | swissquote.com/institutional — sin volcado documentado |
| **Darwinex** | **Sí — única excepción.** Tick con archivos separados de compra y venta, desde octubre 2017, por FTP, para clientes con cuenta real | darwinex.com/tick-data + API comunitaria `darwinex_ticks` |

**Refuerzo de la premisa:** el caso Pepperstone es la prueba más fuerte. Un bróker de ejecución que remite a sus clientes a un proveedor externo para obtener histórico **confirma que las dos funciones son distintas y que la propia industria las separa**.

**Matiz que el paquete no capturó:** Darwinex sí califica técnicamente como fuente masiva (8+ años de profundidad con compra/venta, cumple el criterio de salida de 5 años). Su barrera no es técnica sino comercial (cuenta real financiada) y de canal (FTP autenticado = adaptador nuevo + credenciales, Grupo II de ADR-0020, hoy omitido a propósito en la feature).

---

## 2. Las siete preguntas de diseño

Cada pregunta lleva **recomendación**, **qué se gana**, **qué se pierde**, y **el camino contrario con su propio balance**. Ninguna está sellada.

### Q1 — ¿La fuente masiva es un proveedor de referencia o el bróker de ejecución? (rectora)

**Recomendación: proveedor de referencia.** La fidelidad al bróker concreto se calibra después, por fricción institucional (ADR-0017/ADR-0069), alimentada por el conector de bróker de EPIC-5.

- **Se gana:** la fase cierra con datos disponibles hoy y gratis; una sola serie por símbolo en el lago; el motor se valida contra un dato estable y reproducible; cambiar de bróker no invalida el histórico ni los backtests ya corridos.
- **Se pierde:** el spread simulado es el del proveedor, no el de IC Markets. Todo backtest arrastra un error de fricción hasta que la calibración de EPIC-5 lo corrija. Ese error hay que declararlo en la ficha de cada resultado, no esconderlo.

**Camino contrario — exigir el dato del bróker desde ya:**
- **Se gana:** fidelidad de spread real desde el primer backtest; ninguna calibración posterior.
- **Se pierde:** la fase no puede cerrar (§7.1 lo demuestra formalmente); el lago se multiplica por número de brókers; cambiar de bróker obliga a re-descargar y re-correr todo; solo Darwinex tiene profundidad suficiente, lo que equivale a elegir el bróker por su dato en vez de por sus condiciones de trading.

**Camino intermedio, y es el que recomiendo de verdad:** proveedor de referencia como lago + **datos del bróker real como fuente de calibración** del spread. Convierte la fidelidad en un problema medible en vez de una suposición. Hay dos maneras de obtener ese dato de calibración, y **la segunda la aportó el propietario y es mejor**:

- **Darwinex por FTP** — único de los cuatro brókers que publica descarga propia, pero exige cuenta real allí.
- **Exportación desde el terminal MT5** (§3.2) — ✅ verificado que entrega CSV con compra, venta y volumen desde el servidor del bróker. **Funciona con cualquiera de los cuatro brókers, es gratis y no obliga a elegir bróker por su dato.** Es la vía superior para calibración.

### Q2 — ¿Un proveedor por clase de activo o uno multiactivo?

**Recomendación: uno por clase de activo.** Cripto → Binance Vision (ya construido). Forex/CFD → proveedor de referencia (§6.1). Futuros → proveedor especializado con libro de órdenes (§3.3).

- **Se gana:** cada clase usa la mejor fuente gratuita disponible; ningún proveedor multiactivo verificado cubre bien las tres; el fallo de un proveedor no arrastra a las demás clases.
- **Se pierde:** tres adaptadores en vez de uno; tres formatos que normalizar (TTR-008); tres relaciones legales que auditar en vez de una.

**Camino contrario — un multiactivo (Databento es el único candidato serio):**
- **Se gana:** un adaptador, un formato, un contrato legal, un modelo de linaje.
- **Se pierde:** deja de ser gratis (§3.3), y **no verifiqué que Databento cubra Forex spot** — para el par que bloquea la fase, el candidato multiactivo puede simplemente no servir. Descartarlo hoy sería precipitado; adoptarlo hoy sería asumir cobertura sin verificar.

### Q3 — ¿El tick de Forex debe traer compra y venta, o basta el último precio?

**Recomendación: compra y venta separadas, obligatorio, y decidido AHORA.** Es la pregunta más urgente de las siete.

- **Por qué es urgente:** verifiqué que `crates/shared/src/types/mod.rs:51` define `pub struct Tick;` — un marcador **sin ningún campo**. El esquema físico del tick todavía no existe. Definirlo hoy con compra/venta cuesta cero. Añadirlo después obliga a re-descargar el lago completo e invalida todo backtest anterior, porque el spread deja de ser observado y pasa a ser inventado.
- **Se gana:** el spread es un dato observado, no un parámetro; ADR-0017 (fricción institucional) tiene con qué trabajar; queda abierta la puerta al Level 1 de futuros sin cambio de esquema.
- **Se pierde:** aproximadamente el doble de volumen almacenado por tick; se excluyen las fuentes que solo dan último precio (relevante en futuros, ver §3.3).

**Camino contrario — solo último precio:**
- **Se gana:** la mitad del almacenamiento; más proveedores elegibles, incluidos algunos gratuitos de futuros.
- **Se pierde:** el spread pasa a ser una suposición configurable. En Forex, donde el spread ES el coste principal de operar, eso convierte el backtest en ficción — exactamente lo que EPIC-1 existe para evitar.

**Forma concreta propuesta** (respetando ADR-0141: precios en `INTEGER` escala ×10⁸, prohibido `REAL`): marca de tiempo, precio de compra, precio de venta, volumen de compra y volumen de venta **nulables** (Dukascopy los trae; HistData los deja en cero; Binance no aplica). Es Inundación de Fundaciones (ADR-0020 §7.3): ante duda, incluir.

> ⚠️ **Gate de especialista:** esta decisión pertenece al Quant-Engineer. Debe firmarla antes de canonizarse en un ADR. El Architect es generalista aquí (`critical-domain-reasoning` §2).

### Q4 — ¿Qué formato de volcado soporta cada candidato?

**Recomendación: ampliar el contrato de `BulkSource` para admitir canal autenticado y descompresión conectable, sin tocar la lógica del orquestador.**

Verifiqué el contrato real en `public_interface.rs:77-99`: `BulkSource` expone `list_inventory(range)` y `download_file(file, dest_path)`; `DeltaSource` expone `fetch_range(range) -> Vec<u8>`. **El puerto es más agnóstico de lo que sugiere el paquete de escalamiento** — no menciona HTTP en su firma. La suposición de HTTP vive solo en el adaptador `ReqwestBulkSource` (`orchestrator.rs:347-390`), que construye con una `base_url` y hace `GET` sobre `file.download_url`.

Consecuencia práctica, por candidato, en §5. Resumen: lo que rompe el modelo no es el formato del archivo sino **el canal** (FTP y API de lotes no son `GET` sobre una URL) y **la descompresión** (LZMA crudo no es `.zip`).

- **Se gana con ampliar:** las tres clases de activo entran por el mismo puerto; el orquestador, el deduplicador y la lógica de frontera Bulk/Delta ya probados con QA APTO no se tocan.
- **Se pierde:** el puerto deja de ser trivial; cada adaptador nuevo es una Story con sus propias pruebas.

**Camino contrario — exigir que todo candidato sea HTTP de archivos comprimidos:**
- **Se gana:** cero trabajo de adaptador; el candidato entra por configuración.
- **Se pierde:** elimina a Darwinex (FTP) y a Databento (API de lotes), es decir, la única fuente de bróker propio y el único candidato de futuros con libro de órdenes. **Dejar que el formato del adaptador dicte la estrategia de datos es la cola moviendo al perro.**

### Q5 — ¿El 1 minuto se descarga o se deriva del tick?

**Recomendación: derivar del tick como fuente única de verdad; descargar el 1 minuto solo como control de reconciliación donde sea gratis.**

- **Se gana:** una sola serie canónica por símbolo; imposible que dos series del mismo símbolo se contradigan; el transformador (STORY-051) es el único punto de agregación, y su corrección se prueba una vez; los métodos de agregación (barras de tiempo, de volumen, de dólares) salen gratis del mismo tick.
- **Se pierde:** obliga a almacenar y procesar tick para cualquier temporalidad. Es un coste real y grande — años de tick Forex son cientos de gigabytes por par, frente a megabytes en 1 minuto. Y el 1 minuto derivado **no coincidirá exactamente** con el 1 minuto publicado por el proveedor (marcas de tiempo de frontera, ticks descartados), lo que genera preguntas incómodas al comparar con plataformas externas.

**Camino contrario — descargar ambas:**
- **Se gana:** el 1 minuto está disponible al instante y con poco disco para investigación rápida; comparabilidad directa con MT5/otras plataformas.
- **Se pierde:** dos series que pueden discrepar sin que el sistema sepa cuál es la buena; se duplica la superficie de ingesta y de linaje.

**Matiz de dominio que el paquete no capturó:** en HistData el 1 minuto es **OHLC de precio de compra únicamente**, mientras que su tick trae compra y venta. Derivar el 1 minuto del tick de compra reproduce esa convención; descargarlo mezcla dos convenciones distintas bajo el mismo nombre. Es un argumento fuerte a favor de derivar.

> ⚠️ **Gate de especialista:** la política de agregación (cierre de vela, huecos, sesión, tratamiento del fin de semana) es del Quant-Engineer, no del Architect.

### Q6 — ¿Qué proveedor de futuros preserva la opción de libro de órdenes?

**Recomendación: Databento (conjunto CME Globex MDP 3.0), aun siendo de pago, y con la adopción diferida.** Es el único candidato verificado que sirve el mismo instrumento en L1, L2 y libro completo por orden desde un solo contrato, sin cambiar de proveedor ni de ingesta.

Detalle por proveedor en §3.4 con la columna explícita de "¿cierra la puerta a Level 2?".

- **Se gana:** elegir hoy no obliga a migrar cuando llegue order flow; hay 125 dólares de crédito gratuito para evaluarlo sin comprometerse; el histórico de CME arranca en 2010-06-06 (verificado).
- **Se pierde:** es el único punto de gasto real del plan; el pago por gigabyte descomprimido se vuelve caro rápido en libro completo de órdenes.

**Camino contrario — un proveedor de futuros barato y solo de barras:**
- **Se gana:** coste mucho menor para llegar al primer backtest de futuros.
- **Se pierde:** cuando llegue order flow, hay que cambiar de proveedor, reescribir el adaptador, re-descargar y romper la comparabilidad del histórico. **Es exactamente el escenario que el propietario pidió evitar.**

### Q7 — ¿Cómo se declara la procedencia del dato?

**Recomendación: hacer explícito el proveedor y el método de obtención en el identificador de linaje.**

Verifiqué que `docs/data-model/sovereign_download_records.md:18` ya define `data_snapshot_id` con formato `<exchange>_<symbol>_<timeframe>_<yearmonth>` y `source_endpoint` con la URL exacta. **La base existe, pero tiene dos defectos para el mundo multi-proveedor:**

1. **Confunde mercado con proveedor.** En cripto coinciden (Binance es ambos). En Forex no existe "el exchange": HistData, TrueFX y Dukascopy pueden entregar EURUSD y **no son intercambiables** — proceden de pozos de liquidez distintos, con spreads distintos. Un identificador que dice "EURUSD" sin decir de quién viene permite mezclar en silencio dos series incompatibles.
2. **No distingue descargado de derivado.** Con la recomendación de Q5, un 1 minuto puede ser descargado o computado a partir del tick. Son datos distintos con propiedades distintas y hoy serían indistinguibles.

- **Se gana:** dos particiones del mismo símbolo nunca se mezclan; un resultado de backtest es reproducible sabiendo exactamente de qué proveedor salió; el índice anonimizado del cimiento #9 puede segmentar por proveedor, lo que además **es una necesidad legal** (§4: no todos los proveedores permiten lo mismo).
- **Se pierde:** el formato del identificador cambia, y la restricción `CHECK GLOB` de la migración se toca. En fase greenfield esto es barato (`CLAUDE.md` §1: el baseline es editable in situ); en producción distribuida no lo sería. **Es un argumento para hacerlo ahora.**

**Camino contrario — dejarlo como está y confiar en `source_endpoint`:**
- **Se gana:** cero cambio de esquema.
- **Se pierde:** el proveedor queda en un campo libre y opcional, no en la clave; nada impide físicamente mezclar series. La integridad depende de la disciplina del que escribe, que es exactamente lo que ADR-0141 existe para evitar.

---

## 3. Tabla comparativa de proveedores candidatos

**Convención de lectura:** ✅ verificado en fuente primaria o secundaria consistente · ⚠️ **sin verificar** — no lo confirmé, no debe darse por bueno · ❌ verificado como ausente.

**Regla que apliqué:** donde no pude verificar, escribí "sin verificar". No hay una sola característica, precio ni término de licencia inferido en esta tabla.

### 3.1. Forex / CFD — la clase que bloquea el cierre de la fase

| Proveedor | Clases cubiertas | **Tick** | **1 minuto** | Profundidad real | ¿Compra/venta en el tick? | Formato y canal | Costo | Uso / redistribución |
|---|---|---|---|---|---|---|---|---|
| **Dukascopy** (feed público) | Forex, metales, energía, agrícolas, índices, bonos, CFDs, acciones, cripto (✅ 1000+ instrumentos) | ✅ Sí, tick a tick | ✅ Derivable; export propio ofrece de tick a mensual | ⚠️ **Sin verificar.** La herramienta comunitaria declara "1990-2000 a hoy"; la página oficial no publica rangos. Tratar como *pendiente de medición empírica por instrumento* | ✅ **Sí — compra y venta**, con volúmenes | ⚠️ Export oficial: `.csv` por web. Feed masivo: binario `.bi5` comprimido con LZMA sobre HTTP, **estructura no documentada oficialmente** (ingeniería inversa de la comunidad) | ✅ Gratis | 🔴 ⚠️ **Sin publicar.** La página de export no enuncia términos de licencia ni redistribución → **gate obligatorio de General Counsel** |
| **HistData.com** | 66 pares FX + algunos índices y petróleo (✅) | ✅ Sí — ASCII genérico con precisión de milisegundos | ✅ Sí — barras M1 | ⚠️ **Sin verificar.** Organizado por par/año/mes; el FAQ no declara el año inicial | ✅ **Sí** — formato `Fecha,Compra,Venta,Volumen`. ⚠️ Ojo: el **M1 es OHLC solo de compra** | ✅ `.zip` por par/año/mes vía web, FTP/SFTP o Google Drive. ⚠️ La descarga web usa formulario, no URL directa — **sin verificar** si hay ruta directa | ✅ Gratis | 🔴 ⚠️ **Ambiguo y por eso peligroso.** El FAQ solo dice: *"al ser datos gratuitos no recibes garantía ni certificación; úsalos a tu voluntad y riesgo"*. **Ausencia de permiso ≠ permiso** → gate de General Counsel |
| **TrueFX** (Integral) | ✅ 15 pares mayores | ✅ Sí, tick a tick con milisegundos | ⚠️ Derivable; descarga propia sin verificar | ✅ Desde 2009 | ✅ **Sí** — `Par,Marca de tiempo,Compra,Venta` sin encabezado | ✅ `.csv` en `.zip`, diario o mensual. ⚠️ **Requiere cuenta gratuita** → introduce credenciales | ✅ Gratis con registro | 🟡 ⚠️ **Sin verificar.** Ventaja institucional real: precios de Integral OCX (250+ bancos y brókers) y **es la fuente que Pepperstone recomienda a sus propios clientes** |
| **Darwinex** | ✅ Todo su catálogo | ✅ Sí | ⚠️ Derivable | ✅ Desde octubre 2017 (~8,8 años a hoy — **sí cumple el criterio de 5+ años**) | ✅ **Sí**, en archivos separados de compra y venta | 🔴 **FTP con credenciales** → adaptador nuevo | 🟡 Gratis **pero exige cuenta real de trading** | 🟡 ⚠️ **Sin verificar.** Al ser datos de cliente, probablemente sujetos al contrato de cuenta → gate de General Counsel |
| **Polygon.io** | FX (~1000 pares), cripto, acciones | 🟡 Sí en planes de pago | ✅ Agregados | 🟡 2 años en el plan gratuito | ✅ Sí (mejor oferta y demanda) | 🔴 **API REST, no volcado masivo** — 5 llamadas/minuto en el plan gratuito. **Incompatible con el patrón Bulk de ADR-0034** | 🟡 Gratuito limitado; precio de pago ⚠️ sin verificar | ⚠️ Sin verificar |
| **Exportación manual desde MT5** *(aporte del propietario, 2026-07-18)* | ✅ Lo que cotice el bróker: FX, CFDs, índices, materias primas | ✅ **Sí** — pestaña de Ticks, botón de exportar | ✅ Sí — exportación de barras | 🟡 ✅ Verificado que **depende del bróker y suele arrancar hacia 2015-2016**; con huecos reconocidos. En IC Markets, cuenta real desde 2021 (§1.3) | ✅ **Sí — el CSV exportado trae Compra, Venta y Volumen** | 🔴 **`.csv` en disco local.** No hay descarga de red: el terminal sincroniza contra el servidor del bróker y guarda archivos `.tkc` en `bases/<servidor>/ticks/<símbolo>/` | ✅ Gratis (requiere cuenta y el terminal) | 🟡 ⚠️ Sin verificar — sujeto al contrato de cuenta con el bróker. Ver §3.4 |
| **IC Markets / Pepperstone / Swissquote** *(descarga masiva propia)* | — | ❌ | ❌ | ❌ | — | — | — | **Descartados como fuente masiva de red** (§1.3). Sí alcanzables vía MT5 → fila anterior |

### 3.2. La vía MT5: por qué no es "un proveedor más" sino un tercer canal

El propietario aportó esta vía durante la elaboración del plan. **Es válida, la verifiqué, y resuelve un problema que ninguno de los proveedores anteriores resuelve** — pero no encaja en el modelo de ADR-0034, y esa es justamente su importancia.

**Lo verificado:**
- El terminal exporta ticks a CSV con **columnas de Compra, Venta y Volumen** (Símbolos → pestaña Ticks → Solicitar → Exportar). Cumple el requisito de Q3 sin concesiones.
- El terminal descarga del servidor del bróker bajo demanda y almacena archivos mensuales `.tkc` en `bases/<servidor>/ticks/<símbolo>/`.
- **No hay diferencia entre cuenta demo y real en el mismo bróker**, aunque sí entre brókers distintos: unos guardan cada cambio de precio y otros filtran.
- Profundidad típica desde ~2015-2016 según el bróker, **con huecos reconocidos**. El indicador de "calidad de histórico" de MT5 **no mide calidad**: mide qué porcentaje del periodo pudo reproducirse con ticks reales, y rellena el resto con ticks generados.

**Por qué importa arquitectónicamente — y es el hallazgo que cambia el planteamiento de Q1:**

Esta vía **no es una `BulkSource`**. No hay nada que descargar por red desde Drasus: el terminal ya hizo la descarga y el resultado es un archivo en el disco del usuario. Necesita un **tercer canal**, junto a Bulk y Delta, que podría llamarse importación local. Su forma es más simple que las otras dos (leer, validar, normalizar, particionar; sin inventario, sin concurrencia, sin frontera de unión).

Y hace algo que ningún proveedor de la tabla hace: **entrega el precio del bróker de ejecución real, con compra y venta, gratis y sin depender de que ese bróker publique nada.** Convierte el camino intermedio de Q1 —que planteé como hipótesis— en una opción concreta y barata.

**Lo que gana el plan si se adopta:**
- La calibración de fricción de la Fase C deja de depender de abrir cuenta en Darwinex: sirve cualquiera de los cuatro brókers.
- El propietario puede contrastar empíricamente el spread del proveedor de referencia contra el de su bróker real, que es la pregunta de fondo detrás de todo Q1.
- Coste de adaptador bajo: leer CSV local es lo más barato de todo el plan.

**Lo que NO resuelve, y por eso no sustituye a la Fase A:**
- **Profundidad insuficiente y desigual.** ~2015-2016 en el mejor caso, 2021 o 2024 en IC Markets. El criterio de salida de EPIC-1 pide 5+ años fiables.
- **Huecos reconocidos por la propia herramienta.** Un lago con huecos silenciosos es peor que uno corto y honesto — el validador punto-en-el-tiempo no puede distinguir un hueco de un mercado cerrado sin metadatos.
- **No es automatizable de forma soberana.** Exige que un humano abra el terminal y exporte. Choca con "un comando CLI descarga, sanitiza y particiona", y no se puede reproducir en el equipo de otro usuario.
- **Reproducibilidad comprometida.** Dos usuarios con el mismo bróker pueden exportar series distintas según cuándo sincronizó su terminal. Eso rompe la promesa de backtest reproducible bit a bit de EPIC-2, salvo que el linaje registre el bróker, el servidor y la fecha de exportación (refuerza Q7).

**Recomendación: adoptarla, pero en la Fase C y con un rol distinto — no como lago, sino como referencia de calibración y verificación.** Es el mejor argumento empírico disponible para saber cuánto se aparta el proveedor de referencia del bróker real, que es la incógnita que la Fase A deja abierta por diseño.

**Consecuencia sobre ADR-0034:** la enmienda propuesta en §1.2 crece. Ya no es solo "el canal masivo no siempre es HTTP anónimo"; es que **existe un canal que no es de red en absoluto**. Eso es una ampliación de la decisión, no solo del mecanismo.

### 3.3. Cripto — ya cableado, ¿se conserva?

**Recomendación: sí, se conserva.** Es la única fuente con adaptador construido y QA APTO. Sustituirla sería tirar trabajo validado para resolver un problema que no existe.

| Proveedor | Cobertura | **Tick** | **1 minuto** | Profundidad | ¿Compra/venta? | Formato y canal | Costo | Uso / redistribución |
|---|---|---|---|---|---|---|---|---|
| **Binance Vision** ✅ *(en uso)* | Spot y derivados de Binance | ✅ Operaciones individuales y agregadas | ✅ Velas | ⚠️ Por símbolo, sin verificar; archivos diarios y mensuales | 🔴 **No en el volcado de operaciones** — son ejecuciones, es decir último precio. ⚠️ Sin verificar si el conjunto de mejor oferta/demanda está publicado | ✅ `.zip` sobre HTTP anónimo — **encaje perfecto con el adaptador actual** | ✅ Gratis | 🔴 **Riesgo real.** Los términos de **Binance.US** prohíben expresamente que terceros cobren o lucren con sus datos de mercado, incluida publicidad o comisiones de referido (✅ verificado). Binance global es entidad distinta y sus términos ⚠️ **no los verifiqué**. Drasus es producto **de pago** → **gate de General Counsel, prioridad alta** |
| **Tardis.dev** | 50+ mercados cripto | ✅ Actualizaciones incrementales de libro L2, instantáneas, operaciones | ✅ Derivable | ✅ Desde 2019-03-30 en la mayoría | ✅ Sí, con libro completo | ✅ CSV descargable + API | 🔴 ⚠️ **Precio sin verificar.** Ofrece muestras gratuitas | ⚠️ Sin verificar |

**Nota de riesgo sobre cripto:** si el General Counsel dictamina que los términos de Binance impiden a un producto de pago persistir y agregar sus datos, **el adaptador ya construido queda comprometido**. Esta es la revisión legal más urgente del plan, por encima de las de Forex, porque afecta a código en producción y no a una decisión futura.

### 3.4. Futuros — con la columna que importa

| Proveedor | Cobertura | **Tick** | **1 minuto** | Profundidad | ¿Compra/venta? | **¿Cierra la puerta a Level 2?** | Formato y canal | Costo | Uso / redistribución |
|---|---|---|---|---|---|---|---|---|---|
| **Databento** (CME Globex MDP 3.0) | ✅ CME, ICE, EEX, Eurex — futuros, spreads, opciones | ✅ Operaciones tick a tick | ✅ OHLCV de 1 minuto | ✅ **Desde 2010-06-06** | ✅ Sí (MBP-1 = Level 1) | ✅ **NO la cierra — es el único que la deja abierta de par en par.** Ofrece Level 1, **Level 2 (MBP-10)** y **libro completo por orden (MBO, Level 3)** sobre el mismo conjunto y el mismo contrato | 🟡 API de lotes + binario propietario DBN (también CSV/JSON sin sobrecoste) → **adaptador nuevo** | 🔴 **De pago, con detalle verificado:** $125 de crédito gratuito para usuarios nuevos (caduca a 6 meses, uno por equipo); por uso desde menos de $0,01, facturado **por gigabyte descomprimido**; Standard $179/mes; Plus $1.500/mes (incluye derechos de distribución externa); Ilimitado $4.000/mes con 16+ años de historia | 🟡 **La mayoría de conjuntos son redistribuibles interna y externamente tras 24 horas**; las restricciones vienen del mercado emisor, no de Databento. ⚠️ El caso concreto de CME sin verificar → gate de General Counsel |
| **CME DataMine** | ✅ Fuente oficial CME | ✅ Sí | ✅ Sí | ✅ Profunda (450+ TB) | ✅ Sí | ✅ **No la cierra** — publica conjunto de Market Depth por orden | 🟡 API, SFTP y navegador de archivos → adaptador nuevo | 🔴 ⚠️ **Precio sin verificar** — requiere contacto con ventas. Referencias sugieren paquetes por producto | ⚠️ Sin verificar. Al ser el mercado emisor, sus condiciones son las más restrictivas del mercado → gate de General Counsel |
| **FirstRate Data** | ✅ 130 contratos más activos | 🔴 **En futuros no.** Su tick declarado cubre acciones y cripto (10 años). Futuros son barras de 1 minuto en adelante | ✅ Sí, desde 2007 | ✅ 15+ años intradía | 🔴 **No.** El tick declara precio de operación, volumen y código de mercado — **sin compra/venta** | 🔴 **SÍ la cierra.** Sin compra/venta ni libro: adoptarlo obliga a cambiar de proveedor cuando llegue order flow | ✅ `.csv` en `.zip` por web o API — **encaje bueno con el adaptador actual** | ⚠️ **Sin verificar** — no publica precio en la portada | ⚠️ Sin verificar |
| **Tick Data LLC** | ✅ 150+ futuros globales | ✅ Cotizaciones y operaciones de Level 1, **desde diciembre 1974** | ✅ Derivable | ✅ La más profunda verificada | ✅ Sí (Level 1) | 🟡 **Parcialmente.** Level 1 verificado; ⚠️ Level 2 sin verificar | ⚠️ Sin verificar | 🔴 ⚠️ Sin verificar (perfil institucional) | ⚠️ Sin verificar |
| **Norgate Data** | ~100 mercados, desde ~1980 | ❌ | ❌ | Solo cierre diario | ❌ | 🔴 **SÍ la cierra por completo** | — | Suscripción ⚠️ sin verificar; **el acceso muere si la suscripción caduca** | — |
| | | | | | | **→ DESCARTADO.** ✅ Verificado: **solo fin de día**, sin intradía ni tick. No sirve para el motor de Drasus | | | |
| **Interactive Brokers (API)** | Amplia | ⚠️ Sin verificar | ⚠️ Sin verificar | ⚠️ Sin verificar (conocido por límites estrictos) | ⚠️ Sin verificar | ⚠️ Sin verificar | API, no volcado masivo | Requiere cuenta | ⚠️ Sin verificar |

**Sobre "priorizar lo gratuito" en futuros:** lo agoté y **no encontré ninguna fuente gratuita de futuros CME con tick o libro de órdenes**. La búsqueda de opciones gratuitas devuelve consistentemente proveedores de pago. Es una conclusión negativa honesta, no una omisión: en futuros, gratis y utilizable no coexisten hoy. Por eso la recomendación de futuros va **diferida** en §6 — para no gastar antes de que exista un consumidor real del dato.

---

## 4. Riesgo legal por proveedor y gate del General Counsel

**Por qué Drasus está más expuesto que un trader individual:** el sistema hace dos cosas que muchos términos de servicio prohíben — **persiste** el dato en el equipo del usuario, y **agrega índices anonimizados** en el servidor central (cimiento #9, `data-aggregation`). Un tercer agravante: Drasus **es un producto de pago**, lo que activa las cláusulas de "no lucrar con nuestros datos".

| Proveedor | Riesgo | Motivo | Gate de General Counsel |
|---|---|---|---|
| **Binance Vision** | 🔴 **Alto y ACTIVO** | Términos de Binance.US prohíben cobrar o lucrar con sus datos de mercado (✅ verificado). Entidad global sin verificar. **Es la única fuente ya en producción** | 🔴 **OBLIGATORIO Y URGENTE — antes que cualquier otro** |
| **Dukascopy** | 🔴 Alto | No publica términos de uso del feed. El canal masivo `.bi5` no está documentado oficialmente: usarlo es acceso no publicado, no acceso autorizado | 🔴 **OBLIGATORIO antes de adoptar** |
| **HistData.com** | 🔴 Alto | "Úsalo a tu voluntad y riesgo" no es una licencia. Sin concesión expresa de derechos, la redistribución y la obra derivada quedan sin cobertura | 🔴 **OBLIGATORIO antes de adoptar** |
| **Darwinex** | 🟡 Medio | Datos entregados bajo contrato de cuenta de cliente; el uso probablemente esté acotado a ese cliente | 🟡 Obligatorio si se adopta |
| **TrueFX** | 🟡 Medio | Datos institucionales de Integral bajo registro; términos sin verificar | 🟡 Obligatorio si se adopta |
| **Exportación MT5** | 🟡 Medio | El dato llega bajo el contrato de cuenta con el bróker. Usarlo para calibrar en local es defendible; **alimentar con él el índice anonimizado del cimiento #9 es otra cosa** y es donde está el riesgo real | 🟡 Obligatorio antes de que ese dato toque la agregación |
| **Databento** | 🟢 Bajo, y es su ventaja | Único con política **explícita y publicada**: la mayoría de conjuntos se redistribuyen interna y externamente tras 24 horas; las restricciones son del mercado emisor y se trasladan sin sobrecoste. El plan Plus incluye derechos de distribución externa | 🟡 Obligatorio, pero con documentación real que revisar |
| **CME DataMine** | 🔴 Alto | Los mercados de futuros son los licenciantes más restrictivos del sector | 🔴 Obligatorio |

**Hallazgo de arquitectura derivado del riesgo legal:** si distintos proveedores conceden distintos derechos, entonces **el cimiento #9 no puede agregar ciegamente todo el lago**. Necesita saber, por partición, si ese dato es agregable. Eso refuerza Q7: **el proveedor no es metadato decorativo, es una llave de control legal.** Recomiendo que la ficha de linaje incluya una marca de "permite agregación anonimizada", poblada por la decisión del General Counsel y no por el ingeniero.

---

## 5. Encaje con el motor ya construido

**Punto de partida verificado:** el motor de descarga tiene QA APTO y su lógica (inventario, descarga concurrente, deduplicación, frontera Bulk/Delta) es **independiente de la fuente**. Nada de lo que sigue lo toca.

| Candidato | ¿Entra por configuración o exige Story? | Qué falta exactamente |
|---|---|---|
| **Binance Vision** | ✅ Ya funciona | Nada |
| **HistData.com** | 🟡 **Casi por configuración** | El canal (HTTP) y el formato (`.zip`) encajan con `ReqwestBulkSource`. Falta: construir el inventario por par/año/mes y ⚠️ confirmar si existe URL directa o hay que enviar formulario. **Si hay URL directa, es la vía más barata de todo el plan** |
| **TrueFX** | 🟡 Story pequeña | HTTP + `.csv` en `.zip` encajan. Falta manejo de sesión autenticada (introduce el Grupo II de ADR-0020, hoy omitido a propósito) |
| **Dukascopy** | 🔴 **Story completa** | El canal HTTP encaja, pero el formato no: descompresión LZMA + decodificación de registros binarios `.bi5` + escalado de precios por instrumento. Es el adaptador más caro **y el de mayor calidad de dato** |
| **Darwinex** | 🔴 **Story completa** | Cliente FTP autenticado — `ReqwestBulkSource` no sirve. Además, reunir los archivos separados de compra y venta en un tick único |
| **Databento** | 🔴 **Story completa** | API de lotes con clave + decodificación DBN. La descarga final del lote sí es HTTP de archivos, así que **parte del adaptador actual se reutiliza** |
| **FirstRate Data** | 🟡 Story pequeña | HTTP + `.csv` en `.zip`. Pero ver §3.4: cierra la puerta al Level 2 |
| **Polygon.io** | 🔴 **No encaja con el patrón** | Es API paginada, no volcado masivo. Entraría como `DeltaSource`, nunca como `BulkSource` |

**Corrección a la premisa del paquete:** el paquete afirma que el adaptador "asume descarga HTTP de archivos comprimidos (ADR-0034)". Es cierto del **adaptador**, no del **puerto**. Verifiqué que `BulkSource` no menciona HTTP en su firma: expone listar inventario y descargar un archivo a una ruta local. **Ese puerto ya admite FTP y API de lotes sin cambiar su forma.** Lo que sí cambia es que `BulkFileInfo` lleva un campo `download_url`, que asume direccionamiento por URL. Ese es el punto exacto de fricción, y es más pequeño de lo que el paquete sugiere.

---

## 6. Recomendación final y ruta de adopción

### 6.1. Fase A — cerrar EPIC-1 con dos clases de activo (coste: 0 €)

Cripto se queda como está (Binance Vision). Para Forex hay **dos arranques posibles y quiero que elijas tú**, porque la diferencia no es técnica sino de prioridad entre rapidez y fidelidad:

**Opción A1 — Dukascopy como fuente Forex de referencia** *(recomendada si priorizas calidad del dato)*
- **A favor:** la mejor calidad gratuita verificada — tick real con compra, venta y volúmenes, sobre más de mil instrumentos incluidos índices y materias primas, no solo divisas. Un solo proveedor cubre casi todo el CFD que Drasus quiera tocar.
- **En contra:** el adaptador más caro del plan (formato binario comprimido, no documentado oficialmente) y el riesgo legal más opaco (no publica términos).

**Opción A2 — HistData como fuente Forex de referencia** *(recomendada si priorizas cerrar la fase ya)*
- **A favor:** posiblemente la vía más barata de todo el plan — HTTP y `.zip`, casi lo mismo que ya funciona con Binance. 66 pares con tick de compra y venta.
- **En contra:** menos instrumentos (poco CFD), calidad de dato menos reputada, volumen siempre a cero, y una licencia que no dice nada, lo que legalmente es peor que decir poco.

**Mi recomendación concreta:** **A2 primero, A1 después.** Cierra la fase rápido con la fuente barata, y trata Dukascopy como la mejora de calidad de la fase siguiente, ya con la revisión legal hecha. Así el criterio de salida deja de estar bloqueado por el adaptador más difícil del plan.

**Antes de escribir código, en cualquiera de las dos opciones:**
1. **Gate del General Counsel** sobre Binance (urgente, hay código vivo) y sobre la opción elegida.
2. **Gate del Quant-Engineer** sobre Q3 (forma del tick) y Q5 (política de agregación).
3. Medición empírica de la profundidad real por instrumento — ningún proveedor gratuito la publica de forma fiable, así que se comprueba descargando, no leyendo.

### 6.2. Fase B — segunda fuente Forex para reconciliación cruzada

Añadir **TrueFX** como fuente de contraste, no como lago paralelo. Sirve para tres cosas: medir cuánto discrepan dos proveedores del mismo par (dato que hoy nadie tiene), dar sustento empírico al modelo de fricción de ADR-0017, y ser la primera prueba real del linaje por proveedor de Q7. Argumento extra: es la fuente que Pepperstone recomienda a sus clientes, lo que la acerca a un bróker real del propietario.

### 6.3. Fase C — fidelidad al bróker (diferida)

- **Disparador:** inicio de EPIC-5 (`broker-connector`), o **antes si quieres medir cuánto se aparta el proveedor de referencia de tu bróker real** — con la vía MT5 eso se puede hacer ya, sin esperar a EPIC-5.
- **Contenido:** el dato del bróker no alimenta el lago; alimenta la **calibración de fricción** (ADR-0017/0069).
- **Vía recomendada: importación local desde MT5** (§3.2), por delante de Darwinex. Sirve para los cuatro brókers, es gratis, entrega compra y venta, y no condiciona la elección de bróker. Darwinex queda como alternativa si se prefiere un canal automatizable.
- **Trabajo que implica:** un tercer canal de ingesta (importación de archivo local), más simple que Bulk y Delta. Es una Story pequeña — probablemente la de mejor relación valor/coste de todo el plan, porque responde empíricamente la pregunta que Q1 deja abierta por diseño.

### 6.4. Fase D — futuros (diferida)

- **Disparador:** primera feature que consuma futuros —realistamente, order flow y microestructura—, no antes.
- **Contenido:** Databento sobre CME Globex, evaluado primero con los 125 dólares de crédito gratuito, y **solo entonces** decidiendo entre pago por uso y plan mensual según el volumen medido.
- **Por qué diferirlo y no decidirlo hoy:** es el único gasto real del plan, y el precio se factura por gigabyte descomprimido. Sin un consumidor concreto, no hay forma de estimar el volumen y cualquier plan elegido hoy sería una apuesta.
- **Por qué nombrarlo hoy de todos modos:** para que quede escrito que **FirstRate Data no debe adoptarse** aunque sea más barato y más fácil de conectar. Su tick de futuros no trae compra y venta, y eso obliga a rehacer la ingesta cuando llegue order flow — el escenario exacto que pediste evitar.

### 6.5. Diferidos con disparador (registro obligatorio, `base.md` §4.2.3)

| Diferido | Disparador | Dónde se registra al aprobarse |
|---|---|---|
| Adaptador de futuros (Databento) | Primera feature consumidora de futuros | `docs/ROADMAP.md` + `docs/DEBT.md` |
| Libro de órdenes Level 2 y Level 3 | Feature de order flow / microestructura | `docs/ROADMAP.md` |
| Calibración de fricción contra bróker real | Inicio de EPIC-5, o antes si se adopta la vía MT5 | `docs/ROADMAP.md` |
| Canal de importación local (MT5) | Aprobación de la decisión #11 | `docs/ROADMAP.md` + enmienda a ADR-0034 |
| Muestra Forex congelada para STORY-051 | **Ya disparado** por este plan → DEBT-025 pasa a exigible al cerrar Fase A | `docs/DEBT.md` (DEBT-025) |
| Segunda fuente Forex (TrueFX) | Cierre de Fase A | `docs/ROADMAP.md` |
| Marca de "agregable" por partición (cimiento #9) | Dictamen del General Counsel | `docs/DEBT.md` |

---

## 7. Comprobaciones de razonamiento crítico de dominio

Aplicadas según `.agents/knowledge/critical-domain-reasoning.md`. Dejo constancia de las siete.

**7.1. Límite degenerado (obligatoria).** Empujé la premisa "la fuente masiva debe ser el bróker de ejecución": con B brókers y S símbolos el lago crece como B×S, y el histórico disponible tiende al **mínimo** de todos los brókers — que en IC Markets es 2021, con reportes de recorte a 2024. En el límite donde el propietario opera con los cuatro brókers nombrados, la profundidad común utilizable **tiende a meses, no a años**, y el criterio de salida de EPIC-1 (5+ años) se vuelve **inalcanzable por construcción**, no por falta de trabajo. Resultado: la premisa es degenerada. **Q1 debe resolverse a favor del proveedor de referencia.** Esta prueba es la justificación formal de la recomendación de Q1.

**7.2. Gate de especialista.** Declarado y **no salteado**: Q3 y Q5 requieren firma del **Quant-Engineer**; todos los proveedores requieren dictamen del **General Counsel**. El Architect es generalista en ambos dominios. Ninguna de esas preguntas debe pasar a ADR sin esas firmas.

**7.3. Desconfianza del rigor impresionante.** La afirmación de mayor carga del paquete —"un backtest fiel a IC Markets exige el dato de IC Markets"— suena rigurosa y **es cierta en el límite, pero engañosa como guía operativa**: IC Markets no publica ese dato, así que la afirmación no describe una opción disponible sino una imposible. La formulación útil es la contraria: como el dato del bróker no existe en profundidad, la fidelidad **tiene** que ser un problema de calibración. Reformulé la premisa en vez de repetirla.

**7.4. Re-derivación de supuestos, no de conclusiones.** ADR-0034 concluye "Bulk + Delta por HTTP de archivos comprimidos". Revisé sus supuestos uno a uno: (a) existen volcados públicos —✅ se cumple en Forex; (b) están en HTTP anónimo —🔴 **no se cumple** en Darwinex ni Databento; (c) vienen en `.zip`/`.gz` —🔴 **no se cumple** en Dukascopy ni Databento. **Dos de tres supuestos fallan fuera de cripto.** El ADR se escribió mirando el mundo cripto y se generalizó sin re-derivarse. De ahí que la recomendación sea enmendar el mecanismo y conservar la decisión.

**7.5. Separar el primitivo de la política.** El primitivo es la partición almacenada; la política es qué se puede hacer con ella (agregar, redistribuir, exportar). No los colapsé: por eso §4 propone una marca de "agregable" **separada** del identificador de linaje. Un dato de Binance y uno de Databento pueden coexistir en el mismo lago con permisos distintos, y el sistema debe saberlo.

**7.6. Cierre transitivo del concepto.** La procedencia del dato no toca solo la descarga. Enumerados: la descarga (`sovereign-data-fetcher`), el transformador (STORY-051), el validador punto-en-el-tiempo, la partición Parquet, la ficha de linaje (`data_snapshot_id`, ADR-0141), el registro de expedición de EPIC-2, la agregación anonimizada del cimiento #9, y la exportación del cimiento #13 (`data-portability`) — **este último es el más peligroso, porque exportar dato de terceros al usuario final es redistribución en el sentido legal del término**, y hoy nadie lo ha mirado.

**7.7. Distinguir tipos, no promediar.** Tres distinciones que este plan mantiene separadas y que el planteamiento original mezclaba: **tick ≠ 1 minuto** (proveedores, volúmenes y costes distintos); **último precio ≠ compra/venta** (la diferencia entre medir el spread y suponerlo); y **Level 1 ≠ Level 2 ≠ Level 3** (mejor precio, diez niveles, libro completo por orden — un proveedor puede dar el primero y cerrar los otros dos para siempre).

---

## 8. Lo que necesito que apruebes

| # | Decisión | Recomendación | Referencia |
|---|---|---|---|
| 1 | Q1 — naturaleza de la fuente masiva | Proveedor de referencia, con calibración posterior | §2 Q1, §7.1 |
| 2 | Q2 — uno por clase o multiactivo | Uno por clase de activo | §2 Q2 |
| 3 | **Q3 — forma del tick (urgente)** | **Compra y venta separadas, decidido ahora** | §2 Q3 |
| 4 | Q4 — contrato del puerto de descarga | Ampliar canal y descompresión; no tocar el orquestador | §2 Q4, §5 |
| 5 | Q5 — origen del 1 minuto | Derivar del tick; descarga solo como control | §2 Q5 |
| 6 | Q6 — proveedor de futuros | Databento, adopción diferida; **no** FirstRate | §2 Q6, §6.4 |
| 7 | Q7 — linaje por proveedor | Proveedor y método explícitos en la clave de linaje | §2 Q7, §4 |
| 8 | **Arranque de Fase A** | **A2 (HistData) primero, A1 (Dukascopy) después** | §6.1 |
| 9 | Conservar Binance en cripto | Sí, condicionado al dictamen legal | §3.3 |
| 10 | Orden de los gates | General Counsel sobre Binance **primero de todo** | §4 |
| 11 | **Vía MT5 (tu aporte)** | **Adoptarla como canal de importación local en Fase C — calibración y verificación, no lago** | §3.2, §6.3 |

**Al aprobarse, el Architect ejecuta:** enmienda a ADR-0034 (mecanismo, no decisión) + ADR nuevo de selección de proveedores y linaje + actualización de `docs/features/sovereign-data-fetcher.md` + corrección de la redacción de `docs/ROADMAP.md` §EPIC-1 + registro de los diferidos de §6.5. **Nada de eso se toca antes de tu aprobación.**

---

## Fuentes consultadas

Vía MT5: [MQL5 — CopyTicks](https://www.mql5.com/en/docs/series/copyticks) · [MetaTrader 5 — ticks reales y generados](https://www.metatrader5.com/en/terminal/help/algotrading/tick_generation) · [MQL5 — lectura de histórico de ticks](https://www.mql5.com/en/book/advanced/python/python_copyticks) · [Exportar tick data de MT5](https://support.eareview.net/support/solutions/articles/19000064697-exporting-metatrader-5-tick-data)

Datos verificados en: [HistData especificación](https://www.histdata.com/f-a-q/data-files-detailed-specification/) · [HistData FAQ](https://www.histdata.com/f-a-q/) · [Dukascopy export histórico](https://www.dukascopy.com/swiss/english/marketwatch/historical/) · [dukascopy-node](https://www.dukascopy-node.app/) · [TrueFX descargas](https://www.truefx.com/truefx-historical-downloads/) · [Darwinex tick data](https://www.darwinex.com/algorithmic-trading/tick-data) · [Databento precios](https://databento.com/pricing) · [Databento CME Globex MDP 3.0](https://databento.com/datasets/GLBX.MDP3) · [Databento docs GLBX](https://databento.com/docs/venues-and-datasets/glbx-mdp3) · [FirstRate Data](https://firstratedata.com/) · [Norgate futuros](https://norgatedata.com/futurespackage.php) · [CME DataMine](https://www.cmegroup.com/datamine.html) · [Tardis.dev](https://tardis.dev/) · [Binance Data Collection](https://data.binance.vision/) · [Binance.US datos históricos](https://support.binance.us/en/articles/9843310-introducing-historical-market-data-from-binance-us-download-for-free) · [Swissquote APIs](https://www.swissquote.com/en/institutional/solutions/technology/apis) · [Polygon.io forex](https://polygon.io/docs/forex/ws_getting-started)
