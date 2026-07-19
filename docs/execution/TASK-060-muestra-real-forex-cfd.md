# TASK-060 · Muestra real de Forex/CFD para las pruebas de ingesta

| Campo | Valor |
|---|---|
| **ID** | TASK-060 |
| **Tipo** | Task (sin código — aporte de insumo) |
| **Estado** | 🔴 **BLOQUEADO POR EL USUARIO** |
| **Responsable** | **El propietario** aporta los archivos · Tech-Lead los integra después |
| **Creada** | 2026-07-19 |
| **Salda** | DEBT-025 |

## Qué se necesita, en concreto

**Archivos CSV exportados de una terminal de bróker. NO se necesitan credenciales.**

Las credenciales pertenecen al conector automático, que es otra fase; y por ADR-0093 nunca salen de la máquina del usuario de todos modos. Para probar la ingesta basta con archivos.

### Lista de la compra

| Qué | Detalle | Por qué |
|---|---|---|
| **Un par de divisas** | EURUSD o similar, unos miles de filas | No hace falta volumen: hace falta realidad |
| **Dos granularidades** | Barras de 1 minuto **y** ticks, por separado | Son formatos distintos y ejercitan dos caminos distintos del transformador |
| **Sin limpiar** | Tal cual sale de la terminal | **Lo valioso son las rarezas**: separador decimal, formato de fecha, columna de spread, número de dígitos, cabeceras |
| **Dos brókers del mismo par** | Ej. Pepperstone + IC Markets | Es lo único que prueba de verdad que la tripleta de linaje los mantiene separados |

### Brókers de interés declarados por el propietario

IC Markets, Darwinex, Pepperstone, Swissquote. Cualquiera sirve para empezar; dos distintos es lo ideal.

## Por qué está bloqueado en el propietario

Drasus **no provee datos** (ADR-0154): provee conectores. No podemos descargar esta muestra por nuestra cuenta sin violar la doctrina del producto y, según la fuente, sus condiciones de uso. El propietario tiene cuentas en estos brókers; nosotros no.

**Esta tarea no avanza sin él.** No hay forma de destrabarla desde dentro del equipo.

## Qué se hace cuando lleguen

1. Congelar la muestra en el repositorio como recurso de prueba, igual que la muestra de cripto ya existente.
2. Añadir las pruebas de la segunda tanda del transformador contra datos reales de Forex/CFD.
3. Cerrar **DEBT-025**.
4. Alimentar con evidencia real la decisión de fuentes pendiente (TASK-052) y el diseño del conector universal.

## Nota de método

Registrada como Task con marca de bloqueo del usuario por la regla del 2026-07-19: **todo accionable que dependa de algo que solo el propietario puede aportar** —datos, credenciales, una decisión de negocio, una compra, un acceso— se rastrea así, nunca en prosa dentro de una conversación. El propietario debe poder barrer sus bloqueos sin releer un chat.
