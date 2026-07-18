# Sesión 2026-07-17/18 — Volcado íntegro de decisiones y pendientes

> **Por qué existe este documento:** la sesión se compactó y `NEXT-DISPATCH.md` se borra al terminar. Todo lo acordado vive aquí, en `PROGRESS.md`, en las Tasks de `docs/execution/` y en las fichas de `docs/DEBT.md`. Nada depende de la memoria de la conversación.
>
> Documentos hermanos producidos en esta sesión: `.agents/plans/esquema-canonico-datos-mercado.md` (Quant) · `.agents/plans/veredicto-legal-datos-mercado.md` (General Counsel) · `.agents/plans/plan-decision-fuentes-datos-mercado.md` (Architect, parcialmente obsoleto, con banner).

---

## 1. La doctrina que cambió el producto

**Drasus Engine no provee datos de mercado a nadie.** Cada usuario trae los suyos desde sus propios brókers. Drasus provee los **conectores** y el motor de refinado y análisis. Tres vías canónicas de entrada: interfaz de programación autenticada · credenciales de bróker · exportación manual desde un terminal.

Canonizado en **ADR-0154**. La profundidad histórica nunca fue responsabilidad de Drasus: es del usuario y de lo que su bróker le dé. Dato empírico del propietario: los brókers guardan pocos ticks, pero de barras de minuto, hora y día hay registros de 10 a 20 años.

**Dónde se había perdido (auditoría forense del Architect):** vacío genuino. `ADR-0126:12` cita *"igual que las fuentes de precio en `ingest`"* como si esa regla existiera — nunca se escribió, y ADR-0034 la contradecía nombrando proveedores concretos dentro de una decisión de arquitectura. De ahí bajó a la feature como `[FIJO] Binance Vision`, de ahí al código como literal, y de ahí al criterio de salida "5+ años de 2 símbolos". Ninguno de esos pasos fue una decisión.

## 2. Datos primitivos frente a indicadores (doctrina del propietario, 2026-07-18)

Palabras del propietario, a tatuar donde corresponda:

> *"Los datos del mercado —máximo, mínimo, apertura, cierre, compra, venta, volumen— se manejan como materia prima. Cualquier cosa que use esos datos primitivos debe ser considerado un indicador. El precio promedio ponderado definitivamente es un indicador, y se usa como se usa el ATR, el MACD, el RSI, el perfil de volumen o una media móvil."*

Referencia que el propietario aporta: MT5 permite calcular un indicador eligiendo sobre qué precio se aplica (apertura, máximo, mínimo, cierre, mediana, típico, ponderado) — el patrón `ENUM_APPLIED_PRICE`, `https://www.mql5.com/en/docs/constants/indicatorconstants/prices#enum_applied_price_enum`.

**Consecuencias directas:**
- La **ingesta** persiste primitivos. Los **indicadores** se calculan en el motor (EPIC-2/EPIC-3), nunca en la ingesta.
- El promedio ponderado sale de la barra → **DEBT-028**.
- La frontera "primitivo frente a indicador" merece quedar escrita como decisión de arquitectura, y probablemente el patrón de precio aplicable como parámetro del indicador.

## 3. Responsabilidad y calidad del dato (doctrina del propietario)

> *"Cada usuario es agente de los datos que usa. El motor calcula perfectamente en base a los datos que le proporcionas, y si no tienes los datos para usar una funcionalidad, entonces no te va a dejar usarla."*

Razonamiento del propietario, aceptado como marco: que el volumen que exporta MT5 de muchos brókers sea poco fiable no es problema de Drasus. Operar con barras en vez de ticks implica asumir esa pérdida de exactitud — es una consecuencia que el usuario acepta al elegir sus datos, no un defecto del motor. Si a alguien le funciona una estrategia construida sobre volumen poco fiable, ni el mérito ni la culpa son nuestros.

**Esto refuerza dos frentes ya abiertos:** la habilitación de funciones por capacidad de dato (una función sin el dato que exige no se ofrece), y el juego de cláusulas específicas del corpus legal.

## 4. Decisiones del propietario — vinculantes, no volver a preguntar

