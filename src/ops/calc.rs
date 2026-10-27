use super::convert::{gcd, trim_num, unix_to_string};
use super::Op;
use crate::{numbers_from, parse_f64, split_param};

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
        category: "Calculators",
        description: desc,
        param_hint: hint,
        run,
    }
}

pub fn ops() -> Vec<Op> {
    vec![
        op("calc-expression", "Calculator (Expression)", "Evaluate maths like 2*(3+4)^2, sqrt(16), sin(0.5), pi, e. Functions: sqrt, abs, sin, cos, tan, log, ln, exp, floor, ceil, round.", None, |i, _| { let v = eval_expr(i.trim())?; Ok(trim_num(v)) }),
        op("calc-percentage", "Percentage Calculator", "Input = part, Parameter = whole: what percent the part is of the whole, and the reverse.", Some("whole"), |i, p| { let part = parse_f64(i, "part")?; let whole = parse_f64(p, "whole")?; if whole == 0.0 || part == 0.0 { return Err("Neither value can be zero".into()); } Ok(format!("{} is {}% of {}\n{} is {}% of {}", trim_num(part), trim_num(part / whole * 100.0), trim_num(whole), trim_num(whole), trim_num(whole / part * 100.0), trim_num(part))) }),
        op("calc-percent-change", "Percentage Change", "Input = old value, Parameter = new value.", Some("new value"), |i, p| { let old = parse_f64(i, "old value")?; let new = parse_f64(p, "new value")?; if old == 0.0 { return Err("Old value cannot be zero".into()); } let ch = (new - old) / old.abs() * 100.0; Ok(format!("Change: {} ({}%) — {}", trim_num(new - old), trim_num(ch), if ch >= 0.0 { "increase" } else { "decrease" })) }),
        op("calc-stats", "Statistics (list of numbers)", "Mean, median, mode, min, max, range, sum, standard deviation, variance for a list of numbers.", None, |i, _| stats(i)),
        op("calc-average", "Average / Median / Sum", "Quick summary for a list of numbers.", None, |i, _| { let v = numbers_from(i)?; let sum: f64 = v.iter().sum(); Ok(format!("Count: {}\nSum: {}\nAverage: {}", v.len(), trim_num(sum), trim_num(sum / v.len() as f64))) }),
        op("calc-factorial", "Factorial", "n! for n from 0 to 170 (as a float beyond 20).", None, |i, _| { let n: u64 = i.trim().parse().map_err(|_| "Enter a whole number 0–170".to_string())?; if n > 170 { return Err("Max 170".into()); } if n <= 20 { let mut r: u128 = 1; for k in 2..=n as u128 { r *= k; } Ok(format!("{n}! = {r}")) } else { let mut r = 1.0f64; for k in 2..=n { r *= k as f64; } Ok(format!("{n}! ≈ {r:e}")) } }),
        op("calc-fibonacci", "Fibonacci Number", "The nth Fibonacci number (n = 0,1,2…). Exact up to n=93, approximate after.", None, |i, _| { let n: u64 = i.trim().parse().map_err(|_| "Enter a whole number".to_string())?; if n <= 93 { let (mut a, mut b) = (0u128, 1u128); for _ in 0..n { let t = a + b; a = b; b = t; } Ok(format!("Fibonacci({n}) = {a}")) } else if n <= 10000 { let phi = (1.0 + 5f64.sqrt()) / 2.0; Ok(format!("Fibonacci({n}) ≈ {:.6e} (Binet approximation)", phi.powi(n as i32) / 5f64.sqrt())) } else { Err("Max n = 10,000".into()) } }),
        op("calc-gcd-lcm", "GCD & LCM (list)", "Greatest common divisor and least common multiple of a list of whole numbers.", None, |i, _| { let v = numbers_from(i)?; let ints: Vec<i64> = v.iter().map(|x| *x as i64).collect(); if ints.iter().zip(v.iter()).any(|(a, b)| *a as f64 != *b) { return Err("All values must be whole numbers".into()); } let mut g = ints[0]; for x in &ints[1..] { g = gcd(g, *x); } let mut l = ints[0]; for x in &ints[1..] { l = l / gcd(l, *x) * x; } Ok(format!("GCD: {g}\nLCM: {l}")) }),
        op("calc-primes", "Prime Checker / List Primes", "Input a number to test if it is prime; or two numbers as start|||end via Parameter to list primes in range? Parameter form: use input 'start|||end' with the ||| separator in the INPUT box.", None, |i, _| primes(i)),
        op("calc-tip", "Tip & Split Bill", "Input = bill amount, Parameter = tip % ||| people (defaults 15, 1).", Some("tip%|||people"), |i, p| { let bill = parse_f64(i, "bill")?; let v = split_param(p); let tip_pct: f64 = v.first().and_then(|s| s.trim().parse().ok()).unwrap_or(15.0); let people: f64 = v.get(1).and_then(|s| s.trim().parse().ok()).unwrap_or(1.0); if people < 1.0 { return Err("People must be at least 1".into()); } let tip = bill * tip_pct / 100.0; let total = bill + tip; Ok(format!("Bill: {}\nTip ({}%): {}\nTotal: {}\nPer person: {}", trim_num(bill), trim_num(tip_pct), trim_num(tip), trim_num(total), trim_num(total / people))) }),
        op("calc-bmi", "BMI Calculator", "Input = weight in kg, Parameter = height in cm (or m if under 3).", Some("height in cm"), |i, p| { let w = parse_f64(i, "weight kg")?; let mut h = parse_f64(p, "height")?; if h > 3.0 { h /= 100.0; } if h <= 0.0 { return Err("Height must be positive".into()); } let bmi = w / (h * h); let cat = if bmi < 18.5 { "Underweight" } else if bmi < 25.0 { "Healthy range" } else if bmi < 30.0 { "Overweight" } else { "Obese range" }; Ok(format!("BMI: {:.1} — {cat}\n(General adult guide only, not medical advice.)", bmi)) }),
        op("calc-age", "Age / Date Difference", "Input = start date YYYY-MM-DD, Parameter = end date YYYY-MM-DD (default: today UTC). Reports days, weeks, years/months/days.", Some("end date (optional)"), |i, p| age(i.trim(), p.trim())),
        op("calc-subnet", "IPv4 Subnet Calculator", "Input like 192.168.1.10/24 — network, broadcast, first/last host, host count, netmask, wildcard.", None, |i, _| subnet(i.trim())),
        op("calc-discount", "Discount & Sale Price", "Input = original price, Parameter = discount %.", Some("discount %"), |i, p| { let price = parse_f64(i, "price")?; let d = parse_f64(p, "discount %")?; let save = price * d / 100.0; Ok(format!("You save: {}\nYou pay: {} ({}% off)", trim_num(save), trim_num(price - save), trim_num(d))) }),
        op("calc-loan", "Loan / Mortgage Payment", "Input = amount borrowed, Parameter = annual interest % ||| years. Monthly payment + totals (standard amortisation formula).", Some("rate%|||years"), |i, p| { let v = split_param(p); if v.len() < 2 { return Err("Parameter must be: rate%|||years".into()); } let principal = parse_f64(i, "amount")?; let rate: f64 = v[0].trim().parse().map_err(|_| "Bad rate".to_string())?; let years: f64 = v[1].trim().parse().map_err(|_| "Bad years".to_string())?; let n = years * 12.0; let r = rate / 100.0 / 12.0; let pay = if r == 0.0 { principal / n } else { principal * r / (1.0 - (1.0 + r).powf(-n)) }; Ok(format!("Monthly payment: {:.2}\nTotal paid: {:.2}\nTotal interest: {:.2}\nPayments: {}", pay, pay * n, pay * n - principal, n as i64)) }),
        op("calc-compound", "Compound Interest", "Input = starting amount, Parameter = annual rate % ||| years ||| compounds per year (default 12).", Some("rate%|||years|||per-year"), |i, p| { let v = split_param(p); if v.len() < 2 { return Err("Parameter must be: rate%|||years".into()); } let principal = parse_f64(i, "amount")?; let rate: f64 = v[0].trim().parse().map_err(|_| "Bad rate".to_string())?; let years: f64 = v[1].trim().parse().map_err(|_| "Bad years".to_string())?; let per: f64 = v.get(2).and_then(|s| s.trim().parse().ok()).unwrap_or(12.0); let total = principal * (1.0 + rate / 100.0 / per).powf(per * years); Ok(format!("Final amount: {:.2}\nInterest earned: {:.2}", total, total - principal)) }),
    ]
}

