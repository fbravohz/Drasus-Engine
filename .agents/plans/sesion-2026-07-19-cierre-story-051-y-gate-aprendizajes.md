# Sesión 2026-07-18/19 — Cierre de STORY-051 y Gate de Aprendizajes

> **Propósito:** volcado íntegro de la sesión para no depender del chat. Todo lo decidido, lo hallado y lo pendiente vive aquí o en los documentos que este apunta.
> **Sesión anterior:** [`sesion-2026-07-18-decisiones-y-pendientes.md`](./sesion-2026-07-18-decisiones-y-pendientes.md) (doctrina de datos propios, corpus legal, tripleta).

---

## 1. Qué se cerró

**STORY-051 (transformador de datos de mercado) — ✅ CERRADA.** QA APTO · Quant APTO-CON-CONDICIONES. Primera pieza real completa de EPIC-1.

Recorrido: backend → congelamiento → tres retrofits → identidad de serie → puente → pantalla → gate de calidad (NO APTO) → dos tandas de cierre → gate de calidad (APTO) → validación cuantitativa.

**Órdenes satélite cerradas:** STORY-057 (identidad de serie en la barra), STORY-058 (migración de tipos del descargador; el marcador `Tick` genérico se retiró del catálogo).

**Deuda saldada:** DEBT-028 (el precio medio ponderado salió de la ingesta por ser un indicador).

---

## 2. Decisiones vinculantes del propietario en esta sesión

| # | Decisión | Dónde vive |
|---|---|---|
| 1 | **Los datos primitivos y los indicadores son fases distintas.** OHLC, compra, venta y volumen son materia prima; todo lo calculado a partir de ellos es un indicador y pertenece al motor. | DEBT-028, lección de STORY-051 |
| 2 | **No se descarta ninguna clase de activo.** *"Mientras más activos podamos operar, mejor. 100%."* Sobre el techo de magnitud: la decisión es cómo soportarlo, no si se soporta. | TASK-061, DEBT-033 |
| 3 | **La identidad de la serie se elige de listas cerradas, nunca se teclea.** Bróker, símbolo y temporalidad van en desplegables de opciones soportadas. | DEBT-031 |
| 4 | **El Architect revisa ANTES de que se redacte la Orden.** Etapa 0.3 obligatoria para toda feature. | Skills de Tech-Lead y Architect |
| 5 | **Demostración de discriminación con hechos, no de palabra.** Romper, enseñar el rojo, revertir, enseñar el verde. | Skills de Rust, Flutter y Bridge |
| 6 | **Los accionables que dependen del propietario lo marcan como bloqueante.** Task con `🔴 BLOQUEADO POR EL USUARIO`, nunca prosa en un chat. | Skill del Tech-Lead, TASK-060 |
| 7 | **El molde del ledger se corrige y se declara canónico**, en vez de parchear los sitios que lo copiaron. | `.agents/knowledge/ledger-pattern.md` |
| 8 | **Gate de aprendizajes aprobado íntegro**, las nueve mejoras. | Este documento §6 |

---

## 3. Hallazgos técnicos que sobreviven a la sesión

### 3.1 El techo de magnitud de la escala entera (el más importante)

ADR-0141 eligió la escala ×10⁸ justificándola por **precisión decimal** — cuántos decimales— y nunca comprobó el **rango** — cuánta magnitud. Son preguntas distintas y solo se contestó una.

Un `i64` escalado ×10⁸ solo representa ~92 200 millones de unidades. **Una barra de una hora de SHIB a volumen normal desborda ese techo en un 13%; en día activo lo cuadruplica.** Verificado aritméticamente por el Tech-Lead, no solo reportado.

El sistema falla seguro (error explícito, sin corrupción) pero **rechaza sistemáticamente** esa clase de activo. → **DEBT-033** + **TASK-061**.

### 3.2 Cosas que ya estaban decididas y nadie recordaba

Dos casos en una sola sesión, ambos detectados por el propietario, no por los agentes:

