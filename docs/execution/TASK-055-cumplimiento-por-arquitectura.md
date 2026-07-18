# TASK-055 · Cumplimiento por arquitectura — no rozar el consejo de inversión

| Campo | Valor |
|---|---|
| **ID** | TASK-055 |
| **Tipo** | Task (arquitectura, sin código todavía) |
| **Responsable** | Architect (con consulta al General Counsel donde marque duda) |
| **Estado** | 🟡 Abierta — pendiente de despacho |
| **Creada** | 2026-07-18 |
| **Origen** | Aprobado por el propietario |

## Principio rector

Los reguladores miran **lo que el producto hace**, no cómo se llama. Un descargo de responsabilidad no salva a un software que le dice a la gente qué comprar. Por eso esto se implementa **en la arquitectura**, no en el clausulado.

Formulación del propietario: *"siempre el usuario decide y solo mostramos datos; no tomamos decisiones por el usuario a menos que él indique cuál es la decisión por adelantado."*

## Las cinco reglas a canonizar

1. **Nunca declarar ni sugerir rendimientos esperados**, ni en el producto ni en el material comercial. Una curva de resultados pasados presentada como expectativa es exactamente lo que se persigue.
2. **Tiempo verbal en pasado, siempre.** "Esta estrategia **obtuvo**" en la simulación. Jamás "obtiene" ni "obtendrá". Aplica a la interfaz, a los reportes y a cualquier texto generado.
3. **Cero recomendaciones personalizadas.** El patrón seguro es invariante: el usuario configura, el sistema calcula, el usuario decide.
4. **Nada de gestión discrecional.** Automatizar lo que el usuario configuró de antemano es una cosa; que el sistema decida por él es otra. La frontera es si la decisión estaba declarada antes.
5. **Cuentas verificadas: la superficie más expuesta del producto.** Publicar historiales de rentabilidad se acerca a captar inversores, y eso está regulado casi en todas partes. Necesita más cuidado que la ingesta de datos.

## Qué debe entregar el Architect

- Un ADR que fije las cinco reglas como **invariantes de producto**, no como recomendaciones de estilo.
- **Dónde se hace cumplir cada una:** qué es responsabilidad del generador de reportes, qué de la capa de interfaz, qué del motor de veredictos, y qué del pilar de cuentas verificadas.
- **Barrido de lo ya escrito:** recorrer features y documentos existentes buscando texto que ya viole las reglas —promesas de rendimiento, verbos en futuro, veredictos con forma de recomendación— y corregirlo o reportarlo. El motor de veredictos en lenguaje natural y los reportes institucionales son los sospechosos principales.
- **Regla verificable, no aspiracional:** dejar planteado cómo se comprueba el cumplimiento (revisión del catálogo de textos, prueba automatizada sobre las plantillas de reporte, o lo que proponga).
