use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use ksni::{self, Icon, MenuItem, ToolTip, Tray};
use rdev::{listen, Event, EventType, Key};

static ENABLED: AtomicBool = AtomicBool::new(true);
static BUSY: AtomicBool = AtomicBool::new(false);

struct State {
    last_shift: Option<Instant>,
}

struct Layout {
    en_to_ru: HashMap<char, char>,
    ru_to_en: HashMap<char, char>,
    ru_chars: HashSet<char>,
}

fn build_layout() -> Layout {
    let en = "qwertyuiop[]asdfghjkl;'zxcvbnm,./";
    let ru = "йцукенгшщзхъфывапролджэячсмитьбю.";
    let en_upper = "QWERTYUIOP{}ASDFGHJKL:\"ZXCVBNM<>?";
    let ru_upper = "ЙЦУКЕНГШЩЗХЪФЫВАПРОЛДЖЭЯЧСМИТЬБЮ,";

    let mut en_to_ru = HashMap::new();
    let mut ru_to_en = HashMap::new();
    let mut ru_chars = HashSet::new();

    for (e, r) in en.chars().zip(ru.chars()) {
        en_to_ru.insert(e, r);
        ru_to_en.insert(r, e);
        ru_chars.insert(r);
    }
    for (e, r) in en_upper.chars().zip(ru_upper.chars()) {
        en_to_ru.insert(e, r);
        ru_to_en.insert(r, e);
        ru_chars.insert(r);
    }

    Layout { en_to_ru, ru_to_en, ru_chars }
}

fn convert(text: &str, layout: &Layout) -> String {
    let mut en = 0usize;
    let mut ru = 0usize;
    for c in text.chars() {
        if layout.ru_chars.contains(&c) {
            ru += 1;
        } else if c.is_ascii_alphabetic() || "[]{};':\",./<>?".contains(c) {
            en += 1;
        }
    }
    let from_ru = ru > en;

    text.chars()
        .map(|c| {
            if from_ru {
                layout.ru_to_en.get(&c).copied().unwrap_or(c)
            } else {
                layout.en_to_ru.get(&c).copied().unwrap_or(c)
            }
        })
        .collect()
}

enum Backend {
    X11,
    Wayland,
}

fn detect_backend() -> Backend {
    if std::env::var("WAYLAND_DISPLAY").is_ok()
        || std::env::var("XDG_SESSION_TYPE").map_or(false, |v| v == "wayland")
    {
        Backend::Wayland
    } else {
        Backend::X11
    }
}

fn run_cmd(cmd: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(cmd).args(args).output().ok()?;
    if out.status.success() {
        String::from_utf8(out.stdout).ok()
    } else {
        None
    }
}

fn pipe_to_cmd(cmd: &str, args: &[&str], text: &str) {
    let mut child = Command::new(cmd)
        .args(args)
        .stdin(std::process::Stdio::piped())
        .spawn()
        .expect(&format!("failed to spawn {}", cmd));
    let _ = child.stdin.take().unwrap().write_all(text.as_bytes());
    let _ = child.wait();
}

fn trigger_convert() {
    if BUSY.swap(true, Ordering::SeqCst) {
        return;
    }

    let backend = detect_backend();
    let layout = build_layout();

    match backend {
        Backend::X11 => {
            let saved = run_cmd("xclip", &["-o", "-selection", "clipboard"]);

            let _ = Command::new("xdotool")
                .args(["key", "--clearmodifiers", "ctrl+c"])
                .output();

            std::thread::sleep(Duration::from_millis(80));

            let text = run_cmd("xclip", &["-o", "-selection", "clipboard"]);

            if let Some(ref t) = text {
                if !t.trim().is_empty() && saved.as_deref() != Some(t.as_str()) {
                    let converted = convert(t, &layout);
                    if converted != *t {
                        pipe_to_cmd("xclip", &["-i", "-selection", "clipboard"], &converted);
                        std::thread::sleep(Duration::from_millis(50));

                        let _ = Command::new("xdotool")
                            .args(["key", "--clearmodifiers", "ctrl+v"])
                            .output();
                        std::thread::sleep(Duration::from_millis(100));
                    }
                }
            }

            if let Some(prev) = saved {
                std::thread::sleep(Duration::from_millis(50));
                pipe_to_cmd("xclip", &["-i", "-selection", "clipboard"], &prev);
            }
        }
        Backend::Wayland => {
            let saved = run_cmd("wl-paste", &[]);

            let text = run_cmd("wl-paste", &["--primary"]);

            if let Some(ref t) = text {
                if !t.trim().is_empty() {
                    let converted = convert(t, &layout);
                    if converted != *t {
                        pipe_to_cmd("wl-copy", &[], &converted);
                        std::thread::sleep(Duration::from_millis(50));

                        let _ = Command::new("wtype")
                            .args(["-s", "ctrl+v"])
                            .output();
                        std::thread::sleep(Duration::from_millis(100));
                    }
                }
            }

            if let Some(prev) = saved {
                std::thread::sleep(Duration::from_millis(50));
                pipe_to_cmd("wl-copy", &[], &prev);
            }
        }
    }

    BUSY.store(false, Ordering::SeqCst);
}

