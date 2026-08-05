mod kicad;
use kicad::{Component, Elec, Pin, Schematic, Side};

fn p_in(num: &str, name: &str, net: &str) -> Pin { Pin::new(num, name, Side::Left, Elec::PowerIn, net) }
fn p_out(num: &str, name: &str, net: &str) -> Pin { Pin::new(num, name, Side::Left, Elec::PowerOut, net) }
fn sig(num: &str, name: &str, net: &str) -> Pin { Pin::new(num, name, Side::Right, Elec::Passive, net) }
fn gpio(num: &str, name: &str, net: &str) -> Pin { Pin::new(num, name, Side::Right, Elec::Bidi, net) }
fn pas_l(num: &str, name: &str, net: &str) -> Pin { Pin::new(num, name, Side::Left, Elec::Passive, net) }
fn pas_r(num: &str, name: &str, net: &str) -> Pin { Pin::new(num, name, Side::Right, Elec::Passive, net) }

const HDR6: &str = "Connector_PinHeader_2.54mm:PinHeader_1x06_P2.54mm_Vertical";
const HDR4: &str = "Connector_PinHeader_2.54mm:PinHeader_1x04_P2.54mm_Vertical";
const HDR22: &str = "Connector_PinHeader_2.54mm:PinHeader_1x22_P2.54mm_Vertical";
const R0805: &str = "Resistor_SMD:R_0805_2012Metric";
const C0805: &str = "Capacitor_SMD:C_0805_2012Metric";
const LED0805: &str = "LED_SMD:LED_0805_2012Metric";
const SW_SPDT: &str = "nks:SS12D00";
const SOT236: &str = "Package_TO_SOT_SMD:SOT-23-6";