// ---------- expression evaluator (recursive descent) ----------
pub fn eval_expr(s: &str) -> Result<f64, String> {
    let mut p = Parser {
        chars: s.chars().collect(),
        pos: 0,
    };
    let v = p.expr()?;
    p.skip_ws();
    if p.pos != p.chars.len() {
        return Err(format!("Unexpected character at position {}", p.pos + 1));
    }
    Ok(v)
}
struct Parser {
    chars: Vec<char>,
    pos: usize,
}
impl Parser {
    fn skip_ws(&mut self) {
        while self.pos < self.chars.len() && self.chars[self.pos].is_whitespace() {
            self.pos += 1;
        }
    }
    fn peek(&mut self) -> Option<char> {
        self.skip_ws();
        self.chars.get(self.pos).copied()
    }
    fn expr(&mut self) -> Result<f64, String> {
        let mut v = self.term()?;
        loop {
            match self.peek() {
                Some('+') => {
                    self.pos += 1;
                    v += self.term()?;
                }
                Some('-') => {
                    self.pos += 1;
                    v -= self.term()?;
                }
                _ => break,
            }
        }
        Ok(v)
    }
    fn term(&mut self) -> Result<f64, String> {
        let mut v = self.power()?;
        loop {
            match self.peek() {
                Some('*') => {
                    self.pos += 1;
                    v *= self.power()?;
                }
                Some('/') => {
                    self.pos += 1;
                    let d = self.power()?;
                    if d == 0.0 {
                        return Err("Division by zero".into());
                    }
                    v /= d;
                }
                Some('%') => {
                    self.pos += 1;
                    v %= self.power()?;
                }
                _ => break,
            }
        }
        Ok(v)
    }
    fn power(&mut self) -> Result<f64, String> {
        let base = self.unary()?;
        if self.peek() == Some('^') {
            self.pos += 1;
            let e = self.power()?;
            return Ok(base.powf(e));
        }
        Ok(base)
    }
    fn unary(&mut self) -> Result<f64, String> {
        if self.peek() == Some('-') {
            self.pos += 1;
            return Ok(-self.unary()?);
        }
        if self.peek() == Some('+') {
            self.pos += 1;
            return self.unary();
        }
        self.atom()
    }
    fn atom(&mut self) -> Result<f64, String> {
        self.skip_ws();
        match self.chars.get(self.pos).copied() {
            Some('(') => {
                self.pos += 1;
                let v = self.expr()?;
                if self.peek() != Some(')') {
                    return Err("Missing closing )".into());
                }
                self.pos += 1;
                Ok(v)
            }
            Some(c) if c.is_ascii_digit() || c == '.' => {
                let start = self.pos;
                while self.pos < self.chars.len()
                    && (self.chars[self.pos].is_ascii_digit() || self.chars[self.pos] == '.')
                {
                    self.pos += 1;
                }
                let s: String = self.chars[start..self.pos].iter().collect();
                s.parse::<f64>().map_err(|_| format!("Bad number '{s}'"))
            }
            Some(c) if c.is_alphabetic() => {
                let start = self.pos;
                while self.pos < self.chars.len() && self.chars[self.pos].is_alphabetic() {
                    self.pos += 1;
                }
                let name: String = self.chars[start..self.pos]
                    .iter()
                    .collect::<String>()
                    .to_lowercase();
                match name.as_str() {
                    "pi" => return Ok(std::f64::consts::PI),
                    "e" => return Ok(std::f64::consts::E),
                    "tau" => return Ok(std::f64::consts::TAU),
                    _ => {}
                }
                if self.peek() != Some('(') {
                    return Err(format!(
                        "Unknown name '{name}' — use pi, e, or a function like sqrt(…)"
                    ));
                }
                self.pos += 1;
                let arg = self.expr()?;
                if self.peek() != Some(')') {
                    return Err("Missing closing )".into());
                }
                self.pos += 1;
                match name.as_str() {
                    "sqrt" => {
                        if arg < 0.0 {
                            return Err("sqrt of a negative number".into());
                        }
                        Ok(arg.sqrt())
                    }
                    "abs" => Ok(arg.abs()),
                    "sin" => Ok(arg.sin()),
                    "cos" => Ok(arg.cos()),
                    "tan" => Ok(arg.tan()),
                    "log" => {
                        if arg <= 0.0 {
                            return Err("log of a non-positive number".into());
                        }
                        Ok(arg.log10())
                    }
                    "ln" => {
                        if arg <= 0.0 {
                            return Err("ln of a non-positive number".into());
                        }
                        Ok(arg.ln())
                    }
                    "exp" => Ok(arg.exp()),
                    "floor" => Ok(arg.floor()),
                    "ceil" => Ok(arg.ceil()),
                    "round" => Ok(arg.round()),
                    _ => Err(format!("Unknown function '{name}'")),
                }
            }
            other => Err(format!(
                "Unexpected {}",
                other
                    .map(|c| format!("'{c}'"))
                    .unwrap_or_else(|| "end of input".into())
            )),
        }
    }
}

