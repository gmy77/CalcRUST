use crate::engine::CalcResult;
use chrono::{Datelike, Duration, Local, NaiveDate};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy)]
pub struct Unit {
    pub name: &'static str,
    pub scale: f64,
    pub offset: f64,
}
const fn u(name: &'static str, scale: f64) -> Unit {
    Unit {
        name,
        scale,
        offset: 0.0,
    }
}
pub const CATEGORIES: &[&str] = &[
    "Valuta",
    "Volume",
    "Lunghezza",
    "Peso e massa",
    "Temperatura",
    "Energia",
    "Area",
    "Velocità",
    "Tempo",
    "Potenza",
    "Dati",
    "Pressione",
    "Angolo",
];
pub fn units(category: usize) -> Vec<Unit> {
    match category {
        1 => vec![
            u("Millilitri", 0.000001),
            u("Centimetri cubici", 0.000001),
            u("Litri", 0.001),
            u("Metri cubici", 1.0),
            u("Cucchiaini USA", 0.00000492892159375),
            u("Cucchiai USA", 0.00001478676478125),
            u("Once liquide USA", 0.0000295735295625),
            u("Tazze USA", 0.0002365882365),
            u("Pinte USA", 0.000473176473),
            u("Quarti USA", 0.000946352946),
            u("Galloni USA", 0.003785411784),
            u("Cucchiaini UK", 0.000005919388020833),
            u("Cucchiai UK", 0.0000177581640625),
            u("Once liquide UK", 0.0000284130625),
            u("Pinte UK", 0.00056826125),
            u("Quarti UK", 0.0011365225),
            u("Galloni UK", 0.00454609),
            u("Pollici cubici", 0.000016387064),
            u("Piedi cubici", 0.028316846592),
            u("Iarde cubiche", 0.764554857984),
        ],
        2 => vec![
            u("Nanometri", 1e-9),
            u("Micrometri", 1e-6),
            u("Millimetri", 0.001),
            u("Centimetri", 0.01),
            u("Metri", 1.0),
            u("Chilometri", 1000.0),
            u("Pollici", 0.0254),
            u("Piedi", 0.3048),
            u("Iarde", 0.9144),
            u("Miglia", 1609.344),
            u("Miglia nautiche", 1852.0),
        ],
        3 => vec![
            u("Carati", 0.0002),
            u("Milligrammi", 1e-6),
            u("Centigrammi", 0.00001),
            u("Decigrammi", 0.0001),
            u("Grammi", 0.001),
            u("Decagrammi", 0.01),
            u("Ettogrammi", 0.1),
            u("Chilogrammi", 1.0),
            u("Tonnellate metriche", 1000.0),
            u("Once", 0.028349523125),
            u("Libbre", 0.45359237),
            u("Stone", 6.35029318),
            u("Tonnellate corte USA", 907.18474),
            u("Tonnellate lunghe UK", 1016.0469088),
        ],
        4 => vec![
            Unit {
                name: "Celsius",
                scale: 1.0,
                offset: 273.15,
            },
            Unit {
                name: "Fahrenheit",
                scale: 5.0 / 9.0,
                offset: 255.3722222222222,
            },
            u("Kelvin", 1.0),
        ],
        5 => vec![
            u("Joule", 1.0),
            u("Chilojoule", 1000.0),
            u("Calorie termochimiche", 4.184),
            u("Chilocalorie", 4184.0),
            u("Wattora", 3600.0),
            u("Chilowattora", 3_600_000.0),
            u("Elettronvolt", 1.602176634e-19),
            u("BTU (IT)", 1055.05585262),
            u("Piedi-libbra", 1.3558179483314),
        ],
        6 => vec![
            u("Millimetri quadrati", 1e-6),
            u("Centimetri quadrati", 0.0001),
            u("Metri quadrati", 1.0),
            u("Ettari", 10000.0),
            u("Chilometri quadrati", 1e6),
            u("Pollici quadrati", 0.00064516),
            u("Piedi quadrati", 0.09290304),
            u("Iarde quadrate", 0.83612736),
            u("Acri", 4046.8564224),
            u("Miglia quadrate", 2589988.110336),
        ],
        7 => vec![
            u("Centimetri/secondo", 0.01),
            u("Metri/secondo", 1.0),
            u("Chilometri/ora", 1.0 / 3.6),
            u("Piedi/secondo", 0.3048),
            u("Miglia/ora", 0.44704),
            u("Nodi", 1852.0 / 3600.0),
            u("Mach (aria a 15 °C)", 340.294),
        ],
        8 => vec![
            u("Nanosecondi", 1e-9),
            u("Microsecondi", 1e-6),
            u("Millisecondi", 0.001),
            u("Secondi", 1.0),
            u("Minuti", 60.0),
            u("Ore", 3600.0),
            u("Giorni", 86400.0),
            u("Settimane", 604800.0),
            u("Anni (365 giorni)", 31536000.0),
        ],
        9 => vec![
            u("Watt", 1.0),
            u("Chilowatt", 1000.0),
            u("Megawatt", 1e6),
            u("Cavalli vapore metrici", 735.49875),
            u("Horsepower meccanici", 745.6998715823),
            u("BTU/minuto", 1055.05585262 / 60.0),
            u("Piedi-libbra/minuto", 1.3558179483314 / 60.0),
        ],
        10 => vec![
            u("Bit", 1.0),
            u("Byte", 8.0),
            u("Kilobit", 1e3),
            u("Kilobyte", 8e3),
            u("Megabit", 1e6),
            u("Megabyte", 8e6),
            u("Gigabit", 1e9),
            u("Gigabyte", 8e9),
            u("Terabit", 1e12),
            u("Terabyte", 8e12),
            u("Petabit", 1e15),
            u("Petabyte", 8e15),
            u("Kibibit", 1024.0),
            u("Kibibyte", 8192.0),
            u("Mebibit", 1048576.0),
            u("Mebibyte", 8388608.0),
            u("Gibibit", 1073741824.0),
            u("Gibibyte", 8589934592.0),
            u("Tebibit", 1099511627776.0),
            u("Tebibyte", 8796093022208.0),
            u("Pebibit", 1125899906842624.0),
            u("Pebibyte", 9007199254740992.0),
        ],
        11 => vec![
            u("Atmosfere", 101325.0),
            u("Bar", 100000.0),
            u("Kilopascal", 1000.0),
            u("Pascal", 1.0),
            u("Millimetri di mercurio", 133.322387415),
            u("PSI", 6894.757293168),
            u("Ettopascal", 100.0),
        ],
        12 => vec![
            u("Gradi", std::f64::consts::PI / 180.0),
            u("Radianti", 1.0),
            u("Gradi centesimali", std::f64::consts::PI / 200.0),
        ],
        _ => vec![],
    }
}
pub fn convert(category: usize, value: f64, from: usize, to: usize) -> CalcResult<f64> {
    let list = units(category);
    let a = list.get(from).ok_or("Unità non valida")?;
    let b = list.get(to).ok_or("Unità non valida")?;
    let base = value * a.scale + a.offset;
    if category == 4 && base < -1e-9 {
        return Err("Temperatura sotto lo zero assoluto".into());
    }
    crate::engine::finite((base - b.offset) / b.scale)
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CurrencyRates {
    pub date: String,
    pub rates: BTreeMap<String, f64>,
}
impl CurrencyRates {
    pub fn parse(xml: &str) -> CalcResult<Self> {
        let attr = |s: &str, name: &str| -> Option<String> {
            let start = s.find(&format!("{name}="))? + name.len() + 1;
            let quote = *s.as_bytes().get(start)? as char;
            if quote != '\'' && quote != '"' {
                return None;
            }
            let rest = &s[start + 1..];
            Some(rest[..rest.find(quote)?].into())
        };
        let mut result = Self::default();
        result.rates.insert("EUR".into(), 1.0);
        for element in xml.split('<') {
            if let Some(date) = attr(element, "time") {
                result.date = date
            }
            if let (Some(currency), Some(rate)) = (attr(element, "currency"), attr(element, "rate"))
            {
                let value: f64 = rate.parse().map_err(|_| "Tasso BCE non valido")?;
                if currency.len() != 3 || !value.is_finite() || value <= 0.0 {
                    return Err("Dati BCE non validi".into());
                }
                result.rates.insert(currency, value);
            }
        }
        NaiveDate::parse_from_str(&result.date, "%Y-%m-%d")
            .map_err(|_| "Data BCE assente o non valida")?;
        if result.rates.len() < 3 {
            return Err("Risposta BCE incompleta".into());
        }
        Ok(result)
    }
    pub fn download() -> CalcResult<Self> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(20))
            .build()
            .map_err(|e| e.to_string())?;
        let response = client
            .get("https://www.ecb.europa.eu/stats/eurofxref/eurofxref-daily.xml")
            .send()
            .and_then(|r| r.error_for_status())
            .map_err(|e| format!("BCE non disponibile: {e}"))?;
        let text = response.text().map_err(|e| e.to_string())?;
        if text.len() > 1_000_000 {
            return Err("Risposta BCE troppo grande".into());
        }
        Self::parse(&text)
    }
    pub fn convert(&self, v: f64, a: &str, b: &str) -> CalcResult<f64> {
        let from = self.rates.get(a).ok_or("Cambio non disponibile")?;
        let to = self.rates.get(b).ok_or("Cambio non disponibile")?;
        crate::engine::finite(v / from * to)
    }
    pub fn old(&self) -> bool {
        NaiveDate::parse_from_str(&self.date, "%Y-%m-%d")
            .map(|d| (Local::now().date_naive() - d).num_days() > 7)
            .unwrap_or(true)
    }
}
pub fn date(y: i32, m: u32, d: u32) -> CalcResult<NaiveDate> {
    NaiveDate::from_ymd_opt(y, m, d).ok_or_else(|| "Data non valida".into())
}
pub fn add_date(start: NaiveDate, years: i32, months: i32, days: i64) -> CalcResult<NaiveDate> {
    let total =
        start.year() as i64 * 12 + start.month0() as i64 + years as i64 * 12 + months as i64;
    let year = i32::try_from(total.div_euclid(12)).map_err(|_| "Data fuori intervallo")?;
    let month = (total.rem_euclid(12) + 1) as u32;
    let mut day = start.day();
    let base = loop {
        if let Some(d) = NaiveDate::from_ymd_opt(year, month, day) {
            break d;
        }
        if day == 1 {
            return Err("Data fuori intervallo".into());
        }
        day -= 1
    };
    let duration = Duration::try_days(days).ok_or("Intervallo troppo grande")?;
    base.checked_add_signed(duration)
        .ok_or_else(|| "Data fuori intervallo".into())
}
pub fn difference(a: NaiveDate, b: NaiveDate) -> String {
    let (start, end) = if a <= b { (a, b) } else { (b, a) };
    let mut years = end.year() - start.year();
    if add_date(start, years, 0, 0).unwrap() > end {
        years -= 1
    }
    let mut months =
        (end.year() - start.year() - years) * 12 + end.month() as i32 - start.month() as i32;
    while add_date(start, years, months, 0).unwrap() > end {
        months -= 1
    }
    let pivot = add_date(start, years, months, 0).unwrap();
    let days = (end - pivot).num_days();
    format!("{years} anni, {months} mesi, {days} giorni")
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn units_round_trip() {
        for cat in 1..=12 {
            let list = units(cat);
            for i in 0..list.len() {
                let x = convert(cat, 20.0, 0, i).unwrap();
                let y = convert(cat, x, i, 0).unwrap();
                assert!((y - 20.0).abs() < 1e-7, "cat {cat}, {i}")
            }
        }
        assert!((convert(4, 32.0, 1, 0).unwrap()).abs() < 1e-10);
        assert!(convert(4, -274.0, 0, 2).is_err());
        assert_eq!(convert(10, 1.0, 1, 0).unwrap(), 8.0)
    }
    #[test]
    fn calendars() {
        let leap = date(2024, 2, 29).unwrap();
        assert_eq!(add_date(leap, 1, 0, 0).unwrap(), date(2025, 2, 28).unwrap());
        assert_eq!(add_date(date(2024, 1, 31).unwrap(), 0, 1, 0).unwrap(), leap);
        assert_eq!(
            difference(leap, date(2025, 3, 1).unwrap()),
            "1 anni, 0 mesi, 1 giorni"
        );
        assert!(date(2023, 2, 29).is_err())
    }
    #[test]
    fn currency() {
        let r=CurrencyRates::parse("<Cube time='2026-10-14'><Cube currency='USD' rate='1.2'/><Cube currency='GBP' rate='0.8'/></Cube>").unwrap();
        assert_eq!(r.convert(10.0, "USD", "EUR").unwrap(), 10.0 / 1.2);
        assert!(CurrencyRates::parse("<html>Error</html>").is_err())
    }
    #[test]
    #[ignore = "requires the ECB HTTPS endpoint"]
    fn live_ecb() {
        let rates = CurrencyRates::download().expect("Live ECB feed");
        assert!(rates.rates.len() > 10);
        assert!(rates.convert(100.0, "EUR", "USD").unwrap() > 0.0);
        println!(
            "ECB date: {}, currencies: {}",
            rates.date,
            rates.rates.len()
        );
    }
}
