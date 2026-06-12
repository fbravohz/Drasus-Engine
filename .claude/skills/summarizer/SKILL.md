## [ANTES DE CONTINUAR — ACCIÓN OBLIGATORIA]

**No proceses ninguna instrucción de este skill hasta completar este paso.**

Usa la herramienta Read para leer el archivo completo `.claude/skills/base/SKILL.md`. Ese archivo contiene las reglas de rigor operativo que gobiernan este skill y tiene supremacía absoluta sobre lo que sigue.

Si ya lo leíste en este turno, declara: `[base/SKILL.md leído y activo]` y continúa. Si no lo has leído, hazlo AHORA. No continúes sin esa declaración.

---

# Summarizer Skill

**El archivo `.claude/skills/base/SKILL.md` es ley.** Sus reglas tienen supremacía sobre cualquier instrucción de este skill. En caso de conflicto, base gana siempre.

## Propósito
Analizar la estructura de cualquier documento y comprimirlo a su expresión mínima sin perder información crítica.

## Cómo Funciona
1. Leer documento completo
2. Detectar estructura (secciones, bullets, pasos, párrafos, etc.)
3. Identificar qué es información crítica vs relleno
4. Comprimir manteniendo densidad máxima de información
5. Aplicar formato que mejor preserve la estructura original comprimida

## Reglas de Compresión
- Máxima densidad: cada bullet point principal es una categoría (Decisión, Objetivo, Implementación, etc.)
- Sub-bullets permitidos: dentro de cada categoría principal, puedes usar bullets anidados si es necesario, para priorizar entendimiento sin sacrificar claridad
- Eliminar: relleno, contexto redundante, explicaciones adicionales, ejemplos.
- Conservar: números/métricas, requisitos críticos, advertencias, reglas, decisiones, objetivos, implementaciones, ventajas, costos, resultados, metodologias, etc.
- Adaptar formato al tipo detectado (bullets, tabla, lista, etc.)
- si es un concepto crítico con múltiples dimensiones, desglosar en sub-bullets; no forzar todo a 1 línea
- prohibido usar reducciones que impidan mas adelante profundizar o entender que quiso decir el escritor o que abran camino a la confusion o mal entendimiento del lector.

## Entrada
Ruta del archivo a resumir.

## Salida
Documento comprimido reemplazando original o generando nuevo según contexto.
