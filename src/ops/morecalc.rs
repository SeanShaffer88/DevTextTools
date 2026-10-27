use super::convert::{days_from_civil_pub, trim_num, unix_to_string};
use super::Op;
use crate::{parse_f64, split_param};

fn op(
    id: &'static str,
    name: &'static str,
    cat: &'static str,
    desc: &'static str,
    hint: Option<&'static str>,
    run: fn(&str, &str) -> Result<String, String>,
) -> Op {
    Op {
        id,
        name,
        category: cat,
        description: desc,
        param_hint: hint,
        run,
    }
}

fn parts(i: &str, n: usize, what: &str) -> Result<Vec<f64>, String> {
    let v = split_param(i);
    if v.len() < n {
        return Err(format!("Input must be {what} (values separated by |||)"));
    }
    v.iter().take(n).map(|s| parse_f64(s, "value")).collect()
}

pub fn ops() -> Vec<Op> {
    vec![
        op("calc-circle", "Circle Calculator", "Calculators", "Input = radius. Area, circumference, diameter.", None, |i, _| { let r = parse_f64(i, "radius")?; if r < 0.0 { return Err("Radius cannot be negative".into()); } Ok(format!("Radius: {}\nDiameter: {}\nCircumference: {}\nArea: {}", trim_num(r), trim_num(2.0 * r), trim_num(2.0 * std::f64::consts::PI * r), trim_num(std::f64::consts::PI * r * r))) }),
        op("calc-triangle-area", "Triangle Area", "Calculators", "Input = base|||height.", None, |i, _| { let v = parts(i, 2, "base|||height")?; Ok(format!("Area: {}", trim_num(v[0] * v[1] / 2.0))) }),
        op("calc-rectangle", "Rectangle Calculator", "Calculators", "Input = width|||height. Area, perimeter, diagonal.", None, |i, _| { let v = parts(i, 2, "width|||height")?; Ok(format!("Area: {}\nPerimeter: {}\nDiagonal: {}", trim_num(v[0] * v[1]), trim_num(2.0 * (v[0] + v[1])), trim_num((v[0] * v[0] + v[1] * v[1]).sqrt()))) }),
        op("calc-sphere", "Sphere Calculator", "Calculators", "Input = radius. Volume and surface area.", None, |i, _| { let r = parse_f64(i, "radius")?; Ok(format!("Volume: {}\nSurface area: {}", trim_num(4.0 / 3.0 * std::f64::consts::PI * r.powi(3)), trim_num(4.0 * std::f64::consts::PI * r * r))) }),
        op("calc-cylinder", "Cylinder Calculator", "Calculators", "Input = radius|||height. Volume and total surface area.", None, |i, _| { let v = parts(i, 2, "radius|||height")?; let (r, h) = (v[0], v[1]); Ok(format!("Volume: {}\nSurface area: {}", trim_num(std::f64::consts::PI * r * r * h), trim_num(2.0 * std::f64::consts::PI * r * (r + h)))) }),
        op("calc-quadratic", "Quadratic Solver", "Calculators", "Solve a·x² + b·x + c = 0. Input = a|||b|||c.", None, |i, _| quadratic(i)),
        op("calc-two-points", "Two Points: Slope / Distance / Midpoint", "Calculators", "Input = x1|||y1|||x2|||y2.", None, |i, _| { let v = parts(i, 4, "x1|||y1|||x2|||y2")?; let (x1, y1, x2, y2) = (v[0], v[1], v[2], v[3]); let slope = if (x2 - x1).abs() < f64::EPSILON { "undefined (vertical line)".to_string() } else { trim_num((y2 - y1) / (x2 - x1)) }; Ok(format!("Slope: {slope}\nDistance: {}\nMidpoint: ({}, {})", trim_num(((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt()), trim_num((x1 + x2) / 2.0), trim_num((y1 + y2) / 2.0))) }),
        op("calc-ohms", "Ohm's Law Calculator", "Calculators", "Give any two of V=, I=, R=, P= as input like V=12|||R=4 — the other two are calculated. (V=volts, I=amps, R=ohms, P=watts)", None, |i, _| ohms(i)),
        op("calc-gpa", "GPA Calculator", "Calculators", "One course per line as grade|||credits, e.g. A|||3 or B+|||4. Letter scale A=4.0, A-=3.7, B+=3.3, B=3.0, B-=2.7, C+=2.3, C=2.0, C-=1.7, D+=1.3, D=1.0, F=0.", None, |i, _| gpa(i)),
        op("calc-pace", "Running Pace Calculator", "Calculators", "Input = finishing time as mm:ss or hh:mm:ss. Parameter = distance (in km, or add |||mi for miles).", Some("distance|||unit (km or mi)"), pace),
        op("calc-bmr", "BMR & Daily Calories (TDEE)", "Calculators", "Input = weight in kg. Parameter = sex (m/f) ||| age ||| height in cm ||| activity level: sedentary, light, moderate, active, athlete. Uses the Mifflin-St Jeor formula — a general estimate, not medical advice.", Some("m|||30|||175|||moderate"), bmr),
        op("calc-proportion", "Proportion Solver", "Calculators", "a / b = c / x — input = a|||b|||c, solves for x.", None, |i, _| { let v = parts(i, 3, "a|||b|||c")?; if v[0] == 0.0 { return Err("a cannot be zero".into()); } Ok(format!("x = {}", trim_num(v[1] * v[2] / v[0]))) }),
        op("calc-zscore", "Z-Score Calculator", "Calculators", "Input = value|||mean|||standard deviation. Reports the z-score and the approximate percent of values below it.", None, |i, _| { let v = parts(i, 3, "value|||mean|||stdev")?; if v[2] <= 0.0 { return Err("Standard deviation must be positive".into()); } let z = (v[0] - v[1]) / v[2]; Ok(format!("Z-score: {}\nApprox. percent of values below: {:.2}%", trim_num(z), norm_cdf(z) * 100.0)) }),
        op("date-add-days", "Add / Subtract Days from a Date", "Date & Time", "Input = date YYYY-MM-DD. Parameter = days to add (negative subtracts).", Some("days, e.g. 30 or -7"), |i, p| { let days = parse_date(i.trim())?; let n: i64 = p.trim().parse().map_err(|_| "Parameter must be a whole number of days".to_string())?; Ok(format!("{} {} days = {}", i.trim(), if n >= 0 { "+" } else { "-" }, unix_to_string((days + n) * 86400))) }),
        op("date-weekday", "Day of the Week for a Date", "Date & Time", "Input = date YYYY-MM-DD. Which day of the week it is/was.", None, |i, _| { let d = parse_date(i.trim())?; Ok(format!("{} is a {}", i.trim(), weekday_name(d))) }),
        op("date-week-number", "ISO Week Number", "Date & Time", "Input = date YYYY-MM-DD. Its ISO 8601 week number and week-year.", None, |i, _| { let d = parse_date(i.trim())?; let (wy, w) = iso_week(d); Ok(format!("ISO week: {wy}-W{:02}", w)) }),
        op("date-business-days", "Business Days Between Dates", "Date & Time", "Input = start date YYYY-MM-DD. Parameter = end date. Counts Monday–Friday days in between (start excluded, end included).", Some("end date YYYY-MM-DD"), |i, p| { let s = parse_date(i.trim())?; let e = parse_date(p.trim())?; if e < s { return Err("End date is before start date".into()); } let mut n = 0i64; let mut d = s; while d < e { d += 1; let wd = ((d + 4).rem_euclid(7)) as usize; if (1..=5).contains(&wd) { n += 1; } } Ok(format!("Business days: {n}\nCalendar days: {}", e - s)) }),
        op("date-days-in-month", "Days in a Month", "Date & Time", "Input = YYYY-MM. How many days that month has (leap years handled).", None, |i, _| { let t = i.trim(); let (y, m) = t.split_once('-').ok_or("Use YYYY-MM")?; let y: i64 = y.parse().map_err(|_| "Bad year".to_string())?; let m: u32 = m.parse().map_err(|_| "Bad month".to_string())?; if !(1..=12).contains(&m) { return Err("Month must be 1-12".into()); } let first = days_from_civil_pub(y, m, 1); let next = if m == 12 { days_from_civil_pub(y + 1, 1, 1) } else { days_from_civil_pub(y, m + 1, 1) }; Ok(format!("{t} has {} days", next - first)) }),
    ]
}

pub fn parse_date(s: &str) -> Result<i64, String> {
    let dp: Vec<i64> = s
        .split('-')
        .map(|x| x.parse())
        .collect::<Result<_, _>>()
        .map_err(|_| "Dates must be YYYY-MM-DD".to_string())?;
    if dp.len() != 3 {
        return Err("Dates must be YYYY-MM-DD".into());
    }
    Ok(days_from_civil_pub(dp[0], dp[1] as u32, dp[2] as u32))
}
/// 0 = Sunday … 6 = Saturday (1970-01-01, day 0, was a Thursday = 4).
pub fn weekday_name(days: i64) -> &'static str {
    const NAMES: [&str; 7] = [
        "Sunday",
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
    ];
    NAMES[((days + 4).rem_euclid(7)) as usize]
}
fn iso_week(days: i64) -> (i64, i64) {
    // ISO: week 1 contains the first Thursday; weekday Mon=1..Sun=7
    let iso_wd = (days + 3).rem_euclid(7) + 1; // Mon=1
    let thursday = days + (4 - iso_wd);
    // year of that Thursday
    let (y, _m, _d) = civil(thursday);
    let jan4 = days_from_civil_pub(y, 1, 4);
    let jan4_wd = (jan4 + 3).rem_euclid(7) + 1;
    let week1_monday = jan4 - (jan4_wd - 1);
    let week = (thursday - week1_monday) / 7 + 1;
    (y, week)
}
fn civil(days: i64) -> (i64, u32, u32) {
    // inverse of days_from_civil (Hinnant)
    let z = days + 719468;
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
fn quadratic(i: &str) -> Result<String, String> {
    let v = parts(i, 3, "a|||b|||c")?;
    let (a, b, c) = (v[0], v[1], v[2]);
    if a == 0.0 {
        return Err("a cannot be zero (that is not a quadratic)".into());
    }
    let disc = b * b - 4.0 * a * c;
    if disc < 0.0 {
        Ok(format!(
            "No real roots (the discriminant is {}).\nComplex roots: {} ± {}i",
            trim_num(disc),
            trim_num(-b / (2.0 * a)),
            trim_num((-disc).sqrt() / (2.0 * a.abs()))
        ))
    } else if disc == 0.0 {
        Ok(format!(
            "One repeated root: x = {}",
            trim_num(-b / (2.0 * a))
        ))
    } else {
        let s = disc.sqrt();
        Ok(format!(
            "Two roots:\nx1 = {}\nx2 = {}",
            trim_num((-b + s) / (2.0 * a)),
            trim_num((-b - s) / (2.0 * a))
        ))
    }
}
fn ohms(i: &str) -> Result<String, String> {
    let (mut v_, mut i_, mut r_, mut p_) = (None, None, None, None);
    for pair in split_param(i) {
        let (k, val) = pair.split_once('=').ok_or("Use the form V=12|||R=4")?;
        let x = parse_f64(val, "value")?;
        match k.trim().to_uppercase().as_str() {
            "V" => v_ = Some(x),
            "I" => i_ = Some(x),
            "R" => r_ = Some(x),
            "P" => p_ = Some(x),
            other => return Err(format!("Unknown quantity '{other}' — use V, I, R or P")),
        }
    }
    let known = [v_, i_, r_, p_].iter().filter(|x| x.is_some()).count();
    if known != 2 {
        return Err("Give exactly two of V=, I=, R=, P=".into());
    }
    let (v, cur, r, p) = match (v_, i_, r_, p_) {
        (Some(v), Some(i), None, None) => (v, i, v / i, v * i),
        (Some(v), None, Some(r), None) => (v, v / r, r, v * v / r),
        (Some(v), None, None, Some(p)) => (v, p / v, v * v / p, p),
        (None, Some(i), Some(r), None) => (i * r, i, r, i * i * r),
        (None, Some(i), None, Some(p)) => (p / i, i, p / (i * i), p),
        (None, None, Some(r), Some(p)) => ((p * r).sqrt(), (p / r).sqrt(), r, p),
        _ => unreachable!(),
    };
    if !(v.is_finite() && cur.is_finite() && r.is_finite() && p.is_finite()) {
        return Err("Those values lead to a division by zero".into());
    }
    Ok(format!("V (voltage) = {} volts\nI (current) = {} amps\nR (resistance) = {} ohms\nP (power) = {} watts", trim_num(v), trim_num(cur), trim_num(r), trim_num(p)))
}
fn gpa(i: &str) -> Result<String, String> {
    let grade_pts = |g: &str| -> Option<f64> {
        Some(match g.trim().to_uppercase().as_str() {
            "A+" | "A" => 4.0,
            "A-" => 3.7,
            "B+" => 3.3,
            "B" => 3.0,
            "B-" => 2.7,
            "C+" => 2.3,
            "C" => 2.0,
            "C-" => 1.7,
            "D+" => 1.3,
            "D" => 1.0,
            "D-" => 0.7,
            "F" => 0.0,
            _ => return None,
        })
    };
    let (mut pts, mut credits) = (0.0, 0.0);
    for line in i.lines().filter(|l| !l.trim().is_empty()) {
        let v = split_param(line);
        if v.len() < 2 {
            return Err(format!(
                "Each line must be grade|||credits — problem line: '{line}'"
            ));
        }
        let g = grade_pts(v[0]).ok_or(format!("Unknown grade '{}'", v[0]))?;
        let c = parse_f64(v[1], "credits")?;
        pts += g * c;
        credits += c;
    }
    if credits == 0.0 {
        return Err("Total credits are zero".into());
    }
    Ok(format!(
        "GPA: {:.2}\nTotal credits: {}",
        pts / credits,
        trim_num(credits)
    ))
}
fn pace(i: &str, p: &str) -> Result<String, String> {
    let t: Vec<f64> = i
        .trim()
        .split(':')
        .map(|x| x.parse())
        .collect::<Result<_, _>>()
        .map_err(|_| "Time must be mm:ss or hh:mm:ss".to_string())?;
    let total_secs = match t.len() {
        2 => t[0] * 60.0 + t[1],
        3 => t[0] * 3600.0 + t[1] * 60.0 + t[2],
        _ => return Err("Time must be mm:ss or hh:mm:ss".into()),
    };
    let v = split_param(p);
    if v.is_empty() {
        return Err("Parameter must be: distance (add |||mi for miles)".into());
    }
    let dist = parse_f64(v[0], "distance")?;
    if dist <= 0.0 {
        return Err("Distance must be positive".into());
    }
    let unit = if v
        .get(1)
        .map(|s| s.trim().eq_ignore_ascii_case("mi"))
        .unwrap_or(false)
    {
        "mile"
    } else {
        "km"
    };
    let per = total_secs / dist;
    let speed = dist / (total_secs / 3600.0);
    let secs_part = format!("{:04.1}", per % 60.0);
    Ok(format!(
        "Pace: {}:{secs_part} per {}\nSpeed: {} {} per hour",
        (per / 60.0).floor() as i64,
        unit,
        trim_num(speed),
        unit
    ))
}
fn bmr(i: &str, p: &str) -> Result<String, String> {
    let w = parse_f64(i, "weight kg")?;
    let v = split_param(p);
    if v.len() < 4 {
        return Err("Parameter must be: sex (m/f) ||| age ||| height cm ||| activity".into());
    }
    let sex = v[0].trim().to_lowercase();
    let age = parse_f64(v[1], "age")?;
    let h = parse_f64(v[2], "height cm")?;
    let factor = match v[3].trim().to_lowercase().as_str() {
        "sedentary" => 1.2,
        "light" => 1.375,
        "moderate" => 1.55,
        "active" => 1.725,
        "athlete" => 1.9,
        other => {
            return Err(format!(
                "Unknown activity '{other}' — use sedentary, light, moderate, active or athlete"
            ))
        }
    };
    let base = 10.0 * w + 6.25 * h - 5.0 * age;
    let bmr = if sex.starts_with('m') {
        base + 5.0
    } else if sex.starts_with('f') {
        base - 161.0
    } else {
        return Err("Sex must be m or f for this formula".into());
    };
    Ok(format!(
        "BMR (calories your body burns at complete rest): ~{} kcal/day\nTDEE (with {} activity): ~{} kcal/day\n\nGeneral estimate only, not medical advice.",
        trim_num(bmr),
        v[3].trim(),
        trim_num(bmr * factor)
    ))
}
fn norm_cdf(z: f64) -> f64 {
    // Abramowitz & Stegun approximation
    let t = 1.0 / (1.0 + 0.2316419 * z.abs());
    let poly = t
        * (0.319381530
            + t * (-0.356563782 + t * (1.781477937 + t * (-1.821255978 + t * 1.330274429))));
    let p = 1.0 - (1.0 / (2.0 * std::f64::consts::PI).sqrt()) * (-z * z / 2.0).exp() * poly;
    if z >= 0.0 {
        p
    } else {
        1.0 - p
    }
}
