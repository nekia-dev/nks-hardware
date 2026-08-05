//! Minimal KiCad `.kicad_sch` (v7 / format 20230121) emitter.
//!
//! Design choices for a *generated* schematic:
//! - Every part is drawn as a rectangular symbol with pins on left/right.
//!   A carrier board is almost all connector-like blocks, so one generic
//!   rectangular symbol covers the ESP32-S3 module, the mic breakouts, the
//!   microSD / RTC / SHT30 modules, plus 2-pin passives.
//! - Connectivity is expressed with **net labels** at each pin, never by
//!   trying to route wires between exact pin coordinates. Same label text =
//!   same net. This is what makes a programmatically generated schematic
//!   survive ERC without pixel-perfect geometry.

use std::fmt::Write;

/// Grid unit KiCad works on (1.27 mm). All coordinates are multiples of this.
pub const G: f64 = 1.27;

#[derive(Clone, Copy, PartialEq)]
pub enum Side {
    Left,
    Right,
}

/// Electrical pin type -> KiCad token. Kept small; enough for ERC to be useful.
#[derive(Clone, Copy)]
pub enum Elec {
    PowerIn,
    PowerOut,
    Passive,
    Bidi,
    Input,
    Output,
}

impl Elec {
    fn token(self) -> &'static str {
        match self {
            Elec::PowerIn => "power_in",
            Elec::PowerOut => "power_out",
            Elec::Passive => "passive",
            Elec::Bidi => "bidirectional",
            Elec::Input => "input",
            Elec::Output => "output",
        }
    }
}

/// One pin on a rectangular symbol.
#[derive(Clone)]
pub struct Pin {
    pub number: String,
    pub name: String,
    pub side: Side,
    pub elec: Elec,
    /// Net this pin belongs to. Emitted as a label near the pin end.
    pub net: String,
}

impl Pin {
    pub fn new(number: &str, name: &str, side: Side, elec: Elec, net: &str) -> Self {
        Pin {
            number: number.into(),
            name: name.into(),
            side,
            elec,
            net: net.into(),
        }
    }
}

/// A placed component: a rectangular symbol instance with its pins/nets.
pub struct Component {
    pub reference: String,
    pub value: String,
    pub footprint: String,
    pub pins: Vec<Pin>,
    /// Top-left placement anchor on the sheet, in mm.
    pub x: f64,
    pub y: f64,
    uuid: String,
}

impl Component {
    pub fn new(reference: &str, value: &str, footprint: &str, x: f64, y: f64) -> Self {
        Component {
            reference: reference.into(),
            value: value.into(),
            footprint: footprint.into(),
            pins: Vec::new(),
            x,
            y,
            uuid: uuid_from(&format!("{reference}-{value}-{x}-{y}")),
        }
    }

    pub fn pin(mut self, p: Pin) -> Self {
        self.pins.push(p);
        self
    }

    fn left_pins(&self) -> Vec<&Pin> {
        self.pins.iter().filter(|p| p.side == Side::Left).collect()
    }
    fn right_pins(&self) -> Vec<&Pin> {
        self.pins.iter().filter(|p| p.side == Side::Right).collect()
    }

    /// Body height in grid rows = max pins on either side, min 2.
    fn rows(&self) -> usize {
        self.left_pins().len().max(self.right_pins().len()).max(2)
    }

    fn body_w(&self) -> f64 {
        // wide enough for pin names; forced to an EVEN number of grid units
        // so that half-width (used to place pins) stays on the 1.27mm grid.
        let name_len = self
            .pins
            .iter()
            .map(|p| p.name.len())
            .max()
            .unwrap_or(3) as f64;
        let mut units = ((name_len * 1.3).ceil() as i64).max(12);
        if units % 2 != 0 {
            units += 1;
        }
        units as f64 * G
    }

    fn body_h(&self) -> f64 {
        self.rows() as f64 * 2.0 * G
    }
}

fn uuid_from(seed: &str) -> String {
    // Deterministic pseudo-UUID (FNV-1a based) so re-runs produce stable files.
    let mut h: u64 = 0xcbf29ce484222325;
    for b in seed.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    let a = h;
    let mut h2 = h ^ 0x9e3779b97f4a7c15;
    h2 = h2.wrapping_mul(0x2545f4914f6cdd1d);
    format!(
        "{:08x}-{:04x}-4{:03x}-8{:03x}-{:012x}",
        (a >> 32) as u32,
        (a >> 16) as u16,
        (a as u16) & 0x0fff,
        (h2 >> 48) as u16 & 0x0fff,
        h2 & 0xffff_ffff_ffff
    )
}

