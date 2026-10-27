use super::dev::{parse_hex_color, rgb_to_hsl};
use super::Op;
use crate::{parse_f64, split_param};

fn op(
    id: &'static str,
    name: &'static str,
    desc: &'static str,
    hint: Option<&'static str>,
    run: fn(&str, &str) -> Result<String, String>,
) -> Op {
    Op {
        id,
        name,
        category: "Converters",
        description: desc,
        param_hint: hint,
        run,
    }
}

pub fn ops() -> Vec<Op> {
    let mut v = vec![
        op("conv-base", "Number Base Converter", "Convert an integer between binary/octal/decimal/hex or any base 2–36. Parameter: from-base|||to-base.", Some("from|||to, e.g. 10|||16"), |i, p| {
            let v = split_param(p); if v.len() < 2 { return Err("Parameter must be: from-base|||to-base".into()); }
            let from: u32 = v[0].trim().parse().map_err(|_| "From-base must be 2–36".to_string())?;
            let to: u32 = v[1].trim().parse().map_err(|_| "To-base must be 2–36".to_string())?;
            if !(2..=36).contains(&from) || !(2..=36).contains(&to) { return Err("Bases must be 2–36".into()); }
            let n = i128::from_str_radix(i.trim().trim_start_matches("0x"), from).map_err(|_| format!("'{}' is not a base-{from} number", i.trim()))?;
            Ok(to_base(n, to))
        }),
        op("conv-roman-to-int", "Roman Numerals → Number", "Convert Roman numerals (XIV) to a number.", None, |i, _| Ok(roman_to_int(i.trim())?.to_string())),
        op("conv-int-to-roman", "Number → Roman Numerals", "Convert a number 1–3999 to Roman numerals.", None, |i, _| { let n: i64 = i.trim().parse().map_err(|_| "Enter a whole number 1–3999".to_string())?; int_to_roman(n) }),
        op("conv-temp", "Temperature Converter", "Input a number, parameter from|||to with units C, F or K. Lists all three if param empty.", Some("C|||F (optional)"), temp),
        op("conv-timestamp-to-date", "Unix Timestamp → Date", "Convert seconds since 1970-01-01 UTC to a date/time (UTC). Milliseconds auto-detected.", None, |i, _| { let mut n: i64 = i.trim().parse().map_err(|_| "Enter a Unix timestamp (whole seconds)".to_string())?; if n.abs() > 100_000_000_000 { n /= 1000; } Ok(format!("{n} = {}", unix_to_string(n))) }),
        op("conv-date-to-timestamp", "Date → Unix Timestamp", "Input a date as YYYY-MM-DD or YYYY-MM-DD HH:MM:SS (UTC).", None, |i, _| date_to_unix(i.trim())),
        op("conv-bytes", "Bytes Human-Readable", "Convert a byte count to KB/MB/GB… and back-list all sizes.", None, |i, _| { let n = parse_f64(i, "bytes")?; Ok(bytes_report(n)) }),
        op("conv-number-to-words", "Number → Words", "Write a whole number in English words (up to trillions).", None, |i, _| { let n: i64 = i.trim().parse().map_err(|_| "Enter a whole number".to_string())?; Ok(number_to_words(n)) }),
        op("conv-decimal-to-fraction", "Decimal → Fraction", "Convert a decimal to its simplest fraction.", None, |i, _| { let x = parse_f64(i, "decimal")?; Ok(decimal_to_fraction(x)) }),
        op("conv-color-hex-rgb", "Color HEX → RGB / HSL / HSV", "Full color breakdown from a hex color.", None, |i, _| { let (r, g, b) = parse_hex_color(i)?; let (h, s, l) = rgb_to_hsl(r, g, b); let max = r.max(g).max(b) as f64 / 255.0; let hsv_v = max * 100.0; Ok(format!("#{r:02X}{g:02X}{b:02X}\nrgb({r}, {g}, {b})\nhsl({h:.0}, {s:.1}%, {l:.1}%)\nHSV value/brightness: {hsv_v:.1}%\nLuminance (rough): {:.1}%", (0.2126 * r as f64 + 0.7152 * g as f64 + 0.0722 * b as f64) / 2.55)) }),
        op("conv-color-rgb-hex", "Color RGB → HEX", "Input as r,g,b (e.g. 229,72,77).", None, |i, _| { let parts: Result<Vec<u8>, String> = i.split(',').map(|s| s.trim().parse::<u8>().map_err(|_| "Each part must be 0–255".to_string())).collect(); let p = parts?; if p.len() != 3 { return Err("Enter r,g,b — three numbers".into()); } Ok(format!("#{:02X}{:02X}{:02X}", p[0], p[1], p[2])) }),
        op("conv-color-invert", "Color Invert", "The opposite color (255 minus each channel).", None, |i, _| { let (r, g, b) = parse_hex_color(i)?; Ok(format!("#{:02X}{:02X}{:02X}", 255 - r, 255 - g, 255 - b)) }),
        op("conv-color-lighten", "Color Lighten / Darken", "Parameter: percent — positive lightens toward white, negative darkens toward black.", Some("percent, e.g. 20 or -20"), |i, p| { let (r, g, b) = parse_hex_color(i)?; let pct: f64 = p.trim().parse().map_err(|_| "Parameter must be a percent like 20".to_string())?; let adj = |c: u8| -> u8 { let c = c as f64; let out = if pct >= 0.0 { c + (255.0 - c) * pct / 100.0 } else { c * (1.0 + pct / 100.0) }; out.clamp(0.0, 255.0).round() as u8 }; Ok(format!("#{:02X}{:02X}{:02X}", adj(r), adj(g), adj(b))) }),
        op("conv-color-complement", "Color Complement", "The color opposite on the color wheel (hue + 180°).", None, |i, _| { let (r, g, b) = parse_hex_color(i)?; let (h, s, l) = rgb_to_hsl(r, g, b); let (r2, g2, b2) = hsl_to_rgb((h + 180.0) % 360.0, s, l); Ok(format!("#{:02X}{:02X}{:02X}", r2, g2, b2)) }),
        op("conv-color-palette", "Color Palette (Triad + Shades)", "Three-color triad plus 5 shades of the input color.", None, |i, _| { let (r, g, b) = parse_hex_color(i)?; let (h, s, l) = rgb_to_hsl(r, g, b); let tri = |dh: f64| { let (a, b2, c) = hsl_to_rgb((h + dh) % 360.0, s, l); format!("#{a:02X}{b2:02X}{c:02X}") }; let mut shades = Vec::new(); for f in [0.25, 0.5, 0.75, 1.25, 1.5] { let (a, b2, c) = hsl_to_rgb(h, s, (l * f).clamp(2.0, 98.0)); shades.push(format!("#{a:02X}{b2:02X}{c:02X}")); } Ok(format!("Base: #{r:02X}{g:02X}{b:02X}\nTriad: {} , {}\nAnalogous: {} , {}\nShades (dark→light): {}", tri(120.0), tri(240.0), tri(30.0), tri(330.0), shades.join("  "))) }),
        op("conv-aspect", "Aspect Ratio", "Input width|||height as the text — get the simplified ratio and, for a parameter width, the matching height.", Some("new width (optional) — input is width|||height"), aspect),
    ];
    v.extend(unit_ops());
    v
}

