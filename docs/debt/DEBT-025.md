# DEBT-025 · Muestra real de Forex/CFD ausente en las pruebas del transformador

| Campo | Valor |
|---|---|
| **ID** | DEBT-025 |
| **Severidad** | 🟡 Baja |
| **Estado** | Abierta |
| **Abierta** | 2026-07-17 |
| **Origen** | STORY-051 (transformador de datos) — decisión del propietario: muestra real en dos tandas |
| **Dominio** | Ingesta / pruebas |

## Causa raíz

El transformador se prueba con datos fabricados (casos borde de desbordamiento, empates de redondeo, timestamps colisionados) más una muestra real congelada en el repositorio. Al cerrar STORY-051 la única fuente cableada es cripto (Binance), porque la fuente Forex/CFD del plan de EPIC-1 nunca se construyó (ver TASK-052). La muestra real cierra, por tanto, con una sola clase de activo.

## Impacto actual

Bajo. La corrección numérica del transformador no depende de la clase de activo: la aritmética entera ×10⁸ y la agregación OHLCV son idénticas para cripto y para Forex. Lo que la muestra de cripto **no** ejercita son las particularidades del dato Forex/CFD:

- Precio de compra y de venta separados (spread) frente al precio único de operación de cripto.
- Huecos de fin de semana y de cierre de sesión — el mercado cripto no cierra.
- Convenciones de volumen distintas (lotes frente a unidades de activo).

Si alguna de esas particularidades rompiera una regla del transformador, se detectaría al añadir la segunda tanda, no antes.

## Disparador de pago

**TASK-052 resuelta** — el Architect selecciona la fuente Forex/CFD y existe una descarga real de la que recortar la muestra. En ese momento se añade la segunda tanda de muestra al repositorio y las pruebas del transformador se extienden a las tres particularidades listadas arriba.
