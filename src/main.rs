use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::os::unix::io::AsRawFd;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use ksni::{self, Icon, MenuItem, ToolTip, Tray};
use rdev::{listen, Button, Event, EventType, Key};

static ENABLED: AtomicBool = AtomicBool::new(true);
static BUSY: AtomicBool = AtomicBool::new(false);
static CAPTURE_MODE: AtomicBool = AtomicBool::new(false);
static LANG_EN: AtomicBool = AtomicBool::new(true);

static CONFIG: OnceLock<Mutex<Config>> = OnceLock::new();

fn tr(en: &'static str, ru: &'static str) -> &'static str {
    if LANG_EN.load(Ordering::Relaxed) {
        en
    } else {
        ru
    }
}

struct PressState {
    last: Option<Instant>,
}

#[derive(Clone)]
struct Config {
    trigger: Trigger,
    timeout_ms: u64,
}

#[derive(Clone, PartialEq)]
enum Trigger {
    Key(Key),
    Button(Button),
}

fn trigger_label(t: &Trigger) -> String {
    match t {
        Trigger::Key(k) => key_label(k),
        Trigger::Button(b) => button_label(b),
    }
}

fn key_label(k: &Key) -> String {
    (match k {
        Key::Alt => "Alt",
        Key::AltGr => "Alt Gr",
        Key::Backspace => "Backspace",
        Key::CapsLock => "Caps Lock",
        Key::ControlLeft => "Левый Ctrl",
        Key::ControlRight => "Правый Ctrl",
        Key::Delete => "Delete",
        Key::DownArrow => "Стрелка вниз",
        Key::End => "End",
        Key::Escape => "Escape",
        Key::Home => "Home",
        Key::Insert => "Insert",
        Key::LeftArrow => "Стрелка влево",
        Key::MetaLeft => "Левый Super (Win)",
        Key::MetaRight => "Правый Super (Win)",
        Key::PageDown => "Page Down",
        Key::PageUp => "Page Up",
        Key::Return => "Enter",
        Key::RightArrow => "Стрелка вправо",
        Key::ShiftLeft => "Левый Shift",
        Key::ShiftRight => "Правый Shift",
        Key::Space => "Пробел",
        Key::Tab => "Tab",
        Key::UpArrow => "Стрелка вверх",
        Key::PrintScreen => "Print Screen",
        Key::ScrollLock => "Scroll Lock",
        Key::Pause => "Pause",
        Key::NumLock => "Num Lock",
        Key::BackQuote => "`",
        Key::Minus => "-",
        Key::Equal => "=",
        Key::BackSlash => "\\",
        Key::LeftBracket => "[",
        Key::RightBracket => "]",
        Key::SemiColon => ";",
        Key::Quote => "'",
        Key::Comma => ",",
        Key::Dot => ".",
        Key::Slash => "/",
        Key::IntlBackslash => "\\ (ISO)",
        Key::Function => "Fn",
        Key::F1 => "F1",
        Key::F2 => "F2",
        Key::F3 => "F3",
        Key::F4 => "F4",
        Key::F5 => "F5",
        Key::F6 => "F6",
        Key::F7 => "F7",
        Key::F8 => "F8",
        Key::F9 => "F9",
        Key::F10 => "F10",
        Key::F11 => "F11",
        Key::F12 => "F12",
        Key::KeyA => "A",
        Key::KeyB => "B",
        Key::KeyC => "C",
        Key::KeyD => "D",
        Key::KeyE => "E",
        Key::KeyF => "F",
        Key::KeyG => "G",
        Key::KeyH => "H",
        Key::KeyI => "I",
        Key::KeyJ => "J",
        Key::KeyK => "K",
        Key::KeyL => "L",
        Key::KeyM => "M",
        Key::KeyN => "N",
        Key::KeyO => "O",
        Key::KeyP => "P",
        Key::KeyQ => "Q",
        Key::KeyR => "R",
        Key::KeyS => "S",
        Key::KeyT => "T",
        Key::KeyU => "U",
        Key::KeyV => "V",
        Key::KeyW => "W",
        Key::KeyX => "X",
        Key::KeyY => "Y",
        Key::KeyZ => "Z",
        Key::Num0 => "0",
        Key::Num1 => "1",
        Key::Num2 => "2",
        Key::Num3 => "3",
        Key::Num4 => "4",
        Key::Num5 => "5",
        Key::Num6 => "6",
        Key::Num7 => "7",
        Key::Num8 => "8",
        Key::Num9 => "9",
        Key::Kp0 => "Numpad 0",
        Key::Kp1 => "Numpad 1",
        Key::Kp2 => "Numpad 2",
        Key::Kp3 => "Numpad 3",
        Key::Kp4 => "Numpad 4",
        Key::Kp5 => "Numpad 5",
        Key::Kp6 => "Numpad 6",
        Key::Kp7 => "Numpad 7",
        Key::Kp8 => "Numpad 8",
        Key::Kp9 => "Numpad 9",
        Key::KpDivide => "Numpad /",
        Key::KpMultiply => "Numpad *",
        Key::KpMinus => "Numpad -",
        Key::KpPlus => "Numpad +",
        Key::KpReturn => "Numpad Enter",
        Key::KpDelete => "Numpad .",
        _ => return format!("{:?}", k),
    })
    .to_string()
}

