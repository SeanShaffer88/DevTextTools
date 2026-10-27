pub mod calc;
pub mod convert;
pub mod data2;
pub mod dev;
pub mod devpack;
pub mod devpack2;
pub mod encode;
pub mod fancy;
pub mod files;
pub mod hashgen;
pub mod morecalc;
pub mod text;

#[derive(Clone, Copy)]
pub struct Op {
    pub id: &'static str,
    pub name: &'static str,
    pub category: &'static str,
    pub description: &'static str,
    /// Shown next to the Parameter box, e.g. "find|||replace". None = no param needed.
    pub param_hint: Option<&'static str>,
    pub run: fn(&str, &str) -> Result<String, String>,
}

pub fn all_ops() -> Vec<Op> {
    let mut v = Vec::new();
    v.extend(text::ops());
    v.extend(encode::ops());
    v.extend(dev::ops());
    v.extend(devpack::ops());
    v.extend(devpack2::ops());
    v.extend(hashgen::ops());
    v.extend(convert::ops());
    v.extend(calc::ops());
    v.extend(fancy::ops());
    v.extend(data2::ops());
    v.extend(morecalc::ops());
    v.extend(files::ops());
    v
}

pub fn categories() -> Vec<&'static str> {
    let mut out: Vec<&'static str> = Vec::new();
    for op in all_ops() {
        if !out.contains(&op.category) {
            out.push(op.category);
        }
    }
    out
}

pub fn find_op(id: &str) -> Option<Op> {
    all_ops().into_iter().find(|o| o.id == id)
}

pub fn run_op(id: &str, input: &str, param: &str) -> Result<String, String> {
    match find_op(id) {
        Some(op) => (op.run)(input, param),
        None => Err(format!(
            "Unknown tool '{id}'. Run with --list to see all tools."
        )),
    }
}
