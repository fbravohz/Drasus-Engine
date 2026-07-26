# DEBT-035 · La vía de mapeo manual del asistente de importación no tiene pantalla, y es la que legalmente la exige

| Campo | Valor |
|---|---|
| **ID** | DEBT-035 |
| **Severidad** | 🟠 Media |
| **Estado** | Abierta |
| **Abierta** | 2026-07-25 (cierre de STORY-068) |
| **Feature** | [`data-import-wizard`](../features/data-import-wizard.md) |
| **Origen** | Alcance declarado de [STORY-068](../execution/STORY-068-mt5-bar-reader.md) §3 y §8 |

## Qué queda debiendo

STORY-068 construyó la **vía de plantilla de plataforma**: Drasus afirma el mapeo de columnas de MetaTrader, lo verifica con pruebas, y responde por él. Esa vía no necesita pantalla — el usuario no declara nada, solo entrega el archivo.

Falta la **vía de mapeo manual**: el usuario declara qué es cada columna suya cuando su archivo no encaja en ninguna plantilla conocida. Con ella falta su pantalla: arrastrar el archivo, ver una vista previa de **dato ya transformado**, y confirmar.

## Impacto actual

**Ninguno vivo.** La vía manual no existe, así que no puede usarse mal. El lector funciona hoy por la vía de plantilla, que es la única construida, y su verificación corre por el comando de terminal (`docs/TEST.md`).

## Por qué no es cosmético

LDR-0008 condición 1 es explícita: la protección de responsabilidad de Drasus depende de que el usuario haya confirmado el mapeo **sobre una vista previa de sus propios datos ya transformados**, no sobre una tabla de correspondencias de nombres de columna. Sin esa pantalla, la vía manual no puede existir sin exponer a Drasus — construir el motor de mapeo manual sin su vista previa sería peor que no tenerlo.

Es también una deuda de Definición de Terminado (ADR-0117): la Superficie de Verificación en interfaz es el único canal por el que el propietario prueba el camino completo sin leer código.

## Disparador de pago

**La Story del asistente visual de importación** — que es la misma que habilita la vía de mapeo manual. Las dos cosas nacen juntas o ninguna: el motor de mapeo manual sin vista previa confirmada no debe construirse.

No antes: hoy no hay vía manual que proteger. No después: el día que exista el motor de mapeo manual sin pantalla, la exposición legal es real.

## Relacionadas

- [[DEBT-036]] — la otra deuda abierta por el cierre de STORY-068 (trazabilidad física de la declaración hacia la persistencia).
- [`LDR-0008`](../legal/LDR-0008.md) — veredicto que fija las cuatro condiciones de la protección.
- ADR-0117 — Superficie de Verificación en la misma Story que el backend.
