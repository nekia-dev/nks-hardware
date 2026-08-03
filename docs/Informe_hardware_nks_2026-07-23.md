# INFORME DE INGENIERÍA — Placa NkS (carrier)
# Nekia Systems S.L. · PROYECTO SOMNIA
# Fecha: 23 de julio de 2026
# Alcance: del pinout congelado a los Gerbers verificados

---

## 1. RESUMEN EJECUTIVO

Se cierra el diseño de la placa portadora del NkS: esquemático con ERC a 0,
layout con DRC a 0 errores y 0 conexiones sin rutar, y Gerbers generados y
verificados capa por capa. Placa de **80 × 90 mm, 2 capas**, con plano de masa
en ambas caras.

En paralelo se reorientó el producto (de dispositivo médico a producto de
consumo con la privacidad como bandera), se añadió el SHT30 como sensor de
ambiente y se pasó a **captura estéreo con dos INMP441**.

Queda pendiente lo único que el DRC no puede validar: **el mapeo de pines del
DevKit contra la placa física**.

---

## 2. DECISIONES DE PRODUCTO Y HARDWARE

### 2.1 Segundo micrófono (estéreo)

Se añade un segundo INMP441 aprovechando el pin `L/R` del integrado: un módulo
con `L/R`→GND (canal izquierdo) y otro con `L/R`→3V3 (canal derecho). Ambos
comparten `WS`/`SCK`/`SD`.

**Coste real:** cero GPIO adicionales y coherencia de fase gratuita (comparten
reloj). El trabajo se traslada al software: I2S a 2 canales, formato `.nka` con
dos canales (dobla almacenamiento) y colocación intencional de A1/A2 en el
layout (define el *baseline* estéreo).

**Descartado:** matrices de 4+ micrófonos. El INMP441 no soporta TDM
multicanal; un array mayor exigiría otro sensor y otro diseño.

### 2.2 SHT30 (temperatura + humedad)

Entra al bus I2C existente (0x44) sin GPIO nuevos. Se acepta como **feature de
producto** ("ambiente del dormitorio"), no como dato suelto: el DS3231 ya ofrece
temperatura aproximada, pero mide su propio die y no sirve como medida ambiental.

**Restricción de layout:** aislar del DevKit y su regulador; de lo contrario
reporta temperatura alta y humedad baja.

### 2.3 Sin conector USB-C propio

El DevKitC-1 ya expone el USB-C nativo del ESP32-S3 (vía para el futuro USB Mass
Storage). Se elimina el bloque de conector independiente.

### 2.4 Contorno rectangular

Se aplaza el formato circular: el rectangular abarata el prototipo y el contorno
no aporta valor hasta validar la electrónica.

---

## 3. FLUJO DE TRABAJO ADOPTADO

El esquemático se generó **programáticamente con una herramienta propia en
Rust** (`tools/nks-sch-gen`), que emite `.kicad_sch`, `.kicad_sym` y
`sym-lib-table`. El usuario ejecuta, revisa y ajusta en KiCad 10.

**Justificación:** reproducibilidad y control de versiones del esquemático; se
evita Python por decisión de toolchain del proyecto.

**Limitación asumida:** un esquemático generado queda correcto en conectividad
(net labels) pero no en estética. La reorganización visual, si se quiere, es
manual.

---

## 4. INCIDENCIAS RESUELTAS (y su causa raíz)

### 4.1 Formato `.kicad_sch` rechazado por KiCad

**Causa:** los sub-símbolos anidados se emitían con el prefijo de biblioteca
(`nks:R1_0_1`). KiCad exige el nombre sin prefijo (`R1_0_1`).

### 4.2 Referencias marcadas como "sin anotar"

**Causa:** las instancias apuntaban a la ruta `/` en lugar de a la UUID de la
hoja raíz. Corregido en el generador.

**Relacionado:** referencias tipo `J1H`/`J3H` son inválidas en KiCad (deben
terminar en número). Renombradas a `J2`/`J3`.

### 4.3 "No se ha encontrado la huella 'nks:ESP32-S3-DevKitC-1'"

**Causa:** se creó un footprint a medida en una biblioteca de proyecto
(`nks.pretty` + `fp-lib-table`) que KiCad no cargaba de forma fiable.

**Solución adoptada:** eliminar la biblioteca propia y representar el DevKit
como **dos conectores 1×22 estándar** (`J2`, `J3`) con huellas de la librería
oficial. Cada pin lleva el número de su **posición física** en los headers J1/J3
del DevKitC-1 v1.1.

> **Regla aprendida:** evitar bibliotecas de footprints propias mientras existan
> huellas estándar equivalentes.

### 4.4 90 infracciones de margen en el layout

**Causa:** se había colocado un plano de +3V3 en `F.Cu`, la misma cara por la
que se rutaban las señales. Plano y pistas competían por el mismo espacio.

**Solución:** eliminar el plano de 3V3; el 3V3 pasa a pistas. `F.Cu` queda para
señales.

> **Regla aprendida:** un plano ocupa la cara entera; no se comparte cara entre
> plano y rutado manual denso.

### 4.5 Rutado manual bloqueado

Con las señales en una sola cara, el router interactivo dejaba de encontrar
camino (SD_CS, SD_MISO, SD_SCLK, LED_STAT, +3V3). No era un error de diseño:
falta de espacio.

**Solución:** autorouter externo (**Freerouting**), que usa ambas caras y coloca
vías. Resultado: 42 nets sin rutar → **0**, en 1,27 s.

