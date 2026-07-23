# NkS — Placa portadora (carrier) · PROYECTO SOMNIA

**Nekia Systems S.L.** · Hardware del monitor de sueño NkS
*"Your Data, Your Device."*

Placa portadora para ESP32-S3 DevKitC-1 N16R8. Diseño "árbol de pascua apagado":
hardware mínimo, el valor está en el software.

---

## Estado

| Etapa | Estado |
|---|---|
| Hardware congelado | ✅ |
| Esquemático + ERC | ✅ 0 errores |
| Layout + DRC | ✅ 0 errores / 0 sin conectar |
| Gerbers | ✅ generados y verificados |
| Fabricación | ⏳ pendiente de pedido |
| Validación física | ⏳ pendiente (ver §Validación) |

---

## Datos de fabricación

| Parámetro | Valor |
|---|---|
| Dimensiones | **80 × 90 mm** (8 × 9 cm) |
| Capas | 2 (F.Cu / B.Cu) |
| Grosor | 1,6 mm (estándar) |
| Contorno | Rectangular |
| Plano de masa | GND en **ambas** caras |
| Pistas | 271 segmentos + 4 vías |
| Taladros | 80 (brocas 0,3 / 1,0 / 1,1 mm) |
| Ancho de pista | 0,2 mm (señal) |
| Componentes | 18 |

> El formato circular queda aplazado: el rectangular abarata el prototipo.

---

## Arquitectura

El ESP32-S3 DevKitC-1 **no va soldado**: se enchufa sobre dos tiras de pines
hembra 1×22 (`J2` = fila J1 del DevKit, `J3` = fila J3). Los sensores y
periféricos son módulos comerciales (breakouts) montados sobre headers.

**Consecuencia:** el fabricante entrega la placa **desnuda**. El montaje
(soldadura de headers y pasivos + enchufar el DevKit) es manual.

---

## Pinout congelado

| Bus | Señal | GPIO | Posición en header |
|---|---|---|---|
| I2S | WS | 11 | J2 pos. 17 |
| I2S | SCK | 12 | J2 pos. 18 |
| I2S | SD | 13 | J2 pos. 19 |
| SPI | MOSI | 15 | J2 pos. 8 |
| SPI | MISO | 16 | J2 pos. 9 |
| SPI | SCLK | 17 | J2 pos. 10 |
| SPI | CS | 18 | J2 pos. 11 |
| I2C | SDA | 8 | J2 pos. 12 |
| I2C | SCL | 9 | J2 pos. 15 |
| — | Botón on/off | 14 | J2 pos. 20 |
| — | LED estado | 21 | J3 pos. 18 |

**Alimentación:** 3V3 en J2 pos. 1–2 · GND en J2 pos. 22 y J3 pos. 1, 21, 22.

**Pines reservados (no usar):** strapping 0/3/45/46 · PSRAM octal (N16R8)
33–37 · USB nativo 19/20.

> El pinout físico J1/J3 procede de la guía oficial de Espressif del
> ESP32-S3-DevKitC-1 v1.1. **Verificar contra la placa real antes de soldar**
> (ver §Validación).

---

## BOM

| Ref | Componente | Interfaz | Notas |
|---|---|---|---|
| J2, J3 | Header 1×22 hembra ×2 | — | Zócalo del DevKit (paso 2,54 mm, filas a 22,86 mm) |
| A1 | INMP441 | I2S | Canal izquierdo — `L/R` → GND |
| A2 | INMP441 | I2S | Canal derecho — `L/R` → 3V3 |
| J1 | Lector microSD | SPI | — |
| U2 | RTC DS3231 + AT24C32 | I2C | 0x68 / 0x57 |
| U3 | SHT30 | I2C | 0x44 — temperatura + humedad |
| SW1 | Pulsador 6 mm | — | On/off, activo-bajo |
| R1 | 1 kΩ (0805) | — | Serie del LED |
| R2 | 10 kΩ (0805) | — | Pull-up del botón |
| D1 | LED (0805) | — | Indicador de estado |
| C1–C6 | 100 nF (0805) | — | Desacoplo + antirrebote |
| C7 | 10 µF (0805) | — | Bulk en el riel 3V3 |

**Aparte (no van en la placa):** ESP32-S3 DevKitC-1 N16R8, tarjeta microSD,
cable USB-C, carcasa.

Todas las huellas proceden de las **librerías estándar de KiCad**; el proyecto
no depende de librerías propias de footprints.

---

## Estructura del repositorio

```
nks.kicad_pro          Proyecto KiCad
nks.kicad_sch          Esquemático
nks.kicad_pcb          Placa (rutada, DRC limpio)
nks.kicad_sym          Biblioteca de símbolos del proyecto
sym-lib-table          Registro de la biblioteca de símbolos
docs/                  Informes de ingeniería
tools/nks-sch-gen/     Generador del esquemático en Rust
```

### Abrir el proyecto

Abrir `nks.kicad_pro` con **KiCad 10.0**. Los cuatro archivos de proyecto y
`sym-lib-table` deben estar en la misma carpeta.

---

## Validación (pendiente al recibir la placa)

El DRC a cero garantiza que la placa **coincide con el esquemático** y es
fabricable. **No** garantiza que el mapeo de pines del DevKit sea correcto.

1. **Continuidad con multímetro** (placa desnuda): comprobar que cada pad de
   los módulos pita con la posición esperada de J2/J3. Prioridad: I2C, I2S, SPI.
2. **Alimentación**: 3V3 y GND presentes en cada módulo antes de montar nada.
3. **Bloque a bloque**: alimentación → I2C (¿responde el RTC en 0x68?) → I2S →
   microSD. No montar todo de golpe.

---

## Decisiones de diseño registradas

- **2× INMP441 en estéreo** por el truco `L/R`: comparten los tres pines I2S sin
  GPIO adicionales. Habilita separar el ronquido propio del de la pareja.
- **SHT30 incorporado** como sensor de ambiente (temp + humedad). Restricción de
  layout: aislarlo térmicamente del DevKit y su regulador.
- **Sin conector USB-C propio**: se usa el USB-C nativo del DevKit.
- **BMP280 descartado**; **BH1750 aplazado**.
- **Plano de GND en ambas caras**: necesario porque D1 es SMD (su pad de masa
  vive solo en F.Cu) y el plano inferior quedó fragmentado por el rutado.

---

*Nekia Systems S.L. — CONFIDENCIAL*