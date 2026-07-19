# DEBT-031 · La identidad de la serie se teclea a mano en la interfaz

| Campo | Valor |
|---|---|
| **ID** | DEBT-031 |
| **Severidad** | 🟠 Media |
| **Estado** | Abierta |
| **Abierta** | 2026-07-18 |
| **Origen** | Gap reportado por el Flutter-Engineer en STORY-051 Etapa 4; elevado por la auditoría del Tech-Lead |
| **Dominio** | Ingesta — identidad de series / superficie de usuario |

## Causa raíz

STORY-057 hizo obligatorios los seis campos de identidad de la barra, y el contrato del puente los exige. Pero `SourceDatasetDto` —lo que alimenta el selector de datasets del panel— **no los transporta**: nació antes del retrofit, cuando el panel solo preveía cuatro controles de configuración.

El Flutter-Engineer resolvió el bloqueo añadiendo campos de entrada manuales para instrumento, bróker y proveedor. Fue la decisión correcta para no inventarse datos ni quedarse parado, y la reportó en vez de esconderla. Pero el resultado es que **el usuario teclea a mano la identidad de la serie**.

## Por qué esto importa más de lo que parece

Los tres campos tecleados no son metadatos decorativos. Son la tripleta de linaje (`EURUSD-1M-PEPPERSTONE`) que ADR-0155 canonizó, y desde STORY-057 **entran en el sello de auditoría de las barras**.

La cadena de consecuencias de un simple error de tecleo:

1. El usuario elige el dataset correcto en el desplegable, pero escribe `PEPERSTONE` o deja el bróker de la transformación anterior.
2. Las barras se producen con una identidad que no corresponde a los datos que contienen.
3. Esa identidad falsa queda **sellada dentro del hash de auditoría**, que es justamente el mecanismo con el que probamos que una transformación es fiel y reproducible.
4. Aguas abajo, dos series de brókers distintos pueden fundirse sin que nada lo detecte — exactamente el defecto contra el que existe la doctrina de la tripleta, y que hace que una estrategia validada en un bróker fracase en otro.

Dicho corto: **el sistema certificaría como auténtica una identidad inventada por un dedo**. La identidad debe venir del origen del dato, no de la memoria del usuario.

## Impacto actual

**Ninguno hoy.** El panel inspector no es alcanzable desde la navegación real de la aplicación: el lienzo es todavía un marcador de posición sin nodos pulsables (deuda ya aceptada en STORY-051 §8). Nadie puede teclear nada porque nadie llega a la pantalla.

## Qué hay que hacer

**Directiva del propietario (vinculante, 2026-07-19):**

> *"¿Por qué se teclea a mano? Debemos tener desplegables o selectores para seleccionar el bróker, la temporalidad y el símbolo. No debe estar escrito a mano, debe desplegarse de las opciones soportadas."*

**Queda descartada la opción de campos de texto libre, incluso con validación.** La identidad se elige de una lista cerrada, nunca se escribe.

1. **Enriquecer `SourceDatasetDto`** para que transporte instrumento, bróker y proveedor desde el registro de descargas, que ya los conoce — el dato existe, simplemente no viaja hasta la pantalla.
2. **Poblar automáticamente** al elegir un dataset. Si el origen conoce la identidad, no hay razón legítima para contradecirlo: los campos quedan de solo lectura.
3. **Los tres selectores son listas cerradas de opciones soportadas**: bróker, símbolo y temporalidad. Nada de escritura libre. Esto exige que exista un **catálogo de brókers soportados** y un **vocabulario de temporalidades** —este último ya existe en el transformador (`1m`, `5m`, `15m`, `1h`, `4h`, `1d`) y debe ser la fuente del desplegable, no una lista duplicada en Dart.
4. Solo si el origen realmente no conoce la identidad (importación manual desde una terminal, caso previsto por ADR-0154), se pide al usuario **eligiendo de las mismas listas cerradas** — y ahí sí se marca la serie como de identidad **declarada**, no verificada.

**Nota de diseño:** el catálogo de brókers soportados no existe todavía como artefacto. Probablemente deba salir de la misma decisión que resuelva el conector universal — conviene componer esta deuda con esa Task en vez de inventar un catálogo aparte.

## Disparador de pago

**Antes de que el panel inspector sea alcanzable por un usuario real** — es decir, junto con la infraestructura de nodos del lienzo, o antes si alguna otra ruta de navegación llega primero al panel. No puede quedar detrás de esa deuda sin vigilancia: en cuanto la pantalla se abra, el tecleo es posible.
