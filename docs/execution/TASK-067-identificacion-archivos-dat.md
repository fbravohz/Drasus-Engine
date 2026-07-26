# TASK-067 · Identificación de los tres archivos `.dat` de `datasets/`

| Campo | Valor |
|---|---|
| **ID** | TASK-067 |
| **Tipo** | Task (sin código — aporte de insumo) |
| **Estado** | ✅ **CERRADA (2026-07-26)** — respondida por el propietario: son exportación de StrategyQuant X, y **no es una vía de importación soportada** |
| **Responsable** | **El propietario** identifica el origen · Tech-Lead lo integra después |
| **Épica (Fase)** | EPIC-1 — Soberanía de Datos (`ingest`) |
| **Creada** | 2026-07-25 |
| **Origen** | Pregunta abierta #1 del gate de viabilidad del Quant-Engineer (`.agents/plans/quant-2026-07-25-viabilidad-lector-mt5.md` §2.3) |

---

## Qué se necesita, en concreto

**El nombre de la herramienta, plataforma o librería que generó estos tres archivos:**

- `datasets/EURUSD_M1_dukas_M1.dat`
- `datasets/EURUSD_M1_dukas_H1.dat`
- `datasets/EURUSD_M1_icmarkets_M1.dat`

Basta con el nombre del programa desde el que se exportaron (o el sitio de donde se descargaron). Si existe documentación pública de su formato, el enlace.

## Por qué está bloqueado en el propietario y no se resuelve solo

El Quant-Engineer abrió los bytes crudos de los tres archivos. Comparten cabecera binaria idéntica (`4.2 D ABCDEFGH` + `SnRbTs` + tabla de offsets) y **no coincide con ningún formato documentado ni conocido en el repositorio**:

- No es el histórico `.hst` de MetaTrader 4 (ese arranca con un entero de versión 400/401 y 64 bytes de copyright en texto plano).
- No es el `.bi5` oficial de Dukascopy (ese no lleva cabecera: es un flujo comprimido de registros fijos de 20 bytes, ya documentado en `.agents/plans/plan-decision-fuentes-datos-mercado.md`).

Deducir la estructura de bytes a fuerza de inspección **es exactamente lo que ADR-0156 prohíbe** (rechazo por defecto, nunca adivinanza silenciosa). Un mapeo adivinado que resulte estar desplazado un campo produciría precios plausibles pero falsos — el peor modo de fallo posible, porque no se nota.

## Qué se desbloquea al responder

Los dos archivos de texto de MetaTrader (barras de 1 minuto y ticks) **no dependen de esto** y siguen su curso normal. Lo que queda parado es la única prueba real de que la identidad de la serie mantiene separados dos brókers distintos del mismo par: para eso hacen falta estos tres archivos, que son de Dukascopy e IC Markets.

## Criterio de cierre

El propietario nombra la herramienta de origen. A partir de ahí, el Tech-Lead decide si se construye una plantilla de plataforma para ese formato (ADR-0156 restricción 4) o si se pide una reexportación en formato de texto, que sería más barata.

## Alternativa que no requiere respuesta

Si el propietario no recuerda el origen, **reexportar el mismo par desde dos brókers distintos en formato de texto** (CSV/TSV, como el de MetaTrader que ya entregó) cierra esta Task sin necesidad de identificar nada.

---

## Resolución (propietario, 2026-07-26)

**Son exportación directa de StrategyQuant X.** El propietario los tenía a mano, no los eligió como fuente: *"no diría que es un medio oficial soportado de importación, solo era lo que tenía a la mano"*.

**Decisión: no se construye plantilla de plataforma para este formato.** Razones, en orden:

- **No es una vía de ingesta del producto.** StrategyQuant X es una herramienta de la competencia, no un terminal de bróker ni un proveedor de datos. Importar su formato propietario de trabajo no sirve a la doctrina de ADR-0154 (el usuario trae los datos de **su bróker**), y su formato no está documentado públicamente.
- **Deducir el formato está prohibido por ADR-0156.** Sin especificación, un mapeo adivinado con un campo corrido produciría precios plausibles pero falsos — el peor modo de fallo, porque no se nota.
- **No hay nada que perder.** Lo único que estos archivos aportaban era la prueba de que la identidad de serie mantiene separados dos brókers del mismo par, y eso se cubre igual con dos exportaciones de texto.

**Formatos que sí se soportan, confirmados por el propietario:** CSV de MetaTrader (construido, STORY-068) · Binance (adaptador existente, con la deuda `DEBT-027` pendiente de saldar) · **futuros de CQG o NinjaTrader**, en cuanto el propietario los consiga.

**Lo que sigue pendiente y NO se cierra aquí:** la muestra del mismo símbolo y rango desde un segundo bróker (`DEBT-025`). El propietario tiene acceso a otros brókers pero aún no ha podido instalar sus terminales de MetaTrader 5. Sigue rastreado como pendiente suyo, con la vía de cTrader anotada como alternativa (ver la nota de esta Task en `PROGRESS.md`).
