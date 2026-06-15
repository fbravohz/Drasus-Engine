## 10. Propiedades del Sistema (SLAs, Limitaciones)

| Métrica | Objetivo | Cómo se logra |
|---|---|---|
| **Barras Algorítmicas** | Soporte Nativo | Motor `algorithmic-bars` (Renko, Range, Volume). |
| **Latencia Ingesta** | < 10ms (dato → base datos) | Entrada/salida asincrónica, parsing con compilación automática. |
| **Latencia Señal** | < 50ms (precio → orden propuesta) | Lógica pura, sin consultas a base de datos. |
| **Rendimiento** | Más rápido que MT5/SQX/QuantConnect en igual hardware (ADR-0114; sin KPI absoluto) | Cálculos en paralelo en CPU, hilos nativos Rust. |
| **Reproducibilidad** | 100% (simulación = vivo) | Lógica pura, semilla fija, estados numéricos exactos. |
| **Disponibilidad** | 99.5% (mercado cripto) | Reintentos automáticos, fallback manual. |
| **Velocidad Pruebas** | < 1ms por prueba unitaria | Sin base de datos, lógica pura. |

---

