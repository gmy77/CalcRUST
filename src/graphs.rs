use crate::engine::{fmt, parse, Angle, Expr};
use eframe::egui::{self, Color32, RichText};
use std::collections::BTreeMap;
type Segments = Vec<Vec<[f64; 2]>>;
pub struct Graph {
    pub equations: Vec<String>,
    pub lo: f64,
    pub hi: f64,
    pub angle: Angle,
    pub vars: BTreeMap<String, f64>,
    reset: bool,
    square: bool,
    cache_key: String,
    cache: Vec<(String, Segments, usize, String)>,
    errors: Vec<String>,
}
impl Default for Graph {
    fn default() -> Self {
        Self {
            equations: vec!["sin(x)".into(), "x^2/5".into()],
            lo: -10.0,
            hi: 10.0,
            angle: Angle::Rad,
            vars: BTreeMap::from([("a".into(), 1.0), ("b".into(), 0.0)]),
            reset: false,
            square: false,
            cache_key: String::new(),
            cache: vec![],
            errors: vec![],
        }
    }
}
pub fn color(i: usize) -> Color32 {
    [
        Color32::from_rgb(0, 135, 230),
        Color32::from_rgb(225, 90, 50),
        Color32::from_rgb(40, 170, 110),
        Color32::from_rgb(170, 90, 210),
        Color32::from_rgb(210, 155, 25),
        Color32::from_rgb(30, 180, 190),
        Color32::from_rgb(200, 80, 140),
        Color32::from_gray(140),
    ][i % 8]
}
impl Graph {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Angoli:");
            for (a, t) in [
                (Angle::Rad, "RAD"),
                (Angle::Deg, "DEG"),
                (Angle::Grad, "GRAD"),
            ] {
                ui.selectable_value(&mut self.angle, a, t);
            }
        });
        let mut remove = None;
        for (i, e) in self.equations.iter_mut().enumerate() {
            ui.push_id(i, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("{}", i + 1)).color(color(i)));
                    ui.add(
                        egui::TextEdit::singleline(e)
                            .desired_width((ui.available_width() - 35.0).max(100.0))
                            .hint_text("sin(x), y=x^2, x^2+y^2=9"),
                    );
                    if ui.small_button("×").clicked() {
                        remove = Some(i)
                    }
                });
            });
        }
        if let Some(i) = remove {
            self.equations.remove(i);
        }
        ui.horizontal(|ui| {
            if ui.button("+ Equazione").clicked() && self.equations.len() < 8 {
                self.equations.push(String::new())
            }
            if ui.button("Reimposta vista").clicked() {
                self.reset = true
            }
            ui.label("Campiona x:");
            ui.add(egui::DragValue::new(&mut self.lo).speed(0.2));
            ui.add(egui::DragValue::new(&mut self.hi).speed(0.2));
        });
        ui.horizontal(|ui| {
            ui.label("Variabili:");
            ui.checkbox(&mut self.square, "Assi 1:1");
            for (name, value) in &mut self.vars {
                ui.label(format!("{name} ="));
                ui.add(egui::DragValue::new(value).speed(0.05));
            }
        });
        if !self.lo.is_finite()
            || !self.hi.is_finite()
            || self.lo >= self.hi
            || self.hi - self.lo > 1e8
        {
            ui.label("Intervallo x non valido");
            return;
        }
        let key = format!(
            "{:?}{:?}{}{}{:?}",
            self.equations, self.angle, self.lo, self.hi, self.vars
        );
        if key != self.cache_key {
            self.cache_key = key;
            self.cache.clear();
            self.errors.clear();
            for (i, e) in self.equations.iter().enumerate() {
                if e.trim().is_empty() {
                    continue;
                }
                match series(e, self.lo, self.hi, self.angle, &self.vars) {
                    Ok((s, f)) => self.cache.push((e.clone(), s, i, f)),
                    Err(err) => self.errors.push(format!("Equazione {}: {err}", i + 1)),
                }
            }
        }
        let mut plot = egui_plot::Plot::new("graph-canvas")
            .height((ui.available_height() - 115.0).max(200.0))
            .include_x(self.lo)
            .include_x(self.hi)
            .include_y(self.lo)
            .include_y(self.hi)
            .legend(egui_plot::Legend::default());
        if self.square {
            plot = plot.data_aspect(1.0);
        }
        if self.reset {
            plot = plot.reset();
        }
        self.reset = false;
        plot.show(ui, |plot_ui| {
            for (name, segments, index, _) in &self.cache {
                for (j, points) in segments.iter().enumerate() {
                    let line = egui_plot::Line::new(
                        if j == 0 { name.clone() } else { String::new() },
                        egui_plot::PlotPoints::from(points.clone()),
                    )
                    .color(color(*index))
                    .width(2.0_f32);
                    plot_ui.line(if j == 0 {
                        line.name(name.clone())
                    } else {
                        line
                    });
                }
            }
        });
        for err in &self.errors {
            ui.colored_label(Color32::from_rgb(220, 90, 60), err);
        }
        for (_, _, _, f) in &self.cache {
            ui.label(RichText::new(f).size(12.0));
        }
        ui.label(
            RichText::new("Rotella: zoom · trascina: sposta · doppio clic: vista automatica")
                .size(12.0),
        );
    }
}
fn validate(expr: &Expr, angle: Angle, vars: &BTreeMap<String, f64>) -> Result<(), String> {
    match expr.eval(angle, vars) {
        Err(e) if e.starts_with("Variabile") || e.starts_with("Funzione") => Err(e),
        _ => Ok(()),
    }
}
fn series(
    text: &str,
    lo: f64,
    hi: f64,
    angle: Angle,
    vars: &BTreeMap<String, f64>,
) -> Result<(Segments, String), String> {
    let input = text.trim();
    let relation = if input.contains("<=") {
        Some("<=")
    } else if input.contains(">=") {
        Some(">=")
    } else if input.contains('<') {
        Some("<")
    } else if input.contains('>') {
        Some(">")
    } else if input.contains('=') {
        Some("=")
    } else {
        None
    };
    let explicit = if let Some("=") = relation {
        input.split_once('=').and_then(|(l, r)| {
            if l.trim() == "y" {
                Some(r.trim())
            } else {
                None
            }
        })
    } else if relation.is_none() {
        Some(input)
    } else {
        None
    };
    if let Some(expression) = explicit {
        let ast = parse(expression)?;
        let mut vars = vars.clone();
        vars.insert("x".into(), lo);
        validate(&ast, angle, &vars)?;
        let mut segments = vec![];
        let mut line = vec![];
        let mut last: Option<[f64; 2]> = None;
        let mut roots: Vec<f64> = vec![];
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        for i in 0..=1200 {
            let x = lo + (hi - lo) * i as f64 / 1200.0;
            vars.insert("x".into(), x);
            let y = ast.eval(angle, &vars).ok().filter(|v| v.abs() < 1e10);
            if let Some(y) = y {
                if let Some(prev) = last {
                    if (y - prev[1]).abs() > (hi - lo) * 20.0 {
                        if line.len() > 1 {
                            segments.push(std::mem::take(&mut line));
                        } else {
                            line.clear()
                        }
                    } else if prev[1] * y <= 0.0 && roots.len() < 8 {
                        let mut a = prev[0];
                        let mut b = x;
                        let mut fa = prev[1];
                        let mut valid = true;
                        for _ in 0..40 {
                            let m = (a + b) / 2.0;
                            vars.insert("x".into(), m);
                            let Ok(fm) = ast.eval(angle, &vars) else {
                                valid = false;
                                break;
                            };
                            if fm == 0.0 {
                                a = m;
                                b = m;
                                break;
                            }
                            if fa * fm <= 0.0 {
                                b = m
                            } else {
                                a = m;
                                fa = fm
                            }
                        }
                        let root = (a + b) / 2.0;
                        vars.insert("x".into(), root);
                        if valid
                            && ast
                                .eval(angle, &vars)
                                .map(|v| v.abs() < 1e-6)
                                .unwrap_or(false)
                            && roots
                                .last()
                                .map(|r| (r - root).abs() > 1e-4)
                                .unwrap_or(true)
                        {
                            roots.push(root)
                        }
                    }
                }
                min = min.min(y);
                max = max.max(y);
                line.push([x, y]);
                last = Some([x, y]);
            } else {
                if line.len() > 1 {
                    segments.push(std::mem::take(&mut line));
                } else {
                    line.clear()
                }
                last = None
            }
        }
        if line.len() > 1 {
            segments.push(line)
        }
        if !min.is_finite() {
            return Err("Nessun punto definito nell'intervallo".into());
        }
        let roots = if roots.is_empty() {
            "nessuno rilevato".into()
        } else {
            roots.iter().map(|x| fmt(*x)).collect::<Vec<_>>().join(", ")
        };
        return Ok((
            segments,
            format!(
                "{input}: zeri stimati [{roots}] · y campionata [{}; {}]",
                fmt(min),
                fmt(max)
            ),
        ));
    }
    let relation = relation.ok_or("Equazione non valida")?;
    let (left, right) = input.split_once(relation).ok_or("Equazione non valida")?;
    let ast = parse(&format!("({left})-({right})"))?;
    let mut vars = vars.clone();
    vars.insert("x".into(), lo);
    vars.insert("y".into(), lo);
    validate(&ast, angle, &vars)?;
    let size = 90usize;
    let step = (hi - lo) / size as f64;
    let mut values = vec![None; (size + 1) * (size + 1)];
    for y in 0..=size {
        for x in 0..=size {
            vars.insert("x".into(), lo + x as f64 * step);
            vars.insert("y".into(), lo + y as f64 * step);
            values[y * (size + 1) + x] = ast.eval(angle, &vars).ok();
        }
    }
    let mut segments = vec![];
    for y in 0..size {
        for x in 0..size {
            let px = lo + x as f64 * step;
            let py = lo + y as f64 * step;
            let corners = [
                (px, py, values[y * (size + 1) + x]),
                (px + step, py, values[y * (size + 1) + x + 1]),
                (px + step, py + step, values[(y + 1) * (size + 1) + x + 1]),
                (px, py + step, values[(y + 1) * (size + 1) + x]),
            ];
            let mut points = vec![];
            for j in 0..4 {
                let (a, b) = (corners[j], corners[(j + 1) % 4]);
                if let (Some(va), Some(vb)) = (a.2, b.2) {
                    if (va <= 0.0) != (vb <= 0.0) {
                        let t = va / (va - vb);
                        points.push([a.0 + t * (b.0 - a.0), a.1 + t * (b.1 - a.1)]);
                    }
                }
            }
            for pair in points.chunks(2) {
                if pair.len() == 2 {
                    segments.push(pair.to_vec())
                }
            }
            if relation != "=" {
                if let Some(v) = corners[0].2 {
                    if if relation.starts_with('<') {
                        v < 0.0
                    } else {
                        v > 0.0
                    } {
                        segments.push(vec![[px, py], [px + step * 0.75, py]])
                    }
                }
            }
        }
    }
    Ok((
        segments,
        format!(
            "{input}: contorno numerico 90×90{}",
            if relation != "=" {
                " · tratteggio = regione valida"
            } else {
                ""
            }
        ),
    ))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn plot_functions_and_implicit() {
        let vars = BTreeMap::new();
        assert!(!series("sin(x)", -10.0, 10.0, Angle::Rad, &vars)
            .unwrap()
            .0
            .is_empty());
        assert!(
            series("x^2+y^2=9", -5.0, 5.0, Angle::Rad, &vars)
                .unwrap()
                .0
                .len()
                > 20
        );
        assert!(series("unknown(x)", -1.0, 1.0, Angle::Rad, &vars).is_err());
        assert!(series("x<1", -2.0, 2.0, Angle::Rad, &vars).is_ok());
    }
}