- **La regla del cierre de vela YA EXISTE y es FIJA.** ADR-0114, "Bar-Open Alignment": *"toda señal calculada al cierre de la barra N se ejecuta al precio de apertura de la barra N+1"*, dentro del Contrato de Consistencia Conservadora. El Quant recomendó canonizarla sin saberlo, y el Tech-Lead lo registró sin verificar. → TASK-059 corregida y reducida a su alcance real.
- **La agregación a tipos de barra YA ESTÁ especificada.** `docs/features/algorithmic-bars.md` declara Renko, Rango, Volumen y Tick, consumiendo un flujo de ticks. → TASK-062.

**Esta es la causa raíz que motivó la Etapa 0.3.**

### 3.3 Inconsistencia del catálogo de tipos de barra

ADR-0155 fija `bar_type IN ('TIME','TICK_COUNT','VOLUME','DOLLAR','RENKO')`; `algorithmic-bars.md` especifica Renko, **Rango**, Volumen y Tick. Dos huecos cruzados: la base rechazaría una barra de Rango que la feature dice que produciremos, y admite un tipo "Dólar" que ninguna feature especifica. **Heikin Ashi no aparece en ningún documento del repositorio** — y por la doctrina de primitivos frente a indicadores, probablemente sea un indicador, no un tipo de barra. → **TASK-062**.

### 3.4 El molde del ledger propagaba dos huecos

La primera feature con registro inmutable se volvió "el patrón establecido" **por el mero hecho de haber sido la primera**, con dos defectos: una prueba de clasificador que nunca llamaba a la función, y una migración sin candados anti modificación. Cada Orden nueva decía "sigue ese patrón". STORY-051 los heredó por obedecer.

Lo irónico: seis de ocho ledgers posteriores **sí** corrigieron uno de los huecos por su cuenta, pero la referencia oficial siguió siendo la vieja. → **DEBT-032** + molde canónico escrito.

---

## 4. Deuda abierta al cierre de la sesión

| ID | Qué | Disparador |
|---|---|---|
| DEBT-025 | Falta muestra real de Forex/CFD | TASK-060 (bloqueada en el propietario) |
| DEBT-029 | Los contratos de entrada aceptan campos desconocidos en silencio | Antes del segundo conector |
| DEBT-030 | La reagregación hereda la temporalidad de la sub-barra: etiqueta `1M` una serie horaria | Antes del primer llamador de esa función |
| DEBT-031 | La identidad de la serie se teclea a mano y queda sellada en el hash | Antes de que el panel sea alcanzable |
| DEBT-032 | Retrofit del ledger del descargador (el molde ya está corregido) | Antes del siguiente ledger |
| DEBT-033 | Techo de magnitud de la escala entera | Antes de ingesta real de cripto de oferta extrema |

**Tasks abiertas:** TASK-052 (fuentes de datos), TASK-053 (jurisdicción), TASK-054 (nivel gratuito), TASK-055 (cumplimiento por arquitectura), TASK-056 (clausulado), TASK-059 (momento de conocibilidad), **TASK-060 (🔴 bloqueada en el propietario)**, TASK-061 (techo de magnitud), TASK-062 (catálogo de tipos de barra).

---

## 5. Cómo se trabajó, para que se repita

Lo que funcionó y conviene conservar:

- **Congelar antes de propagar un contrato.** STORY-051 se congeló con el backend hecho y el puente sin despachar. Costó tres retrofits de backend; no hacerlo habría costado esos tres **más** el puente y la pantalla rehechos.
- **Auditar reproduciendo, nunca leyendo el reporte.** Cada entregable se verificó corriendo las pruebas, leyendo el código y trazando la aritmética. Varios reportes eran optimistas o incompletos.
- **Dejar que el ingeniero desobedezca al despacho cuando contradice una especificación sellada.** Ocurrió y fue correcto: la Orden manda sobre la instrucción verbal de quien despacha. Quedó como precedente escrito en STORY-058 §7.
- **Registrar la deuda con disparador, no con "algún día".** Las seis deudas de esta sesión tienen condición de disparo verificable.

---

## 6. Gate de Aprendizajes — las nueve mejoras (aprobadas íntegras)