fn unit_ops() -> Vec<Op> {
    vec![
        Op { id: "conv-length", name: "Length Converter", category: "Converters", description: "mm, cm, m, km, inch, foot, yard, mile, nautical mile. Input = number, Parameter = from|||to.", param_hint: Some("from|||to, e.g. mile|||km"), run: unit_length },
        Op { id: "conv-weight", name: "Weight / Mass Converter", category: "Converters", description: "g, kg, tonne, oz, lb, stone. Input = number, Parameter = from|||to.", param_hint: Some("from|||to, e.g. lb|||kg"), run: unit_weight },
        Op { id: "conv-area", name: "Area Converter", category: "Converters", description: "sqm, sqkm, hectare, acre, sqft, sqin, sqmi. Input = number, Parameter = from|||to.", param_hint: Some("from|||to, e.g. acre|||sqm"), run: unit_area },
        Op { id: "conv-volume", name: "Volume Converter", category: "Converters", description: "litre, ml, cubic-m, gallon-us/uk, cup-us, tbsp, tsp, floz-us. Input = number, Parameter = from|||to.", param_hint: Some("from|||to, e.g. gallon-us|||litre"), run: unit_volume },
        Op { id: "conv-speed", name: "Speed Converter", category: "Converters", description: "mps, kmh, mph, knot, fps. Input = number, Parameter = from|||to.", param_hint: Some("from|||to, e.g. mph|||kmh"), run: unit_speed },
        Op { id: "conv-time-units", name: "Time Duration Converter", category: "Converters", description: "second, minute, hour, day, week, year. Input = number, Parameter = from|||to.", param_hint: Some("from|||to, e.g. day|||hour"), run: unit_time },
        Op { id: "conv-data", name: "Data Storage Converter", category: "Converters", description: "byte, KB/MB/GB/TB (1000-based), KiB/MiB/GiB/TiB (1024-based), bit. Input = number, Parameter = from|||to.", param_hint: Some("from|||to, e.g. GiB|||MB"), run: unit_data },
        Op { id: "conv-pressure", name: "Pressure Converter", category: "Converters", description: "Pa, kPa, bar, psi, atm, mmHg. Input = number, Parameter = from|||to.", param_hint: Some("from|||to, e.g. psi|||bar"), run: unit_pressure },
        Op { id: "conv-energy", name: "Energy Converter", category: "Converters", description: "joule, kJ, calorie, kcal, kWh, BTU. Input = number, Parameter = from|||to.", param_hint: Some("from|||to, e.g. kWh|||joule"), run: unit_energy },
        Op { id: "conv-angle", name: "Angle Converter", category: "Converters", description: "degree, radian, gradian, turn. Input = number, Parameter = from|||to.", param_hint: Some("from|||to, e.g. degree|||radian"), run: unit_angle },
    ]
}
const LENGTH: &[(&str, f64)] = &[
    ("mm", 0.001),
    ("cm", 0.01),
    ("m", 1.0),
    ("km", 1000.0),
    ("inch", 0.0254),
    ("foot", 0.3048),
    ("yard", 0.9144),
    ("mile", 1609.344),
    ("nautical-mile", 1852.0),
];
const WEIGHT: &[(&str, f64)] = &[
    ("g", 0.001),
    ("kg", 1.0),
    ("tonne", 1000.0),
    ("oz", 0.028349523125),
    ("lb", 0.45359237),
    ("stone", 6.35029318),
];
const AREA: &[(&str, f64)] = &[
    ("sqm", 1.0),
    ("sqkm", 1e6),
    ("hectare", 10000.0),
    ("acre", 4046.8564224),
    ("sqft", 0.09290304),
    ("sqin", 0.00064516),
    ("sqmi", 2589988.110336),
];
const VOLUME: &[(&str, f64)] = &[
    ("litre", 1.0),
    ("ml", 0.001),
    ("cubic-m", 1000.0),
    ("gallon-us", 3.785411784),
    ("gallon-uk", 4.54609),
    ("cup-us", 0.2365882365),
    ("tbsp", 0.0147867648),
    ("tsp", 0.00492892159),
    ("floz-us", 0.02957352956),
];
const SPEED: &[(&str, f64)] = &[
    ("mps", 1.0),
    ("kmh", 1.0 / 3.6),
    ("mph", 0.44704),
    ("knot", 0.514444),
    ("fps", 0.3048),
];
const TIMEU: &[(&str, f64)] = &[
    ("second", 1.0),
    ("minute", 60.0),
    ("hour", 3600.0),
    ("day", 86400.0),
    ("week", 604800.0),
    ("year", 31557600.0),
];
const DATA: &[(&str, f64)] = &[
    ("byte", 1.0),
    ("KB", 1000.0),
    ("MB", 1e6),
    ("GB", 1e9),
    ("TB", 1e12),
    ("KiB", 1024.0),
    ("MiB", 1048576.0),
    ("GiB", 1073741824.0),
    ("TiB", 1099511627776.0),
    ("bit", 0.125),
];
const PRESSURE: &[(&str, f64)] = &[
    ("Pa", 1.0),
    ("kPa", 1000.0),
    ("bar", 100000.0),
    ("psi", 6894.757),
    ("atm", 101325.0),
    ("mmHg", 133.322),
];
const ENERGY: &[(&str, f64)] = &[
    ("joule", 1.0),
    ("kJ", 1000.0),
    ("calorie", 4.184),
    ("kcal", 4184.0),
    ("kWh", 3600000.0),
    ("BTU", 1055.06),
];
const ANGLE: &[(&str, f64)] = &[
    ("degree", 1.0),
    ("radian", 57.29577951308232),
    ("gradian", 0.9),
    ("turn", 360.0),
];
fn unit_length(i: &str, p: &str) -> Result<String, String> {
    convert_unit(LENGTH, i, p)
}
fn unit_weight(i: &str, p: &str) -> Result<String, String> {
    convert_unit(WEIGHT, i, p)
}
fn unit_area(i: &str, p: &str) -> Result<String, String> {
    convert_unit(AREA, i, p)
}
fn unit_volume(i: &str, p: &str) -> Result<String, String> {
    convert_unit(VOLUME, i, p)
}
fn unit_speed(i: &str, p: &str) -> Result<String, String> {
    convert_unit(SPEED, i, p)
}
fn unit_time(i: &str, p: &str) -> Result<String, String> {
    convert_unit(TIMEU, i, p)
}
fn unit_data(i: &str, p: &str) -> Result<String, String> {
    convert_unit(DATA, i, p)
}
fn unit_pressure(i: &str, p: &str) -> Result<String, String> {
    convert_unit(PRESSURE, i, p)
}
fn unit_energy(i: &str, p: &str) -> Result<String, String> {
    convert_unit(ENERGY, i, p)
}
fn unit_angle(i: &str, p: &str) -> Result<String, String> {
    convert_unit(ANGLE, i, p)
}

