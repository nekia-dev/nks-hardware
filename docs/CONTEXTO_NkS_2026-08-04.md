# INFORME DE INGENIERÍA / CONTEXTO — NkS
# Nekia Systems S.L. · Ivan (nekia-dev)
# Fecha de cierre: 4 de agosto de 2026
# Alcance: generador Rust del esquematico recreado + refactorizado (Capa 1: SW1->SPDT,
#          quitar R2). Decisiones: SHT30 se mantiene, distribucion fisica, estrategia.
# Destino: sesion siguiente (decidir sincronizacion sch<->pcb; Capas 2-3: ENABLE, TPS22918)

---

## 0. CÓMO TRABAJAMOS
- Español, "nosotros", instrucciones cortas con el porque. Medir antes de teorizar, un
  cambio verificable a la vez, copia de seguridad antes de operaciones de riesgo.
- **NUEVO ENFOQUE DE TRABAJO (definido hoy, valido a futuro):** el **generador en Rust
  (`nks-sch-gen`) es la FUENTE DE VERDAD del esquematico**. Los cambios de diseño se hacen
  en el generador (main.rs), se regenera el `.kicad_sch`, y el LAYOUT (PCB) lo ajusta Ivan
  a mano en KiCad 10. El agente ayuda a escribir/modificar el codigo Rust; Ivan compila y
  ejecuta en su maquina (`ltc`). "Rust si, Python no" (NO usar kiutils; el generador es Rust).

---

## 1. DECISIÓN ESTRATÉGICA DE FONDO (importante)
- **Reducir la dependencia del asistente.** El conocimiento del proyecto debe vivir en
  DOCUMENTACION y PROCEDIMIENTOS reproducibles, no en el asistente (que NO persiste entre
  sesiones; cada sesion es una instancia nueva que solo sabe lo que se le trae en contexto)
  ni solo en Ivan (una sola persona). Riesgo identificado por Ivan: "si no estas, el negocio
  es una ruina".
- **Plan:** cerrar la placa -> fase de documentacion/procedimientos del sistema NkS.
  Objetivo: negocio transferible, defendible ante inversor, y base para la linea de formacion
  (electronica embebida + Rust). La documentacion debe hacer a Ivan AUTONOMO, no perpetuar
  la dependencia.
- **SUBVENCION APROBADA: 7.500 EUR — pago octubre 2026.** Condiciona calendario (alta de
  autonomo, forma juridica societario vs individual, compras, ROI/IVA).

---

## 2. HITO DE HOY — generador Rust recreado + Capa 1 refactorizada

### Contexto: DOS lineas de trabajo desincronizadas (detectado y en via de resolucion)
- Una instancia ANTERIOR construyo un generador de esquematico en Rust (`nks-sch-gen`) que
  produce el `.kicad_sch` por codigo. Pero ese generador reflejaba el diseño VIEJO (SW1
  pulsador, R2 presente) — NO incorporaba el trabajo hecho a mano en las sesiones recientes.
- Esta sesion (y las previas): se edito el `.kicad_pcb` a MANO (SW1->SPDT, R2 borrado,
  rutado, DRC 0). -> Las dos lineas divergian. Hoy se empieza a consolidar en el generador.

### Recreado el proyecto Cargo
- Ubicacion: **`~/Documentos/NEKIA-SYSTEM/nks-hardware/tools/nks-sch-gen/`** (dentro del repo,
  versionado con el diseño). Ficheros: `src/main.rs` (declaracion de bloques) + `src/kicad.rs`
  (motor que escribe el `.kicad_sch`, 485 lineas, solo std). `cargo build` OK.
- NOTA: el proyecto original no estaba en la maquina de Ivan (lo construyo la otra instancia
  en su entorno); Ivan solo tenia los ficheros sueltos en `refactoriza/`. Se recreo el Cargo.
- Aviso rust-analyzer "failed to find any projects in nks-hardware": INOFENSIVO — el editor
  busca Cargo.toml en la raiz del repo, pero esta en tools/nks-sch-gen/. Solucion opcional:
  `.vscode/settings.json` con `"rust-analyzer.linkedProjects":["tools/nks-sch-gen/Cargo.toml"]`.

### Capa 1 aplicada en main.rs (SINCRONIZA generador con la realidad) — VERIFICADA
1. Constante footprint: `const SW_SPDT: &str = "nks:SS12D00";` (antes BTN6MM del pulsador).
2. `R2` (pull-up 10k) ELIMINADO.
3. `SW1` -> SPDT 3 pines: `pin 2 (comun)->BTN`, `pin 1->+3V3`, `pin 3->GND`. Footprint
   `nks:SS12D00`. Value "SW_SPDT".
- `cargo run --release` OK -> genero `nks.kicad_sch` (52409 bytes) en tools/nks-sch-gen/
  (ruta relativa; NO piso el .kicad_sch de la raiz — a proposito).
- **Verificacion (grep sobre el generado):** R2=0 ✓; SS12D00 presente ✓; **17 componentes**
  (antes 18) ✓; SW1 con nets +3V3/BTN/GND (uno de cada) ✓. Capa 1 CERRADA.

---

## 3. DECISIÓN PENDIENTE CLAVE — sincronizacion sch<->pcb (NO resuelta, para mañana)
El `.kicad_sch` generado esta limpio, PERO no se puede volcar sin mas al proyecto:
- El PCB de la raiz ya tiene el LAYOUT trabajado a mano (rutado, DRC 0), con los UUID de
  componentes del esquematico VIEJO.
