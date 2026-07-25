---
name: despacho-no-precocinar-solucion
description: "Al despachar un subagente especialista, entrega el problema y las restricciones — nunca la solución precocinada; si la metes en el prompt, el agente la ejecuta en vez de retarla."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 17a4ae3a-e845-4b19-aa68-2971382567d0
  modified: 2026-07-24T01:06:18.762Z
---

Cuando el Tech-Lead despacha un especialista (Quant, Architect, ingeniero), **entrega el problema y las restricciones, nunca el método/solución ya masticado**. Un subagente Sonnet tiende a ejecutar la premisa que recibe, no a retarla — el criterio senior no se pierde en el agente, se pierde en cómo se enmarca el encargo.

**Caso raíz (2026-07-22/23):** el Tech-Lead despachó Quant + Architect para el frente de ingesta y ambos entregaron trabajo defectuoso que el propietario cazó:
- Al **Quant** le escribió en el prompt que abril 2025 era el "ground truth" de calibración y que el DST se anclaba con la noticia de Nóminas cruzada con el pico de volatilidad. El Quant ató toda la política de spread a un dataset fugaz y toda la zona horaria a una noticia — porque se lo puse en la mano. Correcto: piso de spread por instrumento (`default_spread` del catálogo + distribución del propio archivo); zona horaria **declarada** (selector mundial con bandera DST, patrón SQX), nunca inferida de volatilidad.
- Al **Architect** le enmarcó el encargo como "modelo de entidades / normalización de tablas" y devolvió un esquema de tablas, incumpliendo el invariante de que toda lógica se productiza en features hexagonales (ADR-0137). Correcto: features (`instrument-catalog`, `session-catalog`, capa de `pit-data-validator`) con puertos tipados; las tablas son su persistencia interna.

Descartados como causa (con evidencia): no fue capacidad de Sonnet (ambos corrieron las 7 comprobaciones y añadieron caveats), no fue deterioro de contexto (agentes frescos), no fue la división de `governance.md` (ambos la declararon). Causa primaria: el encuadre del despacho.

**Why:** el propietario exige criterio senior en el equipo y detectó que "mucho falla porque no leen la doc" — pero aquí el fallo fue aguas arriba, en el prompt del Tech-Lead, no en la lectura del agente. Precocinar la solución neutraliza el criterio del especialista.

**How to apply:** en cada despacho, da (1) el problema, (2) las restricciones/invariantes duros, (3) la doc a consultar, (4) el entregable y su forma — pero deja que el especialista **derive el método**. Si crees saber la solución, ponla como "una hipótesis a validar o refutar", no como el camino.

Dos planos, dos reglas canónicas: la contraparte del que despacha vive en `tech-lead/SKILL.md` §"Antes de despachar" (no precocinar el prompt); la del que recibe se canonizó como **`GOVERNANCE-4.7 — Reta la Premisa Recibida`** (imperativa para TODO agente): un agente senior empuja contra una premisa masticada en vez de ejecutarla a ciegas. Relacionado: [[roles-explicitos-y-subagentes]], [[razonamiento-critico-dominio]].