pub fn convert_unit(units: &[(&str, f64)], i: &str, p: &str) -> Result<String, String> {
    let names: Vec<&str> = units.iter().map(|(n, _)| *n).collect();
    if p.trim().is_empty() {
        return Ok(format!("Units: {}\n\nHow to use: put the value in the input box, and set the Parameter to from|||to, e.g. {}|||{}", names.join(", "), names.first().copied().unwrap_or(""), names.get(1).copied().unwrap_or("")));
    }
    let value = parse_f64(i, "value")?;
    let v = split_param(p);
    if v.len() < 2 {
        return Err("Parameter must be: from-unit|||to-unit".into());
    }
    let from = units
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case(v[0].trim()))
        .ok_or(format!(
            "Unknown unit '{}'. Units: {}",
            v[0],
            names.join(", ")
        ))?;
    let to = units
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case(v[1].trim()))
        .ok_or(format!(
            "Unknown unit '{}'. Units: {}",
            v[1],
            names.join(", ")
        ))?;
    let result = value * from.1 / to.1;
    Ok(format!(
        "{} {} = {} {}",
        trim_num(value),
        from.0,
        trim_num(result),
        to.0
    ))
}

pub fn trim_num(x: f64) -> String {
    if x == x.trunc() && x.abs() < 1e15 {
        format!("{}", x as i64)
    } else {
        format!("{x:.10}")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}

fn to_base(mut n: i128, base: u32) -> String {
    if n == 0 {
        return "0".into();
    }
    let neg = n < 0;
    if neg {
        n = -n;
    }
    let digits = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut s = String::new();
    while n > 0 {
        s.push(digits.chars().nth((n % base as i128) as usize).unwrap());
        n /= base as i128;
    }
    if neg {
        s.push('-');
    }
    s.chars().rev().collect()
}
fn roman_to_int(s: &str) -> Result<i64, String> {
    let val = |c: char| -> Option<i64> {
        match c {
            'I' => Some(1),
            'V' => Some(5),
            'X' => Some(10),
            'L' => Some(50),
            'C' => Some(100),
            'D' => Some(500),
            'M' => Some(1000),
            _ => None,
        }
    };
    let up = s.to_uppercase();
    let mut total = 0i64;
    let mut prev = 0i64;
    for c in up.chars().rev() {
        let v = val(c).ok_or(format!("'{c}' is not a Roman numeral letter"))?;
        if v < prev {
            total -= v;
        } else {
            total += v;
            prev = v;
        }
    }
    if total <= 0 {
        return Err("Empty or invalid Roman numeral".into());
    }
    Ok(total)
}
fn int_to_roman(mut n: i64) -> Result<String, String> {
    if !(1..=3999).contains(&n) {
        return Err("Number must be 1–3999".into());
    }
    let table = [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut out = String::new();
    for (v, sym) in table {
        while n >= v {
            out.push_str(sym);
            n -= v;
        }
    }
    Ok(out)
}
fn temp(i: &str, p: &str) -> Result<String, String> {
    let v = parse_f64(i, "temperature")?;
    let parts = split_param(p);
    let to_c = |x: f64, u: &str| -> Result<f64, String> {
        match u.trim().to_uppercase().as_str() {
            "C" => Ok(x),
            "F" => Ok((x - 32.0) * 5.0 / 9.0),
            "K" => Ok(x - 273.15),
            other => Err(format!(
                "Unknown temperature unit '{other}' — use C, F or K"
            )),
        }
    };
    if parts.len() < 2 {
        // assume input is Celsius if no param? Better: report all assuming input unit = first param or C
        let from = if parts.is_empty() {
            "C"
        } else {
            parts[0].trim()
        };
        let c = to_c(v, from)?;
        return Ok(format!(
            "{v} °{from} =\n{:.4} °C\n{:.4} °F\n{:.4} K",
            c,
            c * 9.0 / 5.0 + 32.0,
            c + 273.15
        ));
    }
    let c = to_c(v, parts[0])?;
    let out = match parts[1].trim().to_uppercase().as_str() {
        "C" => c,
        "F" => c * 9.0 / 5.0 + 32.0,
        "K" => c + 273.15,
        other => return Err(format!("Unknown temperature unit '{other}'")),
    };
    Ok(format!(
        "{} {} = {} {}",
        trim_num(v),
        parts[0].trim().to_uppercase(),
        trim_num(out),
        parts[1].trim().to_uppercase()
    ))
}
// --- Unix time <-> civil date (Howard Hinnant's algorithms, public domain math) ---
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}
fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u64;
    let mp = if m > 2 { m - 3 } else { m + 9 } as u64;
    let doy = (153 * mp + 2) / 5 + d as u64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe as i64 - 719468
}
pub fn unix_to_string(ts: i64) -> String {
    let days = ts.div_euclid(86400);
    let secs = ts.rem_euclid(86400);
    let (y, m, d) = civil_from_days(days);
    format!(
        "{y:04}-{m:02}-{d:02} {:02}:{:02}:{:02} UTC",
        secs / 3600,
        (secs % 3600) / 60,
        secs % 60
    )
}
fn date_to_unix(s: &str) -> Result<String, String> {
    let (date, time) = s.split_once(' ').unwrap_or((s, "00:00:00"));
    let dp: Vec<i64> = date
        .split('-')
        .map(|x| x.parse())
        .collect::<Result<_, _>>()
        .map_err(|_| "Date must be YYYY-MM-DD".to_string())?;
    if dp.len() != 3 {
        return Err("Date must be YYYY-MM-DD".into());
    }
    let tp: Vec<i64> = time
        .split(':')
        .map(|x| x.parse())
        .collect::<Result<_, _>>()
        .map_err(|_| "Time must be HH:MM:SS".to_string())?;
    let (h, mi, se) = (
        tp.first().copied().unwrap_or(0),
        tp.get(1).copied().unwrap_or(0),
        tp.get(2).copied().unwrap_or(0),
    );
    let days = days_from_civil(dp[0], dp[1] as u32, dp[2] as u32);
    Ok(format!("{}", days * 86400 + h * 3600 + mi * 60 + se))
}
fn bytes_report(n: f64) -> String {
    let units = [
        ("bytes", 1.0),
        ("KB (1000)", 1e3),
        ("MB", 1e6),
        ("GB", 1e9),
        ("TB", 1e12),
        ("KiB (1024)", 1024.0),
        ("MiB", 1048576.0),
        ("GiB", 1073741824.0),
        ("TiB", 1099511627776.0),
    ];
    units
        .iter()
        .map(|(name, f)| format!("{}: {}", name, trim_num(n / f)))
        .collect::<Vec<_>>()
        .join("\n")
}
fn number_to_words(n: i64) -> String {
    if n == 0 {
        return "zero".into();
    }
    let neg = n < 0;
    let mut n = n.unsigned_abs();
    let ones = [
        "",
        "one",
        "two",
        "three",
        "four",
        "five",
        "six",
        "seven",
        "eight",
        "nine",
        "ten",
        "eleven",
        "twelve",
        "thirteen",
        "fourteen",
        "fifteen",
        "sixteen",
        "seventeen",
        "eighteen",
        "nineteen",
    ];
    let tens = [
        "", "", "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety",
    ];
    let under1000 = |mut x: u64| -> String {
        let mut parts = Vec::new();
        if x >= 100 {
            parts.push(format!("{} hundred", ones[(x / 100) as usize]));
            x %= 100;
        }
        if x >= 20 {
            let mut t = tens[(x / 10) as usize].to_string();
            if x % 10 > 0 {
                t.push('-');
                t.push_str(ones[(x % 10) as usize]);
            }
            parts.push(t);
        } else if x > 0 {
            parts.push(ones[x as usize].to_string());
        }
        parts.join(" ")
    };
    let scales = [
        (1_000_000_000_000u64, "trillion"),
        (1_000_000_000, "billion"),
        (1_000_000, "million"),
        (1_000, "thousand"),
    ];
    let mut parts = Vec::new();
    for (v, name) in scales {
        if n >= v {
            parts.push(format!("{} {}", under1000(n / v), name));
            n %= v;
        }
    }
    if n > 0 {
        parts.push(under1000(n));
    }
    format!("{}{}", if neg { "minus " } else { "" }, parts.join(" "))
}
fn decimal_to_fraction(x: f64) -> String {
    let sign = if x < 0.0 { -1i64 } else { 1 };
    let ax = x.abs();
    let whole = ax.trunc() as i64;
    let frac = ax - whole as f64;
    // continued fraction, tolerance 1e-9
    let (mut num, mut den) = (frac, 1.0f64); // (we iterate on reciprocal below instead)
    let _ = (&mut num, &mut den);
    let mut a = frac;
    let (mut h1, mut h2, mut k1, mut k2) = (1i64, 0i64, 0i64, 1i64);
    for _ in 0..64 {
        if a.abs() < 1e-12 {
            break;
        }
        let term = a.floor() as i64;
        let h = term * h1 + h2;
        let k = term * k1 + k2;
        h2 = h1;
        h1 = h;
        k2 = k1;
        k1 = k;
        let approx = h as f64 / k as f64;
        if (approx - frac).abs() < 1e-9 {
            break;
        }
        let rem = a - term as f64;
        if rem.abs() < 1e-12 {
            break;
        }
        a = 1.0 / rem;
    }
    let (fn_, fd) = (h1, k1.max(1));
    if fn_ == 0 {
        return format!("{}", sign * whole);
    }
    if whole == 0 {
        format!("{}/{}", sign * fn_, fd)
    } else {
        format!(
            "{} {}/{}  (= {}/{})",
            sign * whole,
            fn_,
            fd,
            sign * (whole * fd + fn_),
            fd
        )
    }
}
pub fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
    let (s, l) = (s / 100.0, l / 100.0);
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    (
        ((r + m) * 255.0).round() as u8,
        ((g + m) * 255.0).round() as u8,
        ((b + m) * 255.0).round() as u8,
    )
}
fn aspect(i: &str, p: &str) -> Result<String, String> {
    let v = split_param(i);
    if v.len() < 2 {
        return Err("Input must be: width|||height".into());
    }
    let w = parse_f64(v[0], "width")?;
    let h = parse_f64(v[1], "height")?;
    let g = gcd((w * 1000.0) as i64, (h * 1000.0) as i64).max(1) as f64 / 1000.0;
    let mut out = format!(
        "{} × {} simplifies to {}:{}",
        trim_num(w),
        trim_num(h),
        trim_num(w / g),
        trim_num(h / g)
    );
    if !p.trim().is_empty() {
        let nw = parse_f64(p, "new width")?;
        out.push_str(&format!(
            "\nAt width {} the height is {}",
            trim_num(nw),
            trim_num(nw * h / w)
        ));
    }
    Ok(out)
}
pub fn gcd(mut a: i64, mut b: i64) -> i64 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a.abs()
}

/// Public wrapper used by the calculators (date differences).
pub fn days_from_civil_pub(y: i64, m: u32, d: u32) -> i64 {
    days_from_civil(y, m, d)
}