fn run_rdev_listener() {
    let state = Mutex::new(State { last_shift: None });
    let double_shift_ms: u64 = 400;

    let callback = move |event: Event| {
        if !ENABLED.load(Ordering::Relaxed) {
            return;
        }
        if let EventType::KeyPress(Key::ShiftLeft) | EventType::KeyPress(Key::ShiftRight) =
            event.event_type
        {
            let mut s = state.lock().unwrap();
            let now = Instant::now();
            let should = s.last_shift.map_or(false, |t| {
                now.duration_since(t).as_millis() < double_shift_ms as u128
            });
            s.last_shift = Some(now);
            drop(s);
            if should {
                std::thread::spawn(|| {
                    trigger_convert();
                });
            }
        }
    };

    if let Err(e) = listen(callback) {
        eprintln!("Ошибка: {:?}", e);
    }
}

fn render_svg(svg_data: &[u8], size: u32) -> Vec<u8> {
    let opt = resvg::usvg::Options::default();
    let rtree = resvg::usvg::Tree::from_data(svg_data, &opt).unwrap();
    let sz = rtree.size();
    let (w, h) = (sz.width() as f64, sz.height() as f64);

    let scale = size as f64 / h.max(w);
    let mut pixmap = resvg::tiny_skia::Pixmap::new(size, size).unwrap();
    let ts = resvg::tiny_skia::Transform::from_scale(scale as f32, scale as f32);
    resvg::render(&rtree, ts, &mut pixmap.as_mut());
    pixmap.data().to_vec()
}

struct RaskladkaTray {
    on_rgba: Vec<u8>,
    off_rgba: Vec<u8>,
}

impl Tray for RaskladkaTray {
    fn icon_pixmap(&self) -> Vec<Icon> {
        let pixels = if ENABLED.load(Ordering::Relaxed) {
            &self.on_rgba
        } else {
            &self.off_rgba
        };
        vec![Icon {
            width: 24,
            height: 24,
            data: pixels.clone(),
        }]
    }

    fn tool_tip(&self) -> ToolTip {
        let title = if ENABLED.load(Ordering::Relaxed) {
            "raskladka: включена"
        } else {
            "raskladka: выключена"
        };
        ToolTip {
            title: title.into(),
            ..Default::default()
        }
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        ENABLED.fetch_xor(true, Ordering::SeqCst);
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let label = if ENABLED.load(Ordering::Relaxed) {
            "Выключить"
        } else {
            "Включить"
        };
        vec![
            MenuItem::Standard(ksni::menu::StandardItem {
                label: label.into(),
                activate: Box::new(|_: &mut Self| {
                    ENABLED.fetch_xor(true, Ordering::SeqCst);
                }),
                ..Default::default()
            }),
            MenuItem::Standard(ksni::menu::StandardItem {
                label: "Выход".into(),
                activate: Box::new(|_: &mut Self| {
                    std::process::exit(0);
                }),
                ..Default::default()
            }),
        ]
    }
}

fn main() {
    let on_rgba = render_svg(include_bytes!("../on.svg"), 24);
    let off_rgba = render_svg(include_bytes!("../off.svg"), 24);

    let tray = RaskladkaTray { on_rgba, off_rgba };
    ksni::TrayService::new(tray).spawn();

    std::thread::spawn(|| {
        run_rdev_listener();
    });

    loop {
        std::thread::sleep(Duration::from_secs(3600));
    }
}
