## 22. Cabina de Mando Central y Substrato de Monetización

> Base: ADR-0143 (Tres Planos + Soberanía Condicionada por Tier) y ADR-0144 (Nueve Cimientos de Monetización). Esta sección describe la arquitectura de alto nivel; los contratos concretos viven en las Features del substrato (`docs/features/`).

### 22.1 Los Tres Planos

| Plano | Dónde corre | Responsabilidad | Qué NO hace |
|---|---|---|---|
| **UI** | Máquina del usuario (laptop) | Dibuja y orquesta; State-Driven | No computa, no decide |
| **Ejecución** | Hardware del usuario (PC local o su VPS headless, ADR-0033) | Todo el cómputo: backtesting, ejecución, motor, dato crudo. Manejado por la UI vía gRPC (ADR-0142/0106/0116) | — |
| **Cabina de Mando (Control)** | Servidor central del proveedor (Drasus) | Identidad, licencias, ingesta de telemetría, agregación de datos | **Nunca** ejecuta el motor ni órdenes; nunca está en la ruta crítica de la orden |

**Desambiguación (ADR-0143 vs ADR-0119):** el "Plano de Control" de ADR-0119 es del propio usuario (orquesta sus nodos satélite). La **Cabina de Mando** de esta sección es del proveedor. Coexisten.

### 22.2 Soberanía Condicionada por Tier

| Tier | Telemetría de trabajo | Dueño del dato | Canal de control |
|---|---|---|---|
| Gratuito | Firehose completo (estrategias, semillas, backtests, portafolios, resultados live/demo, instrumentos) | Proveedor (por ToS) | Obligatorio |
| Pago al corriente | **Suprimida en origen** (privacy-by-design) | Usuario | Obligatorio (solo licencia/heartbeat/anti-abuso) |
| Pago vencido | Reactivada (entorno no se borra) | Proveedor | Obligatorio |

**Guardarraíl transversal (FIJO):** secretos (credenciales de bróker, IPs de servidores live) nunca se exfiltran, en ningún tier (ADR-0093). El firehose captura el *trabajo*, no los *secretos*.

### 22.3 Los Nueve Cimientos del Substrato (ADR-0144)

Principio: *"construye la fontanería una vez, vende el agua de mil formas"* — Inundación de Fundaciones (ADR-0020 V2) aplicada al negocio. Cada cimiento = puerto tipado (ADR-0137) + esquema greenfield (ADR-0141/ADR-0020 V2) **ahora**; el producto concreto = adaptador **después**.

1. **Identidad y Cuenta Central** — cuenta, verificación de correo, identidad federada.
2. **Licenciamiento y Activación** — licencia por identidad, huella de hardware, activaciones por tier; gate local del ADR-0143.
3. **Plan / Tier / Cuota** — catálogo configurable de planes y límites.
4. **Medición de Uso / Libro de Nocional** — contador de valor nocional USD por ciclo (el motor ya lo calcula); habilita cobro por volumen o flat-fee.
5. **Consentimiento / ToS** — aceptación versionada y fechada, granular. Columna vertebral legal.
6. **Eventos de Dominio Enriquecidos** — tipos ricos sobre el bus (ADR-0085/0027): orden con fricción, backtest completado, régimen, drawdown, liquidez, correlación, licencia, registro.
7. **Motor de Reportes Institucionales** — puerto + plantilla base; habilita stress test, validación, forense, certificación.
8. **API Pública de Terceros** — gRPC público (extiende ADR-0142) con auth + rate limit.
9. **Anonimización y Agregación** — puerto que anonimiza (ADR-0102) y agrega en índices vendibles.

### 22.4 Mapa de dependencias

El **Bus de Eventos Enriquecido** (#6) es la raíz: sin eventos estructurados no hay telemetría, ni agregación, ni reportes, ni billing. De él cuelgan el Audit Trail inmutable (ya construido), la Anonimización (#9) y el Licenciamiento (#2). Sobre esos se apoyan Reportes (#7), API de Terceros (#8) y Facturación (parte de #3/#4). Multi-tenancy real vive solo en la Cabina de Mando (no calcada en tablas locales; se reutiliza `owner_id`/`institutional_tag`, ADR-0020 V2 Grupo II).

### 22.5 Clasificación de modelos de monetización

- **Cimiento ahora / adaptador después:** los nueve + reportes institucionales (stress test, validación-herramienta, certificación, forense) + feeds agregados (régimen, fricción de bróker, correlación, liquidez).
- **Moonshots (etiquetados "zizaña"):** PFOF/venta de flujo, Capital Allocation Platform, firma de validación regulatoria acreditada.
- **Descartado (veneno reputacional):** venta de flujo retail identificable para front-running contra los propios usuarios.
- **Ya diseñados (asegurar que emitan a estos puertos):** La Colmena, Marketplace de Cajas Negras, Copy-Trading, Collective Intelligence, Transpiler, Microestructura L3.

### 22.6 Pilar de Cuentas Verificadas Drasus (cimiento #10, ADR-0145)

Décimo cimiento del substrato: un pilar de producto análogo a **myFXbook / MT5 Signals** con un diferenciador **soberano**. Donde esos comparables confían en la conexión read-only al bróker, Drasus **atestigua criptográficamente lo que su propio motor ejecutó** (cadena de hash + append-only del audit-log ya construido). Contrato/esquema **ahora**; portal **después**.

**Tres piezas:**
1. **Atestación soberana de ejecución** — Drasus certifica la porción de actividad que fluyó por su módulo `execute` (FillEvent conciliado + PreTradeVerdict, encadenado). Etiqueta **"Ejecución Verificada por Drasus"**.
2. **Registro multi-cuenta** (`verified-account-registry`) — una identidad Drasus (`owner_id`) → N cuentas (fondeo/prop/ICMarkets/Binance/IBKR), cada una con su track record y ámbito de atestación.
3. **Track record publicable opt-in** — el usuario publica (o no) las estadísticas de una cuenta en el portal público.

**Dos ámbitos de atestación por cuenta (coexistentes):** (i) *soberana* (ejecución propia, atestada por hash chain) y (ii) *read-only del bróker* (cuenta-completa, estilo myFXbook, computada en el **Plano de Ejecución del usuario**). La investor password/credencial **nunca** sube a la Cabina de Mando (ADR-0093).

**Quinta clase de telemetría (enmienda a ADR-0143):** "track record publicable" — resultados sin la lógica generadora, **opt-in por cuenta, independiente del tier**. Protege el *trabajo/PI*, permite publicar *resultados*. Identificable por diseño → NO pasa por la anonimización de #9.

**Enriquecimiento del cimiento #6 (necesario para reconstruir la pista):** dos eventos nuevos — **flujo de capital** (depósito/retiro/transferencia; imprescindible para gain% sin depósitos) y **snapshot de estado de cuenta** (equity/balance/margen; alimenta las curvas) — más el refuerzo de la orden-con-fricción con `account_id`/PnL/MAE/MFE/duración.

**Portal público (repo aparte, stack libre, diferido):** el portal (`drasusengine.com/<ruta-por-decidir>`) y la Cabina de Mando web **NO** forman parte del monolito Rust+Flutter; viven en un **repositorio separado** con su propio stack a decidir (Next.js u otro para SEO es admisible). El invariante ADR-0001 se acota al monolito (motor + UI). Ahora solo se construye, dentro del monolito, lo que reportará al portal: eventos de #6, registro #10, consentimiento #5 y el contrato/puerto de reporte.

---

**Documento versión 1.1** | Creado: 2026-07-03 (ADR-0143 / ADR-0144) · §22.6 añadida 2026-07-04 (ADR-0145)
