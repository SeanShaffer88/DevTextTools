#![cfg_attr(windows, windows_subsystem = "windows")]
//! DevTextTools — GUI shell + CLI. All real work lives in the library's
//! operation registry, dispatched identically here, so there are no
//! GUI-only features.

use devtexttools::{all_ops, fmt_clock, run_op, Op};
use std::io::Read;
use std::time::{Duration, Instant};

const GREEN: egui::Color32 = egui::Color32::from_rgb(74, 222, 128);
const GREEN_DIM: egui::Color32 = egui::Color32::from_rgb(18, 42, 27);
const REDT: egui::Color32 = egui::Color32::from_rgb(248, 113, 113);
const LINE: egui::Color32 = egui::Color32::from_rgb(38, 38, 38);

/// Format a Unix timestamp as "YYYY-MM-DD HH:MM UTC" (history rail).
fn fmt_ts(ts: i64) -> String {
    let days = ts.div_euclid(86_400);
    let secs = ts.rem_euclid(86_400);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let mut y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    if m <= 2 {
        y += 1;
    }
    format!(
        "{y:04}-{m:02}-{d:02} {:02}:{:02} UTC",
        secs / 3600,
        (secs % 3600) / 60
    )
}

impl App {
    fn timers_ui(&mut self, ui: &mut egui::Ui) {
        let any_running = self.timers.sw_since.is_some()
            || self.timers.cd_since.is_some()
            || self.timers.pomo_since.is_some();
        if any_running {
            ui.ctx().request_repaint_after(Duration::from_millis(100));
        }
        let big = |ui: &mut egui::Ui, s: String| {
            ui.label(egui::RichText::new(s).size(40.0).strong());
        };
        ui.heading("Stopwatch");
        let sw = self.timers.sw_elapsed
            + self
                .timers
                .sw_since
                .map(|t| t.elapsed())
                .unwrap_or_default();
        big(ui, fmt_clock(sw.as_secs()));
        ui.horizontal(|ui| {
            if self.timers.sw_since.is_some() {
                if ui.button("Pause").clicked() {
                    self.timers.sw_elapsed = sw;
                    self.timers.sw_since = None;
                }
            } else if ui.button("Start").clicked() {
                self.timers.sw_since = Some(Instant::now());
            }
            if ui.button("Reset").clicked() {
                self.timers.sw_since = None;
                self.timers.sw_elapsed = Duration::ZERO;
            }
        });
        ui.add_space(16.0);
        ui.separator();
        ui.heading("Countdown");
        ui.horizontal(|ui| {
            ui.label("Minutes:");
            ui.add(egui::TextEdit::singleline(&mut self.timers.cd_minutes).desired_width(60.0));
            if ui.button("Set").clicked() {
                if let Ok(m) = self.timers.cd_minutes.trim().parse::<f64>() {
                    if m > 0.0 {
                        self.timers.cd_total = Duration::from_secs_f64(m * 60.0);
                        self.timers.cd_remaining = self.timers.cd_total;
                        self.timers.cd_since = None;
                        self.timers.cd_done = false;
                    }
                }
            }
        });
        if let Some(since) = self.timers.cd_since {
            let elapsed = since.elapsed();
            if elapsed >= self.timers.cd_remaining {
                self.timers.cd_remaining = Duration::ZERO;
                self.timers.cd_since = None;
                self.timers.cd_done = true;
            }
        }
        let cd_left = if let Some(since) = self.timers.cd_since {
            self.timers.cd_remaining.saturating_sub(since.elapsed())
        } else {
            self.timers.cd_remaining
        };
        big(ui, fmt_clock(cd_left.as_secs()));
        if self.timers.cd_done {
            ui.colored_label(egui::Color32::from_rgb(0xE5, 0x48, 0x4D), "Time is up!");
        }
        ui.horizontal(|ui| {
            if self.timers.cd_since.is_some() {
                if ui.button("Pause").clicked() {
                    self.timers.cd_remaining = cd_left;
                    self.timers.cd_since = None;
                }
            } else if ui.button("Start").clicked() && cd_left > Duration::ZERO {
                self.timers.cd_since = Some(Instant::now());
                self.timers.cd_done = false;
            }
            if ui.button("Reset").clicked() {
                self.timers.cd_since = None;
                self.timers.cd_remaining = self.timers.cd_total;
                self.timers.cd_done = false;
            }
        });
        ui.add_space(16.0);
        ui.separator();
        ui.heading("Pomodoro (25 minutes work / 5 minutes break)");
        if let Some(since) = self.timers.pomo_since {
            if since.elapsed() >= self.timers.pomo_remaining {
                if self.timers.pomo_work {
                    self.timers.pomo_cycles += 1;
                    self.timers.pomo_work = false;
                    self.timers.pomo_remaining = Duration::from_secs(5 * 60);
                } else {
                    self.timers.pomo_work = true;
                    self.timers.pomo_remaining = Duration::from_secs(25 * 60);
                }
                self.timers.pomo_since = Some(Instant::now());
            }
        }
        let pomo_left = if let Some(since) = self.timers.pomo_since {
            self.timers.pomo_remaining.saturating_sub(since.elapsed())
        } else {
            self.timers.pomo_remaining
        };
        ui.label(if self.timers.pomo_work {
            "Focus time"
        } else {
            "Break time"
        });
        big(ui, fmt_clock(pomo_left.as_secs()));
        ui.label(format!(
            "Completed work sessions: {}",
            self.timers.pomo_cycles
        ));
        ui.horizontal(|ui| {
            if self.timers.pomo_since.is_some() {
                if ui.button("Pause").clicked() {
                    self.timers.pomo_remaining = pomo_left;
                    self.timers.pomo_since = None;
                }
            } else if ui.button("Start").clicked() {
                self.timers.pomo_since = Some(Instant::now());
            }
            if ui.button("Reset").clicked() {
                self.timers.pomo_since = None;
                self.timers.pomo_work = true;
                self.timers.pomo_remaining = Duration::from_secs(25 * 60);
                self.timers.pomo_cycles = 0;
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if !args.is_empty() {
        std::process::exit(cli(&args));
    }
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("DevTextTools — every text & dev tool, offline")
            .with_icon(std::sync::Arc::new(egui::IconData {
                rgba: include_bytes!("../assets/icon-64.rgba").to_vec(),
                width: 64,
                height: 64,
            }))
            .with_inner_size([1180.0, 760.0])
            // Floor: below this the toolbars and editors would start to
            // clip, so the window simply refuses to shrink any further.
            .with_min_inner_size([1120.0, 640.0]),
        ..Default::default()
    };
    eframe::run_native(
        "DevTextTools",
        options,
        Box::new(|cc| {
            configure_style(&cc.egui_ctx);
            configure_fonts(&cc.egui_ctx);
            Ok(Box::new(App::default()))
        }),
    )
}

/// CLI parity with the GUI:
///   devtexttools --list
///   devtexttools <tool-id> [--param <value>]   (input read from stdin)
///   devtexttools --search <word>
fn cli(args: &[String]) -> i32 {
    if args[0] == "--list" || args[0] == "list" {
        for op in all_ops() {
            println!("{:<28} [{}] {}", op.id, op.category, op.name);
        }
        println!("\n{} tools total.", all_ops().len());
        return 0;
    }
    if args[0] == "--search" || args[0] == "search" {
        let q = args.get(1).map(|s| s.to_lowercase()).unwrap_or_default();
        for op in all_ops() {
            if op.name.to_lowercase().contains(&q)
                || op.id.contains(&q)
                || op.description.to_lowercase().contains(&q)
            {
                println!("{:<28} [{}] {}", op.id, op.category, op.name);
            }
        }
        return 0;
    }
    if args[0] == "--help" || args[0] == "help" || args[0] == "-h" {
        println!("DevTextTools — every text & dev tool, offline.\n\nNo arguments: opens the window.\n--list                 list all tools\n--search <word>        find tools\n<tool-id> [--param X]  run a tool, text in via stdin, result to stdout");
        return 0;
    }
    if args[0] == "pdf-merge" {
        // DevTextTools pdf-merge <output.pdf> <in1.pdf> <in2.pdf> [...]
        if args.len() < 4 {
            eprintln!("Usage: DevTextTools pdf-merge <output.pdf> <in1.pdf> <in2.pdf> [...]");
            return 2;
        }
        let out = std::path::PathBuf::from(&args[1]);
        let inputs: Vec<std::path::PathBuf> =
            args[2..].iter().map(std::path::PathBuf::from).collect();
        return match devtexttools::pdf::pdf_merge(&inputs, &out) {
            Ok(report) => {
                println!("{report}");
                0
            }
            Err(e) => {
                eprintln!("Error: {e}");
                1
            }
        };
    }
    if args[0] == "pdf-split" {
        // DevTextTools pdf-split <in.pdf> [output folder]
        if args.len() < 2 {
            eprintln!("Usage: DevTextTools pdf-split <in.pdf> [output folder]");
            return 2;
        }
        let input = std::path::PathBuf::from(&args[1]);
        let outdir = args
            .get(2)
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        return match devtexttools::pdf::pdf_split(&input, &outdir) {
            Ok(report) => {
                println!("{report}");
                0
            }
            Err(e) => {
                eprintln!("Error: {e}");
                1
            }
        };
    }
    let id = &args[0];
    let mut param = String::new();
    let mut k = 1;
    while k < args.len() {
        if args[k] == "--param" {
            if let Some(v) = args.get(k + 1) {
                param = v.clone();
                k += 2;
                continue;
            }
        }
        k += 1;
    }
    let mut input = String::new();
    if std::io::stdin().read_to_string(&mut input).is_err() {
        eprintln!("Could not read input from stdin");
        return 2;
    }
    // stdin input usually ends with a newline the user didn't mean as content for line tools; keep it — ops trim where it matters.
    match run_op(id, input.trim_end_matches(['\n', '\r']), &param) {
        Ok(out) => {
            println!("{out}");
            0
        }
        Err(e) => {
            eprintln!("Error: {e}");
            1
        }
    }
}

/// The fonts bundled with egui have no glyphs for the fancy-text ranges
/// (mathematical letters, circled letters) or for a few symbols (check mark,
/// arrows), so those drew as empty boxes in the field test. On Windows, add
/// Segoe UI Symbol (and Emoji) as fallback fonts — they ship with Windows.
#[cfg(windows)]
fn configure_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let mut added = false;
    // Terminal design: prefer a monospace UI font (Cascadia Mono ships with
    // Windows Terminal / Windows 11; Consolas is on every Windows).
    let mut mono_names: Vec<String> = Vec::new();
    for (name, path) in [
        ("cascadia-mono", "C:\\Windows\\Fonts\\CascadiaMono.ttf"),
        ("cascadia-code", "C:\\Windows\\Fonts\\CascadiaCode.ttf"),
        ("consolas", "C:\\Windows\\Fonts\\consola.ttf"),
    ] {
        if let Ok(bytes) = std::fs::read(path) {
            fonts.font_data.insert(
                name.to_owned(),
                std::sync::Arc::new(egui::FontData::from_owned(bytes)),
            );
            mono_names.push(name.to_owned());
            added = true;
        }
    }
    if !mono_names.is_empty() {
        for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
            let fam = fonts.families.entry(family).or_default();
            for n in mono_names.iter().rev() {
                fam.insert(0, n.clone());
            }
        }
    }
    for (name, path) in [
        ("segoe-symbol", "C:\\Windows\\Fonts\\seguisym.ttf"),
        ("segoe-emoji", "C:\\Windows\\Fonts\\seguiemj.ttf"),
    ] {
        if let Ok(bytes) = std::fs::read(path) {
            fonts.font_data.insert(
                name.to_owned(),
                std::sync::Arc::new(egui::FontData::from_owned(bytes)),
            );
            for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
                fonts
                    .families
                    .entry(family)
                    .or_default()
                    .push(name.to_owned());
            }
            added = true;
        }
    }
    if added {
        ctx.set_fonts(fonts);
    }
}