- El generador crea UUID NUEVOS -> si se reemplaza el .sch y se hace *Update PCB from
  Schematic*, KiCad veria todos los componentes como nuevos -> borraria el layout/rutado
  existente. Riesgo de perder el trabajo manual del PCB.

**Opciones (decidir con mente lucida):**
- **A) El generador manda -> rehacer el layout desde cero.** Coherente con el nuevo enfoque.
  Como la reorganizacion que viene (USB-C detras, panel frontal, +TPS22918/ENABLE) va a
  rehacer el layout DE TODOS MODOS, la perdida del rutado actual es asumible. RECOMENDADA.
- **B) Mantener el PCB actual a mano; el generador queda de referencia.** Conserva el rutado
  pero perpetua la divergencia.

Si se va por A: meter tambien Capas 2-3 (ENABLE, TPS22918) en el generador ANTES de
reconstruir el layout UNA sola vez (no reconstruir dos veces).

---

## 4. REFACTORIZACIÓN PENDIENTE DEL GENERADOR (Capas 2-3)
- **Capa 2:** añadir `ENABLE` -> sacar `GPIO4` (J2 pos.4, libre, RTC-capable, validado) a un
  net ENABLE en el header J2.
- **Capa 3:** añadir `TPS22918` (load switch, parte +3V3 en +3V3_IN/+3V3_SW, EN=ENABLE) +
  divisor VBUS->ADC (5Vin en J2 pos.21; ADC libre GPIO5/6/7) + posible bulk mayor.
  Requiere definir pines del TPS22918 y valores del divisor.
- Nota: C6 (100n) sigue conectado a BTN/GND (antirrebote) — inofensivo con el SPDT, se deja.

---

## 5. DECISIONES DE PRODUCTO DE HOY
- **SHT30 (U3): SE MANTIENE.** Se valoro quitarlo por coste (ajustados), pero: (a) es un
  diferenciador de producto (ambiente del dormitorio = dato clinico/venta); (b) ya esta
  integrado; (c) ahorro modesto (~2-4 EUR). Opcion elegante disponible si hiciera falta:
  dejar el socket y hacer el MODULO opcional (el firmware detecta si responde en I2C 0x44).
  Por ahora se queda como esta.
- **Distribucion fisica de la placa (requisito de LAYOUT, no de esquematico):**
  - Borde FRONTAL: **LED (D1) + interruptor SS12D00G (SW1)** — interaccion del usuario.
  - Borde POSTERIOR: **USB-C del DevKit ("cola")** — alimentacion oculta, no cruza por
    delante. Implica ORIENTAR el DevKit sobre J2/J3 para que su USB-C nativo caiga detras.
  - Esto NO afecta al generador (el esquematico no sabe de frontal/posterior); se aplica al
    colocar componentes en el PCB.

---

## 6. ESTADO GLOBAL
- **Firmware `single-idf`:** terminado para la fase (soft power latch integrado y validado,
  commiteado). USB Mass Storage (ST-023) imprescindible (habilita "caja con USB-C").
- **Placa:** interruptor cerrado en el PCB actual (DRC 0, commiteado). Generador Rust
  sincronizado (Capa 1). Pendiente: decidir estrategia sch<->pcb + Capas 2-3 + reorganizar
  layout (USB-C detras) + regenerar Gerbers.
- **Costes:** curva PCB real (Saving: ~1.95/ud a 100u); BOM en CSV entregado. Los dos
  presupuestos base los construye Ivan (entregable ultima semana agosto, accionista).
- **Carcasa:** referencia visual validada (box router, rejilla, patas, panel USB-C detras +
  LED/switch delante). Cotas reales ~9x9.5cm; alto por confirmar con modelo 3D. Punto abierto:
  mecanica del actuador del switch (SS12D00G sin pestañas -> carro en carcasa o SS12F15).
- **Alimentacion:** por USB-C (regulador del DevKit -> 3V3). Divisor VBUS vigila los 5V.
  Pendiente decidir si la caja incluye cargador USB (recomendado, ~3-5 EUR).
- **Prioritario no-firmware:** rotar credenciales del historial git (`9991c9e`, `d751b87`).
  SIGUE PENDIENTE.

---

## 7. TEMAS ABIERTOS
- Estrategia de sincronizacion sch<->pcb (A vs B) — PRIMERA decision de mañana.
- Mecanica del actuador del switch — con la carcasa.
- Modelos 3D de modulos -> alto real de carcasa.
- Mapeo pines DevKit<->placa sin validar con multimetro (critico antes de soldar).
- Forma juridica + ROI/IVA — pendiente de la subvencion (oct 2026).
- Documentar donde vive y como se ejecuta el generador (hecho parcialmente en este informe:
  tools/nks-sch-gen/, `cargo run --release` genera el .kicad_sch en ese directorio).

---

## 8. SIGUIENTE SESIÓN — arranque
1. Decidir estrategia sch<->pcb (recomendada A: generador manda, rehacer layout).
2. Si A: meter Capa 2 (ENABLE) y Capa 3 (TPS22918 + divisor) en el generador.
3. Regenerar, validar (ERC en KiCad 10), y reconstruir el layout UNA vez con la
   distribucion nueva (USB-C detras, LED+switch delante).
4. Regenerar Gerbers -> precio PCB final.

---

*Nekia Systems S.L. — CONFIDENCIAL · "Your Data, Your Device." 🌙*