| # | Decisión |
|---|---|
| 1 | Drasus no provee datos; cada usuario trae los suyos (ADR-0154). |
| 2 | **Sí se graba el flujo en vivo.** Pendiente el dictamen legal de alcances, no la decisión. |
| 3 | **Conector de Binance:** pasa a credenciales del usuario (DEBT-027). Es su prioridad más baja; descartarlo del producto es alternativa aceptable. **Prohibido pedir permiso a Binance.** Su interés en cripto es **con apalancamiento** (futuros/CFD sobre cripto) → viene de brókers, no de exchanges. |
| 4 | **Responsabilidad del dato: íntegra del usuario.** Drasus responde de que el software calcule bien, **arreglando el software, nunca pagando pérdidas de operar con él**. |
| 5 | **Linaje = tripleta** símbolo × marco temporal × bróker (`EURUSD-1M-PEPPERSTONE`, patrón SQX). No se funden jamás. |
| 6 | **Agregación:** se agrega lo que la licencia de la fuente permite, se omite lo bloqueado. Habilitador: marca por origen (DEBT-026). |
| 7 | ADR-0154 y ADR-0155 aprobados íntegros. |
| 8 | **"Guarda el hecho, calcula la interpretación"** como principio sin excepción, con **todos** los métodos de ajuste y costura definidos, no uno por defecto. |
| 9 | **Habilitación de funciones por capacidad de dato**: se resuelve al conectar; lo no disponible se muestra visible, apagado y con el motivo. |
| 10 | **Nivel gratuito:** se toma la **estrategia completa y los resultados** (el mapa de dónde vive el edge). Los agregados no le sirven. La protección es el **intercambio anunciado**, no tomar menos. |
| 11 | **Modelo de cooperativa de minado aprobado como marco** del nivel gratuito: aportas cómputo y exploración, recibes algo. **Qué es ese algo queda deliberadamente abierto** (acceso preferente, parte de lo encontrado, créditos) — se decide después. |
| 12 | **Bloqueo por jurisdicción** habilitado como opción de producto. Contexto del propietario: *"Drasus Engine es para mí; si yo y mi familia y seres queridos la usamos prósperamente, el objetivo está resuelto."* |
| 13 | **Deudas: se acumulan y se repasan todas juntas** al cerrar los pendientes en curso, antes de arrancar feature nueva. |

## 5. Estado del trabajo de código

**STORY-051 — transformador de datos (TTR-007).** Backend entregado, retrofit a ADR-0155 aplicado, **45 pruebas en verde reproducidas por el Tech-Lead**, comando de verificación devolviendo el esquema nuevo con las cuatro magnitudes de volumen. **NO cerrada.** Pendiente: auditoría de detalle del código y del mapeo criterio→prueba (el Tech-Lead la debía y no la hizo), sacar el promedio ponderado, y la cadena puente → interfaz → calidad → validación cuantitativa.

**Tres decisiones que el ingeniero dejó abiertas** (paró en vez de resolver solo, comportamiento correcto):
1. Promedio ponderado en la barra → resuelto por el propietario: es indicador, sale (DEBT-028).
2. `crates/shared/src/types/mod.rs` sigue declarando `Tick` en vez de `QuoteTick`/`TradeTick`/`DepthLevel`/`OrderEvent` → se actualiza **junto con la tripleta**, por instrucción del propietario.
3. Faltan campos de identidad en la barra (instrumento, mercado, tipo de barra) → encajan con la tripleta, se resuelven ahí.

## 6. Cola de trabajo pendiente (orden por dependencia)

1. **Auditoría del Tech-Lead** del retrofit y de las entregas de ADR-0155 y del corpus legal. Detalle de código y mapeo criterio→prueba. **Deuda de rigor asumida: se prometió y no se hizo.**
2. **Architect (Sonnet) — tripleta de linaje** + actualización del catálogo de tipos compartido + campos de identidad de la barra. Encargo íntegro en `PROGRESS.md`. Fue despachado y **detenido por el propietario**; verificar que no dejó ediciones a medias antes de relanzar.
3. **Architect (Sonnet) — métodos de ajuste y costura.** Todos los métodos, no uno por defecto: ajuste aditivo, proporcional, empalme crudo, contrato suelto; disparadores de rolo por volumen, interés abierto, días antes del vencimiento y primer día de aviso. El método usado forma parte de la identidad de la serie derivada. Guardar además proporción y fecha del split, importe y fecha del dividendo, y código, vencimiento, primer día de aviso y fechas de rolo del contrato. Verificar qué hacen NinjaTrader y TradingView en vez de afirmarlo de memoria.
4. **Architect — frontera primitivo/indicador** (sección 2 de este documento), incluido el patrón de precio aplicable como parámetro.
5. **Architect — habilitación por capacidad de dato.** Clasificar cada feature por el dato mínimo que exige; resolver al conectar; mostrar lo no disponible apagado y con motivo; componer con la habilitación por plan; regla anti-promesa.
6. **Rust — sacar el promedio ponderado de la barra** (DEBT-028) y cerrar STORY-051.
7. **Bridge → Flutter → QA → Quant** para cerrar STORY-051. No despachar el puente antes de que el esquema esté firme.
8. **Task de secuenciación de conectores** (reemplaza a TASK-052, replanteada): qué conectores primero y con qué tabla de capacidades. Insumo verificado: exportación de terminal (cualquier bróker) · interfaz abierta de cTrader (verificar por bróker) · Darwinex · Swissquote · futuros según lo que el usuario pague.
9. **Tasks legales y de arquitectura legal:** TASK-053 a TASK-056 (ver `docs/execution/`).
10. **Repaso conjunto de deudas** antes de arrancar feature nueva (instrucción del propietario).

## 7. Trabajo sin commitear

Toda la sesión. Cinco decisiones de arquitectura, siete decisiones legales, un crate nuevo con su migración, cuatro deudas, dos Tasks y varias features corregidas. **Pendiente de autorización explícita del propietario**, agrupado por tipo.
