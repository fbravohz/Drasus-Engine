## 22. Cabina de Mando Central y Substrato de Monetización

> Base: ADR-0143 (Tres Planos + Soberanía Condicionada por Tier) y ADR-0144 (Nueve Cimientos de Monetización). Esta sección describe la arquitectura de alto nivel; los contratos concretos viven en las Features del substrato (`docs/features/`).

### 22.1 Los Tres Planos

| Plano | Dónde corre | Responsabilidad | Qué NO hace |
|---|---|---|---|
| **UI** | Máquina del usuario (laptop) | Dibuja y orquesta; State-Driven | No computa, no decide |
| **Ejecución** | Hardware del usuario (PC local o su VPS headless, ADR-0033) | Todo el cómputo: backtesting, ejecución, motor, dato crudo. Manejado por la UI vía gRPC (ADR-0142/0106/0116) | — |
| **Cabina de Mando (Control)** | Servidor central del proveedor (Drasus) | Identidad, licencias, ingesta de telemetría, agregación de datos | **Nunca** ejecuta el motor ni órdenes; nunca está en la ruta crítica de la orden |

**Desambiguación (ADR-0143 vs ADR-0119):** el "Plano de Control" de ADR-0119 es del propio usuario (orquesta sus nodos satélite). La **Cabina de Mando** de esta sección es del proveedor. Coexisten.

**Relé cifrado genérico (ADR-0143, añadido 2026-07-06):** toda instancia mantiene una conexión saliente autenticada hacia la Cabina de Mando — eso resuelve NAT/IP dinámica sin abrir puertos entrantes. Ese mismo canal se generaliza a un relé de mensajería dirigida, cifrada E2E (zero-knowledge): un solo componente sirve para telemetría/licencia (esta sección), señales de copy-trading (ADR-0092) y comandos de override fondo→cuenta hija (§22.8). Un nodo con IP pública estable (VPS headless, ADR-0119) sigue alcanzándose directo; el relé aplica cuando el destino es una máquina residencial sin dirección fija.

### 22.2 Soberanía Condicionada por Tier

| Tier | Telemetría de trabajo | Dueño del dato | Canal de control |
|---|---|---|---|
| Gratuito | Firehose completo (estrategias, semillas, backtests, portafolios, resultados live/demo, instrumentos) | Proveedor (por ToS) | Obligatorio |
| Pago al corriente | **Suprimida en origen** (privacy-by-design) | Usuario | Obligatorio (solo licencia/heartbeat/anti-abuso) |
| Pago vencido | Reactivada (entorno no se borra) | Proveedor | Obligatorio |

**Guardarraíl transversal (FIJO):** secretos (credenciales de bróker, IPs de servidores live) nunca se exfiltran, en ningún tier (ADR-0093). El firehose captura el *trabajo*, no los *secretos*.

### 22.3 Los Nueve Cimientos del Substrato (ADR-0144)

Principio: *"construye la fontanería una vez, vende el agua de mil formas"* — Inundación de Fundaciones (ADR-0020) aplicada al negocio. Cada cimiento = puerto tipado (ADR-0137) + esquema greenfield (ADR-0141/ADR-0020) **ahora**; el producto concreto = adaptador **después**.

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

