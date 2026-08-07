# DEBT-045 · Los nodos del lienzo son nombres pelados: no dicen qué hace cada feature ni qué necesita para funcionar

| Campo | Valor |
|---|---|
| **ID** | DEBT-045 |
| **Severidad** | 🟠 Media |
| **Estado** | ✅ **Pagada** (2026-08-02) |
| **Abierta** | 2026-08-01 (prueba manual del propietario sobre STORY-073) |
| **Feature** | Canvas (`ui/lib/tabs/canvas_tab.dart`) — transversal a toda feature con Inspector Panel |
| **Disparador de pago** | La próxima Story que añada una feature al lienzo. A partir de tres o cuatro nodos el problema deja de ser incomodidad y pasa a ser que el usuario no encuentra lo que busca |

## El hecho

La lista de features del lienzo muestra solo el nombre: *Reloj Determinista*, *Async Jobs*, *Audit Log*, *Telemetría*, *MCP Gateway*, *Asistente de Importación*, *Transformador de Datos de Mercado*. Nada dice qué hace cada una, qué necesita para funcionar, ni qué produce.

**Costó una sesión de prueba completa.** El propietario, probando el asistente de importación recién construido, abrió el *Transformador de Datos de Mercado* —la fila de al lado— y concluyó que la pantalla estaba inservible porque no tenía forma de seleccionar un archivo. Y tenía razón sobre lo que veía: el transformador consume un dataset **ya importado**, nunca toca archivos. Las dos features son vecinas en la lista, ambas hablan de datos de mercado, y ninguna anuncia la diferencia.

Que le pasara a quien mandó construir ambas es el dato relevante: no es falta de familiaridad, es que la información no está en pantalla.

## Por qué es Media y no Baja

Porque el lienzo es la superficie de entrada al producto — el sitio donde el usuario elige qué va a hacer. Un nombre pelado obliga a abrir el panel para descubrir si era el correcto, y ese descubrimiento solo funciona si el usuario ya sabe qué esperar de cada feature. Con ~138 features en el diseño, esa apuesta no escala: el coste crece con cada nodo que se añade.

Añadido: el panel de reserva para features sin Inspector Panel ya hace lo correcto (dice explícitamente "todavía no tiene un Inspector Panel construido"). La honestidad existe **después** de abrir; falta **antes**.

## Qué hace falta

Que cada nodo diga, sin abrirlo, qué hace y de qué depende. Una frase por feature, del mismo registro que ya resuelve qué panel abrir — no una tabla nueva ni un sistema de ayuda. La forma exacta (subtítulo en la fila, tooltip al posar el cursor, o ambos) es decisión de la Cáscara Visual, no de esta ficha.

Lo que sí es requisito: la frase debe distinguir **qué consume** de **qué produce**. Ese es el eje que confundió las dos features de esta sesión, y es el mismo eje del pipeline (`ingest → generate → validate → …`), así que sirve para todas.

## Cómo se verifica que quedó pagada

Alguien que no ha visto nunca el producto puede decir, sin abrir ningún panel, cuál de los nodos del lienzo toma un archivo del disco y cuál trabaja sobre datos ya importados.

## Cómo quedó pagada (2026-08-02)

Cada feature del catálogo del lienzo declara ahora su descripción junto a su nombre, y esa descripción **debe distinguir qué consume de qué produce** — el requisito de esta ficha, escrito como comentario sobre el campo para que no se pierda al añadir la feature siguiente.

Se muestra en los dos sitios, y por razones distintas:

- **Bajo el nombre**, recortada a dos líneas: se lee sin hacer nada, que es lo que hace falta para elegir entre features vecinas de un vistazo.
- **Al posar el cursor**, completa y con el nombre: el texto de debajo depende del ancho del panel, que ahora el usuario puede estrechar. El tooltip no depende de nada.

Las dos features que originaron la ficha quedan separadas por lo que dicen: *"Toma un archivo de tu disco (CSV, TXT o TSV)"* frente a *"Toma datos YA importados... No lee archivos."*

**Prueba que lo sostiene:** `ui/test/canvas_panels_test.dart` afirma que ambos textos existen en pantalla sin abrir ningún panel, y que el tooltip lleva nombre y descripción completos.

**Entregado en el mismo trabajo, fuera del alcance de esta ficha:** ambos paneles laterales se ajustan arrastrando su borde, y el de features nace medido contra el nombre de feature más largo del catálogo en vez de con un ancho fijo escrito a mano.

## Relacionadas

- [`ADR-0136`](../adr/ADR-0136.md) — el nodo abre panel lateral; esta ficha es sobre lo que se ve **antes** de abrirlo.
- [`DEBT-044`](./DEBT-044.md) — misma sesión, misma raíz: la interfaz dice menos de lo que el usuario necesita para actuar.
