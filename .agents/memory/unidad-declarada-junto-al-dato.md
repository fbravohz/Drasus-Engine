---
name: unidad-declarada-junto-al-dato
description: Un valor crudo sin su unidad sellada junto al dato es un valor a medias; el nombre del campo no es evidencia
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 3115d992-cd1b-42b1-b5bf-2297d84c3f35
  modified: 2026-07-26T00:30:52.009Z
---

Cuando se persiste un número **tal como lo entregó la fuente** (sin convertir a la escala canónica), llamarlo `..._raw` evita que se confunda con un valor ya convertido, pero **no dice en qué unidad está**. Un nombre de campo no viaja con el dato ni sobrevive a que alguien lea la tabla sin leer el código. La unidad se sella junto al dato — columna propia en el registro, o hecho fijo de la plantilla si la fuente no la varía — y si no se puede establecer, **se rechaza en vez de asumirla**.

**Why:** en STORY-068 (2026-07-25) el lector de MetaTrader le dio el tratamiento completo al volumen crudo (unidad declarada, `NULL` = no declarado y jamás un valor por defecto) y al spread crudo solo el nombre `spread_reported_raw`. El spread de MT5 viene en **puntos** (último dígito del precio), no en pips: un consumidor futuro que eligiera la lectura equivocada se habría equivocado **por un factor de diez, en silencio**. Verificado empíricamente contra el archivo real — junio 2024 tiene pico en `0` con valores de `10`–`14`, coherentes como puntos (1.0–1.4 pips en EURUSD) e imposibles como pips.

Lo cazó **el propietario**, no el gate de mutación ni el QA — porque el código era correcto. Lo que faltaba era decir qué significaba el número. Ningún gate automático detecta esa clase de hueco.

**How to apply:** al entregar o auditar una feature que persista valores crudos, **enumera todos los campos crudos de esa entrega** y verifica que cada uno lleve su unidad sellada — no solo el que la orden de trabajo mencionaba. Es un barrido de familia, no una comprobación puntual. Regla escrita en `.agents/skills/rust-engineer/SKILL.md` §4 y en `.agents/skills/tech-lead/SKILL.md` §Verificación Independiente. Relacionado: [[despacho-no-precocinar-solucion]], [[razonamiento-critico-dominio]].
