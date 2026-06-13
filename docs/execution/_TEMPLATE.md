# <ID> · <Título llano>

> **Plantilla de Orden de Trabajo (Spec-Driven).** Copia este archivo a `docs/execution/<ID>-<slug>.md`.
> La Orden de Trabajo es la **especificación ejecutable**: contiene la instrucción EXACTA que recibió el agente,
> los comandos para que el usuario valide por su cuenta, y el registro de lo que pasó. Vive en git, NO en el chat.
> Si la especificación cambia, se EDITA aquí y se re-despacha — así el cambio queda reflejado y versionado.

| Campo | Valor |
|---|---|
| **ID** | STORY-000 |
| **Título** | <llano, sin códigos> |
| **Tipo** | Story \| Spike \| Bug \| Task |
| **Épica (Fase)** | EPIC-0 — Fundación |
| **Sprint** | <n> |
| **Estado** | Pendiente \| En curso \| Bloqueado \| 🟡 Parcial \| ✅ Implementado |
| **Responsable** | <rol> (<modelo>) · auditó <Tech-Lead / QA> |
| **Creada** | YYYY-MM-DD |
| **Completada** | YYYY-MM-DD |

## 1. Especificación de origen (qué specs implementa)
Enlaces a las unidades de especificación que gobiernan este trabajo:
- **Feature(s):** [`nombre`](../features/nombre.md)
- **TTR(s):** TTR-001, TTR-002…
- **Módulo(s):** [`nombre`](../modules/nombre.md)
- **ADR(s):** ADR-XXXX

## 2. Objetivo (una frase llana)
<qué logra este trabajo, en lenguaje de negocio>

## 3. Instrucciones de despacho (la spec ejecutable)
> El prompt EXACTO que se le pasa al agente. Si cambia la spec, se edita ESTO y se re-despacha.

```
<prompt completo dado al agente>
```

## 4. Criterio de aceptación (cada criterio ↔ su prueba)
> Regla de proceso: ningún criterio se da por cumplido sin una prueba nombrada que lo ejerza. El ingeniero entrega ya en verde; el Tech-Lead reproduce y verifica cobertura del criterio (no solo "tests verdes").

| # | Criterio verificable | Prueba que lo demuestra |
|---|---|---|
| 1 | <condición verificable 1> | `<nombre_del_test>` |
| 2 | <condición verificable 2> | `<nombre_del_test>` |

## 5. Comandos de validación (para el usuario — copy/paste)
```bash
# comandos exactos para reproducir y validar por tu cuenta
cargo test -p <crate>
cargo clippy --workspace --all-targets -- -D warnings
cargo llvm-cov --workspace --summary-only          # % de cobertura de líneas
```

## 6. Registro de ejecución (bitácora cronológica)
- YYYY-MM-DD · <agente/modelo> · <APROBADO / RECHAZADO> · <evidencia de la auditoría del Tech-Lead>

## 7. Pendientes derivados / decisiones
- <pendientes que salieron de este trabajo, con su destino (otra Orden, escalamiento, etc.)>
