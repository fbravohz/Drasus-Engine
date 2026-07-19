# TASK-061 · Revisar el techo de magnitud de la escala entera (ADR-0141)

| Campo | Valor |
|---|---|
| **ID** | TASK-061 |
| **Tipo** | Task (decisión de arquitectura) |
| **Responsable** | Architect |
| **Estado** | 🟡 Abierta — pendiente de despacho |
| **Creada** | 2026-07-19 |
| **Origen** | Quant-Engineer, Etapa 6 de STORY-051; aritmética verificada por el Tech-Lead |
| **Resuelve** | DEBT-033 |

## Directiva del propietario (vinculante, 2026-07-19)

> *"Mi intención no es descartar esos activos. Mientras más activos podamos operar, mejor. 100%."*

**Queda descartada de entrada la opción de declarar la clase de activo fuera de alcance.** La decisión es *cómo* soportarla, no *si* se soporta.

## El problema

ADR-0141 fija la escala entera ×10⁸ y la justifica por **precisión decimal**: "cubre crypto (8 dec.) y forex (5 dec.) sin overflow en `i64`". La justificación es correcta para el problema que resolvía — pero responde a la pregunta de **cuántos decimales**, no a la de **cuánta magnitud**. Son preguntas distintas y solo se contestó una.

Un `i64` con escala ×10⁸ solo representa **~92 200 millones de unidades**. Verificación aritmética (Tech-Lead, 2026-07-18):

| Escenario | Unidades en una barra de 1h | Contra el techo |
|---|---|---|
| SHIB a ~$0.00002, 50 M USD/día (volumen conservador) | ~104 200 millones | **1,13× — desborda** |
| SHIB en día activo, 200 M USD/día | ~416 700 millones | **4,52× — desborda** |
| PEPE a ~$0.000012, 30 M USD/día | ~104 200 millones | **1,13× — desborda** |

No es un extremo teórico: **el volumen de un día cualquiera desborda en una sola barra horaria**, y estos activos se negocian en la misma fuente que usamos como muestra real.

Hoy el sistema falla seguro (`checked_add` → error explícito, sin corrupción), pero rechaza sistemáticamente.

## Qué debe decidir el Architect

### 1. Re-derivar el supuesto contra rango, no solo contra precisión

Declarar explícitamente qué combinaciones de precio unitario y oferta circulante debe soportar el sistema. Es la comprobación que ADR-0141 nunca hizo.

### 2. Elegir la salida, con su coste real

Las opciones **no** son equivalentes y el Architect debe pronunciarse con argumentos, no elegir la más fácil:

- **Volumen en `i128`, precio en `i64`.** El precio no tiene problema de magnitud (ningún activo cuesta 92 000 millones); el volumen sí. Separar los dos anchos ataca exactamente el problema. Coste: SQLite no tiene entero de 128 bits nativo — hay que decidir la representación en persistencia (dos columnas, texto, o `BLOB`), y eso toca el modelo de datos, Parquet y el puente FFI.
- **Escala variable por instrumento.** Precisión adaptada a cada activo. Coste alto y peligroso: la escala pasa a ser un dato por fila, y comparar dos series exige normalizar. Contradice el espíritu de "escala fija para todo el sistema" de ADR-0141.
- **Escala menor para volumen que para precio.** El volumen rara vez necesita 8 decimales. Bajarlo a ×10² o ×10⁰ para volumen multiplica el techo por 10⁶ u 10⁸. Coste bajo, pero hay que verificar que ningún activo negocie fracciones más finas que la escala elegida.

**Recomendación del Tech-Lead, no vinculante:** la tercera opción parece la de mejor relación coste/beneficio y merece evaluarse primero — pero es el Architect quien decide, tras comprobar si existe algún activo real que negocie fracciones de unidad por debajo de la escala que proponga.

### 3. Alcance de la corrección

Qué se toca: ADR-0141, el esquema canónico de ADR-0155, las migraciones afectadas (fase GREENFIELD: se editan in-situ), el crate del transformador, el puente y la interfaz. Estimar el radio antes de despachar implementación.

### 4. Si alguna combinación queda fuera pese a todo

Decirlo **donde el usuario lo vea**. La doctrina es que el usuario trae sus propios datos (ADR-0154); si trae un activo que no podemos procesar, merece un mensaje que lo explique, no un error genérico de desbordamiento.

## Disparador

Antes de habilitar ingesta real de cripto de oferta extrema, y antes de cualquier promesa pública sobre cobertura de mercados.