El **Bus de Eventos Enriquecido** (#6) es la raíz: sin eventos estructurados no hay telemetría, ni agregación, ni reportes, ni billing. De él cuelgan el Audit Trail inmutable (ya construido), la Anonimización (#9) y el Licenciamiento (#2). Sobre esos se apoyan Reportes (#7), API de Terceros (#8) y Facturación (parte de #3/#4). Multi-tenancy real vive solo en la Cabina de Mando (no calcada en tablas locales; se reutiliza `owner_id`/`institutional_tag`, ADR-0020 Grupo II). La jerarquía organizacional (#12, §22.8) es la primera extensión consciente de esa regla: `parent_owner_id` es solo un puntero cacheado localmente (mismo idioma que `parent_id`, Grupo III), nunca una tabla de árbol completo — el árbol sigue viviendo en el plano central.

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

**Corrección 2026-07-06 (ADR-0145):** el modelo de atestación de #10 no es "dos ámbitos" sino **dos ejes ortogonales** por cuenta — Eje A (autoría: `SOVEREIGN`/`BROKER_READONLY`) y Eje B (realidad del capital: `LIVE`/`PAPER`/`DEMO`/`CHALLENGE`). Una cuenta `PAPER`/`CHALLENGE` corre en el mismo entorno determinista que `LIVE` (no backtesting) y es igualmente atestiguable; el rótulo público siempre muestra ambos ejes juntos, nunca uno solo.

**Corrección 2026-07-07 (auditoría de Inundación de Fundaciones, ADR-0020):** el Eje B NO es un campo propio de #10 — es `institutional_tag` (Grupo II, universal, ya usado en 30+ features y los 8 módulos con el mismo dominio `PROD`/`PAPER`/`CHALLENGE`) con su vocabulario extendido a `LIVE`/`PAPER`/`DEMO`/`CHALLENGE`. El código ya escrito (STORY-038) lo implementó como columna nueva `capital_reality` duplicando `institutional_tag` en la misma tabla — retrabajo pendiente de consolidación (dominio Tech-Lead).

### 22.7 Continuidad y Portabilidad de Instancia (cimiento #11, ADR-0146)

Undécimo cimiento: promueve el moonshot [`instance-continuity`](../features/instance-continuity.md) (antes `encrypted-local-backup`) de adaptador diferido a cimiento. Dos piezas:

1. **Respaldo cifrado client-side** de la base de datos local hacia el almacén de objetos del proveedor — el proveedor jamás puede leerlo (ADR-0093).
2. **Maestro itinerante (relevo de custodia):** la misma cuenta maestra opera alternadamente desde varias máquinas activadas, cediendo/reclamando la titularidad de la cadena de auditoría al cerrar/abrir sesión — nunca dos máquinas escriben la misma cadena a la vez. Si la máquina anterior murió, la titularidad se libera desde el panel de cuenta central (mismo flujo self-service que liberar un cupo de activación en `licensing-system`).

Motivo de urgencia: el pilar #10 ya genera datos irremplazables (firma de atestación soberana); sin este cimiento, un disco muerto en tier Sovereign los pierde para siempre.

### 22.8 Jerarquía Organizacional de Cuentas Maestras (cimiento #12, ADR-0147)

Duodécimo cimiento: una **cuenta maestra raíz** (fondo) agrupa N **cuentas maestras hijas** (traders/desks), cada una maestro autónomo por derecho propio (Plano de Control ADR-0119 con sus propios satélites), pero con la raíz teniendo autoridad de auditoría y override total sobre ella. Generaliza el patrón de mando de ADR-0119 un nivel arriba: el "satélite" ahora es otra cuenta maestra completa, no un nodo sin criterio propio.

Reglas fijas: (1) la jerarquía vive en el plano central (`parent_owner_id` cacheado, anti-`tenant_id`); (2) el mando viaja por el relé genérico (§22.1), nunca escritura directa; (3) la autoridad del fondo es un consentimiento contractual vigente (#5), no un backdoor; (4) todo override queda **atestado en ambos extremos** (preserva la integridad de #10); (5) "eliminar" siempre archiva, nunca DELETE físico (ADR-0141); (6) la hija conserva su propio Plano de Control — la jerarquía es una capa encima, no un reemplazo. Ver [`master-account-hierarchy`](../features/master-account-hierarchy.md).

### 22.9 Portabilidad y Exportación de Datos del Usuario (cimiento #13, ADR-0148)

Decimotercer cimiento, producto de la auditoría de cobertura del substrato (barrido de las ~154 features y ~54 moonshots buscando cimientos faltantes, 2026-07-07). Obligación legal transversal (GDPR Art. 15/17/20), no producto: catálogo declarativo de qué tablas tienen `owner_id` (auto-poblado por cada feature nueva, mismo mecanismo de detección→elevación de ADR-0020) + registro append-only de solicitudes de exportación/olvido (Perfil D). El generador real del archivo de export es el adaptador diferido; lo que se cimenta ahora es barato porque `owner_id` ya es universal.

**Distinción obligatoria (no confundir):** el blob de `instance-continuity` (#11) es **opaco** — ni el proveedor ni un tercero pueden leerlo, sirve para disaster recovery entre máquinas del mismo usuario. El export de #13 es **legible y estructurado**, sirve para ejercer un derecho legal. El agregado anónimo de `data-aggregation` (#9) sale **hacia terceros** sin identidad; el export de #13 sale **hacia el mismo dueño** con toda su identidad. Tres cimientos, tres propósitos, ningún solape.

**Olvido con excepción de retención:** los registros con obligación legal de retención (auditoría financiera, atestaciones soberanas de #10, libro de nocional de #4) se **pseudonimizan** (se desvincula el `owner_id` identificable) en vez de purgarse físicamente — misma disciplina de "eliminar = archivar" que el resto del substrato (ADR-0141).

### 22.10 Dos Categorías de Consentimiento (corrección de `consent-registry` #5, 2026-07-07)

El registro de consentimiento (#5) distingue dos bases legales que **no son intercambiables**: (1) **gate de tier** — el firehose de trabajo del tier gratuito y el control/licencia de todos los tiers son la contraprestación contractual del tier elegido (Art. 6(1)(b) GDPR, "necesario para el contrato"), binario, sin opt-out granular dentro del tier — la alternativa real es pagar; (2) **consentimiento genuino** — categorías no necesarias para el servicio (ej. publicación del track record de #10, Clase 5 de telemetría), siempre opt-in real, revocable, y **nunca** puede condicionar el acceso al servicio (Art. 7(4) GDPR prohíbe el "bundling"). El campo `optout_map` del registro modela solo la categoría 2; la categoría 1 se resuelve en `licensing-system`/`plan-tier-quota` según el tier del usuario, no en `consent-registry`.

### 22.11 Roles de Operador a la Carta (cimiento #14, ADR-0149)

Decimocuarto cimiento: dentro de **una sola** cuenta maestra, un catálogo de roles custom por cuenta (nombre libre + matriz de capacidades permitido/denegado) asignable a operadores humanos o a conexiones MCP de agentes LLM bajo el mismo mecanismo — nunca un sistema de permisos separado para IA. **Ancla de capacidad: el puerto de Frontera Pública de cada Feature (ADR-0137), nunca el módulo** — los módulos son preset de composición, no dueños, y un rol custom debe poder otorgar/negar feature por feature aunque el usuario arme flujos de trabajo que ignoren la agrupación por módulo.

**Tres compuertas independientes, todas necesarias para una acción:** (1) ¿la cuenta tiene la capacidad? — `master-account-hierarchy` #12; (2) ¿el operador tiene el rol? — este cimiento; (3) ¿el pipeline/dato lo permite? — `agentic-mcp-gateway`/ADR-0123 (riesgo de pipeline + `institutional_tag`), que este cimiento **extiende** con el insumo de rol de operador, sin reemplazar su compuerta existente.

**Invariante "último admin en pie" (corregido 2026-07-07):** ningún operador queda congelado como admin de por vida — el `owner_id` raíz es el primer admin por defecto y puede designar otros, y cualquiera (incluido él) es reasignable a otro rol siempre que quede al menos un operador más con la capacidad "gestionar operadores y roles" tras el cambio. El guardarraíl protege la *capacidad*, no la *persona* — cubre tanto reasignar a alguien como editar la matriz de un rol para quitarle esa capacidad.

**Cascada de autoridad y dónde vive cada cosa:** el catálogo de roles y las asignaciones son **fuente de verdad en la Cabina de Mando** (cruza máquinas de distintas personas, igual que #1/#12); el motor local cachea el veredicto (`ROLE_CACHE_TTL`, mismo patrón que `IDENTITY_CACHE_TTL`) y el evaluador de permisos corre en local contra esa caché — ninguna acción espera un viaje de ida y vuelta al servidor. La cuenta maestra raíz de un fondo puede ver/cambiar/revocar cualquier asignación de rol de sus cuentas hijas (incluidas las de sus LLMs) vía el mismo canal de override + doble atestación de #12 — no un canal nuevo. Solo un operador con capacidad ADMIN crea cuentas hijas nuevas, y el máximo de cuentas hijas es una cuota de suscripción (`MAX_CHILD_ACCOUNTS`, `plan-tier-quota` #3) fijada por Drasus, nunca por el fondo.

---

**Documento versión 1.5** | Creado: 2026-07-03 (ADR-0143 / ADR-0144) · §22.6 añadida 2026-07-04 (ADR-0145) · §22.7/§22.8 añadidas 2026-07-06 (ADR-0146/ADR-0147) + corrección de atestación de §22.6 · corrección de Inundación de Fundaciones en §22.6 (`institutional_tag` reutilizado) 2026-07-07 · §22.9/§22.10 añadidas 2026-07-07 (ADR-0148, cimiento #13 + corrección de dos categorías de consentimiento) · §22.11 añadida 2026-07-07 (ADR-0149, cimiento #14)