#[cfg(not(windows))]
fn configure_fonts(_ctx: &egui::Context) {}

fn configure_style(ctx: &egui::Context) {
    // Terminal theme (matches CalcDex): near-black panels, phosphor-green
    // accent, square corners, monospace text.
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = egui::Color32::from_rgb(10, 10, 10);
    visuals.window_fill = egui::Color32::from_rgb(16, 16, 16);
    visuals.extreme_bg_color = egui::Color32::from_rgb(13, 13, 13);
    visuals.faint_bg_color = egui::Color32::from_rgb(21, 21, 21);
    visuals.selection.bg_fill = egui::Color32::from_rgba_unmultiplied(74, 222, 128, 60);
    visuals.selection.stroke = egui::Stroke::new(1.0_f32, GREEN);
    visuals.hyperlink_color = GREEN;
    let zero = egui::CornerRadius::ZERO;
    for w in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        w.corner_radius = zero;
    }
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0_f32, LINE);
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(21, 21, 21);
    visuals.widgets.inactive.bg_stroke =
        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(43, 43, 43));
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(29, 29, 29);
    visuals.widgets.hovered.bg_stroke =
        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(64, 64, 64));
    visuals.widgets.active.bg_fill = GREEN_DIM;
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0_f32, GREEN);
    visuals.window_corner_radius = zero;
    visuals.menu_corner_radius = zero;
    ctx.set_visuals(visuals);
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(10.0, 6.0);
    style.text_styles.insert(
        egui::TextStyle::Heading,
        egui::FontId::new(18.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::new(13.5, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::new(12.5, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Monospace,
        egui::FontId::new(13.0, egui::FontFamily::Monospace),
    );
    ctx.set_style(style);
}

#[derive(PartialEq, Clone, Copy)]
enum View {
    Tools,
    Timers,
    Pipelines,
    RegexLab,
    Replace,
    JsonTree,
}

/// Project find & replace results: (file, match count, sample lines).
type RepHits = Vec<(std::path::PathBuf, usize, Vec<(usize, String)>)>;

/// One work tab in the Tools view: its own tool + input/parameter/output.
#[derive(Clone, Default)]
struct WorkTab {
    tool: Option<usize>,
    input: String,
    param: String,
    output: String,
}

#[derive(Default)]
struct Timers {
    sw_since: Option<Instant>,
    sw_elapsed: Duration,
    cd_minutes: String,
    cd_since: Option<Instant>,
    cd_remaining: Duration,
    cd_total: Duration,
    cd_done: bool,
    pomo_work: bool,
    pomo_since: Option<Instant>,
    pomo_remaining: Duration,
    pomo_cycles: u32,
}

/// One recorded run for the History rail (Terminal design).
#[derive(Clone)]
struct HistEntry {
    ts: i64,
    tool: usize,
    input: String,
    param: String,
    output: String,
}

struct App {
    view: View,
    history: Vec<HistEntry>,
    show_history: bool,
    timers: Timers,
    active_cat: String,
    ops: Vec<Op>,
    search: String,
    selected: Option<usize>,
    input: String,
    param: String,
    output: String,
    status: String,
    favorites: Vec<String>,
    recents: Vec<String>,
    settings_dirty: bool,
    launcher_open: bool,
    launcher_focus: bool,
    launcher_query: String,
    launcher_sel: usize,
    pipe_steps: Vec<(String, String)>,
    pipe_name: String,
    saved_pipes: Vec<(String, Vec<(String, String)>)>,
    pipe_input: String,
    pipe_output: String,
    lab_pattern: String,
    lab_text: String,
    lab_icase: bool,
    tabs: Vec<WorkTab>,
    tab_active: usize,
    undo_stack: Vec<(String, String, String)>,
    redo_stack: Vec<(String, String, String)>,
    snippets: Vec<(String, String)>,
    snip_sel: usize,
    snip_name: String,
    saved_regex: Vec<(String, String)>,
    lab_sel: usize,
    lab_name: String,
    paste_autorun: bool,
    autorun_pending: bool,
    tree_text: String,
    tree_path: String,
    rep_folder: String,
    rep_find: String,
    rep_replace: String,
    rep_exts: String,
    rep_regex: bool,
    rep_hits: RepHits,
    rep_note: String,
    rep_key: String,
}

impl Default for App {
    fn default() -> Self {
        let ops = all_ops();
        let settings = load_settings(&ops);
        let history: Vec<HistEntry> = settings
            .history
            .iter()
            .filter_map(|(ts, id, i, p, o)| {
                ops.iter().position(|op| op.id == id).map(|tool| HistEntry {
                    ts: *ts,
                    tool,
                    input: i.clone(),
                    param: p.clone(),
                    output: o.clone(),
                })
            })
            .collect();
        App {
            favorites: settings.favorites,
            recents: settings.recents,
            saved_pipes: settings.pipes,
            snippets: settings.snippets,
            saved_regex: settings.regexes,
            paste_autorun: settings.autorun,
            snip_sel: 0,
            snip_name: String::new(),
            lab_sel: 0,
            lab_name: String::new(),
            autorun_pending: false,
            tree_text: String::new(),
            tree_path: String::new(),
            rep_folder: String::new(),
            rep_find: String::new(),
            rep_replace: String::new(),
            rep_exts: String::new(),
            rep_regex: false,
            rep_hits: Vec::new(),
            rep_note: String::new(),
            rep_key: String::new(),
            settings_dirty: false,
            history,
            show_history: true,
            launcher_open: false,
            launcher_focus: false,
            launcher_query: String::new(),
            launcher_sel: 0,
            pipe_steps: Vec::new(),
            pipe_name: String::new(),
            pipe_input: String::new(),
            pipe_output: String::new(),
            lab_pattern: String::new(),
            lab_text: String::new(),
            lab_icase: false,
            tabs: vec![WorkTab {
                tool: Some(0),
                ..Default::default()
            }],
            tab_active: 0,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            active_cat: ops
                .first()
                .map(|o| o.category.to_string())
                .unwrap_or_default(),
            view: View::Tools,
            timers: Timers {
                pomo_work: true,
                pomo_remaining: Duration::from_secs(25 * 60),
                ..Default::default()
            },
            ops,
            search: String::new(),
            selected: Some(0),
            input: String::new(),
            param: String::new(),
            output: String::new(),
            status: String::new(),
        }
    }
}

/// Measured minimum width of a button showing `text`: the label's
/// laid-out width plus the button's own horizontal padding and a
/// small safety margin, so a row built from these minimums can
/// never push a button off the edge of the window.
fn btn_min_width(ui: &egui::Ui, text: &str) -> f32 {
    let font = egui::TextStyle::Button.resolve(ui.style());
    let text_w = ui
        .painter()
        .layout_no_wrap(text.to_string(), font, egui::Color32::WHITE)
        .size()
        .x;
    text_w + ui.spacing().button_padding.x * 2.0 + 8.0
}

/// Widths for one row of buttons: every button gets its measured
/// minimum plus an equal share of the leftover space, so the row
/// fills the available width exactly AND every label fits in full.
/// (v0.7.1 divided rows into plain equal slots; with a long tool
/// name the slot was narrower than the label, the button refused
/// to shrink below its text, and the last button in the row slid
/// off the right edge — found in the v0.7.1 field check.)
fn row_widths(ui: &egui::Ui, labels: &[&str]) -> Vec<f32> {
    let n = labels.len().max(1);
    let spacing = ui.spacing().item_spacing.x;
    let avail = ui.available_width();
    let mins: Vec<f32> = labels.iter().map(|t| btn_min_width(ui, t)).collect();
    let used: f32 = mins.iter().sum::<f32>() + spacing * (n - 1) as f32;
    let slack = ((avail - used) / n as f32).max(0.0);
    mins.iter().map(|m| (m + slack).min(avail)).collect()
}

/// Group item indexes into rows for a button strip: a new row
/// starts when the row reaches the usual density (about `target`
/// px per item) or when the next item's minimum width would push
/// the row past the available width — whichever comes first.
fn pack_rows(ui: &egui::Ui, labels: &[&str], target: f32) -> Vec<Vec<usize>> {
    let spacing = ui.spacing().item_spacing.x;
    let avail = ui.available_width();
    let cap = per_row(ui, target);
    let mut rows: Vec<Vec<usize>> = Vec::new();
    let mut row: Vec<usize> = Vec::new();
    let mut row_min = 0.0_f32;
    for (i, t) in labels.iter().enumerate() {
        let m = btn_min_width(ui, t);
        if !row.is_empty() && (row.len() >= cap || row_min + spacing + m > avail) {
            rows.push(std::mem::take(&mut row));
            row_min = 0.0;
        }
        row_min += if row.is_empty() { m } else { spacing + m };
        row.push(i);
    }
    if !row.is_empty() {
        rows.push(row);
    }
    rows
}

/// How many items per row so each gets about `target` px (at least 1).
fn per_row(ui: &egui::Ui, target: f32) -> usize {
    let spacing = ui.spacing().item_spacing.x;
    (((ui.available_width() + spacing) / (target + spacing)).floor() as usize).max(1)
}

/// Where DevTextTools keeps its small settings file (favorites, recents,
/// saved pipelines). Per-user app data on Windows; config dir elsewhere.
fn settings_path() -> std::path::PathBuf {
    let base = std::env::var_os("LOCALAPPDATA")
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::var_os("XDG_CONFIG_HOME").map(std::path::PathBuf::from))
        .or_else(|| std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".config")))
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    base.join("DevTextTools").join("settings.txt")
}