fn button_label(b: &Button) -> String {
    (match b {
        Button::Left => "ЛКМ",
        Button::Right => "ПКМ",
        Button::Middle => "нажатие колёсика",
        Button::Unknown(_) => "неизвестная кнопка",
    })
    .to_string()
}

fn trigger_to_str(t: &Trigger) -> String {
    match t {
        Trigger::Key(k) => format!("{:?}", k),
        Trigger::Button(b) => format!("m:{}", button_to_str(b)),
    }
}

fn button_to_str(b: &Button) -> &'static str {
    match b {
        Button::Left => "Left",
        Button::Right => "Right",
        Button::Middle => "Middle",
        Button::Unknown(_) => "Unknown",
    }
}

fn str_to_button(s: &str) -> Option<Button> {
    match s {
        "Left" => Some(Button::Left),
        "Right" => Some(Button::Right),
        "Middle" => Some(Button::Middle),
        _ => None,
    }
}

macro_rules! match_key {
    ($s:expr, $( $k:ident ),+ $(,)?) => {
        match $s {
            $( stringify!($k) => Some(Key::$k), )+
            _ => None,
        }
    };
}

fn str_to_key(s: &str) -> Option<Key> {
    match_key!(s,
        Alt, AltGr, Backspace, CapsLock, ControlLeft, ControlRight,
        Delete, DownArrow, End, Escape,
        F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
        Home, LeftArrow, MetaLeft, MetaRight,
        PageDown, PageUp, Return, RightArrow,
        ShiftLeft, ShiftRight, Space, Tab, UpArrow,
        PrintScreen, ScrollLock, Pause, NumLock,
        BackQuote, Num1, Num2, Num3, Num4, Num5,
        Num6, Num7, Num8, Num9, Num0,
        Minus, Equal, BackSlash, LeftBracket, RightBracket,
        Quote, SemiColon, Comma, Dot, Slash,
        KeyA, KeyB, KeyC, KeyD, KeyE, KeyF, KeyG, KeyH, KeyI, KeyJ,
        KeyK, KeyL, KeyM, KeyN, KeyO, KeyP, KeyQ, KeyR, KeyS, KeyT,
        KeyU, KeyV, KeyW, KeyX, KeyY, KeyZ,
        IntlBackslash, Function, Insert,
        KpDivide, KpMultiply, KpMinus, KpPlus, KpReturn,
        Kp1, Kp2, Kp3, Kp4, Kp5, Kp6, Kp7, Kp8, Kp9, Kp0, KpDelete,
    )
}

fn parse_trigger(s: &str) -> Trigger {
    if let Some(btn) = s.strip_prefix("m:") {
        str_to_button(btn)
            .map(Trigger::Button)
            .unwrap_or(Trigger::Key(Key::ShiftLeft))
    } else {
        str_to_key(s)
            .map(Trigger::Key)
            .unwrap_or(Trigger::Key(Key::ShiftLeft))
    }
}

fn config_path() -> PathBuf {
    let mut path = if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home)
    } else {
        PathBuf::from(".")
    };
    path.push(".config/raskladka/config");
    path
}

fn load_config() -> Config {
    let path = config_path();
    let mut trigger = Trigger::Key(Key::ShiftLeft);
    let mut timeout_ms = 400u64;

    if let Ok(data) = fs::read_to_string(&path) {
        for line in data.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((k, v)) = line.split_once('=') {
                match k.trim() {
                    "trigger" => trigger = parse_trigger(v.trim()),
                    "timeout_ms" => timeout_ms = v.trim().parse().unwrap_or(400),
                    _ => {}
                }
            }
        }
    }

    Config { trigger, timeout_ms }
}

fn save_config(cfg: &Config) {
    let path = config_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let s = format!(
        "# raskladka config\n\
         trigger={}\n\
         timeout_ms={}\n",
        trigger_to_str(&cfg.trigger),
        cfg.timeout_ms,
    );
    let _ = fs::write(&path, s);
}

