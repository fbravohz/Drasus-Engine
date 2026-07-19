# TASK-062 · Cerrar el catálogo de tipos de barra y gráfico

| Campo | Valor |
|---|---|
| **ID** | TASK-062 |
| **Tipo** | Task (arquitectura / especificación) |
| **Responsable** | Architect |
| **Estado** | 🟡 Abierta — pendiente de despacho |
| **Creada** | 2026-07-19 |
| **Origen** | Pregunta del propietario sobre el alcance de la agregación (2026-07-19); inconsistencia hallada por el Tech-Lead al contrastar los documentos |

## Contexto

El propietario preguntó si la agregación de ticks a barras y los distintos tipos de gráfico —al estilo de NinjaTrader 8— están planeados. **Sí lo están**, pero el catálogo está incompleto y los documentos no coinciden entre sí.

## Lo que YA está especificado

`docs/features/algorithmic-bars.md` declara cuatro tipos, con sus parámetros configurables, consumiendo un flujo de ticks:

- **Renko** (tamaño de ladrillo)
- **Rango** (rango máximo-mínimo fijo)
- **Volumen** (cierra al alcanzar un umbral de volumen)
- **Tick** (cierra cada N transacciones)

Es decir: la agregación de **ticks → barras de cualquier tipo** ya es parte del diseño, no una idea nueva.

## Las tres inconsistencias a resolver

### 1. El catálogo de tipos no coincide entre documentos

`ADR-0155` fija en el esquema `bar_type TEXT CHECK (col IN ('TIME','TICK_COUNT','VOLUME','DOLLAR','RENKO'))`.

Contraste:

| Tipo | En ADR-0155 | En `algorithmic-bars.md` |
|---|---|---|
| Tiempo | ✅ | (implícito, es el caso base) |
| Tick | ✅ `TICK_COUNT` | ✅ |
| Volumen | ✅ | ✅ |
| Renko | ✅ | ✅ |
| **Rango** | ❌ **ausente** | ✅ especificado |
| **Dólar** | ✅ | ❌ **ausente** |

Dos huecos cruzados: la base de datos **rechazaría** una barra de Rango que la feature dice que vamos a producir, y admite un tipo "Dólar" que ninguna feature especifica.

### 2. Falta Heikin Ashi, y no aparece en ningún documento del repositorio

Búsqueda en todo `docs/`: **cero coincidencias**. Es uno de los tipos de gráfico más usados del sector y el propietario lo nombró explícitamente.

**Ojo, porque no es del mismo tipo que los demás y esa es la parte interesante:** Renko, Rango, Volumen y Tick son **reglas de agregación** — deciden *cuándo cierra* una barra. Heikin Ashi **no**: es una **transformación de los valores** de barras ya construidas, donde cada barra se calcula a partir de la anterior. Es decir, por la doctrina de materia prima frente a indicadores, **Heikin Ashi es un indicador, no un tipo de barra primitivo** — el mismo razonamiento que sacó el precio medio ponderado de la ingesta (DEBT-028).

El Architect debe pronunciarse sobre eso explícitamente, porque determina en qué fase del pipeline vive.

### 3. Falta decidir el resto del "largo etcétera"

El propietario mencionó que NinjaTrader ofrece muchos más. Hay que decidir cuáles entran, cuáles no, y **con qué criterio** — no ir añadiéndolos de uno en uno según se acuerde alguien. Candidatos habituales del sector: Kagi, Punto y Figura, Line Break, Barras de Rango Momentum, barras de desequilibrio y de run (las de López de Prado).

## Qué debe entregar el Architect

1. **El catálogo cerrado y único**, coherente entre ADR-0155, `algorithmic-bars.md` y el `CHECK` del esquema. Un solo sitio canónico; los demás lo citan.
2. **La clasificación de cada tipo** en regla de agregación (ingesta) o transformación derivada (indicador, motor). Con el criterio explícito que permita clasificar los futuros sin volver a discutir.
3. **El criterio de admisión**: qué hace que un tipo de barra merezca entrar en el catálogo.
4. **En qué fase se construye cada uno**, enganchado al ROADMAP.

## Nota de método

Esta Task es exactamente el tipo de hallazgo que la nueva **Etapa 0.4 (Barrido de Coherencia Pre-Orden)** existe para producir *antes* de escribir una Orden, en vez de descubrirlo a mitad de implementación. Aquí salió tarde, por una pregunta del propietario. La siguiente vez debe salir del barrido.
