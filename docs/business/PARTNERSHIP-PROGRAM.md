# Drasus Engine — Programa de Sociedades y Colaboración

Este documento es un catálogo de mecanismos para que personas del círculo cercano del propietario se apalanquen de la infraestructura tecnológica de Drasus Engine y generen ingresos junto con él, sin que eso implique regalar dinero, ceder control societario, ni arriesgar la relación personal si alguien decide salir.

**No es un contrato.** Es un menú de opciones con rangos ilustrativos, pensado para elegir el mecanismo correcto antes de sentarse a redactar un acuerdo real con alguien. Ningún número de este documento se firma tal cual — ver §5.

**Cómo está organizado:** por **tipo de aporte**, no por persona. El mismo mecanismo aplica a cualquiera que traiga ese tipo de aporte, sea quien sea — pareja, familiar, amigo de toda la vida o conocido reciente. Nadie tiene un mecanismo reservado por su profesión o parentesco; lo que decide el mecanismo es lo que la persona efectivamente aporta, hoy, verificable.

---

## 0. Principios rectores (no negociables)

Estos cuatro principios gobiernan cada mecanismo de este documento. Ningún caso de uso, por atractivo que parezca, puede violarlos.

- **Control máximo del propietario, siempre.** Ningún instrumento de este catálogo permite que un tercero iguale o supere el porcentaje de equity o el poder de voto del propietario. Los socios minoritarios participan de las ganancias en proporción a lo que aportan; la última palabra sobre decisiones del proyecto es siempre del propietario.
- **Los secretos nunca se comparten.** Credenciales de bróker, IPs de servidores de trading en vivo y cualquier secreto de sesión quedan fuera de cualquier acuerdo de colaboración, sin excepción — mismo principio que rige para cualquier usuario del producto (`ADR-0093`, `ADR-0143`).
- **Toda relación empieza pequeña y reversible.** Nadie entra directo a un compromiso mayor. Se empieza en el mecanismo de menor riesgo/menor compromiso que tenga sentido para el aporte de esa persona, y solo se escala con evidencia sostenida de que el aporte es real y continuo.
- **La amistad es el activo más valioso — se protege por diseño, no por buena voluntad.** Toda cláusula de salida se define por adelantado, en frío, nunca se improvisa en el momento de la ruptura. Un desacuerdo de negocio nunca debería poder convertirse en una pelea familiar o de pareja porque el mecanismo de salida ya estaba escrito desde el principio.

---

## 1. El menú de mecanismos

Ocho mecanismos reutilizables. Cada persona puede empezar en uno y, con el tiempo, migrar a otro — nunca al revés de forma automática (no hay upgrade sin evidencia de aporte sostenido).

### 1.1 Comisión pura por adquisición de cliente

**Qué es:** un pago por resultado — cada cliente nuevo que la persona trae genera una comisión, y nada más. Sin equity, sin vesting, sin compromiso de tiempo.

**Cuándo usarlo:** para cualquiera con una red de contactos o capacidad de venta, sin importar si sabe algo de tecnología o de mercados. Es el mecanismo de menor fricción de entrada y de salida de todo el catálogo.

**Rango ilustrativo:** 15%-30% del primer pago o de un porcentaje recurrente sobre lo que ese cliente paga mientras siga activo (referencia de mercado en SaaS/fintech: mediana ~20-23%, algo más conservador en fintech por la carga regulatoria).

**Riesgo a vigilar:** ninguno especialmente delicado — es el mecanismo más simple del catálogo. Sí conviene dejar por escrito qué cuenta como "cliente traído por esta persona" (atribución), para evitar disputas.

**Cómo se sale:** de inmediato. Se deja de pagar comisión sobre clientes nuevos; los clientes ya traídos pueden seguir generando comisión recurrente durante un periodo de gracia corto, o cortarse de tajo — se define de antemano.

---

### 1.2 Cuenta fondeada, estilo prop-firm

**Qué es:** la persona opera (o simplemente explora) con capital que no es suyo — del propietario o de una empresa de fondeo externa real — y se queda con una parte de las ganancias que genere, sin exponerse nunca a las pérdidas.