pub struct Schematic {
    pub title: String,
    pub root_uuid: String,
    pub components: Vec<Component>,
}

/// Snap a coordinate to the 1.27 mm connection grid.
fn snap(v: f64) -> f64 {
    (v / G).round() * G
}

impl Schematic {
    pub fn new(title: &str) -> Self {
        Schematic {
            title: title.into(),
            root_uuid: uuid_from(title),
            components: Vec::new(),
        }
    }

    pub fn add(&mut self, c: Component) {
        self.components.push(c);
    }

    /// Emit the full `.kicad_sch` file text.
    pub fn render(&self) -> String {
        let mut s = String::new();
        writeln!(s, "(kicad_sch").unwrap();
        writeln!(s, "  (version 20230121)").unwrap();
        writeln!(s, "  (generator nks_sch_gen)").unwrap();
        writeln!(s, "  (uuid \"{}\")", self.root_uuid).unwrap();
        writeln!(s, "  (paper \"A3\")").unwrap();
        self.render_lib_symbols(&mut s);
        for c in &self.components {
            self.render_component(&mut s, c);
        }
        writeln!(s, "  (sheet_instances").unwrap();
        writeln!(s, "    (path \"/\" (page \"1\"))").unwrap();
        writeln!(s, "  )").unwrap();
        writeln!(s, ")").unwrap();
        s
    }

    /// Emit a standalone `.kicad_sym` library holding every symbol used,
    /// so the project can register nickname "nks" and clear the
    /// "library not included" ERC warning.
    pub fn render_sym_lib(&self) -> String {
        let mut s = String::new();
        writeln!(s, "(kicad_symbol_lib").unwrap();
        writeln!(s, "  (version 20211014)").unwrap();
        writeln!(s, "  (generator nks_sch_gen)").unwrap();
        for c in &self.components {
            // library symbol names carry NO "nks:" prefix
            self.render_symbol_def_named(&mut s, c, &c.reference);
        }
        writeln!(s, ")").unwrap();
        s
    }

    /// The project-local `sym-lib-table` registering the library above.
    pub fn render_sym_lib_table(&self) -> String {
        let mut s = String::new();
        writeln!(s, "(sym_lib_table").unwrap();
        writeln!(s, "  (version 7)").unwrap();
        writeln!(
            s,
            "  (lib (name \"nks\")(type \"KiCad\")(uri \"${{KIPRJMOD}}/nks.kicad_sym\")(options \"\")(descr \"NkS generated symbols\"))"
        )
        .unwrap();
        writeln!(s, ")").unwrap();
        s
    }

    fn render_lib_symbols(&self, s: &mut String) {
        writeln!(s, "  (lib_symbols").unwrap();
        for c in &self.components {
            self.render_symbol_def(s, c);
        }
        writeln!(s, "  )").unwrap();
    }

    fn render_symbol_def(&self, s: &mut String, c: &Component) {
        let name = format!("nks:{}", c.reference);
        self.render_symbol_def_named(s, c, &name);
    }

    fn render_symbol_def_named(&self, s: &mut String, c: &Component, sym_name: &str) {
        let w = c.body_w();
        let h = c.body_h();
        let hw = w / 2.0;
        let hh = h / 2.0;
        writeln!(s, "    (symbol \"{sym_name}\"").unwrap();
        writeln!(s, "      (pin_names (offset 1.016))").unwrap();
        writeln!(s, "      (in_bom yes) (on_board yes)").unwrap();
        writeln!(
            s,
            "      (property \"Reference\" \"{}\" (at 0 {:.3} 0) (effects (font (size 1.27 1.27))))",
            c.reference,
            hh + 2.54
        )
        .unwrap();
        writeln!(
            s,
            "      (property \"Value\" \"{}\" (at 0 {:.3} 0) (effects (font (size 1.27 1.27))))",
            c.value,
            -(hh + 2.54)
        )
        .unwrap();
        writeln!(
            s,
            "      (property \"Footprint\" \"{}\" (at 0 0 0) (effects (font (size 1.27 1.27)) hide))",
            c.footprint
        )
        .unwrap();
        // body rectangle — nested sub-symbol name must NOT carry the lib prefix
        writeln!(s, "      (symbol \"{}_0_1\"", c.reference).unwrap();
        writeln!(
            s,
            "        (rectangle (start {:.3} {:.3}) (end {:.3} {:.3}) (stroke (width 0.254) (type default)) (fill (type background)))",
            -hw, hh, hw, -hh
        )
        .unwrap();
        writeln!(s, "      )").unwrap();
        // pins — nested sub-symbol name must NOT carry the lib prefix
        writeln!(s, "      (symbol \"{}_1_1\"", c.reference).unwrap();
        let pin_len = 3.81;
        let lefts = c.left_pins();
        let rights = c.right_pins();
        for (i, p) in lefts.iter().enumerate() {
            let py = hh - G - (i as f64) * 2.0 * G;
            let px = -hw - pin_len;
            writeln!(
                s,
                "        (pin {} line (at {:.3} {:.3} 0) (length {pin_len})",
                p.elec.token(),
                px,
                py
            )
            .unwrap();
            writeln!(
                s,
                "          (name \"{}\" (effects (font (size 1.016 1.016)))) (number \"{}\" (effects (font (size 1.016 1.016)))))",
                p.name, p.number
            )
            .unwrap();
        }
        for (i, p) in rights.iter().enumerate() {
            let py = hh - G - (i as f64) * 2.0 * G;
            let px = hw + pin_len;
            writeln!(
                s,
                "        (pin {} line (at {:.3} {:.3} 180) (length {pin_len})",
                p.elec.token(),
                px,
                py
            )
            .unwrap();
            writeln!(
                s,
                "          (name \"{}\" (effects (font (size 1.016 1.016)))) (number \"{}\" (effects (font (size 1.016 1.016)))))",
                p.name, p.number
            )
            .unwrap();
        }
        writeln!(s, "      )").unwrap();
        writeln!(s, "    )").unwrap();
    }

