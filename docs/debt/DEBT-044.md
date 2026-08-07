# DEBT-044 · La zona de carga parece aceptar archivos soltados y no los acepta

| Campo | Valor |
|---|---|
| **ID** | DEBT-044 |
| **Severidad** | 🟡 Baja |
| **Estado** | Abierta |
| **Abierta** | 2026-08-01 (prueba manual del propietario sobre STORY-073) |
| **Feature** | [`data-import-wizard`](../features/data-import-wizard.md) · componente `custom_ui.Dropzone` |
| **Disparador de pago** | La próxima Story que toque el paso de selección de archivo del asistente — o antes, si el propietario quiere el gesto de arrastrar en su flujo real |

## El hecho

El componente `Dropzone` se llama zona de arrastre, se dibuja como una zona de arrastre (recuadro punteado, icono `+`, superficie de cristal) y hasta el 2026-08-01 su texto decía *"Arrastra tu archivo CSV/TXT/TSV, o toca para buscarlo"*.

**Soltar un archivo encima no hace absolutamente nada.** No hay `DragTarget`, ni plugin de arrastre de escritorio (`desktop_drop` o equivalente) en `pubspec.yaml`. El único camino que funciona es tocar la zona, que abre el diálogo del sistema.

## Qué se corrigió ya, y qué no

**Corregido en el momento (no es esta deuda):** el texto dejó de prometerlo. La etiqueta es ahora *"Toca para elegir tu archivo CSV/TXT/TSV"* y el estado de hover ya no anuncia "Suelta aquí" —que era, además, la mitad del bucle de parpadeo que se arregló el mismo día—. Prometer una interacción que no ocurre es un defecto, no una deuda: se sanea donde se encuentra.

**Lo que queda debiendo es la funcionalidad:** arrastrar un archivo desde el gestor de archivos y soltarlo sobre el panel es el gesto natural para importar, y hoy no existe.

## Impacto actual

**Bajo.** El camino por diálogo del sistema está construido, probado y es el que cubre el criterio de aceptación. La ausencia del arrastre cuesta un par de clics, no bloquea nada.

## Por qué no se hizo en el momento

Añadir arrastre real es una dependencia nueva de plataforma con su propia superficie de fallo (rutas con espacios, URI de red, múltiples archivos, permisos de sandbox), y llegó a la mesa mientras se corregía un bug de parpadeo. Mezclar una corrección de defecto con una capacidad nueva habría impedido decir con claridad qué arregló qué.

## Cómo se verifica que quedó pagada

Soltar un archivo CSV desde el gestor de archivos sobre el panel del asistente carga ese archivo — el mismo estado que si se hubiera elegido por diálogo (nombre, tamaño, huella, columnas detectadas). Soltar algo que no es un archivo admisible produce un rechazo explicado, nunca un estado a medias. Y el texto de la zona vuelve a mencionar el arrastre solo cuando el arrastre funcione.

## Relacionadas

- [`DEBT-045`](./DEBT-045.md) — misma sesión de prueba manual, misma raíz: la interfaz dice menos de lo que el usuario necesita saber para actuar.
- `docs/DESIGN.md` §"Convención de Spec y Matriz de Estados" — la regla irrompible de geometría que dejó el bug de parpadeo.