**Nota operativa:** el GUI de Freerouting v2.2.4 falla
(`parent_iconified ... null`). Se usó en modo terminal:

```
./freerouting -de nks.dsn -do nks.ses
```

### 4.6 KiCad se cerraba al importar el `.ses`

**Causa:** el footprint del pulsador tiene pads con número duplicado (`1`,`1`,
`2`,`2`). Freerouting los desambigua como `SW1-1@1`; el importador Specctra de
KiCad no interpreta esa sintaxis y aborta con *"No se ha encontrado la
referencia 'SW1'"*.

**Solución adoptada:** convertir el rutado del `.ses` (271 segmentos + 4 vías) e
insertarlo directamente en el `.kicad_pcb`, evitando el importador. Transformada
de coordenadas verificada contra tres componentes (unidades/10000 = mm, Y
invertida).

### 4.7 Pads de GND sin conectar al plano

Persistían pads de masa sueltos (A1, A2, U2, U3, D1) pese a existir plano en
`B.Cu`.

**Causa raíz:** `D1` es **SMD** — su pad de masa vive solo en `F.Cu` y no puede
alcanzar un plano situado en la cara opuesta. Además, el plano inferior había
quedado **fragmentado** por las pistas que el autorouter trazó en `B.Cu`,
aislando pads pasantes en islas.

**Solución:** añadir una **segunda zona de GND en `F.Cu`**. Con masa en ambas
caras, todo pad —SMD o pasante— encuentra cobre de masa en su propia cara.
Resultado: `unconnected_items` → **0**.

> **Regla aprendida:** con componentes SMD, el plano de masa debe existir en la
> cara donde viven sus pads.

### 4.8 Avisos `starved_thermal`

Tres pads (A1, A2, U3) conectaban al plano con 1 radio de alivio térmico frente
a los 2 exigidos por defecto.

**Solución:** *Ajustes de la placa → Requerimientos → Cantidad mínima de radios
de alivio térmico* = **1**. Los pads estaban conectados; era una exigencia
cosmética.

### 4.9 Incidencia: borrado global excesivo

Un `Editar → Borrado global` para limpiar pistas de GND eliminó buena parte del
rutado (35 nets sin rutar). Se recuperó desde una copia previa del archivo
rutado. **Precaución:** no usar borrado global sin copia de seguridad previa.

---

## 5. VERIFICACIÓN DE GERBERS

Pack analizado archivo por archivo:

| Capa | Estado |
|---|---|
| `F_Cu` | ✅ 4727 trazos |
| `B_Cu` | ✅ 4056 trazos |
| `F_Mask` / `B_Mask` | ✅ |
| `F_Silkscreen` / `B_Silkscreen` | ✅ |
| `Edge_Cuts` | ✅ rectángulo cerrado 80 × 90 mm |
| `PTH.drl` | ✅ 80 agujeros (brocas 0,3 / 1,0 / 1,1 mm) |
| `.gbrjob` | ✅ todas las capas declaradas |

**Recuento cruzado de taladros:** 44 (J2+J3) + 18 (A1, A2, J1) + 10 (U2, U3) +
4 (SW1) + 4 (vías) = **80**. Coincide.

**Conclusión:** pack completo y coherente, apto para fabricación.

---

## 6. PENDIENTES

### Crítico (antes de soldar)
1. **Verificar el mapeo de pines del DevKit con multímetro.** Es el único
   eslabón no validado: el footprint se construyó desde la documentación
   oficial, y ni el DRC ni los Gerbers pueden detectar un error ahí.

### Fabricación
2. Pedir la placa (JLCPCB / PCBWay) y trasladar el **precio real** al modelo de
   costes `NkS_costes.xlsx` (celda de fabricación de placa).
3. Cotizar componentes y carcasa — entregable de costes: **última semana de
   agosto de 2026**.

### Hardware por confirmar
4. **Pull-ups de I2C**: se asumen presentes en los breakouts de RTC y SHT30.
   Verificar; si ninguno los trae, añadir un par de 4,7 kΩ.
5. **Orden pin↔pad** de los headers de módulo frente a los breakouts reales.

### Mejoras aplazadas
6. Ensanchar las pistas de `+3V3` (actualmente 0,2 mm por defecto del
   autorouter). Suficiente para el consumo actual, mejorable.
7. Formato circular y reducción de tamaño (hay margen sobrado en 80 × 90 mm).
8. Modelos 3D de los módulos, para validar volumen frente a la futura carcasa.

### Software (rama aparte)
9. Refactor a `single-idf`: partir del firmware actual y retirar lo que ya no
   monta la placa (BH1750, BMP280, OLED), pasar el I2S a 2 canales y versionar
   el formato `.nka` para audio estéreo + temperatura/humedad.

---

## 7. NOTAS DE MÉTODO

- **El DRC valida coherencia, no corrección.** Comprueba que la placa cumple las
  reglas y coincide con el esquemático; no que el esquemático sea correcto.
- **Distinguir conexión lógica de conexión física.** Las líneas finas
  (*ratsnest*) expresan intención; solo el cobre conecta. El contador "sin
  rutar" es el indicador fiable, no el DRC filtrado por severidad.
- **En una placa de 2 caras:** masa por plano, señales por pista. Los pads de
  masa bajan al plano con vías; no se rutea GND con pistas.
- **Copia de seguridad antes de operaciones masivas** (borrado global, importes
  de sesión, cambios de zona).

---

*Nekia Systems S.L. — CONFIDENCIAL*
*"Your Data, Your Device." 🌙*