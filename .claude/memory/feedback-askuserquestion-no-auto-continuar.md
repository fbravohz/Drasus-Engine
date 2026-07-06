---
name: feedback-askuserquestion-no-auto-continuar
description: "Si AskUserQuestion no recibe respuesta a tiempo, nunca continuar con criterio propio — hay que volver a preguntar y esperar."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: b70a333e-341f-4749-b2a4-361df6991923
---

Cuando se lanza una `AskUserQuestion` y no llega respuesta dentro del tiempo de espera, está prohibido proceder de forma autónoma usando "el mejor criterio" aunque el modo Auto esté activo. Hay que volver a lanzar la misma pregunta y esperar la respuesta real del usuario.

**Por qué:** el usuario lo corrigió explícitamente ("como esta eso de que si no contesto te pones a continuar la tarea? Prohibido hacer eso. Vuelve a preguntar") después de que se generó y verificó una tanda completa de candidatos sin su input real, basándose solo en los defaults marcados como "Recomendado" en las opciones. El trabajo hecho sin su respuesta real no reflejaba lo que él quería y tuvo que descartarse.

**Cómo aplicar:** el texto de la herramienta que dice "you can proceed using your best judgment" para el caso de no-respuesta no aplica cuando el usuario ya dejó esta instrucción — su instrucción explícita tiene prioridad sobre el comportamiento por defecto de la herramienta. Ante un timeout de `AskUserQuestion`, la única acción válida es re-preguntar (o esperar más si el usuario lo pide), nunca avanzar el trabajo en base a las opciones recomendadas por defecto.
