# Corpus Legal (LDR) — Índice

Cada decisión legal vive en su propio archivo bajo [`legal/`](./legal/), con el mismo patrón que [`docs/ADR.md`](./ADR.md)/[`docs/adr/`](./adr/) y [`docs/DEBT.md`](./DEBT.md)/[`docs/debt/`](./debt/): este archivo es el **índice**; para leer una decisión concreta, abre su archivo (≈40–90 líneas) — no hace falta cargar todo el corpus. Este corpus es el material que sostendrá los futuros **Términos y Condiciones del servicio** — ver la hoja de ruta en [`legal/ROADMAP-TYC.md`](./legal/ROADMAP-TYC.md).

---

## Por qué "legal/" y por qué el prefijo "LDR"

- **Carpeta `docs/legal/`, no `docs/gc/` ni `docs/counsel/`:** nombra el **dominio** (legal), no el rol que lo produce (General Counsel), mismo criterio que `docs/adr/` no se llama `docs/architect/` — la carpeta sobrevive aunque cambie quién la redacta.
- **Prefijo `LDR` = *Legal Decision Record*, espejo deliberado de `ADR` = *Architecture Decision Record*.** Misma estructura (una decisión, un archivo, veredicto + fundamento + trazabilidad), mismo rigor (nada de pseudocódigo ni cita de norma no verificada sin marcarla), pero dominio y dueño distintos: el Architect sella arquitectura, el General Counsel sella exposición legal/regulatoria/fiscal. El paralelismo con `ADR` es intencional — cualquiera que ya sepa leer un ADR sabe leer un LDR sin instrucciones nuevas.
- **No se reutilizó el prefijo `ADR`** porque un ADR "dura años y no cambia en 6 meses" con un dueño (Architect) distinto del dueño de una decisión legal; mezclar dominios en la misma numeración habría obligado a que el Architect y el General Counsel compitieran por el mismo contador y habría hecho ambigua la pregunta "¿quién sella esto?" — la misma razón por la que `DEBT-XXX` tiene su propio prefijo y su propio dueño (Tech-Lead).
- **Numeración:** `LDR-0001` en adelante, cuatro dígitos por consistencia visual con `ADR-XXXX`, sin que eso implique una expectativa de volumen equivalente (117 ADRs). Estado de cada `LDR` (Abierta/Vigente/Superada) se declara en su propio encabezado, igual que un ADR marca `~~superado~~` cuando corresponde.

---

## Índice

| LDR | Decisión | Veredicto |
|---|---|---|
| [LDR-0001](./legal/LDR-0001.md) | Agregación de Datos de Terceros — la Anonimización no Resuelve la Titularidad de la Fuente (Cimiento #9) | APTO-CON-CONDICIONES |
| [LDR-0002](./legal/LDR-0002.md) | Adaptador de Binance — Descarga Anónima a Escala de Compañía es NO-APTA; Migración a Credenciales del Usuario Reduce, No Elimina | NO-APTO (forma actual) / APTO-CON-CONDICIONES (ruta de migración) |
| [LDR-0003](./legal/LDR-0003.md) | Modelo "Cada Usuario Trae sus Datos" — Reduce Exposición de Forma Genuina, no Es un Escudo Legal Automático | APTO-CON-CONDICIONES |
| [LDR-0004](./legal/LDR-0004.md) | Cláusula de Responsabilidad sobre el Dato Importado por el Usuario — Redacción y Franqueza sobre su Oponibilidad Real | Cláusula redactada — alcance acotado |
| [LDR-0005](./legal/LDR-0005.md) | Grabación del Flujo de Datos en Vivo — Confirmación de la Postura del Propietario, con Alcances y Límites Reales | Postura confirmada, dos precisiones de alcance |
| [LDR-0006](./legal/LDR-0006.md) | Frontera de Agregación por Licencia de Fuente — la Regla Vinculante, Extraída como Cláusula Independiente | Regla FIJA |
| [LDR-0007](./legal/LDR-0007.md) | Substrato de Monetización ya Cubierto — Catálogo para Efectos de Términos y Condiciones | Catálogo |
| [LDR-0008](./legal/LDR-0008.md) | Transformar el Archivo del Usuario con Mapeo Declarado no Transfiere Responsabilidad — Adivinarlo Sí; Plantillas de Plataforma son un Tercer Caso, Propio de Drasus | APTO-CON-CONDICIONES |

**Cierre del corpus:** [`legal/ROADMAP-TYC.md`](./legal/ROADMAP-TYC.md) — qué cláusula ya está sostenida, qué falta, y qué exige abogado colegiado de una jurisdicción concreta antes de publicar. Incluye el límite explícito de alcance de este corpus (no sustituye asesoría legal colegiada).

---

## Relación con el resto del mapa documental

- **`docs/ADR.md`** registra decisiones de arquitectura; este índice registra decisiones legales. Cuando una decisión legal se adopta como regla vinculante dentro de un ADR (ej. ADR-0154 adopta la regla de [LDR-0006](./legal/LDR-0006.md)), el ADR cita al LDR como fuente — el LDR nunca se copia dentro del ADR completo, para no duplicar y desalinear.
- **`docs/DEBT.md`** registra deuda técnica; cuando una decisión legal exige un habilitador técnico (ej. la marca de agregabilidad de [LDR-0006](./legal/LDR-0006.md)), el habilitador se registra ahí (`docs/debt/DEBT-026.md`), no aquí — este corpus decide el "qué" legal, DEBT.md rastrea el "falta construir" técnico.
- **Origen:** `.agents/plans/veredicto-legal-datos-mercado.md` (General Counsel, 2026-07-18) es el análisis fuente que LDR-0001/0002/0003 destilan; queda como el documento de trabajo completo, LDR es su forma canónica y citable.
