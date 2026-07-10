---
name: drasus-portal-cabina-de-mando
description: "El repo hermano ../drasus-portal es la Cabina de Mando (Plano #3, servidor central + web pública), stack libre fuera del monolito; construcción diferida con disparador 'primer cobro real, tras EPIC-5'."
metadata: 
  node_type: memory
  type: reference
  originSessionId: 895b148d-9d25-4bc3-9f03-0e0f81cced43
---

La **Cabina de Mando Central** (Plano #3 de ADR-0143: autentica, licencia, factura, ingiere/agrega telemetría — **nunca computa** el trabajo del usuario) se materializa en el **repo hermano `../drasus-portal/`** (junto a `Drasus-Engine/`, no dentro). Es el servidor central del proveedor + su cara web pública (marketing/SEO, área de cliente, panel público de Cuentas Verificadas, marketplace).

**Stack libre, fuera del monolito:** NO hereda el invariante Rust+Flutter (ADR-0001 quedó **acotado al monolito** por ADR-0145); Next.js u otro es admisible. El repo **referencia** el diseño canónico de `Drasus-Engine/docs/` (ADR-0143/0144/0145/0093/0137), no lo redefine. Su `README.md` (esqueleto 2026-07-04) mapea cada sección del portal al cimiento del substrato que consume (#1–#10).

**Guardarraíl (FIJO, ADR-0093):** los secretos del usuario (credenciales de bróker, investor passwords, IPs live) JAMÁS llegan al portal, en ningún tier. El engine **empuja** al portal (identidad, licencia, telemetría por tier, track record atestado); el portal nunca jala cómputo ni accede a la DB del engine. El transporte exacto (gRPC/REST) y los DTO se definen al cimentar el **contrato de reporte** (`verified-account-registry` TTR-004) + `third-party-api-gateway` (#8).

**Estado en el ROADMAP:** existe como fila en el mapa de entregas ("Cabina de Mando", **sin nº de EPIC** por secuenciación) + en el bloque "Diferido — anclaje y disparadores". **Disparador de construcción: primer cobro real (tras EPIC-5).** Precondición: Spike gRPC (ADR-0142). Ver [[pricing-foundations-saas]] y [[arquitectura-visual-canvas]].
