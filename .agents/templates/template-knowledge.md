# Plantilla de Documento de Knowledge

- Cómo se ensambla un documento completo de `.agents/knowledge/` — no un bloque suelto, para eso [[.agents/templates/template-instruction-formats.md]].
- SLUG-ID: `TEMPLATE-KNOWLEDGE`
- Tamaño: `2137 caracteres (~534 tokens aprox., ratio 4:1)`
- Lecturas:
  - [[.agents/templates/template-instruction-formats.md]] -> SIEMPRE -> Context Engineer

## TEMPLATE-KNOWLEDGE-1: Estructura

Título + cabecera → `<DOC>-P` (Pipeline, opcional, solo si el orden de pasos no es inferible de las secciones mismas) → N secciones `<DOC>-N: <Título>` sin límite → Ejemplos al cierre si aplican, namespace propio `<DOC>-EJ`, nunca compite por número con las secciones.

```
# <Título>

- <Objetivo: 1-2 líneas, qué agrupa y para quién — sin backstory>
- SLUG-ID: `<DOC>`
- Tamaño: `<N> caracteres (~<M> tokens aprox.)`
- Tags: `tag-uno`, `tag-dos` (opcional — solo si ya tiene tags asignados del vocabulario controlado)
- Lecturas:
  - [[ruta/doc1.md]] -> SIEMPRE -> Todos
  - [[ruta/doc2.md]] -> `keyword1`, `keyword2` -> <alcance>

## <DOC>-1: <Título de la sección>

- **<Nombre corto>:** cuerpo en una línea.
- **<Nombre corto>:** cuerpo en una línea.

## <DOC>-N: <Título de la sección N-ésima — secuencia sin límite, append-only>

## <DOC>-EJ: Ejemplos

**Caso: <qué transforma>**

**Antes:**
> <texto>

**Después:**
> <texto, con medición real inline si existe>
```

## TEMPLATE-KNOWLEDGE-2: Notas

- Cabecera, Secciones, IDs y espaciado siguen el catálogo completo, sin excepción aquí: [[.agents/templates/template-instruction-formats.md]].
- Único matiz propio de un documento de knowledge: el campo Tamaño se recalcula (`wc -m`) en cada edición que cambie el contenido — `validate-agent-docs.py --audit` solo compara y avisa si quedó desfasado, no lo corrige.
- Toda referencia a otro documento es wikilink `[[ruta/archivo.md]]`, nunca ruta suelta en texto plano.
- Campo Tags (cuando aplica): valor debe existir en [[.agents/knowledge/tags-vocabulary.md]] — vocabulario controlado, append-only, con 4 tags semilla. La asignación real sobre el corpus es progresiva (ver plan de aligeramiento, Sección H).