**Cuándo usarlo:** para cualquiera que quiera usar la plataforma sin necesidad de saber programar ni de entender el negocio a fondo. Puede ser exploración pasiva (copiar operaciones de otra cuenta vía `copy-trading-engine`, sin tomar ninguna decisión) o gestión activa de una cuenta fondeada, según cuánto quiera involucrarse la persona.

**Rango ilustrativo:** split de ganancias 70/30 hasta 90/10 a favor de quien opera, sin responsabilidad sobre pérdidas (referencia de mercado: FTMO y comparables arrancan en 80/20 y suben a 90/10 según desempeño sostenido).

**Riesgo a vigilar:** si el capital viene de una empresa de fondeo externa real (no del propietario), esa relación es entre la persona y la fondeadora — el propietario actúa como habilitador tecnológico, no como empleador ni como parte de ese contrato. Fondear con dinero de terceros y gestionar cuentas de otras personas es terreno regulado (posible actividad de asesoría de inversión no registrada); **cualquier variante que mueva dinero real de un tercero pasa por el skill `general-counsel` antes de operar**, no después.

**Cómo se sale:** se revoca el acceso a la cuenta fondeada. No hay nada que desenredar — la persona nunca tuvo el capital, solo el acceso a operarlo.

---

### 1.3 Socio-empleado sin sueldo, con reparto de ganancias contractual

**Qué es:** la persona trabaja de forma regular y sostenida en el proyecto — código, soporte, contenido, atención a usuarios, lo que sea — sin recibir un sueldo fijo, a cambio de un porcentaje de las ganancias que el proyecto genere, pagado como honorarios variables.

**Cuándo usarlo:** para quien puede comprometer tiempo real de forma sostenida hoy, cuando el proyecto todavía no factura lo suficiente para pagar un sueldo.

**Rango ilustrativo:** honorarios variables ligados a un porcentaje de ingresos o de ahorro generado, definido caso por caso; sin referencia de mercado única porque depende del tiempo comprometido.

**Riesgo a vigilar — el más delicado del catálogo:** en México, si esta relación tiene horario fijo, supervisión directa o dependencia económica total, la autoridad fiscal o laboral puede reclasificarla como relación laboral encubierta ("simulación laboral"), con consecuencias retroactivas (aguinaldo, vacaciones, seguridad social, multas). Para mantenerse en terreno seguro: **sin horario fijo impuesto, sin supervisión de subordinación, entregables definidos por objetivo, no por presencia**.

**Cómo se sale:** se define un preaviso corto (2-4 semanas) y una liquidación de lo ya devengado hasta ese punto. Sin cláusula de permanencia forzosa — nadie queda atrapado.

---

### 1.4 Equity dinámica por contribución (tipo Slicing Pie)

**Qué es:** en vez de fijar un porcentaje de entrada arbitrario, cada aporte (tiempo, dinero, ventas) se convierte en "unidades" acumulables, y el porcentaje de cada quien se recalcula sobre el total acumulado. Se ajusta solo con el tiempo, sin renegociar cada vez que cambia el aporte de alguien.

**Cuándo usarlo:** en la fase actual, de side-project, antes de que exista un vehículo legal formal (SAPI) — para quien aporta tiempo regular y sostenido (mecanismo 1.3) y se le quiere reconocer también en participación futura, no solo en honorarios presentes.

**Rango ilustrativo:** techo agregado de todas las unidades repartibles fijado por adelantado, siempre muy por debajo del porcentaje del propietario — este techo se define en el ledger de cap-table informal (ver §2).

**Riesgo a vigilar:** es un compromiso informal hasta que exista un vehículo legal — debe quedar explícito por escrito que la conversión a equity real (mecanismo 1.6) no es automática ni garantizada, depende de que el vehículo legal se constituya y de que el aporte se sostenga.

**Cómo se sale:** las unidades acumuladas hasta la fecha de salida se "congelan" y se liquidan según la fórmula de recompra (§2) — no se pierden retroactivamente, pero tampoco siguen creciendo tras la salida.

---

### 1.5 Phantom equity — bono contractual atado a valuación

**Qué es:** un derecho económico contractual, sin acciones reales, que paga como si la persona tuviera un porcentaje de la empresa, calculado sobre el valor de la empresa en un evento futuro (venta, ronda de inversión, distribución de utilidades relevante). No es propiedad, es un compromiso de pago.