fn stats(i: &str) -> Result<String, String> {
    let mut v = numbers_from(i)?;
    let n = v.len() as f64;
    let sum: f64 = v.iter().sum();
    let mean = sum / n;
    let mut sorted = v.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = if sorted.len() % 2 == 1 {
        sorted[sorted.len() / 2]
    } else {
        (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0
    };
    let var = v.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
    // mode
    let mut counts: Vec<(i64, usize)> = Vec::new();
    for x in &v {
        let key = (x * 1e9) as i64;
        if let Some(e) = counts.iter_mut().find(|e| e.0 == key) {
            e.1 += 1;
        } else {
            counts.push((key, 1));
        }
    }
    counts.sort_by_key(|e| std::cmp::Reverse(e.1));
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    Ok(format!("Count: {}\nSum: {}\nMean (average): {}\nMedian: {}\nMode: {} ({} times)\nMin: {}\nMax: {}\nRange: {}\nVariance (population): {}\nStandard deviation: {}", sorted.len(), trim_num(sum), trim_num(mean), trim_num(median), trim_num(counts[0].0 as f64 / 1e9), counts[0].1, trim_num(sorted[0]), trim_num(sorted[sorted.len() - 1]), trim_num(sorted[sorted.len() - 1] - sorted[0]), trim_num(var), trim_num(var.sqrt())))
}
fn primes(i: &str) -> Result<String, String> {
    let is_prime = |n: u64| -> bool {
        if n < 2 {
            return false;
        }
        let mut d = 2u64;
        while d * d <= n {
            if n % d == 0 {
                return false;
            }
            d += 1;
        }
        true
    };
    if let Some((a, b)) = i.split_once("|||") {
        let start: u64 = a.trim().parse().map_err(|_| "Bad start".to_string())?;
        let end: u64 = b.trim().parse().map_err(|_| "Bad end".to_string())?;
        if end < start || end - start > 1_000_000 {
            return Err("Range must be increasing and at most 1,000,000 wide".into());
        }
        let list: Vec<String> = (start..=end)
            .filter(|n| is_prime(*n))
            .map(|n| n.to_string())
            .collect();
        return Ok(format!(
            "{} primes between {start} and {end}:\n{}",
            list.len(),
            list.join(", ")
        ));
    }
    let n: u64 = i
        .trim()
        .parse()
        .map_err(|_| "Enter a whole number, or start|||end".to_string())?;
    if is_prime(n) {
        Ok(format!("{n} IS prime"))
    } else if n < 2 {
        Ok(format!("{n} is not prime (primes start at 2)"))
    } else {
        let d = (2..).find(|d| n % d == 0).unwrap();
        Ok(format!("{n} is NOT prime — divisible by {d}"))
    }
}
fn parse_date(s: &str) -> Result<i64, String> {
    let dp: Vec<i64> = s
        .split('-')
        .map(|x| x.parse())
        .collect::<Result<_, _>>()
        .map_err(|_| "Dates must be YYYY-MM-DD".to_string())?;
    if dp.len() != 3 {
        return Err("Dates must be YYYY-MM-DD".into());
    }
    // days_from_civil is in convert; recompute via unix conversion round trip is circular — duplicate small impl here using the same algorithm through a helper import:
    Ok(super::convert::days_from_civil_pub(
        dp[0],
        dp[1] as u32,
        dp[2] as u32,
    ))
}
fn age(start_s: &str, end_s: &str) -> Result<String, String> {
    let start = parse_date(start_s)?;
    let end = if end_s.is_empty() {
        // today UTC from system time
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| "clock error".to_string())?
            .as_secs() as i64;
        now.div_euclid(86400)
    } else {
        parse_date(end_s)?
    };
    let days = end - start;
    if days < 0 {
        return Err("End date is before start date".into());
    }
    Ok(format!("From {} to {}:\n{} days\n{} weeks and {} days\n~{} years, ~{} months (approximate)\nEnd date as timestamp check: {}", start_s, if end_s.is_empty() { unix_to_string(end * 86400) } else { end_s.to_string() }, days, days / 7, days % 7, days / 365, days / 30, end * 86400))
}
fn subnet(i: &str) -> Result<String, String> {
    let (ip_s, cidr_s) = i.split_once('/').ok_or("Use the form 192.168.1.10/24")?;
    let prefix: u32 = cidr_s
        .trim()
        .parse()
        .map_err(|_| "Bad prefix length".to_string())?;
    if prefix > 32 {
        return Err("Prefix must be 0–32".into());
    }
    let octets: Vec<u32> = ip_s
        .trim()
        .split('.')
        .map(|x| x.parse())
        .collect::<Result<_, _>>()
        .map_err(|_| "Bad IPv4 address".to_string())?;
    if octets.len() != 4 || octets.iter().any(|o| *o > 255) {
        return Err("Bad IPv4 address".into());
    }
    let ip = (octets[0] << 24) | (octets[1] << 16) | (octets[2] << 8) | octets[3];
    let mask: u32 = if prefix == 0 {
        0
    } else {
        u32::MAX << (32 - prefix)
    };
    let network = ip & mask;
    let broadcast = network | !mask;
    let hosts: u64 = if prefix >= 31 {
        0
    } else {
        (1u64 << (32 - prefix)) - 2
    };
    let fmt = |x: u32| {
        format!(
            "{}.{}.{}.{}",
            x >> 24,
            (x >> 16) & 255,
            (x >> 8) & 255,
            x & 255
        )
    };
    Ok(format!("Address: {}\nNetmask: {} (/{})\nWildcard: {}\nNetwork: {}\nBroadcast: {}\nFirst host: {}\nLast host: {}\nUsable hosts: {}", fmt(ip), fmt(mask), prefix, fmt(!mask), fmt(network), fmt(broadcast), fmt(if hosts > 0 { network + 1 } else { network }), fmt(if hosts > 0 { broadcast - 1 } else { broadcast }), hosts))
}
