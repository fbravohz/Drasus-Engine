sp# Central Identity

**Carpeta:** `./features/central-identity/`
**Estado:** En Diseño
**Última actualización:** 2026-07-03
**Decisión Arquitectónica Asociada:** ADR-0143 (Tres Planos) · ADR-0144 (Substrato de Monetización, cimiento #1)

## ¿Qué es esta feature?

La cuenta del usuario en la **Cabina de Mando Central** del proveedor (ADR-0143). Verifica identidad (correo + identidad federada OAuth) y produce un identificador de cuenta estable que ancla licencias, atribución de telemetría y anti-abuso.

- **Problema:** sin identidad central no se puede licenciar, ni segmentar tiers, ni atribuir el firehose de datos gratuitos, ni prevenir el abuso multi-cuenta.
- **Comportamiento observable:** el usuario se registra/inicia sesión; el motor local queda vinculado a esa cuenta.
- **Por qué:** es la raíz de todo cobro y de toda gobernanza de datos.

## Comportamientos Observables

- Cuando el usuario se registra con correo → el sistema envía verificación y no activa la cuenta hasta confirmarla.
- Cuando el usuario inicia sesión con identidad federada (Google/GitHub) → el sistema crea o vincula la cuenta a esa identidad.
- Cuando el motor local arranca → consulta a la Cabina de Mando la identidad vinculada y la cachea para operación offline.
- Cuando se crean N identidades desde el mismo hardware → se marcan para revisión anti-abuso (señal, no bloqueo automático).

## Restricciones

- NUNCA se almacena la contraseña en texto plano (hash con sal, estándar).
- NUNCA se exfiltran secretos de bróker ni IPs de servidores live junto a la identidad (ADR-0093).
- La cuenta es del plano central; el motor local nunca es fuente de verdad de identidad — solo cachea.

## Parámetros Configurables (ADR-0008)

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| IDENTITY_CACHE_TTL | 24 h | 1 h – 30 d | Cuánto vale la identidad cacheada sin revalidar | CONFIG |
| EMAIL_VERIFICATION_REQUIRED | true | true/false | Exigir verificación de correo antes de operar | FIJO |
| OAUTH_PROVIDERS | Google, GitHub | conjunto | Proveedores de identidad federada aceptados | CONFIG |

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** validación de formato de correo, verificación de firma de token OAuth, cálculo de la huella de hardware (hash determinista de identificadores de máquina).
- **Shell (Infraestructura):** persistencia de la cuenta, envío de verificación, llamada gRPC a la Cabina de Mando, caché local.
- **Frontera Pública:** puerto que responde "¿quién es el dueño de esta instancia?" y expone el identificador de cuenta a las demás features del substrato.

## Ciclo de Vida de la Feature — Central Identity

### Entrada
Credenciales del usuario (correo+contraseña o token OAuth) y los identificadores de la máquina.

### Proceso
Verifica la identidad contra la Cabina de Mando, deriva la huella de hardware, y vincula la instancia local a la cuenta.

### Salida
Un identificador de cuenta estable (`owner_id`, ADR-0020 V2 Grupo II) cacheado localmente, con su estado de verificación.

## Tareas (TTRs)

- **TTR-001:** Registro y verificación de correo contra la Cabina de Mando.
- **TTR-002:** Vinculación de identidad federada (OAuth) y derivación de huella de hardware.
- **TTR-003:** Caché local de identidad con TTL y operación offline.

## Puertos de Integración (ADR-0137)

| Puerto | ID de tipo | Dirección | Cardinalidad | Descripción |
|---|---|---|---|---|
| `identity_out` | `AccountIdentity` (tipo técnico nuevo — plomería, ADR-0144) | Output | `1` | Identidad de cuenta vinculada a la instancia; consumida por `licensing-system`, `usage-metering`, `consent-registry`. |

> Tipo técnico nuevo del substrato (no de dominio del canvas), análogo a `AuditEvent`/`TelemetrySample`. Extiende el catálogo de ADR-0137 vía ADR-0144.

## Cáscara Visual (Thin Shell)

> Pendiente de la Etapa 0.5 (UI-Designer). Superficie prevista: panel de cuenta/sesión en el cajón de ajustes (Inspector de Ajustes). El Architect NO rellena esta sección.

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016 enmendado por ADR-0143):** la identidad vive en el plano central; el motor local solo cachea. Justificado: es cimiento de la Cabina de Mando.
- **Inundación de Fundaciones (ADR-0020 V2):** Grupo I completo + **Perfil D (Ops/Auditoría)**: Identidad(I) + Soberanía(II: `owner_id`, `institutional_tag`, `access_token_id`) + Hardware(IV: `node_id`).

## Persistencia (Inundación de Fundamentos — ADR-0020 V2)

Tabla de cuenta con Grupo I (`id`, `created_at`, `updated_at`, `audit_hash`, `audit_chain_hash`, `event_sequence_id`) + `owner_id`, `institutional_tag`, `access_token_id`, `node_id` (huella de hardware). Campos propios fuera del catálogo (marcados como tales): estado de verificación de correo, proveedor OAuth. `STRICT`, UUIDv7 (ADR-0141). Perfil D.

## Dependencias y Bloqueantes

- **Bloquea a:** `licensing-system`, `usage-metering`, `consent-registry` (todas necesitan `owner_id`).
- **Contrato de Integración UI (ADR-0117) — Ventana de Verificación:** su observable (cuenta vinculada + estado de verificación) queda visible en el panel de cuenta del cajón de ajustes; hasta que exista, se registra como deuda de integración contra la feature consumidora `licensing-system`.