**Cuándo usarlo:** es la vía más limpia legalmente para dar upside real a alguien de alto valor sin ceder acciones — y la única forma directa de reconocer "equity por trabajo" en México hoy, porque la LGSM prohíbe la aportación de trabajo puro a cambio de acciones reales en sociedades de capital.

**Rango ilustrativo:** 0.5%-2% de "phantom" (equivalente económico, no accionario) para un aporte de alto valor sostenido, pagadero solo si ocurre un evento de liquidez o de distribución definido de antemano.

**Riesgo a vigilar:** al pagarse, se trata fiscalmente como ingreso ordinario (no como ganancia de capital) — validar tratamiento con un contador antes de comprometerse a un monto.

**Cómo se sale:** si la persona sale antes de un evento de pago, el phantom equity acumulado se calcula proporcional al tiempo/vesting cumplido (ver vesting en 1.6) y se liquida o se cancela según lo pactado — nunca se negocia en caliente.

---

### 1.6 Equity real diferida, vía pool de opciones (cuando exista SAPI formal)

**Qué es:** el único camino a equity accionaria real del catálogo. Requiere que exista una sociedad formal (SAPI de C.V.) con un pool de opciones reservado. La persona recibe el derecho a comprar acciones a un precio de ejercicio simbólico (nominal), no a que se le regalen por su trabajo — así se sortea la prohibición de la LGSM sobre aportación de trabajo.

**Cuándo usarlo:** es el techo superior del catálogo — reservado para compromiso operativo crítico y sostenido, con trayectoria ya demostrada en otro mecanismo (nunca como punto de entrada).

**Rango ilustrativo:** vesting de 4 años con cliff de 1 año (nadie consolida nada antes de cumplir un año), vesting mensual después — estándar de mercado para equity de largo plazo.

**Riesgo a vigilar:** no existe hoy — depende de que se constituya el vehículo legal. Prometer esto como punto de partida a alguien es prometer algo que no se puede entregar todavía; se ofrece como destino, con 1.5 como puente mientras tanto.

**Cómo se sale:** cláusula de "reverse vesting" estándar — lo no consolidado se pierde al salir; lo consolidado se recompra a fórmula (§2), nunca se queda flotando como acción minoritaria sin salida.

---

### 1.7 Equity de asesor (grid tipo FAST)

**Qué es:** un porcentaje pequeño de equity (real o phantom, según exista o no el vehículo legal) a cambio de asesoría puntual de alto valor, sin ejecución del día a día.

**Cuándo usarlo:** para experticia que aporta dirección estratégica o técnica sin comprometer tiempo operativo regular.

**Rango ilustrativo:** 0.15%-1.00% según la etapa del proyecto y la profundidad del involucramiento (referencia: grid FAST del Founder Institute), con vesting corto — 2 años, cliff de 0 a 3 meses.

**Riesgo a vigilar:** sin métrica de valor aportado, este mecanismo se vuelve simbólico y puede generar resentimiento en otros socios que sí ejecutan a diario — usar solo cuando el aporte de asesoría es real y recurrente, no una cortesía.

**Cómo se sale:** al dejar de asesorar, se detiene el vesting; lo ya consolidado se mantiene o se recompra según la fórmula de §2.

---

### 1.8 Royalty por estrategia o modelo aportado

**Qué es:** no es equity. Es un pago de un porcentaje del fee de performance (la comisión que ya cobra el sistema sobre las ganancias reales que genera una estrategia) atado específicamente a una estrategia, modelo o pieza de investigación cuantitativa concreta que la persona aportó, mientras esa pieza siga viva en producción generando resultado.

**Cuándo usarlo:** para cualquiera que aporte una pieza de investigación cuantitativa propia y verificable — no importa si es alguien con formación formal en ciencia de datos o alguien que simplemente desarrolló algo que funciona. Desacopla el incentivo del cap table por completo: se paga por resultado medible, no por percepción de valor.

**Rango ilustrativo:** 10%-25% del fee de performance específico que esa pieza genere, mientras siga en producción — se apaga solo si la pieza deja de usarse.

**Riesgo a vigilar:** exige trazabilidad clara de qué resultado es atribuible a qué pieza aportada — sin esa trazabilidad, el mecanismo es imposible de auditar y genera disputas.

