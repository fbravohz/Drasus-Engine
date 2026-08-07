# DEBT-040 · La importación carga el archivo entero en memoria, y el archivo real pesa 535 MB

| Campo | Valor |
|---|---|
| **ID** | DEBT-040 |
| **Severidad** | 🟠 Media |
| **Estado** | Abierta |
| **Abierta** | 2026-07-31 |
| **Origen** | Medición del Tech-Lead sobre el archivo real del propietario durante la Etapa 3 de [STORY-073](../execution/STORY-073-asistente-importacion-pantalla.md) |
| **Disparador de pago** | La Story de conexión con la persistencia real (la entrega inmediatamente posterior a STORY-073), donde el camino de escritura se diseña por primera vez |

---

## El hecho, medido

Sobre `datasets/EURUSD_M1_2000-01-01_2025-05-01.csv` — **535 MB, 9 231 695 filas**, ejecutado contra la superficie pública real del crate en perfil `release`:

| Operación | Coste |
|---|---|
| Lectura de disco | 0,4 – 0,7 s |
| Parseo completo a estructuras | **5,3 s** |
| Memoria del vector resultante | **704 MB** (`Mt5RawBarFields`; `CanonicalBarRow` es mayor) |

El camino de importación toma `file_bytes: Vec<u8>` — el archivo completo en memoria — y produce un `Vec<CanonicalBarRow>` con todas las filas antes de emitir nada. Pico transitorio: **más de 1,2 GB** para un archivo de 535 MB, sin contar la copia que la interfaz tuvo que hacer para entregarlo.

## Por qué no es un problema de rendimiento

Un tick de un mercado activo, o varios años de un símbolo de cripto, no son 535 MB: son varios gigas. En esa escala esto no se vuelve lento — **se vuelve imposible**, y falla por agotamiento de memoria en la máquina del usuario, que es donde el motor corre por diseño (Local-First, ADR-0143). El fallo no llega gradualmente: llega de golpe con el primer archivo que no cabe.

Añadido menor pero real: `CanonicalBarRow.timeframe_value` es `String`, es decir **una asignación de heap por fila** para un valor idéntico en todas ellas. Sobre 9,2 M de filas es coste puro sin beneficio; el valor pertenece al tramo, no a la barra.

## Qué NO es esta deuda

**No es la vista previa.** Que la vista previa procesara el archivo entero para mostrar diez filas se corrigió dentro de STORY-073 — era barato y afectaba a cada iteración del usuario.

Esto es el camino de **confirmación**: el que sí tiene que recorrer el archivo completo porque tiene que importarlo entero. Lo que está mal no es que lo recorra, es que lo tenga todo en memoria a la vez en vez de procesarlo por partes.

## Qué habría que hacer

Lectura y emisión **por partes**: recorrer el archivo en bloques, transformar cada bloque y entregarlo al consumidor sin acumular el total. El puerto de salida ya emite a un consumidor (`bar_sink`), así que la forma del contrato no lo impide — lo impide que el orquestador construya el vector completo antes de llamarlo.

**Por qué se paga en la Story de persistencia y no antes:** ahí es donde el consumidor real (escritura a disco con particionado) se diseña por primera vez. Diseñar ese consumidor asumiendo que recibe todo de golpe, y rehacerlo después, es hacer el trabajo dos veces. Y al revés: resolver la lectura por partes sin saber cómo escribe el consumidor es optimizar a ciegas.

## Relación con otras deudas

- [`DEBT-036`](./DEBT-036.md) — mismo disparador (la Story de persistencia real) y misma superficie (el puerto de salida de barras). **Se evalúan juntas:** una cambia qué viaja por ese puerto, la otra cómo y en qué tandas.