fn main() {
    let mut sch = Schematic::new("NkS - SOMNIA carrier");

    // DevKit = two stock 1x22 female headers it plugs into. Pin NUMBER = physical
    // position on the official J1/J3 header (DevKitC-1 v1.1). One driver per rail
    // (3V3 + GND power_out, rest passive) to avoid multiple-power-output ERC errors.
    // J1 header row:
    sch.add(Component::new("J2", "DevKit-J1", HDR22, 20.0, 20.0)
        .pin(p_out("1", "3V3", "+3V3_IN")).pin(pas_l("2", "3V3", "+3V3_IN")).pin(p_out("22", "GND", "GND"))
        .pin(gpio("8", "IO15", "SD_MOSI")).pin(gpio("9", "IO16", "SD_MISO"))
        .pin(gpio("10", "IO17", "SD_SCLK")).pin(gpio("11", "IO18", "SD_CS"))
        .pin(gpio("12", "IO8", "I2C_SDA")).pin(gpio("15", "IO9", "I2C_SCL"))
        .pin(gpio("17", "IO11", "I2S_WS")).pin(gpio("18", "IO12", "I2S_SCK"))
        .pin(gpio("19", "IO13", "I2S_SD")).pin(gpio("20", "IO14", "BTN"))
        .pin(gpio("4", "IO4", "ENABLE"))
        .pin(gpio("21", "5VIN", "VBUS"))
        .pin(gpio("5", "IO5", "VBUS_ADC")));    // J3 header row:
    
    sch.add(Component::new("J3", "DevKit-J3", HDR22, 20.0, 90.0)
        .pin(pas_l("1", "GND", "GND")).pin(pas_l("21", "GND", "GND")).pin(pas_l("22", "GND", "GND"))
        .pin(gpio("18", "IO21", "LED_STAT")));

    // Load switch: parte +3V3_IN -> +3V3_SW, gobernado por ENABLE (GPIO4).
    sch.add(Component::new("U4", "TPS22918", SOT236, 90.0, 60.0)
        .pin(p_in("1", "VIN", "+3V3_IN")).pin(p_in("2", "GND", "GND"))
        .pin(sig("3", "ON", "ENABLE")).pin(sig("4", "CT", "CT_NC"))
        .pin(p_out("5", "VOUT", "+3V3_SW")).pin(sig("6", "QOD", "+3V3_SW")));

    sch.add(Component::new("A1", "INMP441", HDR6, 130.0, 20.0)
        .pin(p_in("1", "VDD", "+3V3_SW")).pin(p_in("2", "GND", "GND")).pin(pas_l("6", "L/R", "GND"))
        .pin(sig("3", "SD", "I2S_SD")).pin(sig("4", "SCK", "I2S_SCK")).pin(sig("5", "WS", "I2S_WS")));
    sch.add(Component::new("A2", "INMP441", HDR6, 130.0, 80.0)
        .pin(p_in("1", "VDD", "+3V3_SW")).pin(p_in("2", "GND", "GND")).pin(pas_l("6", "L/R", "+3V3_SW"))
        .pin(sig("3", "SD", "I2S_SD")).pin(sig("4", "SCK", "I2S_SCK")).pin(sig("5", "WS", "I2S_WS")));

    sch.add(Component::new("J1", "microSD", HDR6, 210.0, 20.0)
        .pin(p_in("1", "VCC", "+3V3_SW")).pin(p_in("2", "GND", "GND"))
        .pin(sig("3", "CS", "SD_CS")).pin(sig("4", "SCK", "SD_SCLK")).pin(sig("5", "MOSI", "SD_MOSI")).pin(sig("6", "MISO", "SD_MISO")));

    sch.add(Component::new("U2", "RTC_DS3231+AT24C32", HDR6, 210.0, 80.0)
        .pin(p_in("1", "VCC", "+3V3_IN")).pin(p_in("2", "GND", "GND"))
        .pin(sig("3", "SCL", "I2C_SCL")).pin(sig("4", "SDA", "I2C_SDA")));

    sch.add(Component::new("U3", "SHT30", HDR4, 210.0, 140.0)
        .pin(p_in("1", "VIN", "+3V3_SW")).pin(p_in("2", "GND", "GND"))
        .pin(sig("3", "SCL", "I2C_SCL")).pin(sig("4", "SDA", "I2C_SDA")));

    sch.add(Component::new("R1", "1k", R0805, 130.0, 140.0).pin(pas_l("1", "~", "LED_STAT")).pin(pas_r("2", "~", "LED_A")));
    sch.add(Component::new("D1", "LED", LED0805, 130.0, 180.0).pin(pas_l("1", "A", "LED_A")).pin(pas_r("2", "K", "GND")));

    sch.add(Component::new("SW1", "SW_SPDT", SW_SPDT, 60.0, 180.0)
    .pin(pas_l("2", "~", "BTN")).pin(pas_r("1", "~", "+3V3_SW")).pin(pas_r("3", "~", "GND")));

    sch.add(Component::new("C6", "100n", C0805, 60.0, 220.0).pin(pas_l("1", "~", "BTN")).pin(pas_r("2", "~", "GND")));

    for (r, x) in [("C1", 130.0), ("C2", 175.0), ("C3", 220.0), ("C4", 265.0), ("C5", 310.0)] {
        sch.add(Component::new(r, "100n", C0805, x, 250.0).pin(pas_l("1", "~", "+3V3_SW")).pin(pas_r("2", "~", "GND")));
    }
    sch.add(Component::new("C7", "10u", C0805, 20.0, 250.0).pin(pas_l("1", "~", "+3V3_IN")).pin(pas_r("2", "~", "GND")));

    // Divisor VBUS->ADC: vigila los 5V del USB (deteccion tiron de cable).
    // 5V -> R3(100k) -> nodo(VBUS_ADC) -> R4(100k) -> GND  => ~2.5V al ADC (GPIO5).
    sch.add(Component::new("R3", "100k", R0805, 90.0, 200.0)
        .pin(pas_l("1", "~", "VBUS")).pin(pas_r("2", "~", "VBUS_ADC")));
    sch.add(Component::new("R4", "100k", R0805, 90.0, 230.0)
        .pin(pas_l("1", "~", "VBUS_ADC")).pin(pas_r("2", "~", "GND")));



    let out = sch.render();
    std::fs::write("nks.kicad_sch", &out).expect("write .kicad_sch");

    let pro = format!(
        "{{\n  \"board\": {{ \"design_settings\": {{}} }},\n  \"meta\": {{ \"filename\": \"nks.kicad_pro\", \"version\": 1 }},\n  \"schematic\": {{}},\n  \"sheets\": [ [ \"{}\", \"Root\" ] ]\n}}\n",
        sch.root_uuid
    );
    std::fs::write("nks.kicad_pro", &pro).expect("write .kicad_pro");
    std::fs::write("nks.kicad_sym", sch.render_sym_lib()).expect("write .kicad_sym");
    std::fs::write("sym-lib-table", sch.render_sym_lib_table()).expect("write sym-lib-table");

    println!("wrote nks.kicad_sch ({} bytes) + project (stock footprints only)", out.len());
}
