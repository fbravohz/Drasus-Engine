# Third-Party API Gateway

**Carpeta:** `./features/third-party-api-gateway/`
**Estado:** En Diseño
**Última actualización:** 2026-07-03
**Decisión Arquitectónica Asociada:** ADR-0144 (cimiento #8) · ADR-0142 (gRPC/CLI) · ADR-0093 (seguridad)

## ¿Qué es esta feature?

La capa gRPC **pública** que permite a sistemas externos consumir Drasus: certificar estrategias, leer feeds de datos agregados, o rutear ejecución. Extiende el gRPC ya planificado (ADR-0142) añadiendo **autenticación y limitación de tasa**. Ahora se entregan los **contratos (protos por dominio) + auth**; el servidor público es un adaptador posterior.

- **Problema:** cada producto que se vende a terceros (Execution-as-a-Service, feeds, certificación) necesita una puerta de entrada autenticada y con control de abuso. Sin contrato común, cada uno inventa el suyo.
- **Comportamiento observable:** un tercero autenticado invoca un endpoint y recibe la respuesta del motor, con su uso medido y limitado.
- **Por qué:** convierte cada capacidad interna en un producto vendible por API sin reabrir el core.

## Comportamientos Observables

- Cuando un tercero se autentica con su credencial de API → obtiene acceso a los endpoints de su plan.
- Cuando supera su límite de tasa → recibe rechazo con "límite alcanzado", sin tumbar el motor.
- Cuando invoca certificación/feed/ruteo → el gateway delega en el puerto interno correspondiente y devuelve el resultado.
- Cuando la credencial se revoca → el acceso cesa de inmediato.

## Restricciones

- NUNCA un tercero accede al motor sin autenticación (mTLS + credencial de API).
- NUNCA el gateway expone datos crudos que violen consentimiento (respeta `consent-registry`).
- El gateway no computa: delega en los puertos internos.

## Parámetros Configurables (ADR-0008)

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| RATE_LIMIT_DEFAULT | (por plan) | 0 – ∞ | Solicitudes por ventana por credencial | CONFIG |
| AUTH_REQUIRED | true | true/false | Exigir autenticación | FIJO |
| ENDPOINTS_ENABLED | (conjunto) | conjunto | Qué endpoints se exponen | CONFIG |

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** validación de la solicitud, cómputo de la ventana de rate-limit.
- **Shell (Infraestructura):** servidor gRPC (tonic, ADR-0142), autenticación mTLS, delegación a los puertos internos, registro de uso.
- **Frontera Pública:** los protos por dominio (contratos públicos) — el contrato externo de todo el ecosistema.

## Ciclo de Vida de la Feature — Third-Party API Gateway

### Entrada
Una solicitud externa autenticada (certificar / feed / rutear) + la credencial de API.

### Proceso
Autentica, verifica rate-limit y consentimiento, delega en el puerto interno, mide el uso.

### Salida
La respuesta del puerto interno + el registro de uso de esa credencial.

## Tareas (TTRs)

- **TTR-001:** Protos por dominio de los endpoints públicos (contrato).
- **TTR-002:** Autenticación mTLS + credencial de API y rate-limit (Core: ventana).
- **TTR-003:** Delegación a los puertos internos respetando consentimiento.

## Puertos de Integración (ADR-0137)

| Puerto | ID de tipo | Dirección | Cardinalidad | Descripción |
|---|---|---|---|---|
| `api_request_in` | `ThirdPartyRequest` (tipo técnico nuevo — plomería, ADR-0144) | Input | `0..N` | Solicitud externa autenticada. |
| `api_response_out` | `ThirdPartyResponse` (tipo técnico nuevo — plomería, ADR-0144) | Output | `0..N` | Respuesta delegada del puerto interno. |

## Cáscara Visual (Thin Shell)

> Plomería (Ventana de Verificación). El UI-Designer escribe la nota de observable. El Architect NO rellena esta sección.

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016 enmendado por ADR-0143):** el gateway corre en el plano de ejecución o en la Cabina de Mando según el producto; nunca mueve cómputo al proveedor sin justificación.
- **Inundación de Fundaciones (ADR-0020):** Grupo I completo + **Perfil D (Ops/Auditoría)**: Identidad(I) + Soberanía(II: `owner_id`, `access_token_id`) + Hardware(IV: `node_id`).

## Persistencia (Inundación de Fundamentos — ADR-0020)

Tabla de credenciales de API + registro de uso (append-only) con Grupo I + Perfil D. Campos propios fuera del catálogo (marcados): endpoint invocado, ventana de rate-limit, credencial (referencia, nunca el secreto en claro — ADR-0093). `STRICT`, UUIDv7 (ADR-0141).

## Dependencias y Bloqueantes

- **Distinción de `saas-gateway` / `saas-cloud-engine` (moonshots):** NO confundir. El `saas-gateway` es el ingreso del cliente headless del **propio usuario** hacia su clúster de ejecución (auth/RBAC/rate-limit para su sesión, modo `SaaSCloudEngine` de ADR-0033). Esta feature (`third-party-api-gateway`) es la **API pública para terceros externos** (fondos, plataformas, bots) que consumen certificación/feeds/ejecución de Drasus. Ambos usan gRPC + auth + rate-limit, pero el sujeto y el propósito son distintos.
- **Depende de:** gRPC (ADR-0142), `central-identity`, `consent-registry`, `institutional-report-engine`, `data-aggregation`.
- **Bloquea a:** los productos vendidos por API (Execution-as-a-Service, feeds, certificación externa).
- **Contrato de Integración UI (ADR-0117) — Ventana de Verificación:** su observable (credenciales activas + conteo de solicitudes) queda visible en un panel de administración de API en ajustes; hasta entonces, deuda de integración registrada.
