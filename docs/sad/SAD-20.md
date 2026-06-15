## 20. Gobernanza y Soberanía de Datos

Drasus Engine es un sistema **Local-First (ADR-0016)**. La persistencia se realiza en el sistema de archivos del usuario mediante SQLite para estados y Parquet para datos históricos. El usuario retiene el control total de su IP (estrategias) y capital, sin dependencia obligatoria de servicios en la nube. Toda entidad de datos obedece el **Contrato Global (ADR-0020 V2)**: el grupo I de Identidad & Integridad es universal y el resto del contrato lógico se inyecta de forma selectiva por perfil, asegurando auditabilidad institucional sin replicar 25 columnas en cada tabla.

---

**Documento versión 4.2** | Última actualización: 2026-06-14
