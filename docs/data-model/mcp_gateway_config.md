# mcp_gateway_config

**Feature dueña:** [`agentic-mcp-gateway`](../features/agentic-mcp-gateway.md)
**Migración:** `migrations/0005_mcp_gateway.sql`
**Naturaleza:** Estado clave-valor (mutable, una fila por clave)
**Perfil ADR-0020:** No aplica el contrato de 25 campos — es una tabla de configuración pura.

## Columnas

| Columna | Tipo SQL | Null | Descripción |
|---|---|---|---|
| `key` | TEXT | NO | PK |
| `value` | TEXT | NO | Valor de la clave |

## Claves

- **PK:** `key`
- **FK salientes:** ninguna

## Quién la referencia (FK entrante)

- Ninguna.

## Notas de diseño

- Valor inicial sembrado por la migración: `('production_override_active', '0')` — el interruptor de producción del Gateway MCP arranca desactivado por defecto (ADR-0123). `permission_decisions` es el log forense de cada evaluación; esta tabla es el estado vigente.