    fn render_component(&self, s: &mut String, c: &Component) {
        let lib_id = format!("nks:{}", c.reference);
        let w = c.body_w();
        let h = c.body_h();
        let hw = w / 2.0;
        let hh = h / 2.0;
        // center of the symbol on the sheet, snapped to the connection grid
        let cx = snap(c.x + hw + 5.08);
        let cy = snap(c.y + hh + 5.08);
        writeln!(s, "  (symbol").unwrap();
        writeln!(s, "    (lib_id \"{lib_id}\")").unwrap();
        writeln!(s, "    (at {:.3} {:.3} 0) (unit 1)", cx, cy).unwrap();
        writeln!(s, "    (in_bom yes) (on_board yes)").unwrap();
        writeln!(s, "    (uuid \"{}\")", c.uuid).unwrap();
        writeln!(
            s,
            "    (property \"Reference\" \"{}\" (at {:.3} {:.3} 0) (effects (font (size 1.27 1.27))))",
            c.reference,
            cx,
            cy - hh - 2.54
        )
        .unwrap();
        writeln!(
            s,
            "    (property \"Value\" \"{}\" (at {:.3} {:.3} 0) (effects (font (size 1.27 1.27))))",
            c.value,
            cx,
            cy + hh + 2.54
        )
        .unwrap();
        writeln!(
            s,
            "    (property \"Footprint\" \"{}\" (at {cx:.3} {cy:.3} 0) (effects (font (size 1.27 1.27)) hide))",
            c.footprint
        )
        .unwrap();
        // instance pin uuids
        for p in &c.pins {
            writeln!(
                s,
                "    (pin \"{}\" (uuid \"{}\"))",
                p.number,
                uuid_from(&format!("{}-{}-pin{}", c.reference, c.uuid, p.number))
            )
            .unwrap();
        }
        writeln!(
            s,
            "    (instances (project \"nks\" (path \"/{}\" (reference \"{}\") (unit 1))))",
            self.root_uuid, c.reference
        )
        .unwrap();
        writeln!(s, "  )").unwrap();

        // Now the net labels: a short wire stub from each pin end outward,
        // with a label carrying the net name.
        let pin_len = 3.81;
        let lefts = c.left_pins();
        let rights = c.right_pins();
        for (i, p) in lefts.iter().enumerate() {
            let py = cy - (hh - G - (i as f64) * 2.0 * G);
            let pin_end_x = cx - hw - pin_len;
            self.render_wire(s, pin_end_x, py, pin_end_x - 2.54, py);
            self.render_label(s, &p.net, pin_end_x - 2.54, py, 180.0);
        }
        for (i, p) in rights.iter().enumerate() {
            let py = cy - (hh - G - (i as f64) * 2.0 * G);
            let pin_end_x = cx + hw + pin_len;
            self.render_wire(s, pin_end_x, py, pin_end_x + 2.54, py);
            self.render_label(s, &p.net, pin_end_x + 2.54, py, 0.0);
        }
    }

