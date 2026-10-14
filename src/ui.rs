use crate::{
    converters::{self, CurrencyRates, CATEGORIES},
    engine::{self, fmt, Angle, Calculator, History, Programmer},
    graphs::Graph,
};
use chrono::{Datelike, Local};
use eframe::egui::{self, Color32, FontId, RichText, Vec2};
use serde::{Deserialize, Serialize};
use std::sync::mpsc;
const VERSION: &str = "2.0.0";
const RELEASE: &str = "14/10/2026";
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
enum Theme {
    #[default]
    System,
    Light,
    Dark,
}
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
enum Mode {
    #[default]
    Standard,
    Scientific,
    Programmer,
    Date,
    Graph,
    Converter(usize),
    Settings,
}
impl Mode {
    fn name(self) -> &'static str {
        match self {
            Self::Standard => "Standard",
            Self::Scientific => "Scientifica",
            Self::Programmer => "Programmatore",
            Self::Date => "Calcolo della data",
            Self::Graph => "Grafica",
            Self::Converter(i) => CATEGORIES[i],
            Self::Settings => "Impostazioni",
        }
    }
}
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
struct Saved {
    theme: Theme,
    mode: Mode,
    history: Vec<History>,
    memory: Vec<String>,
    programmer_memory: Vec<u64>,
    rates: Option<CurrencyRates>,
}
#[derive(Clone, Copy)]
struct Palette {
    bg: Color32,
    number: Color32,
    operator: Color32,
    text: Color32,
    muted: Color32,
    accent: Color32,
    accent_text: Color32,
    border: Color32,
}
impl Palette {
    fn new(dark: bool) -> Self {
        let c = Color32::from_rgb;
        if dark {
            Self {
                bg: c(32, 32, 32),
                number: c(59, 59, 59),
                operator: c(50, 50, 50),
                text: c(249, 249, 249),
                muted: c(175, 175, 175),
                accent: c(118, 185, 237),
                accent_text: c(20, 32, 42),
                border: c(65, 65, 65),
            }
        } else {
            Self {
                bg: c(243, 243, 243),
                number: Color32::WHITE,
                operator: c(249, 249, 249),
                text: c(26, 26, 26),
                muted: c(100, 100, 100),
                accent: c(0, 103, 192),
                accent_text: Color32::WHITE,
                border: c(224, 224, 224),
            }
        }
    }
}
struct Preview {
    path: String,
    frames: u32,
    requested: bool,
}
pub struct App {
    calc: Calculator,
    programmer: Programmer,
    mode: Mode,
    theme: Theme,
    nav: bool,
    applied_theme: Option<Theme>,
    history_open: bool,
    memory_tab: bool,
    pinned: bool,
    shift: bool,
    hyp: bool,
    from_date: [i32; 3],
    to_date: [i32; 3],
    date_add: bool,
    subtract: bool,
    offsets: [i32; 3],
    unit_from: usize,
    unit_to: usize,
    convert_input: String,
    rates: Option<CurrencyRates>,
    currency_from: String,
    currency_to: String,
    currency_error: Option<String>,
    receiver: Option<mpsc::Receiver<Result<CurrencyRates, String>>>,
    graph: Graph,
    notice: String,
    preview: Option<Preview>,
    rng: u64,
}
impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let saved: Saved = cc
            .storage
            .and_then(|s| eframe::get_value(s, "calculator-v2"))
            .unwrap_or_default();
        let mut fonts = egui::FontDefinitions::default();
        if let Ok(bytes) = std::fs::read("C:\\Windows\\Fonts\\segoeui.ttf") {
            fonts
                .font_data
                .insert("Segoe UI".into(), egui::FontData::from_owned(bytes).into());
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "Segoe UI".into());
        }
        if let Ok(bytes) = std::fs::read("C:\\Windows\\Fonts\\seguisym.ttf") {
            fonts.font_data.insert(
                "Segoe Symbols".into(),
                egui::FontData::from_owned(bytes).into(),
            );
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(1, "Segoe Symbols".into());
        }
        cc.egui_ctx.set_fonts(fonts);
        let mut calc = Calculator::new();
        calc.history = saved.history;
        calc.memory = saved.memory;
        let mut programmer = Programmer::new();
        programmer.memory = saved.programmer_memory;
        let today = Local::now().date_naive();
        let dt = [today.year(), today.month() as i32, today.day() as i32];
        let mut app = Self {
            calc,
            programmer,
            mode: saved.mode,
            theme: saved.theme,
            nav: false,
            applied_theme: None,
            history_open: false,
            memory_tab: false,
            pinned: false,
            shift: false,
            hyp: false,
            from_date: dt,
            to_date: dt,
            date_add: false,
            subtract: false,
            offsets: [0; 3],
            unit_from: 0,
            unit_to: 1,
            convert_input: "1".into(),
            rates: saved.rates,
            currency_from: "EUR".into(),
            currency_to: "USD".into(),
            currency_error: None,
            receiver: None,
            graph: Graph::default(),
            notice: String::new(),
            preview: None,
            rng: today.num_days_from_ce() as u64 + 1,
        };
        let args: Vec<String> = std::env::args().collect();
        if let Some(i) = args.iter().position(|s| s == "--preview") {
            if let Some(path) = args.get(i + 1) {
                app.theme = if args.iter().any(|s| s == "--dark") {
                    Theme::Dark
                } else {
                    Theme::Light
                };
                app.mode = if args.iter().any(|s| s == "--scientific") {
                    Mode::Scientific
                } else if args.iter().any(|s| s == "--programmer") {
                    Mode::Programmer
                } else if args.iter().any(|s| s == "--graph") {
                    Mode::Graph
                } else if args.iter().any(|s| s == "--date") {
                    Mode::Date
                } else if args.iter().any(|s| s == "--converter") {
                    Mode::Converter(2)
                } else {
                    Mode::Standard
                };
                app.calc.set_value(12345.6789);
                app.programmer.set(0xA7);
                app.history_open = matches!(app.mode, Mode::Standard | Mode::Scientific);
                if args.iter().any(|s| s == "--compact") {
                    app.history_open = false;
                }
                app.preview = Some(Preview {
                    path: path.clone(),
                    frames: 0,
                    requested: false,
                });
            }
        }
        if matches!(app.mode,Mode::Converter(i)if i>=CATEGORIES.len()) {
            app.mode = Mode::Standard
        }
        app
    }
    fn dark(&self, ctx: &egui::Context) -> bool {
        match self.theme {
            Theme::Light => false,
            Theme::Dark => true,
            Theme::System => ctx.system_theme() == Some(egui::Theme::Dark),
        }
    }
    fn style(&mut self, ctx: &egui::Context) -> Palette {
        let dark = self.dark(ctx);
        let p = Palette::new(dark);
        let mut v = if dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        v.panel_fill = p.bg;
        v.window_fill = p.bg;
        v.override_text_color = Some(p.text);
        v.selection.bg_fill = p.accent;
        v.selection.stroke = egui::Stroke::new(1.0_f32, p.accent_text);
        v.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0_f32, p.border);
        v.widgets.inactive.bg_fill = p.operator;
        v.widgets.hovered.bg_fill = if dark {
            Color32::from_gray(75)
        } else {
            Color32::from_gray(230)
        };
        ctx.set_visuals(v);
        if self.applied_theme != Some(self.theme) {
            ctx.send_viewport_cmd(egui::ViewportCommand::SetTheme(match self.theme {
                Theme::System => egui::SystemTheme::SystemDefault,
                Theme::Dark => egui::SystemTheme::Dark,
                Theme::Light => egui::SystemTheme::Light,
            }));
            self.applied_theme = Some(self.theme);
        }
        ctx.style_mut(|s| {
            s.spacing.item_spacing = Vec2::splat(5.0);
            s.spacing.button_padding = Vec2::new(10.0, 7.0);
            s.text_styles
                .insert(egui::TextStyle::Body, FontId::proportional(16.0));
            s.text_styles
                .insert(egui::TextStyle::Button, FontId::proportional(16.0));
        });
        p
    }
    fn switch(&mut self, m: Mode) {
        self.mode = m;
        self.nav = false;
        self.notice.clear();
        self.calc.clear();
        self.calc.scientific = m == Mode::Scientific;
        if let Mode::Converter(_) = m {
            self.unit_from = 0;
            self.unit_to = 1;
            self.convert_input = "1".into();
            if m == Mode::Converter(0) && self.rates.is_none() {
                self.fetch_rates()
            }
        }
    }
    fn fetch_rates(&mut self) {
        if self.receiver.is_some() {
            return;
        }
        self.currency_error = None;
        let (tx, rx) = mpsc::channel();
        self.receiver = Some(rx);
        std::thread::spawn(move || {
            let _ = tx.send(CurrencyRates::download());
        });
    }
    fn currency_poll(&mut self, ctx: &egui::Context) {
        if let Some(rx) = &self.receiver {
            match rx.try_recv() {
                Ok(r) => {
                    match r {
                        Ok(r) => {
                            self.rates = Some(r);
                            self.currency_error = None
                        }
                        Err(e) => self.currency_error = Some(e),
                    }
                    self.receiver = None
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.currency_error = Some("Aggiornamento interrotto".into());
                    self.receiver = None
                }
                _ => ctx.request_repaint_after(std::time::Duration::from_millis(200)),
            }
        }
    }
    fn keyboard(&mut self, ctx: &egui::Context) {
        if ctx.wants_keyboard_input() {
            return;
        }
        for e in ctx.input(|i| i.events.clone()) {
            match e {
                egui::Event::Copy => ctx.copy_text(if self.mode == Mode::Programmer {
                    self.programmer.text(self.programmer.radix)
                } else {
                    self.calc.display.clone()
                }),
                egui::Event::Paste(s) => {
                    if self.mode == Mode::Programmer {
                        self.programmer.input = s.trim().into();
                        self.programmer.equals();
                        continue;
                    }
                    if matches!(self.mode, Mode::Converter(_)) {
                        self.convert_input = s.trim().replace(',', ".");
                        continue;
                    }
                    if matches!(self.mode, Mode::Standard | Mode::Scientific) {
                        let s = s.trim().replace(',', ".");
                        if crate::precise::evaluate(&s, self.calc.angle).is_ok() {
                            self.calc.input = s;
                            self.calc.fresh = false;
                            self.calc.preview()
                        } else {
                            self.notice = "Testo incollato non valido".into()
                        }
                    }
                }
                egui::Event::Text(t) => {
                    for c in t.chars() {
                        if self.mode == Mode::Programmer {
                            match c {
                                '0'..='9' | 'a'..='f' | 'A'..='F' => self.programmer.digit(c),
                                '+' | '-' | '*' | '/' => self.programmer.operator(&c.to_string()),
                                '=' => self.programmer.equals(),
                                _ => {}
                            }
                        } else if matches!(self.mode, Mode::Standard | Mode::Scientific) {
                            match c {
                                '0'..='9' => self.calc.digit(&c.to_string()),
                                '.' | ',' => self.calc.dot(),
                                '+' | '-' | '*' | '/' | '^' => self.calc.operator(&c.to_string()),
                                '=' => self.calc.equals(),
                                '(' | ')' => self.calc.digit(&c.to_string()),
                                '%' => self.calc.percent(),
                                _ => {}
                            }
                        }
                    }
                }
                egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } => {
                    if modifiers.alt {
                        let m = match key {
                            egui::Key::Num1 => Some(Mode::Standard),
                            egui::Key::Num2 => Some(Mode::Scientific),
                            egui::Key::Num3 => Some(Mode::Graph),
                            egui::Key::Num4 => Some(Mode::Programmer),
                            egui::Key::Num5 => Some(Mode::Date),
                            _ => None,
                        };
                        if let Some(m) = m {
                            self.switch(m)
                        }
                        continue;
                    }
                    if modifiers.ctrl {
                        let action = match key {
                            egui::Key::L => Some("MC"),
                            egui::Key::M => Some("MS"),
                            egui::Key::R => Some("MR"),
                            egui::Key::P => Some("M+"),
                            egui::Key::Q => Some("M−"),
                            _ => None,
                        };
                        if let Some(action) = action {
                            if self.mode == Mode::Programmer {
                                self.programmer.memory_action(action)
                            } else {
                                self.calc.memory_action(action)
                            }
                        }
                        if key == egui::Key::D && modifiers.shift {
                            self.calc.history.clear()
                        }
                        continue;
                    }
                    if matches!(
                        self.mode,
                        Mode::Standard | Mode::Scientific | Mode::Programmer
                    ) && matches!(
                        key,
                        egui::Key::Enter
                            | egui::Key::Escape
                            | egui::Key::Delete
                            | egui::Key::Backspace
                            | egui::Key::F9
                    ) {
                        ctx.input_mut(|i| {
                            i.consume_key(modifiers, key);
                        });
                    }
                    if key == egui::Key::F9 {
                        if self.mode == Mode::Programmer {
                            self.programmer.set(self.programmer.result.wrapping_neg())
                        } else if matches!(self.mode, Mode::Standard | Mode::Scientific) {
                            self.calc.unary("neg")
                        }
                        continue;
                    }
                    if self.mode == Mode::Programmer {
                        match key {
                            egui::Key::Enter => self.programmer.equals(),
                            egui::Key::Escape => self.programmer.set(0),
                            egui::Key::Delete => self.programmer.ce(),
                            egui::Key::Backspace => self.programmer.back(),
                            egui::Key::F5 => self.programmer.switch(16, self.programmer.bits),
                            egui::Key::F6 => self.programmer.switch(10, self.programmer.bits),
                            egui::Key::F7 => self.programmer.switch(8, self.programmer.bits),
                            egui::Key::F8 => self.programmer.switch(2, self.programmer.bits),
                            egui::Key::F12 => self.programmer.switch(self.programmer.radix, 8),
                            _ => {}
                        }
                    } else if matches!(self.mode, Mode::Standard | Mode::Scientific) {
                        match key {
                            egui::Key::Enter => self.calc.equals(),
                            egui::Key::Escape => self.calc.clear(),
                            egui::Key::Delete => self.calc.ce(),
                            egui::Key::Backspace => self.calc.back(),
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }
    fn header(&mut self, ctx: &egui::Context, p: Palette) {
        egui::TopBottomPanel::top("header")
            .frame(
                egui::Frame::NONE
                    .fill(p.bg)
                    .inner_margin(egui::Margin::symmetric(12, 10)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .add(egui::Button::new(RichText::new("☰").size(23.0)).frame(false))
                        .on_hover_text("Navigazione")
                        .clicked()
                    {
                        self.nav = !self.nav
                    }
                    ui.label(RichText::new(self.mode.name()).size(23.0).strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .button(if self.dark(ctx) { "☀" } else { "☾" })
                            .on_hover_text("Tema chiaro / scuro")
                            .clicked()
                        {
                            self.theme = if self.dark(ctx) {
                                Theme::Light
                            } else {
                                Theme::Dark
                            }
                        }
                        if matches!(
                            self.mode,
                            Mode::Standard | Mode::Scientific | Mode::Programmer
                        ) {
                            if ui
                                .selectable_label(self.pinned, "▣")
                                .on_hover_text("Sempre in primo piano")
                                .clicked()
                            {
                                self.pinned = !self.pinned;
                                ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                                    if self.pinned {
                                        egui::WindowLevel::AlwaysOnTop
                                    } else {
                                        egui::WindowLevel::Normal
                                    },
                                ));
                            }
                            if ui
                                .selectable_label(self.history_open, "◷")
                                .on_hover_text("Cronologia e memoria")
                                .clicked()
                            {
                                self.history_open = !self.history_open
                            }
                        }
                    });
                });
            });
    }
    fn navigation(&mut self, ctx: &egui::Context, p: Palette) {
        if !self.nav {
            return;
        }
        egui::SidePanel::left("nav")
            .resizable(false)
            .exact_width(222.0)
            .frame(egui::Frame::NONE.fill(p.bg).inner_margin(12))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.label(RichText::new("CALCOLATRICE").size(11.0).color(p.muted));
                    for m in [
                        Mode::Standard,
                        Mode::Scientific,
                        Mode::Graph,
                        Mode::Programmer,
                        Mode::Date,
                    ] {
                        if ui
                            .add_sized(
                                [198.0, 35.0],
                                egui::Button::new(m.name()).selected(self.mode == m),
                            )
                            .clicked()
                        {
                            self.switch(m)
                        }
                    }
                    ui.add_space(14.0);
                    ui.label(RichText::new("CONVERTITORE").size(11.0).color(p.muted));
                    for (i, n) in CATEGORIES.iter().enumerate() {
                        if ui
                            .add_sized(
                                [198.0, 32.0],
                                egui::Button::new(*n).selected(self.mode == Mode::Converter(i)),
                            )
                            .clicked()
                        {
                            self.switch(Mode::Converter(i))
                        }
                    }
                    ui.separator();
                    if ui.button("⚙  Impostazioni").clicked() {
                        self.switch(Mode::Settings)
                    }
                });
            });
    }
    fn side_history(&mut self, ctx: &egui::Context, p: Palette) {
        if !self.history_open
            || !matches!(
                self.mode,
                Mode::Standard | Mode::Scientific | Mode::Programmer
            )
        {
            return;
        }
        if ctx.screen_rect().width() > 780.0 {
            egui::SidePanel::right("history")
                .resizable(true)
                .default_width(260.0)
                .min_width(220.0)
                .frame(egui::Frame::NONE.fill(p.bg).inner_margin(15))
                .show(ctx, |ui| self.history_panel(ui, p));
        } else {
            let mut open = true;
            egui::Window::new("Cronologia e memoria")
                .open(&mut open)
                .default_width(300.0)
                .default_height(330.0)
                .show(ctx, |ui| self.history_panel(ui, p));
            if !open {
                self.history_open = false
            }
        }
    }
    fn history_panel(&mut self, ui: &mut egui::Ui, p: Palette) {
        if self.mode == Mode::Programmer {
            ui.heading("Memoria intera");
            let values = self.programmer.memory.clone();
            if values.is_empty() {
                ui.label("La memoria è vuota");
            }
            for (i, value) in values.iter().enumerate() {
                ui.push_id(i, |ui| {
                    if ui
                        .button(RichText::new(format!("0x{value:X}")).size(20.0))
                        .clicked()
                    {
                        self.programmer.set(*value)
                    }
                });
            }
            if !values.is_empty() && ui.button("Svuota memoria").clicked() {
                self.programmer.memory.clear()
            }
            return;
        }
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.memory_tab, false, "Cronologia");
            ui.selectable_value(&mut self.memory_tab, true, "Memoria");
        });
        ui.add_space(15.0);
        if self.memory_tab {
            if self.calc.memory.is_empty() {
                ui.label(RichText::new("La memoria è vuota").color(p.muted));
            }
            let items = self.calc.memory.clone();
            let mut remove = None;
            egui::ScrollArea::vertical()
                .id_salt("memory-list")
                .show(ui, |ui| {
                    for (i, x) in items.iter().enumerate() {
                        ui.push_id(i, |ui| {
                            if ui
                                .button(
                                    RichText::new(
                                        crate::precise::evaluate(x, self.calc.angle)
                                            .unwrap_or_else(|_| x.clone()),
                                    )
                                    .size(23.0),
                                )
                                .clicked()
                            {
                                self.calc.set_text(x)
                            }
                            ui.horizontal(|ui| {
                                if ui.small_button("MC").clicked() {
                                    remove = Some(i)
                                }
                                for (k, op) in [("M+", "+"), ("M−", "-")] {
                                    if ui.small_button(k).clicked() {
                                        if let Ok(next) = crate::precise::internal(
                                            &format!(
                                                "({}){op}({})",
                                                self.calc.memory[i], self.calc.input
                                            ),
                                            self.calc.angle,
                                        ) {
                                            self.calc.memory[i] = next
                                        }
                                    }
                                }
                            });
                            ui.separator();
                        });
                    }
                });
            if let Some(i) = remove {
                self.calc.memory.remove(i);
            }
            if !items.is_empty() && ui.button("Svuota memoria").clicked() {
                self.calc.memory.clear()
            }
        } else {
            if self.calc.history.is_empty() {
                ui.label(RichText::new("Non c'è ancora una cronologia").color(p.muted));
            }
            let entries = self.calc.history.clone();
            egui::ScrollArea::vertical()
                .id_salt("history-list")
                .max_height((ui.available_height() - 50.0).max(50.0))
                .show(ui, |ui| {
                    for e in entries.iter().rev() {
                        ui.label(
                            RichText::new(format!("{} =", e.expression))
                                .size(13.0)
                                .color(p.muted),
                        );
                        if ui
                            .add_sized(
                                [ui.available_width(), 38.0],
                                egui::Button::new(RichText::new(&e.result).size(25.0)).frame(false),
                            )
                            .clicked()
                        {
                            self.calc.set_text(&e.expression)
                        }
                        ui.add_space(12.0);
                    }
                });
            if !entries.is_empty() && ui.button("Cancella cronologia").clicked() {
                self.calc.history.clear()
            }
        }
    }
    fn display(&mut self, ui: &mut egui::Ui, p: Palette) {
        ui.add_space(6.0);
        let width = ui.available_width();
        let expr = self.calc.input.clone();
        ui.allocate_ui_with_layout(
            Vec2::new(width, 22.0),
            egui::Layout::right_to_left(egui::Align::Center),
            |ui| {
                ui.label(RichText::new(expr).color(p.muted).size(15.0));
            },
        );
        let text = self
            .calc
            .error
            .clone()
            .unwrap_or_else(|| self.calc.display.clone());
        let size = if self.calc.error.is_some() {
            22.0
        } else {
            (width / (text.chars().count().max(7) as f32 * 0.62)).clamp(25.0, 48.0)
        };
        ui.allocate_ui_with_layout(
            Vec2::new(width, size + 12.0),
            egui::Layout::right_to_left(egui::Align::Center),
            |ui| {
                ui.add(egui::Label::new(RichText::new(&text).size(size).strong()).selectable(true))
                    .context_menu(|ui| {
                        if ui.button("Copia risultato").clicked() {
                            ui.ctx().copy_text(text.clone());
                            ui.close_menu();
                        }
                    });
            },
        );
        egui::CollapsingHeader::new("Modifica espressione")
            .id_salt("expression-edit")
            .show(ui, |ui| {
                let r = ui.add(
                    egui::TextEdit::singleline(&mut self.calc.input)
                        .desired_width(f32::INFINITY)
                        .hint_text("2 + 3 * (4 - 1)"),
                );
                if r.changed() {
                    self.calc.fresh = false;
                    self.calc.error = None;
                    self.calc.preview()
                }
                if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.calc.equals()
                }
            });
    }
    fn memory_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let w = (ui.available_width() - 25.0) / 6.0;
            for k in ["MC", "MR", "M+", "M−", "MS", "M▾"] {
                let enabled = !self.calc.memory.is_empty() || !["MC", "MR", "M▾"].contains(&k);
                if ui
                    .add_enabled_ui(enabled, |ui| {
                        ui.add_sized(
                            [w, 30.0],
                            egui::Button::new(RichText::new(k).size(14.0)).frame(false),
                        )
                    })
                    .inner
                    .clicked()
                {
                    if k == "M▾" {
                        self.history_open = true;
                        self.memory_tab = true
                    } else {
                        self.calc.memory_action(k)
                    }
                }
            }
        });
    }
    fn key(ui: &mut egui::Ui, k: &str, w: f32, h: f32, p: Palette, kind: u8) -> bool {
        let fill = if kind == 2 {
            p.accent
        } else if kind == 1 {
            p.number
        } else {
            p.operator
        };
        let text = if kind == 2 { p.accent_text } else { p.text };
        ui.scope(|ui| {
            ui.spacing_mut().button_padding = Vec2::new(6.0, 3.0);
            ui.add_sized(
                [w, h],
                egui::Button::new(
                    RichText::new(k)
                        .size(if k.chars().count() > 5 {
                            15.0
                        } else if h < 44.0 {
                            16.0
                        } else {
                            21.0
                        })
                        .color(text),
                )
                .fill(fill)
                .stroke(egui::Stroke::new(0.6_f32, p.border))
                .corner_radius(4.0),
            )
            .clicked()
        })
        .inner
    }
    fn calc_key(&mut self, k: &str) {
        match k {
            "C" => self.calc.clear(),
            "CE" => self.calc.ce(),
            "⌫" => self.calc.back(),
            "=" => self.calc.equals(),
            "." => self.calc.dot(),
            "±" => self.calc.unary("neg"),
            "%" => self.calc.percent(),
            "+" | "−" | "×" | "÷" | "^" => self.calc.operator(match k {
                "−" => "-",
                "×" => "*",
                "÷" => "/",
                _ => k,
            }),
            "mod" => self.calc.operator(" mod "),
            "1/x" => self.calc.unary("inv"),
            "x²" => self.calc.unary("sq"),
            "x³" => self.calc.unary("cube"),
            "√x" => self.calc.unary("sqrt"),
            "∛x" => self.calc.unary("cbrt"),
            "|x|" => self.calc.unary("abs"),
            "n!" => self.calc.unary("fact"),
            "π" => self.calc.constant("pi"),
            "e" => self.calc.constant("e"),
            "xʸ" => self.calc.operator("^"),
            "10ˣ" => self.calc.unary("pow10"),
            "2ˣ" => self.calc.unary("pow2"),
            "eˣ" => self.calc.unary("exp"),
            "log" => self.calc.unary("log"),
            "ln" => self.calc.unary("ln"),
            "exp" => self.calc.enter_exponent(),
            "ʸ√x" | "logʸx" => {
                self.calc
                    .start_binary_function(if k == "ʸ√x" { "root" } else { "logy" });
            }
            "2nd" => self.shift = !self.shift,
            _ => self.calc.digit(k),
        }
    }
    fn calculator(&mut self, ui: &mut egui::Ui, p: Palette) {
        self.calc.scientific = self.mode == Mode::Scientific;
        self.display(ui, p);
        if self.mode == Mode::Scientific {
            ui.horizontal(|ui| {
                for (a, t) in [
                    (Angle::Deg, "DEG"),
                    (Angle::Rad, "RAD"),
                    (Angle::Grad, "GRAD"),
                ] {
                    if ui.selectable_label(self.calc.angle == a, t).clicked() {
                        self.calc.angle = a;
                        self.calc.preview()
                    }
                }
                if ui.selectable_label(self.calc.exponential, "F-E").clicked() {
                    self.calc.exponential = !self.calc.exponential;
                    self.calc.preview()
                }
                if ui.button("CE").on_hover_text("Cancella voce").clicked() {
                    self.calc.ce()
                }
            });
        }
        self.memory_bar(ui);
        if self.mode == Mode::Scientific {
            ui.horizontal(|ui| {
                ui.menu_button("Trigonometria ▾", |ui| {
                    ui.checkbox(&mut self.hyp, "Iperboliche (hyp)");
                    let names = if self.hyp {
                        [
                            "sinh", "cosh", "tanh", "sech", "csch", "coth", "asinh", "acosh",
                            "atanh", "asech", "acsch", "acoth",
                        ]
                    } else {
                        [
                            "sin", "cos", "tan", "sec", "csc", "cot", "asin", "acos", "atan",
                            "asec", "acsc", "acot",
                        ]
                    };
                    for f in names {
                        if ui.button(f).clicked() {
                            self.calc.unary(f);
                            ui.close_menu()
                        }
                    }
                });
                ui.menu_button("Funzioni ▾", |ui| {
                    for (f, t) in [
                        ("abs", "Valore assoluto"),
                        ("floor", "Arrotonda per difetto"),
                        ("ceil", "Arrotonda per eccesso"),
                        ("round", "Arrotonda"),
                        ("dms", "Decimali → DMS"),
                        ("degrees", "DMS → decimali"),
                    ] {
                        if ui.button(t).clicked() {
                            self.calc.unary(f);
                            ui.close_menu()
                        }
                    }
                    if ui.button("Numero casuale").clicked() {
                        self.rng ^= self.rng << 13;
                        self.rng ^= self.rng >> 7;
                        self.rng ^= self.rng << 17;
                        self.calc
                            .set_value((self.rng >> 11) as f64 / ((1u64 << 53) as f64));
                        ui.close_menu()
                    }
                });
            });
        }
        let standard = [
            "%", "CE", "C", "⌫", "1/x", "x²", "√x", "÷", "7", "8", "9", "×", "4", "5", "6", "−",
            "1", "2", "3", "+", "±", "0", ".", "=",
        ];
        let scientific = if self.shift {
            [
                "2nd", "π", "e", "C", "⌫", "x³", "1/x", "|x|", "exp", "mod", "∛x", "(", ")", "n!",
                "÷", "ʸ√x", "7", "8", "9", "×", "2ˣ", "4", "5", "6", "−", "logʸx", "1", "2", "3",
                "+", "eˣ", "±", "0", ".", "=",
            ]
        } else {
            [
                "2nd", "π", "e", "C", "⌫", "x²", "1/x", "|x|", "exp", "mod", "√x", "(", ")", "n!",
                "÷", "xʸ", "7", "8", "9", "×", "10ˣ", "4", "5", "6", "−", "log", "1", "2", "3",
                "+", "ln", "±", "0", ".", "=",
            ]
        };
        let (keys, cols) = if self.mode == Mode::Scientific {
            (scientific.as_slice(), 5)
        } else {
            (standard.as_slice(), 4)
        };
        let w = (ui.available_width() - (cols - 1) as f32 * 5.0) / cols as f32;
        let rows = keys.len() / cols;
        let h = ((ui.clip_rect().bottom() - ui.cursor().top() - rows as f32 * 5.0 - 8.0)
            / rows as f32)
            .clamp(35.0, 95.0);
        for row in keys.chunks(cols) {
            ui.horizontal(|ui| {
                for k in row {
                    let kind = if *k == "=" {
                        2
                    } else if k.chars().all(|c| c.is_ascii_digit()) || *k == "." {
                        1
                    } else {
                        0
                    };
                    if Self::key(ui, k, w, h, p, kind) {
                        self.calc_key(k)
                    }
                }
            });
        }
    }
    fn programmer(&mut self, ui: &mut egui::Ui, p: Palette) {
        ui.horizontal(|ui| {
            for key in ["MC", "MR", "M+", "M−", "MS"] {
                if ui.button(key).clicked() {
                    self.programmer.memory_action(key)
                }
            }
            if ui.button("C").clicked() {
                self.programmer.set(0)
            }
        });
        if let Some(e) = &self.programmer.error {
            ui.colored_label(Color32::from_rgb(224, 90, 70), e);
        }
        ui.label(RichText::new(&self.programmer.input).color(p.muted));
        ui.allocate_ui_with_layout(
            Vec2::new(ui.available_width(), 48.0),
            egui::Layout::right_to_left(egui::Align::Center),
            |ui| {
                ui.label(
                    RichText::new(self.programmer.text(self.programmer.radix))
                        .size(36.0)
                        .strong(),
                );
            },
        );
        for (radix, label) in [(16, "HEX"), (10, "DEC"), (8, "OCT"), (2, "BIN")] {
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(self.programmer.radix == radix, label)
                    .clicked()
                {
                    self.programmer.switch(radix, self.programmer.bits)
                }
                ui.label(
                    RichText::new(self.programmer.text(radix))
                        .size(if radix == 2 { 13.0 } else { 16.0 })
                        .monospace(),
                );
            });
        }
        ui.horizontal(|ui| {
            for (bits, label) in [(64, "QWORD"), (32, "DWORD"), (16, "WORD"), (8, "BYTE")] {
                if ui
                    .selectable_label(self.programmer.bits == bits, label)
                    .clicked()
                {
                    self.programmer.switch(self.programmer.radix, bits)
                }
            }
        });
        ui.horizontal(|ui| {
            ui.menu_button("Bit per bit ▾", |ui| {
                for op in ["AND", "OR", "XOR", "NAND", "NOR"] {
                    if ui.button(op).clicked() {
                        self.programmer.operator(&op.to_lowercase());
                        ui.close_menu()
                    }
                }
                if ui.button("NOT").clicked() {
                    self.programmer.shift("NOT");
                    ui.close_menu()
                }
            });
            ui.menu_button("Spostamento ▾", |ui| {
                for (label, op) in [
                    ("LSH di n bit", "lsh"),
                    ("ASR di n bit", "rsh"),
                    ("ROL di n bit", "rol"),
                    ("ROR di n bit", "ror"),
                ] {
                    if ui.button(label).clicked() {
                        self.programmer.operator(op);
                        ui.close_menu()
                    }
                }
                ui.separator();
                for op in ["LSH", "LSR", "ASR", "ROL", "ROR", "ROLC", "RORC"] {
                    if ui.button(op).clicked() {
                        self.programmer.shift(op);
                        ui.close_menu()
                    }
                }
            });
            ui.label(format!("Carry: {}", self.programmer.carry as u8));
        });
        egui::CollapsingHeader::new("Modifica singoli bit").show(ui, |ui| {
            for row in (0..self.programmer.bits)
                .rev()
                .collect::<Vec<_>>()
                .chunks(8)
            {
                ui.horizontal(|ui| {
                    let w = (ui.available_width() - 7.0 * 5.0) / 8.0;
                    for bit in row {
                        let on = self.programmer.result & (1u64 << bit) != 0;
                        if ui
                            .add_sized(
                                [w, 27.0],
                                egui::Button::new(if on { "1" } else { "0" }).selected(on),
                            )
                            .on_hover_text(format!("Bit {bit}"))
                            .clicked()
                        {
                            self.programmer.flip(*bit)
                        }
                    }
                });
            }
        });
        let keys = [
            "A", "B", "C", "D", "E", "F", "(", ")", "CE", "⌫", "%", "÷", "7", "8", "9", "×", "4",
            "5", "6", "−", "1", "2", "3", "+", "±", "0", "NOT", "=",
        ];
        let w = (ui.available_width() - 15.0) / 4.0;
        let h = ((ui.clip_rect().bottom() - ui.cursor().top() - 43.0) / 7.0).clamp(30.0, 65.0);
        for row in keys.chunks(4) {
            ui.horizontal(|ui| {
                for k in row {
                    let digit = k.len() == 1 && k.chars().next().unwrap().is_ascii_hexdigit();
                    let enabled = !digit
                        || k.chars()
                            .next()
                            .unwrap()
                            .to_digit(self.programmer.radix)
                            .is_some();
                    if ui
                        .add_enabled_ui(enabled, |ui| {
                            Self::key(
                                ui,
                                k,
                                w,
                                h,
                                p,
                                if *k == "=" {
                                    2
                                } else if digit {
                                    1
                                } else {
                                    0
                                },
                            )
                        })
                        .inner
                    {
                        match *k {
                            "CE" => self.programmer.ce(),
                            "⌫" => self.programmer.back(),
                            "=" => self.programmer.equals(),
                            "NOT" => self.programmer.shift("NOT"),
                            "±" => self.programmer.set(self.programmer.result.wrapping_neg()),
                            "+" | "−" | "×" | "÷" | "%" => self.programmer.operator(match *k {
                                "−" => "-",
                                "×" => "*",
                                "÷" => "/",
                                x => x,
                            }),
                            "(" | ")" => {
                                if self.programmer.fresh {
                                    self.programmer.input.clear();
                                    self.programmer.fresh = false
                                }
                                self.programmer.input.push_str(k)
                            }
                            _ => self.programmer.digit(k.chars().next().unwrap()),
                        }
                    }
                }
            });
        }
    }
    fn date_control(ui: &mut egui::Ui, id: &str, v: &mut [i32; 3]) {
        ui.push_id(id, |ui| {
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut v[2]).range(1..=31).prefix("G "));
                ui.add(egui::DragValue::new(&mut v[1]).range(1..=12).prefix("M "));
                ui.add(egui::DragValue::new(&mut v[0]).range(1..=9999).prefix("A "));
            });
        });
    }
    fn date_view(&mut self, ui: &mut egui::Ui, p: Palette) {
        ui.add_space(16.0);
        egui::ComboBox::from_id_salt("date-type")
            .selected_text(if self.date_add {
                "Aggiungi o sottrai giorni"
            } else {
                "Differenza tra date"
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.date_add, false, "Differenza tra date");
                ui.selectable_value(&mut self.date_add, true, "Aggiungi o sottrai giorni");
            });
        ui.add_space(22.0);
        ui.label("Da");
        Self::date_control(ui, "from", &mut self.from_date);
        let from = converters::date(
            self.from_date[0],
            self.from_date[1] as u32,
            self.from_date[2] as u32,
        );
        ui.add_space(22.0);
        let result = if self.date_add {
            ui.horizontal(|ui| {
                ui.radio_value(&mut self.subtract, false, "Aggiungi");
                ui.radio_value(&mut self.subtract, true, "Sottrai");
            });
            for (i, n) in ["Anni", "Mesi", "Giorni"].iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut self.offsets[i]).range(0..=9999));
                    ui.label(*n);
                });
            }
            from.and_then(|d| {
                let sign = if self.subtract { -1 } else { 1 };
                converters::add_date(
                    d,
                    sign * self.offsets[0],
                    sign * self.offsets[1],
                    sign as i64 * self.offsets[2] as i64,
                )
                .map(|r| r.format("%d/%m/%Y").to_string())
            })
        } else {
            ui.label("A");
            Self::date_control(ui, "to", &mut self.to_date);
            let to = converters::date(
                self.to_date[0],
                self.to_date[1] as u32,
                self.to_date[2] as u32,
            );
            from.and_then(|a| {
                to.map(|b| {
                    format!(
                        "{}\n{} giorni",
                        converters::difference(a, b),
                        (b - a).num_days().abs()
                    )
                })
            })
        };
        ui.add_space(28.0);
        ui.label(RichText::new("Risultato").color(p.muted));
        match result {
            Ok(s) => {
                ui.label(RichText::new(&s).size(27.0).strong());
                if ui.button("Copia risultato").clicked() {
                    ui.ctx().copy_text(s)
                }
            }
            Err(e) => {
                ui.colored_label(Color32::from_rgb(220, 80, 60), e);
            }
        }
    }
    fn read_convert(&self) -> Result<f64, String> {
        engine::evaluate(&self.convert_input.replace(',', "."), Angle::Deg)
    }
    fn converted(&self, c: usize) -> Result<f64, String> {
        converters::convert(c, self.read_convert()?, self.unit_from, self.unit_to)
    }
    fn converter(&mut self, ui: &mut egui::Ui, c: usize, p: Palette) {
        ui.add_space(18.0);
        ui.horizontal(|ui| {
            ui.label("Valore");
            ui.add(
                egui::TextEdit::singleline(&mut self.convert_input)
                    .desired_width(180.0)
                    .font(FontId::proportional(28.0)),
            );
        });
        ui.add_space(20.0);
        if c == 0 {
            self.currency_view(ui, p);
            return;
        }
        let list = converters::units(c);
        self.unit_from = self.unit_from.min(list.len() - 1);
        self.unit_to = self.unit_to.min(list.len() - 1);
        egui::ComboBox::from_id_salt("from-unit")
            .width(260.0)
            .selected_text(list[self.unit_from].name)
            .show_ui(ui, |ui| {
                for (i, u) in list.iter().enumerate() {
                    ui.selectable_value(&mut self.unit_from, i, u.name);
                }
            });
        if ui.button("⇅  Scambia unità").clicked() {
            if let Ok(v) = self.converted(c) {
                self.convert_input = fmt(v)
            }
            std::mem::swap(&mut self.unit_from, &mut self.unit_to)
        }
        egui::ComboBox::from_id_salt("to-unit")
            .width(260.0)
            .selected_text(list[self.unit_to].name)
            .show_ui(ui, |ui| {
                for (i, u) in list.iter().enumerate() {
                    ui.selectable_value(&mut self.unit_to, i, u.name);
                }
            });
        ui.add_space(28.0);
        match self.converted(c) {
            Ok(v) => {
                ui.label(RichText::new(fmt(v)).size(40.0).strong());
                ui.label(list[self.unit_to].name);
                if ui.button("Copia risultato").clicked() {
                    ui.ctx().copy_text(fmt(v))
                }
                ui.add_space(22.0);
                egui::CollapsingHeader::new("Equivale anche a").show(ui, |ui| {
                    for (i, u) in list
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| *i != self.unit_from && *i != self.unit_to)
                        .take(4)
                    {
                        if let Ok(value) =
                            converters::convert(c, self.read_convert().unwrap(), self.unit_from, i)
                        {
                            ui.label(
                                RichText::new(format!("{} {}", fmt(value), u.name)).color(p.muted),
                            );
                        }
                    }
                });
            }
            Err(e) => {
                ui.colored_label(Color32::from_rgb(220, 80, 60), e);
            }
        }
        ui.add_space(24.0);
        self.converter_pad(ui, p);
    }
    fn converter_pad(&mut self, ui: &mut egui::Ui, p: Palette) {
        let w = (ui.available_width() - 10.0) / 3.0;
        for row in [
            ["CE", "⌫", "±"],
            ["7", "8", "9"],
            ["4", "5", "6"],
            ["1", "2", "3"],
            [".", "0", "00"],
        ] {
            ui.horizontal(|ui| {
                for k in row {
                    if Self::key(ui, k, w, 36.0, p, 1) {
                        match k {
                            "CE" => self.convert_input = "0".into(),
                            "⌫" => {
                                self.convert_input.pop();
                                if self.convert_input.is_empty() {
                                    self.convert_input = "0".into()
                                }
                            }
                            "±" => {
                                if self.convert_input.starts_with('-') {
                                    self.convert_input.remove(0);
                                } else {
                                    self.convert_input.insert(0, '-')
                                }
                            }
                            "." => {
                                if !self.convert_input.contains('.') {
                                    self.convert_input.push('.')
                                }
                            }
                            _ => {
                                if self.convert_input == "0" {
                                    self.convert_input.clear()
                                }
                                self.convert_input.push_str(k)
                            }
                        }
                    }
                }
            });
        }
    }
    fn currency_view(&mut self, ui: &mut egui::Ui, p: Palette) {
        let codes: Vec<String> = self
            .rates
            .as_ref()
            .map(|r| r.rates.keys().cloned().collect())
            .unwrap_or_else(|| vec!["EUR".into(), "USD".into(), "GBP".into()]);
        for (i, v) in [&mut self.currency_from, &mut self.currency_to]
            .into_iter()
            .enumerate()
        {
            egui::ComboBox::from_id_salt(("currency", i))
                .selected_text(v.clone())
                .show_ui(ui, |ui| {
                    for code in &codes {
                        ui.selectable_value(v, code.clone(), code);
                    }
                });
        }
        if ui.button("⇅  Scambia valute").clicked() {
            std::mem::swap(&mut self.currency_from, &mut self.currency_to)
        }
        ui.add_space(15.0);
        match self
            .rates
            .as_ref()
            .ok_or_else(|| "Cambi non disponibili: aggiorna dalla BCE".to_string())
            .and_then(|r| r.convert(self.read_convert()?, &self.currency_from, &self.currency_to))
        {
            Ok(v) => {
                ui.label(
                    RichText::new(format!("{} {}", fmt(v), self.currency_to))
                        .size(35.0)
                        .strong(),
                );
                if ui.button("Copia risultato").clicked() {
                    ui.ctx().copy_text(fmt(v))
                }
            }
            Err(e) => {
                ui.label(e);
            }
        }
        if let Some(r) = &self.rates {
            ui.label(RichText::new(format!("BCE · dati del {}", r.date)).color(p.muted));
            if r.old() {
                ui.colored_label(
                    Color32::from_rgb(210, 130, 20),
                    "Dati più vecchi di 7 giorni",
                );
            }
        }
        ui.label(
            RichText::new("Cambi di riferimento, non quotazioni di transazione")
                .size(12.0)
                .color(p.muted),
        );
        if self.receiver.is_some() {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Aggiornamento BCE…");
            });
        } else if ui.button("Aggiorna cambi").clicked() {
            self.fetch_rates()
        }
        if let Some(e) = &self.currency_error {
            ui.colored_label(Color32::from_rgb(220, 90, 60), e);
            if self.rates.is_some() {
                ui.label("Mostrati gli ultimi cambi salvati, con la loro data.");
            }
        }
        ui.add_space(22.0);
        self.converter_pad(ui, p);
    }
    fn settings(&mut self, ui: &mut egui::Ui, p: Palette) {
        ui.add_space(16.0);
        ui.heading("Aspetto");
        for (t, label) in [
            (Theme::System, "Usa impostazione di Windows"),
            (Theme::Light, "Chiaro"),
            (Theme::Dark, "Scuro"),
        ] {
            ui.radio_value(&mut self.theme, t, label);
        }
        ui.add_space(24.0);
        ui.heading("Calcolatrice Rust");
        ui.label(format!("Versione {VERSION} · {RELEASE}"));
        ui.label("Interfaccia desktop ispirata alla Calcolatrice Microsoft.");
        ui.add_space(12.0);
        ui.label(RichText::new("Tastiera").strong());
        ui.label("Invio: risultato · Esc: C · Canc: CE · Backspace: cancella");
        ui.label("Alt+1…5: Standard, Scientifica, Grafica, Programmatore, Data");
        ui.label("Ctrl+M: MS · Ctrl+R: MR · Ctrl+P/Q: M+/M− · Ctrl+L: MC");
        ui.label("Programmatore: F5 HEX · F6 DEC · F7 OCT · F8 BIN · F12 BYTE");
        ui.add_space(16.0);
        ui.label(RichText::new("Precisione e grafici").strong());
        ui.label("Operazioni razionali esatte su decimali e interi grandi; visualizzazione fino a 32 decimali. Trigonometria e funzioni non razionali: f64 (circa 15 cifre). Programmatore: interi esatti a 64 bit con overflow modulare.");
        ui.label("Grafici campionati: contorni e zeri sono stime numeriche, non una dimostrazione simbolica.");
        ui.label(
            RichText::new(
                "Tema, cronologia, memoria e cambi BCE sono salvati nel profilo dell'app.",
            )
            .size(13.0)
            .color(p.muted),
        );
        if ui.button("Cancella cronologia e memoria").clicked() {
            self.calc.history.clear();
            self.calc.memory.clear();
            self.notice = "Cronologia e memoria cancellate".into();
        }
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        if self.preview.as_ref().is_some_and(|p| p.frames == 0) {
            let compact = std::env::args().any(|s| s == "--compact");
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(if compact {
                Vec2::new(400.0, 660.0)
            } else {
                Vec2::new(960.0, 790.0)
            }));
        }
        let p = self.style(ctx);
        self.currency_poll(ctx);
        self.keyboard(ctx);
        self.header(ctx, p);
        egui::TopBottomPanel::bottom("status")
            .frame(
                egui::Frame::NONE
                    .fill(p.bg)
                    .inner_margin(egui::Margin::symmetric(12, 5)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&self.notice).size(12.0).color(p.muted));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new(format!("v{VERSION} · {RELEASE}"))
                                .size(11.0)
                                .color(p.muted),
                        );
                    });
                });
            });
        self.navigation(ctx, p);
        self.side_history(ctx, p);
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(p.bg).inner_margin(12))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| match self.mode {
                        Mode::Standard | Mode::Scientific => self.calculator(ui, p),
                        Mode::Programmer => self.programmer(ui, p),
                        Mode::Date => self.date_view(ui, p),
                        Mode::Converter(i) => self.converter(ui, i, p),
                        Mode::Settings => self.settings(ui, p),
                        Mode::Graph => self.graph.show(ui),
                    });
            });
        if let Some(preview) = &mut self.preview {
            preview.frames += 1;
            ctx.request_repaint();
            if preview.frames > 8 && !preview.requested {
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                preview.requested = true
            }
            for e in ctx.input(|i| i.events.clone()) {
                if let egui::Event::Screenshot { image, .. } = e {
                    let rgba: Vec<u8> = image.pixels.iter().flat_map(|p| p.to_array()).collect();
                    let _ = image::save_buffer(
                        &preview.path,
                        &rgba,
                        image.width() as u32,
                        image.height() as u32,
                        image::ColorType::Rgba8,
                    );
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
    }
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if self.preview.is_none() {
            eframe::set_value(
                storage,
                "calculator-v2",
                &Saved {
                    theme: self.theme,
                    mode: self.mode,
                    history: self.calc.history.clone(),
                    memory: self.calc.memory.clone(),
                    programmer_memory: self.programmer.memory.clone(),
                    rates: self.rates.clone(),
                },
            );
        }
    }
}
