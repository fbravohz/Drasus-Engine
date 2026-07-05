## 20. Gobernanza y Soberanía de Datos

> 🔶 **Enmendado por ADR-0143 (2026-07-03)** — la soberanía absoluta de datos que describe esta sección aplica al **cómputo** (el motor corre en hardware del usuario) y al **usuario de pago al corriente** (telemetría suprimida en origen). Deja de ser absoluta para el **usuario gratuito**, cuyo trabajo fluye a la Cabina de Mando Central del proveedor por Términos y Condiciones. Zero-Telemetry queda derogado. La arquitectura de la Cabina de Mando y el substrato de monetización se detallan en **SAD-22**.

Drasus Engine mantiene el **cómputo Local-First (ADR-0016, enmendado por ADR-0143)**. La persistencia de trabajo se realiza en el sistema de archivos del usuario mediante SQLite para estados y Parquet para datos históricos. El usuario de pago retiene el control total de su IP (estrategias) y capital, sin que su trabajo salga de su máquina. El usuario gratuito acepta que su trabajo alimenta al proveedor (modelo "gratis = tus datos son del proveedor", ADR-0143). En todo caso, **los secretos (credenciales de bróker, IPs de servidores live) nunca se exfiltran** (ADR-0093). Toda entidad de datos obedece el **Contrato Global (ADR-0020)**: el grupo I de Identidad & Integridad es universal y el resto del contrato lógico se inyecta de forma selectiva por perfil, asegurando auditabilidad institucional sin replicar 25 columnas en cada tabla.

---

### Reconciler de Integridad Parquet (ADR-0141)

La soberanía de datos incluye garantizar que los registros en SQLite que referencian particiones Parquet no queden huérfanos (la partición fue registrada pero el archivo no existe en disco). Esta situación no puede detectarse mediante FK de base de datos (el constraint es cross-motor y es imposible en SQL).

**Mecanismo:** el módulo Ingest ejecuta un **reconciler de startup** en cada arranque de la aplicación:
1. Consulta todos los `data_snapshot_id` registrados en SQLite.
2. Verifica que el path Parquet correspondiente existe en disco.
3. Los registros sin partición física se reportan en `audit_events` con `action_type = 'PARQUET_ORPHAN_DETECTED'`.
4. El reconciler no borra ni corrige automáticamente — reporta para que el usuario decida.

**Regla de integridad de campo:** todo campo que referencie una partición Parquet incluye en el comentario SQL el formato canónico del valor y un `CHECK` de formato. Formato canónico de `data_snapshot_id`: `<exchange>_<symbol>_<timeframe>_<year><month>`.

---

**Documento versión 4.3** | Última actualización: 2026-06-28