    fn render_wire(&self, s: &mut String, x1: f64, y1: f64, x2: f64, y2: f64) {
        writeln!(s, "  (wire (pts (xy {x1:.3} {y1:.3}) (xy {x2:.3} {y2:.3}))").unwrap();
        writeln!(
            s,
            "    (stroke (width 0) (type default)) (uuid \"{}\"))",
            uuid_from(&format!("wire-{x1}-{y1}-{x2}-{y2}"))
        )
        .unwrap();
    }

    fn render_label(&self, s: &mut String, net: &str, x: f64, y: f64, rot: f64) {
        writeln!(
            s,
            "  (label \"{net}\" (at {x:.3} {y:.3} {rot}) (effects (font (size 1.27 1.27)) (justify {})) (uuid \"{}\"))",
            if rot == 0.0 { "left bottom" } else { "right bottom" },
            uuid_from(&format!("lbl-{net}-{x}-{y}"))
        )
        .unwrap();
    }
}

// ---------------------------------------------------------------------------
// Custom footprint for the ESP32-S3-DevKitC-1, generated so its pad numbers
// match U1's schematic pins (GPIO-numbered). Physical positions come from the
// official Espressif J1/J3 header tables (v1.1). Two 1x22 rows @ 2.54mm pitch,
// rows 22.86mm (0.9") apart. Multiple GND positions share pad "2"; both 3V3
// positions share pad "1"; unused positions are mechanical pads with no number.
pub fn devkitc1_footprint() -> String {
    // (physical J1 position 1..22, pad number as used by the symbol or "" if unused)
    // J1 row
    let j1: [(&str,&str); 22] = [
        ("1","1"),("2","1"),("3",""),("4",""),("5",""),("6",""),("7",""),
        ("8","15"),("9","16"),("10","17"),("11","18"),("12","8"),("13",""),
        ("14",""),("15","9"),("16",""),("17","11"),("18","12"),("19","13"),
        ("20","14"),("21",""),("22","2"),
    ];
    // J3 row
    let j3: [(&str,&str); 22] = [
        ("1","2"),("2",""),("3",""),("4",""),("5",""),("6",""),("7",""),
        ("8",""),("9",""),("10",""),("11",""),("12",""),("13",""),("14",""),
        ("15",""),("16",""),("17",""),("18","21"),("19",""),("20",""),
        ("21","2"),("22","2"),
    ];
    let pitch = 2.54;
    let row_gap = 22.86; // distance between the two header rows
    let n = 22.0;
    let span = (n - 1.0) * pitch;
    let y0 = -span / 2.0;
    let x_left = -row_gap / 2.0;
    let x_right = row_gap / 2.0;

    let mut s = String::new();
    s.push_str("(footprint \"ESP32-S3-DevKitC-1\" (version 20221018) (generator nks_sch_gen)\n");
    s.push_str("  (layer \"F.Cu\")\n");
    s.push_str("  (attr through_hole)\n");
    s.push_str("  (fp_text reference \"U1\" (at 0 -18 0) (layer \"F.SilkS\") (effects (font (size 1 1) (thickness 0.15))))\n");
    s.push_str("  (fp_text value \"ESP32-S3-DevKitC-1\" (at 0 18 0) (layer \"F.Fab\") (effects (font (size 1 1) (thickness 0.15))))\n");

    let mut pad = |num: &str, x: f64, y: f64, first: bool| -> String {
        let shape = if first { "rect" } else { "oval" };
        if num.is_empty() {
            // mechanical pad, no net (numberless -> use "" pad name = unconnected)
            format!(
                "  (pad \"\" thru_hole {shape} (at {x:.3} {y:.3}) (size 1.7 1.7) (drill 1.0) (layers \"*.Cu\" \"*.Mask\"))\n"
            )
        } else {
            format!(
                "  (pad \"{num}\" thru_hole {shape} (at {x:.3} {y:.3}) (size 1.7 1.7) (drill 1.0) (layers \"*.Cu\" \"*.Mask\"))\n"
            )
        }
    };

    for (i, (_pos, num)) in j1.iter().enumerate() {
        let y = y0 + (i as f64) * pitch;
        s.push_str(&pad(num, x_left, y, i == 0));
    }
    for (i, (_pos, num)) in j3.iter().enumerate() {
        let y = y0 + (i as f64) * pitch;
        s.push_str(&pad(num, x_right, y, false));
    }

    // simple courtyard/silk rectangle around the module
    let hw = row_gap / 2.0 + 2.0;
    let hh = span / 2.0 + 3.0;
    s.push_str(&format!(
        "  (fp_rect (start {:.2} {:.2}) (end {:.2} {:.2}) (layer \"F.CrtYd\") (stroke (width 0.05) (type solid)))\n",
        -hw, -hh, hw, hh
    ));
    s.push_str(")\n");
    s
}