**Cómo se sale:** al dejar la pieza de usarse en producción (por decisión del propietario o degradación de resultados), el royalty se apaga. La persona conserva lo ya devengado hasta ese punto.

---

## 2. Salvaguardas transversales de control y salida

Estas cláusulas aplican sobre **cualquier** mecanismo del §1, sin excepción:

- **Ledger de cap-table informal, llevado desde hoy.** El propietario mantiene un registro simple de qué techo agregado tiene comprometido en cada mecanismo con participación acumulable (1.4, 1.5, 1.6, 1.7), sin esperar a que exista una SAPI formal para empezar a llevarlo.
- **Voto no decisorio para minoritarios.** Ningún instrumento de este catálogo trae derecho de voto que pueda bloquear o forzar una decisión del propietario.
- **Recompra a fórmula, no a negociación ad-hoc.** La salida de cualquier persona con participación acumulada se liquida con una fórmula predefinida (ej. valor en libros más un múltiplo simple del aporte acumulado), no con una negociación improvisada en el momento. El propietario es el árbitro designado de cualquier desacuerdo sobre el aporte real, con esa fórmula como referencia objetiva.
- **Período de prueba antes de cualquier compromiso mayor.** Nadie entra directo a 1.4, 1.5, 1.6 o 1.7 — se pasa primero por un mecanismo de bajo compromiso (1.1, 1.2 o el arranque de 1.3) durante un periodo razonable.
- **Cláusula de no cesión.** Ningún instrumento de este catálogo es transferible a un tercero — ni por venta, ni por herencia, ni por cesión informal — sin el consentimiento explícito del propietario.
- **Cláusula de confidencialidad y propiedad intelectual.** Cualquiera que tenga acceso a lógica, algoritmos o estrategias del sistema firma confidencialidad y cede la propiedad intelectual de lo que construya dentro del proyecto al propietario/la sociedad, sin excepción, sin importar el mecanismo bajo el que colabore.
- **Cláusula anti-coalición.** Los socios minoritarios no pueden actuar en bloque para presionar una decisión, ni agregar peso cediéndose instrumentos entre ellos — la cláusula de no cesión ya lo impide, pero se deja explícito.
- **Cláusula de no-punto-único-de-fallo.** Cualquier rol operativamente crítico exige documentación y transferencia de conocimiento como condición del acuerdo — nadie se vuelve indispensable ni gana apalancamiento informal sobre el propietario por ser el único que entiende una parte del sistema.
- **Plantilla de conversación de salida.** Toda salida se trata como "cierre de cuentas, no cierre de relación": se liquida lo pactado según la fórmula, se agradece el aporte, y se aclara desde el principio que salir del proyecto no significa salir de la relación personal.

---

## 3. Casos de uso, organizados por tipo de aporte

**Regla rectora: ningún caso está reservado a una profesión, parentesco o cercanía.** Lo que determina el mecanismo es el tipo de aporte real que la persona trae hoy, no su título ni su relación con el propietario. La misma persona puede calificar para un caso genérico en un momento y para uno particular más adelante — o al revés.

### 3.1 Casos genéricos (abiertos a cualquiera, sin curva técnica exigida)

| Lo que la persona hace | Mecanismo | Exigencia técnica |
|---|---|---|
| Solo usa la plataforma con capital fondeado, como exploradora o como copier pasivo | 1.2 | Ninguna |
| Solo trae clientes o hace networking | 1.1 | Ninguna |
| Aporta tiempo operativo regular sin sueldo (soporte, contenido, atención a usuarios, código, lo que sea sostenido) | 1.3 + 1.4, con umbral objetivo explícito de "graduación" a sueldo/equity mayor (monto de facturación o fecha fija, nunca a discreción del momento) | Variable, según el tipo de tarea |

### 3.2 Casos particulares de involucramiento profundo (abiertos a cualquiera que efectivamente traiga ese aporte)

| Lo que la persona aporta | Mecanismo | Nota |
|---|---|---|
| Una estrategia, modelo o pieza de investigación cuantitativa propia y verificable | 1.5 + 1.8 hoy; 1.6/1.7 como destino futuro cuando exista SAPI | Nunca prometer 1.6/1.7 como punto de entrada — es una entidad legal que todavía no existe |
| Experticia puntual de asesoría de alto valor, sin ejecución diaria | 1.7 | Requiere aporte real y recurrente, no cortesía |
| Compromiso operativo crítico y sostenido de largo plazo, con trayectoria ya demostrada | 1.6 | El techo superior del catálogo — solo alcanzable por trayectoria |

