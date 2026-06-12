# Feature Router — Inyección Dinámica de Features

**Carpeta:** `./features/feature-router/`  
**Estado:** Lista para implementar  
**Última actualización:** 2026-04-08  
**Decisión Arquitectónica Asociada:** ADR-0007 (Inyección Dinámica de Comportamiento)

---

## ¿Qué es?

Feature Router implementa un mecanismo para activar/desactivar features dinámicamente en tiempo de startup, sin hardcodear qué features están disponibles en el código.

**Problema:** Si cada módulo está hardcodeado para usar feature X, es imposible cambiar comportamientos sin modificar código. Algunos traders quieren feature X, otros no.

**Solución:** Centralizar en un registry la lista de features disponibles. En startup, cargar configuración de usuario que especifica qué features activar. Sistema valida combinaciones válidas y rechaza configuraciones inválidas.

**Resultado observable:** Usuario puede cambiar qué features están activos solo editando configuración, sin tocar código.

---

## Comportamientos Observables

- [ ] Usuario crea archivo `config/features.yaml` especificando qué features activar:
  ```yaml
  features:
    active: ["audit-log", "order-fsm", "clock", "notification"]
    disabled: ["pysr-signal-discovery"]
  ```
  → En startup, sistema carga configuración
  → Busca cada feature en FEATURE_REGISTRY
  → Todas existen y están disponibles → Startup exitoso

- [ ] Usuario configura una feature que NO existe en FEATURE_REGISTRY:
  ```yaml
  features:
    active: ["audit-log", "nonexistent-feature"]
  ```
  → Sistema busca "nonexistent-feature" en registry
  → NO ENCUENTRA → Falla en startup con error claro: "Feature 'nonexistent-feature' not found in registry"
  → Sistema nunca inicia

- [ ] Usuario intenta desactivar una feature que es requerida por otra feature:
  ```yaml
  features:
    active: ["order-fsm"]  # Activado
    disabled: ["clock"]    # Desactivado
  # Pero order-fsm REQUIERE clock
  ```
  → Sistema valida dependencias en startup
  → Detecta: "Feature 'order-fsm' requires 'clock' but 'clock' is disabled"
  → Falla rápido con error claro

- [ ] Una feature `notification` intenta consumir capability que NO tiene porque fue desactivada:
  ```yaml
  features:
    active: ["execute"]
    disabled: ["notification"]
  # execute intenta notificar al usuario (usa notification)
  ```
  → En tiempo de init de execute, intenta obtener notification feature del router
  → Router retorna `None` o lanza error explícito: "Feature 'notification' not available"
  → execute maneja gracefully (ej: no notifica)

- [ ] Sistema carga una feature, la inicializa con parámetros de configuración:
  ```yaml
  features:
    active: ["audit-log"]
  audit_log_config:
    retention_days: 365
    batch_size: 1000
  ```
  → Feature Router busca `audit_log_config` en configuración
  → Pasa a AuditLogFeature.__init__(retention_days=365, batch_size=1000)
  → Feature se inicializa con parámetros del usuario

---

## Restricciones

- **NUNCA se inicializa un feature que no está registrado en FEATURE_REGISTRY.** Si no está en registry, es rechazado.
- **NUNCA un módulo se comunica directamente con una feature sin pasar por Feature Router.** El router es el único punto de acceso.
- **NUNCA se activa una feature que viola dependencias de otras features.** Sistema valida en startup (fail-fast).
- **NUNCA hay dead code paths.** Si una feature está desactivada, ninguna otra feature intenta usarla.
- **NUNCA se persiste Feature Router en disco.** Es un componente de arranque (stateless después de init).

---

## Parámetros Configurables

| Parámetro | Default | Tipo | Descripción | CONFIG/FIJO |
|-----------|---------|------|-------------|------------|
| `features.active` | `["audit-log", "clock", "order-fsm"]` | List[str] | Lista de features a activar en startup | CONFIG |
| `features.disabled` | `[]` | List[str] | Lista explícita de features a desactivar (opcional, override de active) | CONFIG |
| `strict_mode` | `true` | bool | Si true, rechaza configuración si hay features no registradas. Si false, ignora (NO RECOMENDADO en prod) | FIJO |
| Feature-specific params | var. | var. | Cada feature puede tener sus propios parámetros (ej: `audit_log_config`, `notification_config`) | CONFIG |

---

## Ciclo de Vida de la Feature

### Entrada
- **Quién llama:** Main entry point del sistema (orquestador principal)
- **Qué recibe:** Ruta a configuración de features (`config/features.yaml`)

### Proceso
1. **Load Config:** Leer `config/features.yaml`
2. **Validate Syntax:** ¿YAML válido? Si no, falla rápido
3. **Check Registry:** Para cada feature en `active`, ¿existe en FEATURE_REGISTRY?
4. **Check Dependencies:** Para cada feature, ¿todas sus dependencias están activas?
5. **Instantiate:** Crear instancia de cada feature con parámetros específicos
6. **Boot Check:** ¿Cada feature pasó su propio init sin errores?
7. **Store in Router:** Guardar instancias en `router.features_dict` accesible por módulos

