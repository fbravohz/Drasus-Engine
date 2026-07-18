# DEBT-027 · La descarga de Binance está fijada en el código y es anónima

| Campo | Valor |
|---|---|
| **ID** | DEBT-027 |
| **Severidad** | 🔴 Alta |
| **Estado** | Abierta |
| **Abierta** | 2026-07-18 |
| **Origen** | Veredicto NO-APTO del General Counsel (Frente 2) + ADR-0154 §restricción 1 |
| **Dominio** | Ingesta / cumplimiento |

## Causa raíz

`crates/features/data/sovereign-data-fetcher/` lleva la dirección de Binance escrita como literal en el código (`schemas.rs:29` y `public_interface.rs:338`) y descarga de forma **anónima**, sin credenciales de ningún usuario. Es Drasus quien baja el dato, no el usuario.

Eso viola dos cosas a la vez:

- **ADR-0154:** Drasus provee conectores, no datos. La fuente es configuración elegida por el usuario, nunca un invariante del producto. La feature ya recogió la corrección de doctrina; el código no.
- **Veredicto legal (NO-APTO):** los términos verificados de Binance.US prohíben expresamente que un tercero lucre con sus datos de mercado. Drasus es producto de pago. Los términos de Binance global quedaron sin verificar.

Causa de fondo: ADR-0034 nombró proveedores concretos dentro de una decisión de arquitectura, y la feature lo marcó como `[FIJO]`. De ahí bajó al código como literal.

## Impacto actual

El adaptador funciona y pasó calidad, pero no puede distribuirse tal cual. Mientras el producto no se distribuya (fase greenfield), el riesgo no se materializa — pero cada Story que se construya encima aumenta el costo de corregirlo.

## Qué hay que construir

1. Eliminar el literal: la fuente pasa a configuración sin valor por defecto, elegida por el usuario.
2. Autenticación con las credenciales del propio usuario, respetando que los secretos nunca salen de su máquina (ADR-0093).
3. Declarar la fuente en la tabla de capacidades (ADR-0154), incluido su permiso de agregación (ver [DEBT-026](./DEBT-026.md)).
4. Verificar los términos de desarrollador de la interfaz de programación de la fuente antes de cablearla — requisito del veredicto legal, no opcional.

## Disparador de pago

**Inmediato en cuanto el propietario decida el destino del conector** (migrar a credenciales del usuario, retirarlo del producto, o solicitar permiso a la fuente). No debe cerrarse ninguna Story nueva que construya sobre este adaptador antes de esa decisión.
