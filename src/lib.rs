//! DevTextTools core library.
//! Every tool is an *operation*: a pure function (input, param) -> output.
//! The GUI, the CLI and tests all dispatch through the same registry —
//! there are no GUI-only features.

// Guard-clause style: several ops validate with consecutive `if { return Err }`
// checks on one line, which trips clippy::possible_missing_else even though no
// else is logically missing (each guard returns). Allowed once, here, on purpose.
#![allow(clippy::possible_missing_else)]

pub mod ops;

pub use ops::{all_ops, find_op, run_op, Op};

/// Simple deterministic-when-seeded RNG (xorshift64*) so tools that need
/// randomness work offline with no extra dependencies. Seed 0 is remapped.
pub struct Rng(pub u64);
impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(if seed == 0 { 0x9E3779B97F4A7C15 } else { seed })
    }
    pub fn from_time() -> Self {
        let n = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x2545F4914F6CDD1D);
        Rng::new(n ^ (std::process::id() as u64) << 32)
    }
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }
    /// Value in [0, n). n must be > 0.
    pub fn below(&mut self, n: u64) -> u64 {
        if n == 0 {
            return 0;
        }
        self.next_u64() % n
    }
    pub fn f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// Split a param field on `|||` — the one multi-value convention for all ops.
pub fn split_param(param: &str) -> Vec<&str> {
    if param.is_empty() {
        return Vec::new();
    }
    param.split("|||").collect()
}

pub fn parse_f64(s: &str, what: &str) -> Result<f64, String> {
    s.trim()
        .parse::<f64>()
        .map_err(|_| format!("'{s}' is not a valid number for {what}"))
}

pub fn numbers_from(input: &str) -> Result<Vec<f64>, String> {
    let mut out = Vec::new();
    for tok in input.split(|c: char| c.is_whitespace() || c == ',' || c == ';') {
        if tok.is_empty() {
            continue;
        }
        out.push(
            tok.parse::<f64>()
                .map_err(|_| format!("'{tok}' is not a number"))?,
        );
    }
    if out.is_empty() {
        return Err("Enter at least one number.".into());
    }
    Ok(out)
}

/// Format a duration in seconds as HH:MM:SS (used by the Timers view and
/// available to every surface).
pub fn fmt_clock(total_secs: u64) -> String {
    let h = total_secs / 3600;
    let m = (total_secs % 3600) / 60;
    let s = total_secs % 60;
    format!("{h:02}:{m:02}:{s:02}")
}

pub mod pdf;