type SavedPipes = Vec<(String, Vec<(String, String)>)>;

#[derive(Default)]
struct Settings {
    favorites: Vec<String>,
    recents: Vec<String>,
    pipes: SavedPipes,
    snippets: Vec<(String, String)>,
    regexes: Vec<(String, String)>,
    autorun: bool,
    history: Vec<(i64, String, String, String, String)>,
}

/// Escape a value for one settings line (backslash scheme).
fn esc(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\t', "\\t")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\u{1}', "\\x01")
}

/// Inverse of `esc`.
fn unesc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some('x') => {
                // only \x01 is produced by esc; swallow the two digits
                let _ = chars.next();
                let _ = chars.next();
                out.push('\u{1}');
            }
            Some(other) => out.push(other),
            None => out.push('\\'),
        }
    }
    out
}

/// Load favorites / recents / pipelines / snippets / saved patterns.
/// Unknown tool ids are dropped.
fn load_settings(ops: &[Op]) -> Settings {
    let mut st = Settings::default();
    let known = |id: &str| ops.iter().any(|o| o.id == id);
    if let Ok(text) = std::fs::read_to_string(settings_path()) {
        for line in text.lines() {
            let mut parts = line.split('\t');
            match parts.next() {
                Some("fav") => {
                    if let Some(id) = parts.next() {
                        if known(id) && !st.favorites.contains(&id.to_string()) {
                            st.favorites.push(id.to_string());
                        }
                    }
                }
                Some("recent") => {
                    if let Some(id) = parts.next() {
                        if known(id) && !st.recents.contains(&id.to_string()) {
                            st.recents.push(id.to_string());
                        }
                    }
                }
                Some("pipe") => {
                    if let Some(name) = parts.next() {
                        let steps: Vec<(String, String)> = parts
                            .filter_map(|s| {
                                let (id, param) = s.split_once('\u{1}')?;
                                if known(id) {
                                    Some((id.to_string(), param.to_string()))
                                } else {
                                    None
                                }
                            })
                            .collect();
                        if !steps.is_empty() {
                            st.pipes.push((name.to_string(), steps));
                        }
                    }
                }
                Some("snip") => {
                    if let (Some(name), Some(text)) = (parts.next(), parts.next()) {
                        st.snippets.push((name.to_string(), unesc(text)));
                    }
                }
                Some("regex") => {
                    if let (Some(name), Some(pat)) = (parts.next(), parts.next()) {
                        st.regexes.push((name.to_string(), unesc(pat)));
                    }
                }
                Some("autorun") => {
                    st.autorun = parts.next() == Some("1");
                }
                Some("hist") => {
                    if let (Some(ts), Some(id), Some(i), Some(pp), Some(o)) = (
                        parts.next(),
                        parts.next(),
                        parts.next(),
                        parts.next(),
                        parts.next(),
                    ) {
                        if known(id) {
                            if let Ok(ts) = ts.parse::<i64>() {
                                st.history.push((
                                    ts,
                                    id.to_string(),
                                    unesc(i),
                                    unesc(pp),
                                    unesc(o),
                                ));
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    st.recents.truncate(10);
    st
}

impl App {
    fn save_settings(&self) {
        let clean = |s: &str| s.replace(['\t', '\n', '\r', '\u{1}'], " ");
        let mut out = String::new();
        for id in &self.favorites {
            out.push_str(&format!("fav\t{id}\n"));
        }
        for id in &self.recents {
            out.push_str(&format!("recent\t{id}\n"));
        }
        for (name, steps) in &self.saved_pipes {
            let step_text: Vec<String> = steps
                .iter()
                .map(|(id, param)| format!("{id}\u{1}{}", clean(param)))
                .collect();
            out.push_str(&format!(
                "pipe\t{}\t{}\n",
                clean(name),
                step_text.join("\t")
            ));
        }
        for (name, text) in &self.snippets {
            out.push_str(&format!("snip\t{}\t{}\n", clean(name), esc(text)));
        }
        for (name, pat) in &self.saved_regex {
            out.push_str(&format!("regex\t{}\t{}\n", clean(name), esc(pat)));
        }
        if self.paste_autorun {
            out.push_str("autorun\t1\n");
        }
        for h in self.history.iter().take(20) {
            out.push_str(&format!(
                "hist\t{}\t{}\t{}\t{}\t{}\n",
                h.ts,
                self.ops[h.tool].id,
                esc(&h.input),
                esc(&h.param),
                esc(&h.output)
            ));
        }
        let path = settings_path();
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(&path, out);
    }

    fn filtered(&self) -> Vec<usize> {
        let q = self.search.trim().to_lowercase();
        self.ops
            .iter()
            .enumerate()
            .filter(|(_, op)| {
                q.is_empty()
                    || op.name.to_lowercase().contains(&q)
                    || op.category.to_lowercase().contains(&q)
                    || op.description.to_lowercase().contains(&q)
                    || op.id.contains(&q)
            })
            .map(|(i, _)| i)
            .collect()
    }

    /// The History rail (Terminal design): recent runs, newest first, each
    /// with a RE-OPEN that restores the tool, its input and its parameter.
    fn history_ui(&mut self, ui: &mut egui::Ui) {
        ui.label(
            egui::RichText::new(format!("HISTORY ({})", self.history.len()))
                .color(GREEN)
                .size(11.0),
        );
        ui.label("Recent runs, newest first. RE-OPEN loads the tool with its input and parameter restored.");
        let mut clear = false;
        let mut reopen: Option<usize> = None;
        if ui
            .add(egui::Button::new(
                egui::RichText::new("CLEAR HISTORY").color(REDT),
            ))
            .on_hover_text("Remove all history entries. This cannot be undone.")
            .clicked()
        {
            clear = true;
        }
        ui.separator();
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (n, h) in self.history.iter().enumerate() {
                let name = self.ops[h.tool].name;
                egui::Frame::new()
                    .stroke(egui::Stroke::new(1.0_f32, LINE))
                    .inner_margin(egui::Margin::same(8))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(fmt_ts(h.ts)).weak().small());
                            ui.label(egui::RichText::new(name).color(GREEN).small());
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui
                                        .small_button("RE-OPEN")
                                        .on_hover_text(
                                            "Load this tool with the input and parameter it ran with.",
                                        )
                                        .clicked()
                                    {
                                        reopen = Some(n);
                                    }
                                },
                            );
                        });
                        let one = |s: &str| -> String {
                            s.chars().take(90).collect::<String>().replace('\n', " ")
                        };
                        ui.label(egui::RichText::new(format!("Input: {}", one(&h.input))).small());
                        ui.label(
                            egui::RichText::new(format!("Result: {}", one(&h.output))).small(),
                        );
                    });
                ui.add_space(6.0);
            }
        });
        if clear {
            self.history.clear();
            self.settings_dirty = true;
        }
        if let Some(n) = reopen {
            if let Some(h) = self.history.get(n).cloned() {
                self.snapshot();
                self.selected = Some(h.tool);
                self.active_cat = self.ops[h.tool].category.to_string();
                self.input = h.input;
                self.param = h.param;
                self.output.clear();
                self.view = View::Tools;
                self.status = "Re-opened from history".into();
            }
        }
    }

    #[cfg(windows)]
    fn file_buttons(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let fws = row_widths(
                ui,
                &[
                    "\u{1F4C2} Open file \u{2192} Input",
                    "\u{1F4BE} Save output\u{2026}",
                    "\u{1F4C2} Batch: run tool on files\u{2026}",
                    "PDF path \u{2192} Input",
                ],
            );
            if ui
                .add_sized([fws[0], 26.0], egui::Button::new("\u{1F4C2} Open file → Input"))
                .on_hover_text("Open a text file and load its contents into the Input box (up to 10 MB).")
                .clicked()
            {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    match std::fs::read(&path) {
                        Ok(bytes) => {
                            if bytes.len() > 10_000_000 {
                                self.status =
                                    "That file is over 10 MB — too big to load as text".into();
                            } else {
                                self.input = String::from_utf8_lossy(&bytes).into_owned();
                                self.status = format!("Loaded {}", path.display());
                            }
                        }
                        Err(e) => self.status = format!("⚠ Could not read the file: {e}"),
                    }
                }
            }
            if ui
                .add_sized([fws[1], 26.0], egui::Button::new("\u{1F4BE} Save output…"))
                .on_hover_text("Save the Output box contents to a file.")
                .clicked()
            {
                if let Some(path) = rfd::FileDialog::new()
                    .set_file_name("output.txt")
                    .save_file()
                {
                    match std::fs::write(&path, &self.output) {
                        Ok(()) => self.status = format!("Saved to {}", path.display()),
                        Err(e) => self.status = format!("⚠ Could not save: {e}"),
                    }
                }
            }
            if ui
                .add_sized(
                    [fws[2], 26.0],
                    egui::Button::new("\u{1F4C2} Batch: run tool on files\u{2026}"),
                )
                .on_hover_text("Run the selected tool on many files at once; one result file is written per input file.")
                .clicked()
            {
                self.batch_run();
            }
            if ui
                .add_sized([fws[3], 26.0], egui::Button::new("PDF path → Input"))
                .on_hover_text("Put a PDF file's path in the Input box, for the PDF tools.")
                .clicked()
            {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("PDF", &["pdf"])
                    .pick_file()
                {
                    self.input = path.display().to_string();
                    self.status =
                        "PDF path loaded — pick PDF Info or PDF \u{2192} Text on the left, then Run"
                            .into();
                }
            }
        });
        ui.horizontal(|ui| {
            let pws = row_widths(
                ui,
                &[
                    "PDF Info\u{2026}",
                    "PDF \u{2192} Text\u{2026}",
                    "Merge PDFs\u{2026}",
                    "Split PDF\u{2026}",
                ],
            );
            if ui
                .add_sized([pws[0], 26.0], egui::Button::new("PDF Info…"))
                .on_hover_text("Pick a PDF and show its page count, size and document details.")
                .clicked()
            {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("PDF", &["pdf"])
                    .pick_file()
                {
                    match devtexttools::pdf::pdf_info(&path.to_string_lossy()) {
                        Ok(r) => {
                            self.output = r;
                            self.status = "\u{2713} PDF info".into();
                        }
                        Err(e) => self.status = format!("⚠ {e}"),
                    }
                }
            }
            if ui
                .add_sized([pws[1], 26.0], egui::Button::new("PDF \u{2192} Text…"))
                .on_hover_text("Pick a PDF and extract its text into the Output box.")
                .clicked()
            {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("PDF", &["pdf"])
                    .pick_file()
                {
                    match devtexttools::pdf::pdf_extract_text(&path.to_string_lossy()) {
                        Ok(r) => {
                            self.output = r;
                            self.status = "\u{2713} Text extracted from PDF".into();
                        }
                        Err(e) => self.status = format!("⚠ {e}"),
                    }
                }
            }
            if ui
                .add_sized([pws[2], 26.0], egui::Button::new("Merge PDFs…"))
                .on_hover_text("Pick two or more PDFs and join them into one file.")
                .clicked()
            {
                if let Some(files) = rfd::FileDialog::new()
                    .add_filter("PDF", &["pdf"])
                    .pick_files()
                {
                    if files.len() < 2 {
                        self.status = "Pick at least two PDFs to merge".into();
                    } else if let Some(out) = rfd::FileDialog::new()
                        .add_filter("PDF", &["pdf"])
                        .set_file_name("merged.pdf")
                        .save_file()
                    {
                        match devtexttools::pdf::pdf_merge(&files, &out) {
                            Ok(r) => self.status = format!("\u{2713} {r}"),
                            Err(e) => self.status = format!("⚠ {e}"),
                        }
                    }
                }
            }
            if ui
                .add_sized([pws[3], 26.0], egui::Button::new("Split PDF…"))
                .on_hover_text("Pick a PDF and split it into one file per page.")
                .clicked()
            {
                if let Some(file) = rfd::FileDialog::new()
                    .add_filter("PDF", &["pdf"])
                    .pick_file()
                {
                    if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                        match devtexttools::pdf::pdf_split(&file, &dir) {
                            Ok(r) => {
                                self.output = r;
                                self.status = "\u{2713} PDF split".into();
                            }
                            Err(e) => self.status = format!("⚠ {e}"),
                        }
                    }
                }
            }
        });
    }

    #[cfg(not(windows))]
    fn file_buttons(&mut self, _ui: &mut egui::Ui) {}

    /// Remember the current (input, param, output) so Undo can restore it.
    fn snapshot(&mut self) {
        self.undo_stack
            .push((self.input.clone(), self.param.clone(), self.output.clone()));
        if self.undo_stack.len() > 50 {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    fn undo(&mut self) {
        if let Some(prev) = self.undo_stack.pop() {
            let cur = (self.input.clone(), self.param.clone(), self.output.clone());
            self.redo_stack.push(cur);
            self.input = prev.0;
            self.param = prev.1;
            self.output = prev.2;
            self.status = "Undid the last action".into();
        }
    }

    fn redo(&mut self) {
        if let Some(next) = self.redo_stack.pop() {
            let cur = (self.input.clone(), self.param.clone(), self.output.clone());
            self.undo_stack.push(cur);
            self.input = next.0;
            self.param = next.1;
            self.output = next.2;
            self.status = "Redone".into();
        }
    }

    /// Keep the active work tab in sync with the visible fields.
    fn stash_tab(&mut self) {
        if let Some(tab) = self.tabs.get_mut(self.tab_active) {
            tab.tool = self.selected;
            tab.input = self.input.clone();
            tab.param = self.param.clone();
            tab.output = self.output.clone();
        }
    }

    fn load_tab(&mut self, n: usize) {
        self.stash_tab();
        if let Some(tab) = self.tabs.get(n) {
            let tab = tab.clone();
            self.tab_active = n;
            self.selected = tab.tool;
            self.input = tab.input;
            self.param = tab.param;
            self.output = tab.output;
            if let Some(idx) = self.selected {
                self.active_cat = self.ops[idx].category.to_string();
            }
            self.view = View::Tools;
        }
    }

    fn run_selected(&mut self) {
        self.snapshot();
        if let Some(idx) = self.selected {
            let op = self.ops[idx];
            match (op.run)(&self.input, &self.param) {
                Ok(out) => {
                    let ts = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs() as i64)
                        .unwrap_or(0);
                    self.history.insert(
                        0,
                        HistEntry {
                            ts,
                            tool: idx,
                            input: self.input.clone(),
                            param: self.param.clone(),
                            output: out.clone(),
                        },
                    );
                    self.history.truncate(50);
                    self.output = out;
                    self.status = format!("✓ {} done", op.name);
                    self.recents.retain(|id| id != op.id);
                    self.recents.insert(0, op.id.to_string());
                    self.recents.truncate(10);
                    self.settings_dirty = true;
                }
                Err(e) => {
                    self.status = format!("⚠ {e}");
                }
            }
        }
    }

    /// Guess what the input most likely is, and offer matching tools.
    /// Returns (kind label, op indices) for the strongest single guess.
    fn suggestions(&self) -> Option<(&'static str, Vec<usize>)> {
        let text = self.input.trim();
        if text.len() < 4 || text.len() > 200_000 {
            return None;
        }
        let find = |id: &str| self.ops.iter().position(|o| o.id == id);
        let pick = |ids: &[&str]| -> Vec<usize> { ids.iter().filter_map(|id| find(id)).collect() };
        // JWT: three dot-separated parts, header decodes to JSON with "alg".
        let parts: Vec<&str> = text.split('.').collect();
        if parts.len() == 3 && !text.contains(char::is_whitespace) {
            use base64::Engine;
            if let Ok(bytes) = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(parts[0]) {
                if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                    if v.get("alg").is_some() {
                        return Some(("a JWT (login token)", pick(&["dev-jwt-decode"])));
                    }
                }
            }
        }
        // JSON document.
        if (text.starts_with('{') || text.starts_with('['))
            && serde_json::from_str::<serde_json::Value>(text).is_ok()
        {
            return Some((
                "JSON",
                pick(&[
                    "dev-json-pretty",
                    "dev-json-validate",
                    "dev-jsonpath",
                    "dev-json-to-ts",
                ]),
            ));
        }
        // Stack trace.
        let at_frames = text
            .lines()
            .filter(|l| l.trim_start().starts_with("at "))
            .count();
        if text.contains("Traceback (most recent call last)") || at_frames >= 2 {
            return Some(("a stack trace", pick(&["an-stack-clean", "an-log-analyze"])));
        }
        // UUID.
        let is_uuid = text.len() == 36
            && text.chars().enumerate().all(|(i, c)| match i {
                8 | 13 | 18 | 23 => c == '-',
                _ => c.is_ascii_hexdigit(),
            });
        if is_uuid {
            return Some(("a UUID", pick(&["dev-uuid-validate"])));
        }
        // Hash (hex of a known digest length).
        if [32usize, 40, 56, 64, 96, 128].contains(&text.len())
            && text.chars().all(|c| c.is_ascii_hexdigit())
        {
            return Some(("a hash fingerprint", pick(&["dev-hash-identify"])));
        }
        // Web address.
        if (text.starts_with("http://") || text.starts_with("https://"))
            && !text.contains(char::is_whitespace)
        {
            return Some(("a web address", pick(&["dev-url-parse"])));
        }
        // Base64 that decodes to mostly readable text.
        if text.len() >= 20 && !text.contains(char::is_whitespace) {
            use base64::Engine;
            if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(text) {
                let printable = bytes
                    .iter()
                    .filter(|b| matches!(b, 9 | 10 | 13 | 32..=126))
                    .count();
                if !bytes.is_empty() && printable * 10 >= bytes.len() * 9 {
                    return Some(("Base64", pick(&["enc-base64-decode"])));
                }
            }
        }
        None
    }

    fn handle_drop(&mut self, path: &std::path::Path) {
        self.view = View::Tools;
        let is_pdf = path
            .extension()
            .map(|e| e.eq_ignore_ascii_case("pdf"))
            .unwrap_or(false);
        if is_pdf {
            self.input = path.display().to_string();
            if let Some(idx) = self.ops.iter().position(|o| o.id == "pdf-info") {
                self.selected = Some(idx);
                self.active_cat = self.ops[idx].category.to_string();
                self.run_selected();
                self.status = format!("Dropped PDF: {}", path.display());
            }
            return;
        }
        match std::fs::read(path) {
            Ok(bytes) if bytes.len() <= 10_000_000 => {
                self.input = String::from_utf8_lossy(&bytes).into_owned();
                self.status = format!("Dropped file loaded: {}", path.display());
            }
            Ok(_) => {
                self.status = "That file is over 10 MB \u{2014} too big to load as text".into();
            }
            Err(e) => {
                self.status = format!("\u{26A0} Could not read the dropped file: {e}");
            }
        }
    }

    fn launcher_ui(&mut self, ctx: &egui::Context) {
        let q = self.launcher_query.trim().to_lowercase();
        let rows: Vec<(usize, String)> = self
            .ops
            .iter()
            .enumerate()
            .filter(|(_, op)| {
                q.is_empty()
                    || op.name.to_lowercase().contains(&q)
                    || op.id.contains(&q)
                    || op.category.to_lowercase().contains(&q)
            })
            .take(50)
            .map(|(i, op)| (i, format!("{}  \u{00B7}  {}", op.name, op.category)))
            .collect();
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)) && !rows.is_empty() {
            self.launcher_sel = (self.launcher_sel + 1).min(rows.len() - 1);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
            self.launcher_sel = self.launcher_sel.saturating_sub(1);
        }
        let mut pick: Option<usize> = None;
        let mut close = ctx.input(|i| i.key_pressed(egui::Key::Escape));
        if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
            if let Some((idx, _)) = rows.get(self.launcher_sel) {
                pick = Some(*idx);
            }
        }
        let sel = self.launcher_sel;
        egui::Window::new("\u{2328} Quick launcher \u{2014} Ctrl+K")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_TOP, [0.0, 90.0])
            .show(ctx, |ui| {
                ui.set_min_width(520.0);
                let resp = ui.add(
                    egui::TextEdit::singleline(&mut self.launcher_query)
                        .desired_width(f32::INFINITY)
                        .hint_text("Start typing a tool name\u{2026} \u{2191}\u{2193} to move, Enter to open"),
                );
                if self.launcher_focus {
                    resp.request_focus();
                    self.launcher_focus = false;
                }
                if resp.changed() {
                    self.launcher_sel = 0;
                }
                ui.separator();
                egui::ScrollArea::vertical()
                    .max_height(320.0)
                    .show(ui, |ui| {
                        for (row, (idx, label)) in rows.iter().enumerate() {
                            if ui
                                .add_sized(
                                    [ui.available_width(), 24.0],
                                    egui::Button::new(label).selected(row == sel),
                                )
                                .clicked()
                            {
                                pick = Some(*idx);
                            }
                        }
                        if rows.is_empty() {
                            ui.label("No tool matches that.");
                        }
                    });
            });
        if let Some(idx) = pick {
            self.selected = Some(idx);
            self.active_cat = self.ops[idx].category.to_string();
            self.view = View::Tools;
            self.param.clear();
            self.status.clear();
            close = true;
        }
        if close {
            self.launcher_open = false;
        }
    }

    fn run_pipeline(&mut self) {
        if self.pipe_steps.is_empty() {
            self.pipe_output = "Add at least one step first.".into();
            return;
        }
        let mut text = self.pipe_input.clone();
        for (n, (id, param)) in self.pipe_steps.iter().enumerate() {
            match run_op(id, &text, param) {
                Ok(out) => text = out,
                Err(e) => {
                    self.pipe_output = format!("Step {} ({id}) failed: {e}", n + 1);
                    return;
                }
            }
        }
        self.pipe_output = text;
    }

    fn pipelines_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Pipelines \u{2014} run tools one after another");
        ui.label("Each step's output becomes the next step's input.");
        ui.columns(2, |cols| {
            cols[0].label(egui::RichText::new("Input").strong());
            cols[0].add(
                egui::TextEdit::multiline(&mut self.pipe_input)
                    .desired_rows(8)
                    .desired_width(f32::INFINITY),
            );
            cols[1].label(egui::RichText::new("Output").strong());
            cols[1].add(
                egui::TextEdit::multiline(&mut self.pipe_output)
                    .desired_rows(8)
                    .desired_width(f32::INFINITY),
            );
        });
        ui.separator();
        ui.heading("Current chain");
        if self.pipe_steps.is_empty() {
            ui.label("No steps yet \u{2014} pick a tool on the Tools tab, then add it here.");
        }
        let mut remove: Option<usize> = None;
        for n in 0..self.pipe_steps.len() {
            let id = self.pipe_steps[n].0.clone();
            let name = self
                .ops
                .iter()
                .find(|o| o.id == id)
                .map(|o| o.name.to_string())
                .unwrap_or(id);
            ui.horizontal(|ui| {
                ui.label(format!("{}. {}", n + 1, name));
                ui.add(
                    egui::TextEdit::singleline(&mut self.pipe_steps[n].1)
                        .desired_width(280.0)
                        .hint_text("parameter (optional)"),
                );
                if ui.button("\u{2715}").clicked() {
                    remove = Some(n);
                }
            });
        }
        if let Some(n) = remove {
            self.pipe_steps.remove(n);
        }
        ui.horizontal(|ui| {
            if ui
                .button("\u{FF0B} Add the tool picked on the Tools tab")
                .clicked()
            {
                if let Some(idx) = self.selected {
                    let op = self.ops[idx];
                    self.pipe_steps
                        .push((op.id.to_string(), self.param.clone()));
                }
            }
            if ui.button("\u{25B6} Run chain").clicked() {
                self.run_pipeline();
            }
            if ui.button("Clear chain").clicked() {
                self.pipe_steps.clear();
            }
        });
        ui.separator();
        ui.heading("Saved chains");
        ui.horizontal(|ui| {
            ui.label("Name:");
            ui.add(egui::TextEdit::singleline(&mut self.pipe_name).desired_width(240.0));
            if ui.button("Save chain").clicked() {
                let name = self.pipe_name.trim().to_string();
                if name.is_empty() {
                    self.pipe_output = "Give the chain a name first.".into();
                } else if self.pipe_steps.is_empty() {
                    self.pipe_output = "The chain is empty.".into();
                } else if let Some(existing) = self.saved_pipes.iter_mut().find(|(n, _)| *n == name)
                {
                    existing.1 = self.pipe_steps.clone();
                    self.settings_dirty = true;
                    self.pipe_output = format!("Updated chain \u{2018}{name}\u{2019}.");
                } else {
                    self.saved_pipes
                        .push((name.clone(), self.pipe_steps.clone()));
                    self.settings_dirty = true;
                    self.pipe_output = format!("Saved chain \u{2018}{name}\u{2019}.");
                }
            }
        });
        let mut load: Option<usize> = None;
        let mut delete: Option<usize> = None;
        for (n, (name, steps)) in self.saved_pipes.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.label(format!("{name} ({} steps)", steps.len()));
                if ui.button("Load").clicked() {
                    load = Some(n);
                }
                if ui.button("Delete").clicked() {
                    delete = Some(n);
                }
            });
        }
        if let Some(n) = load {
            self.pipe_steps = self.saved_pipes[n].1.clone();
        }
        if let Some(n) = delete {
            self.saved_pipes.remove(n);
            self.settings_dirty = true;
        }
    }

    fn regex_lab_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Regex Lab \u{2014} live pattern tester");
        ui.horizontal(|ui| {
            ui.label("Pattern:");
            ui.add(
                egui::TextEdit::singleline(&mut self.lab_pattern)
                    .desired_width(480.0)
                    .hint_text("e.g. \\b\\w+@\\w+\\.\\w+\\b"),
            );
            ui.checkbox(&mut self.lab_icase, "Case-insensitive");
        });
        ui.horizontal(|ui| {
            ui.label("Saved:");
            let names: Vec<&str> = self.saved_regex.iter().map(|(n, _)| n.as_str()).collect();
            let mut sel = self.lab_sel.min(names.len().saturating_sub(1));
            egui::ComboBox::from_id_salt("lab_box")
                .selected_text(names.get(sel).copied().unwrap_or("\u{2014} none \u{2014}"))
                .show_ui(ui, |ui| {
                    for (i, n) in names.iter().enumerate() {
                        ui.selectable_value(&mut sel, i, *n);
                    }
                });
            self.lab_sel = sel;
            if ui.button("Load").clicked() {
                if let Some((_, pat)) = self.saved_regex.get(self.lab_sel) {
                    self.lab_pattern = pat.clone();
                }
            }
            ui.add(
                egui::TextEdit::singleline(&mut self.lab_name)
                    .desired_width(130.0)
                    .hint_text("name for Save"),
            );
            if ui.button("Save pattern").clicked() {
                let name = if self.lab_name.trim().is_empty() {
                    format!("Pattern {}", self.saved_regex.len() + 1)
                } else {
                    self.lab_name.trim().to_string()
                };
                if let Some(existing) = self.saved_regex.iter_mut().find(|(n, _)| *n == name) {
                    existing.1 = self.lab_pattern.clone();
                } else {
                    self.saved_regex.push((name, self.lab_pattern.clone()));
                }
                self.settings_dirty = true;
            }
            if ui.button("Delete").clicked() && !self.saved_regex.is_empty() {
                let n = self.lab_sel.min(self.saved_regex.len() - 1);
                self.saved_regex.remove(n);
                self.lab_sel = 0;
                self.settings_dirty = true;
            }
        });
        ui.label(egui::RichText::new("Test text").strong());
        ui.add(
            egui::TextEdit::multiline(&mut self.lab_text)
                .desired_rows(8)
                .desired_width(f32::INFINITY),
        );
        if self.lab_pattern.trim().is_empty() {
            ui.label("Type a pattern above and the matches light up here as you type.");
            return;
        }
        match regex::RegexBuilder::new(&self.lab_pattern)
            .case_insensitive(self.lab_icase)
            .build()
        {
            Err(e) => {
                ui.label(
                    egui::RichText::new(format!("\u{26A0} {e}"))
                        .color(egui::Color32::from_rgb(229, 72, 77)),
                );
            }
            Ok(re) => {
                let text = self.lab_text.clone();
                let plain = egui::text::TextFormat {
                    font_id: egui::FontId::monospace(13.0),
                    ..Default::default()
                };
                let matched = egui::text::TextFormat {
                    font_id: egui::FontId::monospace(13.0),
                    background: egui::Color32::from_rgb(0x6b, 0x07, 0x00),
                    ..Default::default()
                };
                let mut job = egui::text::LayoutJob::default();
                let mut listing = String::new();
                let mut last = 0usize;
                let mut count = 0usize;
                for caps in re.captures_iter(&text) {
                    let m = caps.get(0).expect("group 0 always exists");
                    count += 1;
                    if m.start() > last {
                        job.append(&text[last..m.start()], 0.0, plain.clone());
                    }
                    job.append(m.as_str(), 0.0, matched.clone());
                    last = m.end().max(last);
                    listing.push_str(&format!(
                        "{}\u{2013}{}: {}\n",
                        m.start(),
                        m.end(),
                        m.as_str()
                    ));
                    for gi in 1..caps.len() {
                        if let Some(g) = caps.get(gi) {
                            listing.push_str(&format!("    group {gi}: {}\n", g.as_str()));
                        }
                    }
                }
                if last < text.len() {
                    job.append(&text[last..], 0.0, plain);
                }
                ui.label(egui::RichText::new(format!("{count} match(es)")).strong());
                if count > 0 {
                    egui::ScrollArea::vertical()
                        .max_height(150.0)
                        .id_salt("lab_highlight")
                        .show(ui, |ui| {
                            ui.add(egui::Label::new(job).wrap());
                        });
                    ui.label(egui::RichText::new(listing).monospace());
                }
            }
        }
    }

    /// Batch run: apply the selected tool to every picked file's text and
    /// write one `<name>.out.txt` per file into a picked folder.
    #[cfg(windows)]
    fn batch_run(&mut self) {
        let Some(idx) = self.selected else {
            self.status = "Pick a tool first, then batch-run it".into();
            return;
        };
        let op = self.ops[idx];
        if op.category == "Files" {
            self.status =
                "Batch works with text tools \u{2014} the Files tools take paths, not contents"
                    .into();
            return;
        }
        let Some(files) = rfd::FileDialog::new().pick_files() else {
            return;
        };
        let Some(folder) = rfd::FileDialog::new().pick_folder() else {
            return;
        };
        self.snapshot();
        let mut summary = String::new();
        let (mut ok, mut failed) = (0usize, 0usize);
        for path in &files {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| "file".into());
            let result = std::fs::read(path)
                .map_err(|e| e.to_string())
                .and_then(|bytes| {
                    if bytes.len() > 10_000_000 {
                        return Err("over 10 MB".into());
                    }
                    (op.run)(
                        &String::from_utf8_lossy(&bytes).into_owned(),
                        &self.param.clone(),
                    )
                })
                .and_then(|out| {
                    let out_name = format!("{name}.out.txt");
                    std::fs::write(folder.join(&out_name), out)
                        .map_err(|e| e.to_string())
                        .map(|()| out_name)
                });
            match result {
                Ok(out_name) => {
                    ok += 1;
                    summary.push_str(&format!("\u{2713} {name} \u{2192} {out_name}\n"));
                }
                Err(e) => {
                    failed += 1;
                    summary.push_str(&format!("\u{2717} {name}: {e}\n"));
                }
            }
        }
        self.output = format!(
            "Batch run of {} on {} file(s): {ok} done, {failed} failed.\n\n{summary}",
            op.name,
            files.len()
        );
        self.status = format!("\u{2713} Batch done: {ok} ok, {failed} failed");
    }

    #[cfg(not(windows))]
    #[allow(dead_code)]
    fn batch_run(&mut self) {
        self.status = "Batch run is available in the Windows build".into();
    }

    /// Scan the replace folder and return (file, count, samples) hits.
    fn rep_scan(&self) -> Result<RepHits, String> {
        let root = std::path::PathBuf::from(self.rep_folder.trim());
        if !root.is_dir() {
            return Err("Pick a folder first.".into());
        }
        if self.rep_find.is_empty() {
            return Err("Type what to find.".into());
        }
        let re = if self.rep_regex {
            Some(regex::Regex::new(&self.rep_find).map_err(|e| format!("Bad pattern: {e}"))?)
        } else {
            None
        };
        let exts: Vec<String> = self
            .rep_exts
            .split(',')
            .map(|e| e.trim().trim_start_matches('.').to_lowercase())
            .filter(|e| !e.is_empty())
            .collect();
        let mut hits: RepHits = Vec::new();
        let mut stack = vec![root];
        let mut scanned = 0usize;
        while let Some(dir) = stack.pop() {
            let Ok(rd) = std::fs::read_dir(&dir) else {
                continue;
            };
            for entry in rd.flatten() {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().into_owned();
                if path.is_dir() {
                    if !matches!(name.as_str(), ".git" | "target" | "node_modules") {
                        stack.push(path);
                    }
                    continue;
                }
                scanned += 1;
                if scanned > 5000 {
                    break;
                }
                if !exts.is_empty() {
                    let ext = path
                        .extension()
                        .map(|e| e.to_string_lossy().to_lowercase())
                        .unwrap_or_default();
                    if !exts.contains(&ext) {
                        continue;
                    }
                }
                let Ok(bytes) = std::fs::read(&path) else {
                    continue;
                };
                if bytes.len() > 2_000_000 {
                    continue;
                }
                let Ok(text) = String::from_utf8(bytes) else {
                    continue;
                };
                let mut count = 0usize;
                let mut samples: Vec<(usize, String)> = Vec::new();
                for (ln, line) in text.lines().enumerate() {
                    let c = match &re {
                        Some(re) => re.find_iter(line).count(),
                        None => line.matches(self.rep_find.as_str()).count(),
                    };
                    if c > 0 {
                        count += c;
                        if samples.len() < 3 {
                            samples.push((ln + 1, line.trim().chars().take(120).collect()));
                        }
                    }
                }
                if count > 0 {
                    hits.push((path, count, samples));
                }
            }
        }
        hits.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(hits)
    }

    fn replace_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Project find & replace");
        ui.label("Searches text files under a folder (skips .git, target, node_modules; files over 2 MB are skipped). Preview first \u{2014} Apply only writes the files the preview found.");
        ui.horizontal(|ui| {
            ui.label("Folder:");
            ui.add(
                egui::TextEdit::singleline(&mut self.rep_folder)
                    .desired_width(420.0)
                    .hint_text("e.g. C:\\code\\my-project"),
            );
            #[cfg(windows)]
            if ui.button("Browse\u{2026}").clicked() {
                if let Some(f) = rfd::FileDialog::new().pick_folder() {
                    self.rep_folder = f.display().to_string();
                }
            }
        });
        ui.horizontal(|ui| {
            ui.label("Find:");
            ui.add(egui::TextEdit::singleline(&mut self.rep_find).desired_width(300.0));
            ui.label("Replace with:");
            ui.add(egui::TextEdit::singleline(&mut self.rep_replace).desired_width(300.0));
            ui.checkbox(&mut self.rep_regex, "Regex");
        });
        ui.horizontal(|ui| {
            ui.label("File extensions:");
            ui.add(
                egui::TextEdit::singleline(&mut self.rep_exts)
                    .desired_width(240.0)
                    .hint_text("rs, toml, md \u{2014} blank = all"),
            );
            if ui.button("\u{1F50D} Preview").clicked() {
                match self.rep_scan() {
                    Ok(hits) => {
                        let total: usize = hits.iter().map(|h| h.1).sum();
                        self.rep_note = format!("{} match(es) in {} file(s).", total, hits.len());
                        self.rep_hits = hits;
                        self.rep_key = format!(
                            "{}\u{1}{}\u{1}{}\u{1}{}",
                            self.rep_folder, self.rep_find, self.rep_replace, self.rep_regex
                        );
                    }
                    Err(e) => {
                        self.rep_hits.clear();
                        self.rep_note = e;
                    }
                }
            }
            let key_now = format!(
                "{}\u{1}{}\u{1}{}\u{1}{}",
                self.rep_folder, self.rep_find, self.rep_replace, self.rep_regex
            );
            let fresh = !self.rep_hits.is_empty() && key_now == self.rep_key;
            if ui
                .add_enabled(fresh, egui::Button::new("\u{2713} Apply to these files"))
                .clicked()
            {
                let re = if self.rep_regex {
                    regex::Regex::new(&self.rep_find).ok()
                } else {
                    None
                };
                let (mut files, mut edits) = (0usize, 0usize);
                for (path, count, _) in &self.rep_hits {
                    if let Ok(text) = std::fs::read_to_string(path) {
                        let new = match &re {
                            Some(re) => re
                                .replace_all(&text, self.rep_replace.as_str())
                                .into_owned(),
                            None => text.replace(self.rep_find.as_str(), &self.rep_replace),
                        };
                        if new != text && std::fs::write(path, new).is_ok() {
                            files += 1;
                            edits += count;
                        }
                    }
                }
                self.rep_note = format!("Applied: {edits} replacement(s) in {files} file(s).");
                self.rep_hits.clear();
            }
        });
        if !self.rep_note.is_empty() {
            ui.label(egui::RichText::new(&self.rep_note).strong());
        }
        ui.separator();
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (path, count, samples) in &self.rep_hits {
                ui.label(
                    egui::RichText::new(format!("{} \u{2014} {count} match(es)", path.display()))
                        .strong(),
                );
                for (ln, line) in samples {
                    ui.label(
                        egui::RichText::new(format!("  {ln}: {line}"))
                            .monospace()
                            .small(),
                    );
                }
            }
        });
    }

    fn json_tree_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("JSON Tree Explorer");
        ui.horizontal(|ui| {
            if ui.button("Load from Input (Tools tab)").clicked() {
                self.tree_text = self.input.clone();
                self.tree_path.clear();
            }
            if ui.button("Paste JSON here").clicked() {
                self.tree_text.clear();
                self.tree_path.clear();
            }
        });
        ui.add(
            egui::TextEdit::multiline(&mut self.tree_text)
                .desired_rows(6)
                .desired_width(f32::INFINITY)
                .hint_text("Paste JSON here, or load it from the Tools tab Input"),
        );
        if !self.tree_path.is_empty() {
            ui.label(format!("Copied path: {}", self.tree_path));
        }
        ui.separator();
        match serde_json::from_str::<serde_json::Value>(self.tree_text.trim()) {
            Ok(v) => {
                let mut clicked: Option<String> = None;
                egui::ScrollArea::vertical().show(ui, |ui| {
                    json_tree_node(ui, &v, "$", "$", &mut clicked);
                });
                if let Some(path) = clicked {
                    ui.ctx().copy_text(path.clone());
                    self.tree_path = path;
                }
            }
            Err(e) => {
                if !self.tree_text.trim().is_empty() {
                    ui.label(
                        egui::RichText::new(format!("\u{26A0} Not valid JSON: {e}"))
                            .color(egui::Color32::from_rgb(229, 72, 77)),
                    );
                } else {
                    ui.label("Nothing to show yet.");
                }
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.stash_tab();
        // Ctrl+K / Cmd+K quick launcher.
        if ctx.input(|i| (i.modifiers.command || i.modifiers.ctrl) && i.key_pressed(egui::Key::K)) {
            self.launcher_open = !self.launcher_open;
            if self.launcher_open {
                self.launcher_query.clear();
                self.launcher_sel = 0;
                self.launcher_focus = true;
            }
        }
        // Drag & drop: a dropped PDF shows its info; any other file loads as text.
        let dropped: Vec<std::path::PathBuf> = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .filter_map(|f| f.path.clone())
                .collect()
        });
        if let Some(path) = dropped.into_iter().next() {
            self.handle_drop(&path);
        }
        if self.launcher_open {
            self.launcher_ui(ctx);
        }
        // Auto-run on paste (opt-in): a paste into the window runs the
        // currently selected tool once the pasted text has landed.
        if self.paste_autorun
            && self.view == View::Tools
            && ctx.input(|i| i.events.iter().any(|e| matches!(e, egui::Event::Paste(_))))
        {
            self.autorun_pending = true;
        }
        // ===== Upper panel: title, tabs + search, category dropdowns =====
        // Snapshot of the menu structure (owned data, so the menu closures
        // never fight the borrow checker).
        let mut menu_data: Vec<(&'static str, Vec<(usize, &'static str)>)> = Vec::new();
        for (idx, op) in self.ops.iter().enumerate() {
            match menu_data.iter_mut().find(|(c, _)| *c == op.category) {
                Some((_, items)) => items.push((idx, op.name)),
                None => menu_data.push((op.category, vec![(idx, op.name)])),
            }
        }
        // Synthetic categories first: ★ Favorites, then ◷ Recent.
        let resolve = |ids: &[String]| -> Vec<(usize, &'static str)> {
            ids.iter()
                .filter_map(|id| {
                    self.ops
                        .iter()
                        .position(|o| o.id == id)
                        .map(|i| (i, self.ops[i].name))
                })
                .collect()
        };
        let fav_items = resolve(&self.favorites);
        let recent_items = resolve(&self.recents);
        if !recent_items.is_empty() {
            menu_data.insert(0, ("\u{25F7} Recent", recent_items));
        }
        if !fav_items.is_empty() {
            menu_data.insert(0, ("\u{2605} Favorites", fav_items));
        }
        let selected_now = self.selected;

        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("DEVTEXTTOOLS")
                        .strong()
                        .size(16.0)
                        .color(GREEN),
                );
                ui.label(
                    egui::RichText::new(format!(
                        "{} TOOLS \u{2014} OFFLINE \u{2014} NO ADS",
                        self.ops.len()
                    ))
                    .weak()
                    .size(11.0),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(
                            egui::Button::new(format!("HISTORY ({})", self.history.len()))
                                .selected(self.show_history),
                        )
                        .on_hover_text(
                            "Show or hide the History rail \u{2014} your recent runs, newest first.",
                        )
                        .clicked()
                    {
                        self.show_history = !self.show_history;
                    }
                });
            });
            ui.add_space(2.0);
            // Category chips (Terminal design): rows that STRETCH to
            // fill the toolbar, like every other bar in the program.
            let mut chip_pick: Option<usize> = None;
            let mut chip_cat: Option<String> = None;
            let mut chip_view: Option<View> = None;
            let cats: Vec<&(&str, Vec<(usize, &str)>)> = menu_data.iter().collect();
            let cat_labels: Vec<&str> = cats.iter().map(|e| e.0).collect();
            for row in pack_rows(ui, &cat_labels, 150.0) {
                let row_labels: Vec<&str> = row.iter().map(|&i| cat_labels[i]).collect();
                let widths = row_widths(ui, &row_labels);
                let chunk: Vec<&(&str, Vec<(usize, &str)>)> =
                    row.iter().map(|&i| cats[i]).collect();
                ui.horizontal(|ui| {
                    for (k, (cat, items)) in chunk.iter().enumerate() {
                        let w = widths[k];
                        let active = self.view == View::Tools && self.active_cat == **cat;
                        let mut btn = egui::Button::new(*cat);
                        if active {
                            btn = btn.fill(GREEN_DIM).stroke(egui::Stroke::new(1.0_f32, GREEN));
                        }
                        if ui
                            .add_sized([w, 26.0], btn)
                            .on_hover_text(format!("Show the {cat} tools"))
                            .clicked()
                        {
                            chip_cat = Some(cat.to_string());
                            if let Some((first, _)) = items.first() {
                                chip_pick = Some(*first);
                            }
                        }
                    }
                });
            }
            // View chips: one stretched row of their own.
            ui.horizontal(|ui| {
                let view_widths = row_widths(
                    ui,
                    &["Timers", "Pipelines", "Regex Lab", "Replace", "JSON Tree"],
                );
                for (k, (v, name)) in [
                    (View::Timers, "Timers"),
                    (View::Pipelines, "Pipelines"),
                    (View::RegexLab, "Regex Lab"),
                    (View::Replace, "Replace"),
                    (View::JsonTree, "JSON Tree"),
                ]
                .into_iter()
                .enumerate()
                {
                    let w = view_widths[k];
                    let active = self.view == v;
                    let mut btn = egui::Button::new(name);
                    if active {
                        btn = btn.fill(GREEN_DIM).stroke(egui::Stroke::new(1.0_f32, GREEN));
                    }
                    if ui
                        .add_sized([w, 26.0], btn)
                        .on_hover_text(format!("Open the {name} view"))
                        .clicked()
                    {
                        chip_view = Some(v);
                    }
                }
            });
            if let Some(c) = chip_cat {
                self.active_cat = c;
                self.view = View::Tools;
                // Follow the chip: if the current tool is not in this
                // category, open the category's first tool.
                let in_cat = selected_now
                    .map(|s| self.ops[s].category == self.active_cat)
                    .unwrap_or(false);
                if !in_cat {
                    if let Some(i) = chip_pick {
                        self.selected = Some(i);
                        self.param.clear();
                        self.status.clear();
                    }
                }
            }
            if let Some(v) = chip_view {
                self.view = v;
            }
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("SEARCH").color(GREEN).size(11.0));
                let resp = ui.add(
                    egui::TextEdit::singleline(&mut self.search)
                        .desired_width(ui.available_width() - 190.0)
                        .hint_text("filter tools..."),
                );
                if resp.changed() {
                    let f = self.filtered();
                    if let Some(first) = f.first() {
                        self.selected = Some(*first);
                        self.active_cat = self.ops[*first].category.to_string();
                    }
                }
                if !self.search.is_empty()
                    && ui
                        .button("Clear")
                        .on_hover_text("Clear the search box.")
                        .clicked()
                {
                    self.search.clear();
                }
                ui.label(egui::RichText::new("Ctrl+K quick jump").weak().small());
            });
        });

        // ===== Ribbon: the active category's tools as buttons =====
        if self.view == View::Tools {
            let ribbon_frame = egui::Frame::new()
                .fill(egui::Color32::from_rgb(0x1d, 0x1d, 0x1d))
                .inner_margin(egui::Margin::symmetric(10, 8));
            egui::TopBottomPanel::top("ribbon")
                .frame(ribbon_frame)
                .show(ctx, |ui| {
                    let mut ribbon_pick: Option<usize> = None;
                    ui.label(
                        egui::RichText::new(format!(
                            "TOOLS \u{2014} {}",
                            self.active_cat.to_uppercase()
                        ))
                        .color(GREEN)
                        .size(11.0),
                    );
                    // Cap the ribbon at three button rows; taller categories
                    // scroll inside the band so the workspace keeps its room
                    // (field finding on v0.6.0: Developer/Format's 7 rows
                    // squeezed Input/Output to nothing).
                    egui::ScrollArea::vertical()
                        .id_salt("ribbon-scroll")
                        .max_height(88.0)
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
                            if let Some((_, items)) =
                                menu_data.iter().find(|(c, _)| *c == self.active_cat)
                            {
                                let tool_labels: Vec<&str> =
                                    items.iter().map(|(_, n)| *n).collect();
                                for row in pack_rows(ui, &tool_labels, 168.0) {
                                    let row_labels: Vec<&str> =
                                        row.iter().map(|&i| tool_labels[i]).collect();
                                    let widths = row_widths(ui, &row_labels);
                                    let chunk: Vec<&(usize, &str)> =
                                        row.iter().map(|&i| &items[i]).collect();
                                    ui.horizontal(|ui| {
                                        for (k, (idx, name)) in chunk.iter().enumerate() {
                                            let w = widths[k];
                                            let is_sel = selected_now == Some(*idx);
                                            let mut btn = egui::Button::new(*name);
                                            if is_sel {
                                                btn = btn
                                                    .fill(GREEN_DIM)
                                                    .stroke(egui::Stroke::new(1.0_f32, GREEN));
                                            }
                                            if ui
                                                .add_sized([w, 26.0], btn)
                                                .on_hover_text(self.ops[*idx].description)
                                                .clicked()
                                            {
                                                ribbon_pick = Some(*idx);
                                            }
                                        }
                                    });
                                }
                            }
                        });
                    if let Some(idx) = ribbon_pick {
                        self.selected = Some(idx);
                        self.param.clear();
                        self.status.clear();
                    }
                });
        }

        if self.settings_dirty {
            self.save_settings();
            self.settings_dirty = false;
        }
        if self.view == View::Timers {
            egui::CentralPanel::default().show(ctx, |ui| {
                self.timers_ui(ui);
            });
            return;
        }
        if self.view == View::Pipelines {
            egui::CentralPanel::default().show(ctx, |ui| {
                self.pipelines_ui(ui);
            });
            return;
        }
        if self.view == View::RegexLab {
            egui::CentralPanel::default().show(ctx, |ui| {
                self.regex_lab_ui(ui);
            });
            return;
        }
        if self.view == View::Replace {
            egui::CentralPanel::default().show(ctx, |ui| {
                self.replace_ui(ui);
            });
            return;
        }
        if self.view == View::JsonTree {
            egui::CentralPanel::default().show(ctx, |ui| {
                self.json_tree_ui(ui);
            });
            return;
        }

        // ===== Bottom panel: the command buttons =====
        egui::TopBottomPanel::bottom("commands").show(ctx, |ui| {
            ui.add_space(4.0);
            let cmd_widths = row_widths(
                ui,
                &[
                    "RUN  (Ctrl+Enter)",
                    "Output \u{2192} Input (chain tools)",
                    "Copy output",
                    "\u{21A9} Undo",
                    "\u{21AA} Redo",
                    "Clear",
                ],
            );
            ui.horizontal(|ui| {
                if ui
                    .add_sized(
                        [cmd_widths[0], 28.0],
                        egui::Button::new(
                            egui::RichText::new("RUN  (Ctrl+Enter)")
                                .strong()
                                .color(GREEN),
                        ),
                    )
                    .on_hover_text("Run the selected tool on the Input text (Ctrl+Enter).")
                    .clicked()
                {
                    self.run_selected();
                }
                if ui
                    .add_sized(
                        [cmd_widths[1], 28.0],
                        egui::Button::new("Output → Input (chain tools)"),
                    )
                    .on_hover_text("Feed the Output back into the Input box, to chain another tool onto the result.")
                    .clicked()
                {
                    self.snapshot();
                    self.input = self.output.clone();
                    self.output.clear();
                }
                if ui
                    .add_sized([cmd_widths[2], 28.0], egui::Button::new("Copy output"))
                    .on_hover_text("Copy the Output box contents to the clipboard.")
                    .clicked()
                {
                    ui.ctx().copy_text(self.output.clone());
                    self.status = "Output copied".into();
                }
                if ui
                    .add_sized([cmd_widths[3], 28.0], egui::Button::new("\u{21A9} Undo"))
                    .on_hover_text("Undo the last whole action (Run, Clear, chaining, drops).")
                    .clicked()
                {
                    self.undo();
                }
                if ui
                    .add_sized([cmd_widths[4], 28.0], egui::Button::new("\u{21AA} Redo"))
                    .on_hover_text("Redo an action you undid.")
                    .clicked()
                {
                    self.redo();
                }
                if ui
                    .add_sized(
                        [cmd_widths[5], 28.0],
                        egui::Button::new(egui::RichText::new("Clear").color(REDT)),
                    )
                    .on_hover_text(
                        "Clear the Input, Parameter and Output boxes. Undo can bring them back.",
                    )
                    .clicked()
                {
                    self.snapshot();
                    self.input.clear();
                    self.output.clear();
                    self.param.clear();
                    self.status.clear();
                }
            });
            self.file_buttons(ui);
            ui.add_space(4.0);
        });

        // ===== History rail (Tools view, Terminal design) =====
        if self.view == View::Tools && self.show_history {
            egui::SidePanel::right("history")
                .resizable(false)
                .default_width(300.0)
                .show(ctx, |ui| {
                    self.history_ui(ui);
                });
        }

        // ===== Middle panel: selected tool + input/output =====
        egui::CentralPanel::default().show(ctx, |ui| {
            // Work tabs: several jobs side by side, each with its own state.
            // The row scrolls sideways, so many tabs never get cut off.
            let mut tab_action: Option<(usize, char)> = None;
            egui::ScrollArea::horizontal()
                .id_salt("worktabs")
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                for (n, tab) in self.tabs.iter().enumerate() {
                    let name = tab
                        .tool
                        .and_then(|i| self.ops.get(i))
                        .map(|o| o.name)
                        .unwrap_or("New tab");
                    if ui
                        .add(egui::Button::new(name).selected(n == self.tab_active))
                        .on_hover_text("Switch to this work tab. Each tab keeps its own tool and text.")
                        .clicked()
                    {
                        tab_action = Some((n, 's'));
                    }
                    if ui.small_button("\u{2715}").on_hover_text("Close this work tab").clicked() {
                        tab_action = Some((n, 'x'));
                    }
                }
                if ui.button("\u{FF0B} Tab").on_hover_text("Open a new work tab").clicked() {
                    tab_action = Some((usize::MAX, 'n'));
                }
                    });
                });
            match tab_action {
                Some((_, 'n')) => {
                    self.stash_tab();
                    self.tabs.push(WorkTab {
                        tool: self.selected,
                        ..Default::default()
                    });
                    let n = self.tabs.len() - 1;
                    self.load_tab(n);
                }
                Some((n, 'x')) => {
                    if self.tabs.len() > 1 {
                        self.tabs.remove(n);
                        let next = self.tab_active.min(self.tabs.len() - 1);
                        self.tab_active = usize::MAX;
                        self.load_tab(next);
                    } else {
                        self.tabs[0] = WorkTab {
                            tool: self.selected,
                            ..Default::default()
                        };
                        self.input.clear();
                        self.param.clear();
                        self.output.clear();
                    }
                }
                Some((n, 's')) => self.load_tab(n),
                _ => {}
            }
            if ui
                .checkbox(&mut self.paste_autorun, "\u{26A1} Auto-run on paste")
                .changed()
            {
                self.settings_dirty = true;
            }
            ui.separator();
            let Some(idx) = self.selected else {
                ui.label("Pick a tool from the category menus above.");
                return;
            };
            let op = self.ops[idx];
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(op.name.to_uppercase())
                        .strong()
                        .size(17.0)
                        .color(GREEN),
                );
                let is_fav = self.favorites.iter().any(|id| id == op.id);
                let star = if is_fav {
                    "\u{2605} In Favorites \u{2014} click to remove"
                } else {
                    "\u{2606} Add to Favorites"
                };
                if ui.button(star).on_hover_text("Add or remove this tool from the \u{2605} Favorites category.").clicked() {
                    if is_fav {
                        self.favorites.retain(|id| id != op.id);
                    } else {
                        self.favorites.push(op.id.to_string());
                    }
                    self.settings_dirty = true;
                }
            });
            ui.label(
                egui::RichText::new(format!("[{}]  id: {}", op.category, op.id))
                    .weak()
                    .small(),
            );
            ui.label(op.description);
            // DISPLAY box (Terminal design): the one-line instruction for
            // the current tool, mirroring CalcDex's display panel.
            egui::Frame::new()
                .stroke(egui::Stroke::new(1.0_f32, LINE))
                .inner_margin(egui::Margin::same(10))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(format!(
                            "DISPLAY \u{2014} {}",
                            op.name.to_uppercase()
                        ))
                        .weak()
                        .small(),
                    );
                    let line = match op.param_hint {
                        Some(hint) => format!(
                            "Fill in INPUT and the PARAMETER (format: {hint}), then press RUN (or Ctrl+Enter)."
                        ),
                        None => {
                            "Type or paste in INPUT, then press RUN (or Ctrl+Enter).".to_string()
                        }
                    };
                    ui.label(line);
                });
            ui.add_space(4.0);
            // Smart paste: if the input looks like a known kind of data,
            // offer the matching tools as one-click chips.
            if let Some((kind, idxs)) = self.suggestions() {
                let mut jump: Option<usize> = None;
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        egui::RichText::new(format!("Looks like {kind}:"))
                            .strong()
                            .color(GREEN),
                    );
                    for i in idxs {
                        if ui.button(self.ops[i].name).clicked() {
                            jump = Some(i);
                        }
                    }
                });
                if let Some(i) = jump {
                    self.selected = Some(i);
                    self.active_cat = self.ops[i].category.to_string();
                    self.param.clear();
                    self.run_selected();
                }
            }
            // Snippet library: reusable inputs, saved between sessions.
            ui.horizontal(|ui| {
                ui.label("Snippets:");
                let names: Vec<&str> = self.snippets.iter().map(|(n, _)| n.as_str()).collect();
                let mut sel = self.snip_sel.min(names.len().saturating_sub(1));
                egui::ComboBox::from_id_salt("snip_box")
                    .selected_text(names.get(sel).copied().unwrap_or("\u{2014} none \u{2014}"))
                    .show_ui(ui, |ui| {
                        for (i, n) in names.iter().enumerate() {
                            ui.selectable_value(&mut sel, i, *n);
                        }
                    });
                self.snip_sel = sel;
                if ui.button("Insert").clicked() {
                    if let Some((_, txt)) = self.snippets.get(self.snip_sel) {
                        let txt = txt.clone();
                        self.snapshot();
                        self.input = txt;
                    }
                }
                ui.add(
                    egui::TextEdit::singleline(&mut self.snip_name)
                        .desired_width((ui.available_width() - 470.0).max(130.0))
                        .hint_text("name for Save"),
                );
                if ui.button("Save input").clicked() {
                    let name = if self.snip_name.trim().is_empty() {
                        format!("Snippet {}", self.snippets.len() + 1)
                    } else {
                        self.snip_name.trim().to_string()
                    };
                    if let Some(existing) = self.snippets.iter_mut().find(|(n, _)| *n == name) {
                        existing.1 = self.input.clone();
                    } else {
                        self.snippets.push((name, self.input.clone()));
                    }
                    self.settings_dirty = true;
                    self.status = "Snippet saved".into();
                }
                if ui.button("Delete").clicked() && !self.snippets.is_empty() {
                    let n = self.snip_sel.min(self.snippets.len() - 1);
                    self.snippets.remove(n);
                    self.snip_sel = 0;
                    self.settings_dirty = true;
                }
            });
            ui.add_space(4.0);
            ui.columns(2, |cols| {
                cols[0].label(egui::RichText::new("INPUT").color(GREEN).size(11.0));
                cols[0].add(
                    egui::TextEdit::multiline(&mut self.input)
                        .desired_rows(18)
                        .desired_width(f32::INFINITY)
                        .code_editor(),
                );
                cols[0].add_space(6.0);
                cols[0].label(egui::RichText::new("PARAMETER").color(GREEN).size(11.0));
                if let Some(hint) = op.param_hint {
                    cols[0].add(
                        egui::TextEdit::singleline(&mut self.param)
                            .desired_width(f32::INFINITY)
                            .hint_text(hint),
                    );
                    cols[0].label(
                        egui::RichText::new(format!(
                            "Format: {hint} \u{2014} separate multiple values with |||"
                        ))
                        .small()
                        .weak(),
                    );
                } else {
                    cols[0].label(
                        egui::RichText::new("\u{2014} no parameter for this tool \u{2014}").weak(),
                    );
                }
                cols[1].label(egui::RichText::new("OUTPUT").color(GREEN).size(11.0));
                let mut out = self.output.clone();
                if cols[1]
                    .add(
                        egui::TextEdit::multiline(&mut out)
                            .desired_rows(18)
                            .desired_width(f32::INFINITY)
                            .code_editor(),
                    )
                    .changed()
                {
                    self.output = out;
                }
                cols[1].add_space(6.0);
                cols[1].label(egui::RichText::new("STATUS").color(GREEN).size(11.0));
                if !self.status.is_empty() {
                    let col = if self.status.starts_with('\u{26A0}') {
                        REDT
                    } else {
                        GREEN
                    };
                    cols[1].label(egui::RichText::new(&self.status).color(col));
                }
            });
        });

        if self.autorun_pending {
            self.autorun_pending = false;
            if self.selected.is_some() {
                self.run_selected();
            }
        }
        if ctx.input(|s| s.modifiers.ctrl && s.key_pressed(egui::Key::Enter)) {
            self.run_selected();
        }
    }
}