| # | Mejora | Estado |
|---|---|---|
| 1 | El gate de mutación lo corre solo el Tech-Lead, con script determinista | ✅ aplicado (skills TL/Rust/QA + `scripts/mutation-gate.sh`) |
| 2 | Demostración de discriminación con hechos (rojo → verde) | ✅ aplicado (skills Rust/Flutter/Bridge) |
| 3 | Prohibida la aserción autorreferencial | ✅ aplicado (skill Rust + `mutation-gate.md`) |
| 4 | Espacio de numeración único para Órdenes | ✅ aplicado (skill Architect; ya existía en el del TL) |
| 5 | Referencias canónicas declaradas y auditadas, no "la primera que se construyó" | ✅ aplicado (`ledger-pattern.md`) |
| 6 | Entorno del gate de mutación documentado con su script | ✅ aplicado (`mutation-gate.md` + script) |
| 7 | Propiedad del vocabulario: quien lo define lo expone, nadie lo reimplementa | ✅ aplicado (skill Bridge) |
| 8 | Sincronía documental por etapa, no al final | ✅ aplicado (skill TL) |
| 9 | Correr mutación antes del gate final, no solo al cierre | ✅ aplicado (skill TL) |

---

## 7. Segunda tanda de directivas (2026-07-19, tarde)

| # | Decisión | Dónde vive |
|---|---|---|
| 9 | **El Quant tiene acceso documental completo** (ADRs, SAD, features, módulos, moonshots, TTRs) y obligación de citar. Deja de ser solo cazador de vacíos: pasa a decir qué documento aplica y por qué. | Skill del Quant, §1a |
| 10 | **El Quant va PRIMERO, el Architect después.** Se reordenan las etapas: 0.3 = Quant/Counsel, 0.4 = Architect. Los hallazgos del Quant se le entregan íntegros al Architect como insumo. | Skills de TL, Architect, Quant |
| 11 | **Gate Doc-First para TODOS**, incluido el Tech-Lead: buscar en la documentación antes de proponer. "No encontré nada" solo vale declarando dónde se buscó. | `base.md` §7.4 |
| 12 | **DEBT ≠ TASK, con criterio escrito.** Hueco en algo que ya corre → DEBT. Pregunta que alguien debe contestar → TASK. No se fusionan. Un hallazgo con las dos caras genera las dos. | `debt-management.md` |
| 13 | **No se ingesta formato arbitrario.** Formato estándar + plantillas por plataforma + mapeo declarado con vista previa. Preferencia: que el usuario transforme afuera. | TASK-063 |
| 14 | **La ejecución simulada solo lee compra/venta**, nunca un campo genérico de precio. Matiz del TL: el hecho negociado se conserva como materia prima para barras Renko/Tick/Volumen/Rango. | TASK-063 |

**Ponytail no se reparte al equipo.** Análisis del TL: Ponytail gobierna *cuánto construir*, no *cuánto leer*. Dos de sus reglas ("la escalera es un reflejo, no un proyecto de investigación"; "si la explicación es más larga que el código, borra la explicación") **empeorarían** el problema real, que es gente que no lee ni cita. El fallo diagnosticado se cura con el gate Doc-First (#11), no con Ponytail. Ponytail se mantiene donde ya sirve: decisiones de alcance.

### Errores de la sesión, para memoria

- **Del Tech-Lead:** anunció tres veces que no le alcanzaría el contexto para auditar, cuando tenía de sobra. Se medía contra un límite imaginario. Costó una corrección del propietario y retrasó la auditoría.
- **Del Tech-Lead:** registró TASK-059 sobre una premisa falsa sin verificar que la regla ya existía en ADR-0114.
- **Del Architect:** numeró dos Órdenes con números ya ocupados.
- **De tres agentes:** agotaron su presupuesto completo esperando corridas de mutación, sin entregar nada.
- **Del Rust-Engineer:** escribió una prueba tautológica que habría pasado con una función vacía.
- **Del entorno:** el gate de mutación reventó por falta de RAM con un error que aparentaba fallo de compilación.
