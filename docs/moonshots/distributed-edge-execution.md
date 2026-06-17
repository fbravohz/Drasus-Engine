# Ejecución Distribuida en el Borde (Edge Execution / Central Control)

## 1. Visión del Moonshot

Topología para operar **varios brókers en vivo simultáneamente** desde un único panel de control, colocando un **nodo satélite por bróker** (opcionalmente geo-localizado cerca del bróker) que ejecuta de forma autónoma, mientras el operador conserva su panel Local-First que ve la flota **como un portafolio**.

No es una arquitectura nueva inventada desde cero: **compone** decisiones que el sistema ya tiene, formalizadas en **ADR-0119** (separación Plano de Control / Plano de Ejecución). Su valor primario es **resiliencia, autonomía 24/7 y aislamiento de fallos** — la latencia es un beneficio secundario, no la bandera (ver §5).

## 2. Topología

```
[ Panel Central (Local-First, operador) ]
        │  gRPC/mTLS (comandos)  +  pub/sub (telemetría)  — NUNCA en la ruta de orden
        ├──────────────► [ Nodo satélite · Bróker A ] ── conector/terminal ──► Bróker A
        ├──────────────► [ Nodo satélite · Bróker B ] ── conector/terminal ──► Bróker B
        └──────────────► [ Nodo satélite · Bróker C ] ── conector/terminal ──► Bróker C
                                   │ (cada nodo)
                                   ▼ snapshot asíncrono
                            [ S3 / Cloudflare R2 ]  (cold storage por nodo, DR)
```

- **Plano de Control (operador):** orquesta, ve telemetría consolidada (PnL, estado, métricas) y emite comandos (`StopNode`, `SuspendStrategy`, `Rebalance`). Es Local-First; puede apagarse sin detener la operación.
- **Plano de Ejecución (cada nodo):** instancia headless de Drasus (ADR-0033 modo SaaSCloudEngine) anclada a UN bróker. Ejecuta autónoma; si pierde el enlace, sigue gestionando posiciones bajo reglas de riesgo locales.

## 3. Piezas que ya existen (esto es composición, no invención)

| Capacidad | Ya cubierta por |
|---|---|
| Motor headless remoto + UI local por gRPC | ADR-0033 (modo SaaSCloudEngine) |
| Daemon de ejecución remoto, autónomo al desconectarse, reconciliación de ledger | ADR-0094 (HybridComputeCooperative) |
| Reconciliación de estado contra el bróker al rearrancar | [`crash-recovery`](../features/crash-recovery.md) / ADR-0027 |
| Sync SQLite→S3 + rehidratar nodo desde cold storage | [`saas-cloud-engine`](saas-cloud-engine.md) §3 |
| Telemetría/consulta remota segura (JWT, masking, audit) | [`remote-portfolio-access-protocol`](../features/remote-portfolio-access-protocol.md) / ADR-0090 |
| Bus pub/sub de eventos/telemetría | ADR-0085 (zero-copy, sin NATS/MQTT nuevo) |
| Conectores por bróker (trait `BrokerConnector`) | [`broker-connector`](../features/broker-connector.md) + [`nautilus-integration`](../features/nautilus-integration.md) / ADR-0107 |
| Bridge para brókers MT5-only | [`multiplatform-execution-bridge`](../features/multiplatform-execution-bridge.md) |
| Vista de "portafolio" sobre nodos | ADR-0090 (federated portfolio) |
| mTLS / cifrado de credenciales | [`sovereign-security`](../features/sovereign-security.md) / ADR-0093 |
| **Topología de satélites por bróker + split control/ejecución** | **ADR-0119 (esta decisión)** |

## 4. Recuperación ante Desastres (Cold Storage)

1. **Hot path:** el nodo escribe en SQLite WAL local (<1ms).
2. **Snapshot en frío:** cada N minutos (o al cierre de sesión) un hilo secundario sube un snapshot comprimido a S3/R2 con clave por `node_id` (asíncrono, el hilo de trading no se entera).
3. **Rehidratación:** si un nodo muere, se levanta una instancia limpia que descarga su último snapshot, consulta al bróker el balance/posiciones reales, **reconcilia** (ADR-0027) y reanuda. Si la divergencia es insalvable, entra en `EMERGENCY_LOCK` y notifica.

## 5. Latencia: el encuadre correcto

La justificación **no** es HFT sub-milisegundo. Para estrategias retail/prop-firm swing/algo, 45–60 ms desde Guadalajara hacia Nueva York ya es aceptable (el KPI de EPIC-5 es ≤100ms end-to-end).

Lo que el modelo distribuido sí resuelve:

- **Saca el roundtrip del operador de la ruta de orden.** Co-localizando el nodo satélite + el terminal MT5 cerca del bróker, la orden viaja `nodo→bróker` (salto corto), no `casa→bróker`. El enlace del operador solo lleva control/telemetría.
- **Hace viable el bridge MT5** para brókers que no ofrecen REST/FIX, sin penalización geográfica: el "costo del bridge" que se temía era sobre todo red (casa→bróker), que la co-location elimina; queda solo el overhead interno de MT5, pequeño.
- **24/7 sin depender de la luz/internet de casa** (el verdadero punto único de fallo) y **aislamiento de fallos por bróker**.

La co-location real con FIX (sub-ms) es otra liga: aplica solo al `broker-connector` nativo y solo si una familia de estrategias lo exige.

## 6. Seguridad

- **mTLS** en ambos extremos del canal de control: el nodo solo acepta comandos firmados por la llave del Plano de Control, bloqueando escaneos maliciosos.
- **Credenciales del bróker cifradas en reposo** y/o inyectadas en memoria por el canal seguro al arranque, nunca en texto plano (ADR-0093).

## 7. Estado de Investigación

*Fase actual: Ideación Teórica (Moonshot), EPIC-9+.*

Compuerta **Client Zero**: no iniciar la codificación de esta infraestructura hasta que el Core Local FFI sea estable y genere Alpha real (misma regla que [`saas-cloud-engine`](saas-cloud-engine.md) §5). El modo monomáquina LocalPowerUser (ADR-0033) es el default y cubre la operación hasta entonces.

**Decisión arquitectónica asociada:** ADR-0119. Comparte patrón y compuerta con `saas-cloud-engine`, La Colmena (ADR-0086) y el portafolio federado (ADR-0090).