### Salida
- **Produce:** Feature Router inicializado con diccionario de features activas, listo para ser inyectado en módulos

### Contextos de Uso
- **Startup global:** Orquestador principal inicializa Feature Router antes de cualquier módulo
- **Module injection:** Cada módulo recibe Feature Router (o subset de features) en su constructor
- **Runtime access:** Módulo consulta router: `router.get("order-fsm")` → obtiene feature o None

---

## Tareas (TTRs)

### **TTR-FEATURE-ROUTER-001: Implementar FEATURE_REGISTRY**

**Qué hace:** Diccionario centralizado que mapea nombre_feature → clase Feature.

**Entrada:**
- Nada (es un constant registry)

**Salida:**
- Diccionario: `{"audit-log": AuditLogFeature, "order-fsm": OrderFSMFeature, ...}`

**Reglas de Negocio:**
- Cada feature está registrada exactamente una vez
- El registry no cambia en runtime (es estático)

**Precondiciones:**
- Todos los archivos de feature están disponibles en importación

**Postcondiciones:**
- FEATURE_REGISTRY es accesible globalmente
- Sistema puede validar qué features existen

---

### **TTR-FEATURE-ROUTER-002: Implementar Validación de Dependencias**

**Qué hace:** Dados una lista de features activas, valida que todas las dependencias están satisfechas.

**Entrada:**
- Lista de features activos (de config)
- Diccionario de dependencias (ej: `{"order-fsm": ["clock"], "execute": ["order-fsm", "broker-connector"]}`)

**Salida:**
- True si todas las dependencias están satisfechas
- False + lista de features faltantes si hay conflictos

**Reglas de Negocio:**
- Si feature A requiere feature B, y B no está en lista de activos, rechazar
- Las dependencias deben ser transitivasclose (si A requiere B y B requiere C, A indirectamente requiere C)

**Precondiciones:**
- Config de features cargado

**Postcondiciones:**
- Lista de features validada es segura para inicialización

---

### **TTR-FEATURE-ROUTER-003: Implementar Feature Router (Gestor Central)**

**Qué hace:** Inicializa todas las features activas y proporciona API para acceso.

**Entrada:**
- Ruta a configuración (`config/features.yaml`)
- FEATURE_REGISTRY
- Diccionario de dependencias

**Salida:**
- Router inicializado con `features_dict: Dict[str, Feature]`

**Reglas de Negocio:**
- Si un feature falla en init, falla todo el startup
- El error debe ser claro indicando cuál feature falló y por qué
- No se inicializa feature desactivada

**Precondiciones:**
- Config de features es válida (sintaxis YAML correcta)
- Dependencias validadas

**Postcondiciones:**
- Router listo, cada feature accesible por `router.get(name)`
- Cada feature recibió sus parámetros de configuración

---

### **TTR-FEATURE-ROUTER-004: Inyectar Features en Módulos**

**Qué hace:** Pasar Feature Router (o subset) a cada módulo en su constructor.

**Entrada:**
- Feature Router inicializado
- Módulo que necesita features

**Salida:**
- Módulo con acceso a features específicas

**Reglas de Negocio:**
- Cada módulo recibe SOLO las features que necesita (no todas)
- Si módulo consulta feature que no tiene, obtiene None o error explícito

**Precondiciones:**
- Feature Router inicializado exitosamente

**Postcondiciones:**
- Módulo puede llamar `self.features.get("order-fsm")` para acceder

---

### **TTR-FEATURE-ROUTER-005: Documentar Matriz de Dependencias**

**Qué hace:** Tabla clara de qué features requieren cuáles otras.

**Entrada:**
- Análisis de importaciones entre features/módulos

**Salida:**
- Tabla o diagrama de dependencias
- Lista de combinaciones válidas/inválidas de features

**Reglas de Negocio:**
- Documentación debe ser actualizada cada que una feature se agrega/modifica
- Sistema rechaza configuraciones que violen la matriz

**Precondiciones:**
- Todas las features especificadas

**Postcondiciones:**
- Documentación clara disponible para usuarios
- Sistema de validación implementado

---

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016):** 100% Local. La resolución de dependencias y activación de features ocurre en el arranque local.
- **Inundación de Fundaciones (ADR-0020 V2):** 
    - Cada sesión de ruteo y despliegue de features registra el set completo de **25 campos mandatorios** (ver ADR-0020 V2 V2).
    - Metadatos de configuración y seguridad: `audit_chain_hash` (Secuencia de booteo), `logic_hash` (Kernel version), `node_id`, `access_token_id`.
    - Soberanía: `owner_id`, `manifest_id`.


---

## Dependencias y Bloqueantes

- **Bloqueante:** Requiere que todas las features existan con interfaz pública clara (ver ADR-0003)
- **Requiere:** FEATURE_REGISTRY implementado (TTR-001)
- **Requiere:** Validación de dependencias (TTR-002)
- **Habilita:** ADR-0007 (Inyección Dinámica) puede ser implementado

---

## Referencias

- `ADR.md` → ADR-0007: Inyección Dinámica de Comportamiento
- `ADR.md` → ADR-0008: Configurabilidad Universal
- `features/*.md` → Cada feature debe declarar dependencias