### 3.3 Notas de riesgo transversal por tipo de vínculo (no por caso de uso)

Estas notas aplican sobre cualquier caso de 3.1 o 3.2, según el tipo de vínculo con la persona — no dictan qué mecanismo le toca, solo qué precaución adicional tomar:

- **Vínculo de pareja:** advertir siempre el riesgo de que un instrumento con sabor a equity (1.4, 1.5, 1.6) se vea afectado por el régimen de bienes conyugales si la relación de pareja escala (matrimonio, unión libre reconocida legalmente). Mientras el vínculo sea de pareja, preferir los mecanismos más simples y desechables del catálogo (1.1, 1.2) sin importar qué tan involucrada quiera estar esa persona.
- **Vínculo con dependencia económica real** (alguien atravesando dificultades económicas): priorizar mecanismos que no dependan de una curva de aprendizaje ni de resultados inciertos (1.1, 1.2) — dejando la puerta abierta a que, con el tiempo, esa persona aporte algo que la mueva a un caso de 3.2.
- **Cualquiera que toque lógica o algoritmos**, sin importar el caso de uso: la cláusula de confidencialidad y propiedad intelectual de §2 es obligatoria, sin excepción.

---

## 4. Matriz resumen

| Caso de uso | Mecanismo(s) | Rango ilustrativo | Riesgo a vigilar | Cómo se sale |
|---|---|---|---|---|
| Trae clientes / networking | 1.1 | 15%-30% | Definir atribución de cliente | Inmediata |
| Usa la plataforma con capital fondeado (explorador o copier) | 1.2 | 70/30 a 90/10 | Fondeo de terceros = terreno regulado, pasa por `general-counsel` | Se revoca el acceso, no hay nada que desenredar |
| Aporta tiempo regular sin sueldo | 1.3 | Honorarios variables, definidos caso por caso | Riesgo de simulación laboral — sin horario fijo ni subordinación | Preaviso corto + liquidación de lo devengado |
| Aporta tiempo regular + quiere participación futura | 1.3 + 1.4 | Techo agregado definido en el ledger de §2 | Conversión a equity real no es automática | Unidades se congelan y liquidan a fórmula |
| Aporta una estrategia/modelo propio y verificable | 1.5 + 1.8 | 0.5%-2% phantom + 10%-25% del fee de performance de esa pieza | Exige trazabilidad clara del resultado atribuible | Phantom proporcional al vesting; royalty se apaga al dejar de usarse la pieza |
| Aporta asesoría puntual de alto valor | 1.7 | 0.15%-1.00%, vesting 2 años | Sin métrica de valor, se vuelve simbólico | Se detiene el vesting al dejar de asesorar |
| Compromiso operativo crítico y sostenido, con trayectoria | 1.6 (destino, no entrada) | Vesting 4 años, cliff 1 año | Depende de que exista SAPI formal | Reverse vesting + recompra a fórmula |

---

## 5. Próximos pasos recomendados

Este documento es el menú, no el contrato. Antes de mover dinero o equity real con cualquier persona, en este orden:

1. **El propietario define, por escrito, los números duros de cada caso** — el porcentaje exacto y el techo agregado — porque un abogado valida legalidad, no decide montos. El ledger de cap-table informal de §2 es la verdadera línea de defensa, no este catálogo.
2. **Pasar el caso concreto por el skill `general-counsel`** para blindaje fiscal, laboral y regulatorio específico de México — obligatorio antes de firmar cualquier variante de 1.2 con fondeo de terceros, y recomendado para cualquier variante de 1.3, 1.5 o 1.6.
3. **En paralelo, no después, validar con un contador** el tratamiento de ISR/IVA de comisiones y reparto de ganancias — está fuera del alcance de `general-counsel`.
4. **Si el caso toca equity real o fondeo de terceros**, considerar activar el skill `partner` para la mecánica de cap table y las decisiones estratégicas del vehículo legal cuando exista.

Ningún acuerdo de este documento reemplaza un contrato redactado y revisado para la persona y el caso concreto.
