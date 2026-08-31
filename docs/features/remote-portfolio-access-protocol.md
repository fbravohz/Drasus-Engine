# Remote Portfolio Access Protocol (RPAP)

**Carpeta:** `./features/remote-portfolio-access-protocol/`
**Estado:** En Diseño
**Última actualización:** 2026-06-04
**Decisión Arquitectónica Asociada:** ADR-0090

## 1. ¿Qué es esta feature?
Protocolo de acceso remoto autenticado con seguridad a nivel de campo (Field-Level Security). Expone una interfaz analítica de solo lectura (Read-Only) para que empleados o instancias esclavas (copiers) validen su trabajo remotamente contra un clúster maestro sin que la lógica interna de las estrategias sea nunca expuesta.

## Narrativa de Producto

**Capacidad:** `CAP-06` — Ejecución real y defensa del capital

**Historia de usuario:** Como operador que dirige un equipo de sub-traders o empleados, quiero que puedan validar remotamente su portafolio contra el capital principal sin que jamás vean la lógica interna de mis estrategias, para poder delegar sin regalar el activo más valioso del negocio.

**El problema (técnico):** dar acceso a un empleado para que valide su trabajo contra el portafolio maestro normalmente implica exponerle datos operativos completos, y entre esos datos, sin darse cuenta, se filtra la lógica de la estrategia —el árbol de reglas, los parámetros de optimización—, que es la propiedad intelectual real del negocio.

**La solución (técnica):** un protocolo de acceso remoto de solo lectura que enmascara dinámicamente los campos sensibles antes de que salgan del servidor — nunca exporta el árbol de reglas ni los parámetros de optimización, solo resultados agregados (correlación, drawdown cruzado). Cada consulta requiere un token con permisos acotados, se limita en volumen y velocidad, y queda registrada de forma inmutable; revocar el token corta el acceso al instante.

**Explicado sin tecnicismos:** es como dejar entrar a un contador a revisar los números de tu negocio sin darle la fórmula secreta de tu producto. Puede ver cuánto vendiste y compararlo contra el presupuesto, pero nunca ve la receta.

Esta pieza le da a cada persona de tu equipo exactamente esa vista: puede comparar su desempeño contra el portafolio principal y confirmar que su trabajo aporta algo distinto, pero jamás se lleva a casa el secreto que hace funcionar las estrategias.

## 2. Comportamientos Observables
- [ ] Empleado conecta su Drasus Engine local al clúster maestro usando un token JWT provisto por el administrador.
- [ ] Empleado solicita el histórico de trades del portafolio maestro. El sistema retorna los trades enmascarando campos sensibles (`strategy_id`) y omitiendo por completo el AST.
- [ ] Empleado solicita una matriz de correlación temporal cruzada; el clúster maestro computa la métrica y devuelve exclusivamente el resultado numérico.
- [ ] El administrador revoca un token JWT comprometido; el acceso del empleado se corta inmediatamente.

## 3. Restricciones
- NUNCA se exporta el JSON AST o los parámetros de optimización de las estrategias del clúster maestro.
- Límite estricto de Rate Limiting por token para prevenir ataques de extracción de datos.
- Toda consulta realizada se registra incondicionalmente en un Audit Log inmutable.

## 4. Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| RPAP_RATE_LIMIT | 1000 | 100-5000 | Máximo de requests por minuto por token | CONFIG |
| MAX_ROWS_PER_QUERY | 10000 | 1k-100k | Máximo de filas operativas exportables por llamada | CONFIG |
| TOKEN_EXPIRY_DAYS | 30 | 1-365 | Periodo de validez en días para la expiración automática del JWT | CONFIG |

## 5. Estructura Interna (FCIS)
- **Core:** Motores de enmascaramiento dinámico (Field Masking) y funciones puras para el cálculo de distancias/correlación.
- **Shell:** Servidor de red en puerto perimetral (gRPC vía `tonic` / HTTP vía `axum`), gestión de autenticación JWT y persistencia de accesos en SQLite.
- **Frontera Pública:** Endpoints expuestos (Ej. `/rpap/v1/trades/range`, `/rpap/v1/analytics/correlation`).

## 6. Ciclo de Vida de la Feature

### Entrada
- Petición de red (HTTP/gRPC) que incluye un token JWT válido y los scopes requeridos.
- Payload con parámetros de filtrado (ventanas temporales, métricas a comparar).

### Proceso
- Verifica la criptografía del token y su vigencia.
- Intercepta la consulta dirigida a la base de datos transaccional e inyecta dinámicamente la capa de Field Masking.
- Computa agregaciones cruzadas (ej. correlación de curva de equidad).
- Escribe de forma asíncrona en el log de auditoría.

### Salida
- Payload serializado en Parquet/JSON que contiene exclusivamente los datos operativos permitidos.

### Contextos de Uso
**Contexto 1: Validación Pre-Absorción por Sub-Traders**
- Entrada: Portafolio del sub-trader vs Portafolio del clúster maestro.
- Preguntas que responde: "¿La nueva estrategia aporta verdadera diversificación o solo solapa riesgos?"
- Impacto: Permite justificar matemáticamente la inclusión de una nueva estrategia en el portafolio principal de capital.

## 7. Tareas (TTRs)

### **TTR-001: Implementación de Autenticación y Scopes (JWT)**
- Establecer validación de tokens y mapeo de scopes (ej. `trades:read_operational`, `analytics:correlation`).

### **TTR-002: Motor de Enmascaramiento de Datos (Field Masking)**
- Filtrar de manera proactiva campos prohibidos y censurar el `strategy_ast` antes de la serialización hacia la capa de red.

### **TTR-003: Construcción de Analíticas Comparativas Remotas**
- Desarrollar las interfaces para responder a peticiones de correlación temporal, drawdown cruzado y detección de solapamientos.

### **TTR-004: Trazabilidad y Auditoría de Acceso**
- Persistencia inmutable de todas las queries realizadas a través de RPAP en la tabla relacional `rpap_access_log`.

## 8. Gobernanza y Estándares (Fijos)
- **Local-First (ADR-0016):** El servidor maestro opera en la infraestructura física del dueño, accesado a través de túneles SSH/VPN. Cero delegación a SaaS de terceros.
- **Inundación de Fundaciones (ADR-0020):**
  - **Perfil: Ops / Auditoría.** 
  - Foco en la trazabilidad inmutable. Las tablas de acceso de RPAP DEBEN inyectar: `id` (UUID), `created_at` (Nanosegundos), `updated_at`, `audit_hash`, `audit_chain_hash`, `event_sequence_id`, `owner_id`, `institutional_tag`, `access_token_id`, y `node_id`.

## 9. Dependencias y Bloqueantes
**Depende de:**
- [`audit-log`](../features/audit-log.md) (Rastreo irrefutable de queries y accesos).
- [`portfolio-backtest`](../features/portfolio-backtest.md) (Fuente de verdad de los datos operativos e históricos a compartir).
