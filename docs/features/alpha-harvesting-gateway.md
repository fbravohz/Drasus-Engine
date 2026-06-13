# Alpha Harvesting Gateway

**Carpeta:** `./features/alpha-harvesting-gateway/`
**Estado:** En Diseño
**Última actualización:** 2026-04-29

## 1. ¿Qué es esta feature?

Es un portal de ingesta (Gateway) que permite recibir, desencriptar y refinar estrategias anonimizadas provenientes de la mente colectiva (o peers) en la máquina local. 
Permite incorporar "Alpha" descubierto por otros nodos sin revelar su código fuente nativo ni comprometer la seguridad local, re-compilando la estrategia en el `Strategy AST` interno.

## 2. Comportamientos Observables

- Usuario importa un archivo encriptado `*.qf_alpha`.
- El sistema extrae el esquema Serde, valida que no contenga código malicioso, y lo inyecta como una tesis o candidato en el módulo de generación.

## 3. Restricciones

- NUNCA se ejecuta código arbitrario Rust (eval/exec) del archivo importado. Solo parsea JSON AST.
- NUNCA envía datos de rendimiento hacia afuera (Privacidad Local).

## 4. Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| TRUST_MODE | strict | strict / liberal | Nivel de validación del AST importado | CONFIG |

## 5. Estructura Interna (FCIS — ADR-0002)

- **Core:** Validador AST y sanitizador.
- **Shell:** Puerto de entrada (Upload).
- **Frontera Pública:** `import_harvested_alpha(payload)`.

## 6. Ciclo de Vida de la Feature

### Entrada
- Payload anonimizado (Strategy AST v3.0 exportado de otra máquina).

### Proceso
- Parsea y sanitiza cada nodo AST.
- Realiza dry-run estático de compatibilidad de indicadores.
- Asigna un nuevo `version_node_id` local marcado como "Importado".

### Salida
- Estrategia importada lista para optimización local (bayesian tuning).

### Contextos de Uso
**Contexto 1: Acelerador de R&D en Generate**
- Salta la necesidad de descubrir desde cero, usando semillas externas.

## 7. Tareas (TTRs)

### **TTR-001: Deserialización Segura AST**
* **¿Cuál es el problema?** Recibir algoritmos de externos es peligroso si se usa pickle o eval.
* **¿Qué tiene que pasar?** El archivo externo se parsea usando estrictamente esquemas Serde, rechazando atributos no registrados.
* **¿Cómo sé que está hecho?**
  - [ ] Subo AST malicioso y es rechazado.
* **¿Qué no puede pasar?**
  - Cero inyección de dependencias externas.

## 8. Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016):** 100% Local. No llama "home" al origen.
- **Inundación de Fundaciones (ADR-0020 V2):**
  - **Perfil elegido:** B. Perfil IA / R&D
  - **Identidad (Grupo I completo):** `id` del import, `created_at`, `updated_at`, `audit_hash`, `audit_chain_hash`, `event_sequence_id`.
  - **Soberanía:** `institutional_tag` fijado como EXTERNAL_ALPHA.
  - **Pesos/Arquitectura:** `logic_hash` del AST importado verificado en base a firma criptográfica.

## 9. Decisión Arquitectónica Asociada
- ADR-0020 V2.