/// Render one JSON value as a fold-out tree; clicking "copy path" on a
/// scalar reports its JSONPath through `clicked`.
fn json_tree_node(
    ui: &mut egui::Ui,
    v: &serde_json::Value,
    label: &str,
    path: &str,
    clicked: &mut Option<String>,
) {
    use serde_json::Value;
    match v {
        Value::Object(map) => {
            egui::CollapsingHeader::new(format!("{label}  {{{}}}", map.len()))
                .id_salt(path)
                .show(ui, |ui| {
                    for (k, val) in map {
                        let child = if k.chars().all(|c| c.is_alphanumeric() || c == '_') {
                            format!("{path}.{k}")
                        } else {
                            format!("{path}[\"{k}\"]")
                        };
                        json_tree_node(ui, val, k, &child, clicked);
                    }
                });
        }
        Value::Array(items) => {
            egui::CollapsingHeader::new(format!("{label}  [{}]", items.len()))
                .id_salt(path)
                .show(ui, |ui| {
                    for (i, val) in items.iter().enumerate() {
                        let child = format!("{path}[{i}]");
                        json_tree_node(ui, val, &format!("[{i}]"), &child, clicked);
                    }
                });
        }
        _ => {
            ui.horizontal(|ui| {
                let val = match v {
                    Value::String(s) => format!("\"{s}\""),
                    Value::Null => "null".to_string(),
                    other => other.to_string(),
                };
                ui.label(egui::RichText::new(format!("{label}:")).weak());
                ui.label(egui::RichText::new(val).monospace());
                if ui.small_button("copy path").clicked() {
                    *clicked = Some(path.to_string());
                }
            });
        }
    }
}