fn read_config() -> Config {
    CONFIG.get().map(|m| m.lock().unwrap().clone()).unwrap_or_else(|| {
        let cfg = load_config();
        let _ = CONFIG.set(Mutex::new(cfg.clone()));
        cfg
    })
}

fn write_config(cfg: &Config) {
    save_config(cfg);
    if let Some(m) = CONFIG.get() {
        *m.lock().unwrap() = cfg.clone();
    }
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
    let press_state = Mutex::new(PressState { last: None });

    let callback = move |event: Event| {
        let event_type = &event.event_type;

        if CAPTURE_MODE.load(Ordering::Acquire) {
            let captured = match event_type {
                EventType::KeyPress(k) => Some(Trigger::Key(*k)),
                EventType::ButtonPress(b) if *b != Button::Left && *b != Button::Right => {
                    Some(Trigger::Button(*b))
                }
                _ => None,
            };

            if let Some(trigger) = captured {
                CAPTURE_MODE.store(false, Ordering::Release);
                let cfg = Config {
                    trigger,
                    timeout_ms: 400,
                };
                write_config(&cfg);
            }
            return;
        }

        if !ENABLED.load(Ordering::Relaxed) {
            return;
        }

        let cfg = read_config();

        let matches = match &cfg.trigger {
            Trigger::Key(k) => matches!(event_type, EventType::KeyPress(ev) if *ev == *k),
            Trigger::Button(b) => matches!(event_type, EventType::ButtonPress(ev) if *ev == *b),
        };

        if matches {
            let mut s = press_state.lock().unwrap();
            let now = Instant::now();
            let should = s.last.map_or(false, |t| {
                now.duration_since(t).as_millis() < cfg.timeout_ms as u128
            });
            s.last = Some(now);
            drop(s);
            if should {
                std::thread::spawn(|| {
                    trigger_convert();
                });
            }
        }
    };

    if let Err(e) = listen(callback) {
        eprintln!("ERR: {:?}", e);
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
        let title = if CAPTURE_MODE.load(Ordering::Relaxed) {
            tr("press a key...", "нажмите клавишу...")
        } else if ENABLED.load(Ordering::Relaxed) {
            tr("on", "вкл")
        } else {
            tr("off", "вык")
        };
        ToolTip {
            title: title.into(),
            ..Default::default()
        }
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        if !CAPTURE_MODE.load(Ordering::Relaxed) {
            ENABLED.fetch_xor(true, Ordering::SeqCst);
        }
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let toggle_label = if ENABLED.load(Ordering::Relaxed) {
            tr("off", "вык")
        } else {
            tr("on", "вкл")
        };
        let lang_label = if LANG_EN.load(Ordering::Relaxed) {
            "Ru"
        } else {
            "En"
        };
        let cfg = read_config();
        let hotkey_label = format!(
            "{}: {}",
            tr("rebind key", "выбрать клавишу"),
            trigger_label(&cfg.trigger)
        );
        vec![
            MenuItem::Standard(ksni::menu::StandardItem {
                label: toggle_label.into(),
                activate: Box::new(|_: &mut Self| {
                    ENABLED.fetch_xor(true, Ordering::SeqCst);
                }),
                ..Default::default()
            }),
            MenuItem::Standard(ksni::menu::StandardItem {
                label: format!("{} ({})", tr("language", "язык"), lang_label).into(),
                activate: Box::new(|_: &mut Self| {
                    LANG_EN.fetch_xor(true, Ordering::SeqCst);
                }),
                ..Default::default()
            }),
            MenuItem::Standard(ksni::menu::StandardItem {
                label: hotkey_label.into(),
                activate: Box::new(|_: &mut Self| {
                    CAPTURE_MODE.store(true, Ordering::Release);
                }),
                ..Default::default()
            }),
            MenuItem::Standard(ksni::menu::StandardItem {
                label: tr("exit", "выйти").into(),
                activate: Box::new(|_: &mut Self| {
                    std::process::exit(0);
                }),
                ..Default::default()
            }),
        ]
    }
}

fn lock_singleton() -> std::fs::File {
    let mut path = if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home)
    } else {
        PathBuf::from("/tmp")
    };
    path.push(".config/raskladka/lock");
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let file = fs::File::create(&path).expect("cannot create lock file");
    let fd = file.as_raw_fd();
    let ret = unsafe { libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) };
    if ret != 0 {
        eprintln!("raskladka: another instance is already running");
        std::process::exit(1);
    }
    file
}

fn main() {
    let _lock = lock_singleton();
    let cfg = load_config();
    let _ = CONFIG.set(Mutex::new(cfg));

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
