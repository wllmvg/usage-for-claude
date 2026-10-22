#![windows_subsystem = "windows"]

mod config;
mod draw;
mod usage;

use config::{Accent, Config, Shape, Theme, TrayMetric};
use std::cell::Cell;
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use usage::{FetchError, Limit, Usage};
use windows::core::{w, Result, HSTRING};
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Dwm::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::ProcessStatus::*;
use windows::Win32::System::Threading::{CreateMutexW, GetCurrentProcess};
use windows::Win32::UI::HiDpi::*;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_ESCAPE;
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::WindowsAndMessaging::*;

const WM_TRAY: u32 = WM_APP + 1;
const WM_DATA: u32 = WM_APP + 2;
const POLL: usize = 1;
const LIVE: usize = 2;
const POLL_MS: u32 = 300_000;
const NIN_KEYSELECT: u32 = WM_USER + 1;

struct App {
    hwnd: HWND,
    cfg: Config,
    r: draw::Renderer,
    usage: Option<Usage>,
    updated: Option<i64>,
    error: Option<FetchError>,
    loading: bool,
    levels: Option<[u8; 3]>,
    icon: HICON,
    hidden_at: Instant,
    inbox: Arc<Mutex<Option<std::result::Result<Usage, FetchError>>>>,
    taskbar_created: u32,
}

thread_local! { static APP: Cell<*mut App> = const { Cell::new(std::ptr::null_mut()) }; }

fn app() -> Option<&'static mut App> {
    unsafe { APP.with(|a| a.get().as_mut()) }
}

fn now() -> i64 { SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0) }

fn level(pct: f32, cfg: &Config) -> u8 {
    if pct >= cfg.crit_at { 2 } else if pct >= cfg.warn_at { 1 } else { 0 }
}

fn ram_mb() -> f64 {
    let mut c = PROCESS_MEMORY_COUNTERS_EX::default();
    unsafe {
        let _ = GetProcessMemoryInfo(GetCurrentProcess(), &mut c as *mut _ as *mut PROCESS_MEMORY_COUNTERS, std::mem::size_of_val(&c) as u32);
    }
    c.PrivateUsage as f64 / 1_048_576.0
}

fn is_dark(cfg: &Config) -> bool {
    match cfg.theme {
        Theme::Claro => false,
        Theme::Oscuro => true,
        Theme::Sistema => config::theme_dword(w!("AppsUseLightTheme")) == Some(0),
    }
}

fn copy_w(dst: &mut [u16], s: &str) {
    let v: Vec<u16> = s.encode_utf16().take(dst.len() - 1).collect();
    dst[..v.len()].copy_from_slice(&v);
    dst[v.len()] = 0;
}

fn nid(hwnd: HWND) -> NOTIFYICONDATAW {
    NOTIFYICONDATAW { cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32, hWnd: hwnd, uID: 1, ..Default::default() }
}

fn limits(u: &Usage) -> [(&'static str, &'static str, Option<Limit>, i64); 3] {
    [
        ("Sesión", "ventana de 5 h", u.session, 5 * 3600),
        ("Semana", "todos los modelos", u.week, 7 * 86400),
        ("Opus", "semanal", u.opus, 7 * 86400),
    ]
}

impl App {
    fn view(&self) -> draw::View {
        let (cfg, t) = (&self.cfg, now());
        let shown = [cfg.show_session, cfg.show_week, cfg.show_opus];
        let pace_on = cfg.show_pace && cfg.shape == Shape::Barras;
        let mut rows = Vec::new();
        if let Some(u) = &self.usage {
            for (i, (label, sub, lim, span)) in limits(u).into_iter().enumerate() {
                let Some(l) = lim else { continue };
                if !shown[i] { continue; }
                let pace = l.resets_at.map(|at| (100.0 * (1.0 - (at - t) as f32 / span as f32)).clamp(0.0, 100.0));
                let lv = level(l.pct, cfg);
                let status = match lv {
                    2 => "Casi agotado",
                    1 => "Cerca del límite",
                    _ if pace_on && pace.is_some_and(|p| l.pct > p + 5.0) => "Por encima del ritmo",
                    _ => "",
                };
                let reset = if cfg.show_reset { l.resets_at.map(|at| usage::reset_text(at, t)).unwrap_or_default() } else { String::new() };
                let meta = [reset.as_str(), status].into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" · ");
                rows.push(draw::Row { label, sub, pct: l.pct, pace: if pace_on { pace } else { None }, meta: if meta.is_empty() { sub.into() } else { meta }, level: lv });
            }
        }
        let message = if !rows.is_empty() {
            None
        } else if let Some(e) = &self.error {
            Some(match e {
                FetchError::RateLimited(_) => FetchError::RateLimited(self.cfg.quiet_until - t).message(),
                e => e.message(),
            })
        } else if self.usage.is_none() {
            Some(("Cargando…".into(), String::new()))
        } else {
            Some(("Nada que mostrar".into(), "Activa alguna métrica en Ajustes ▸ Mostrar.".into()))
        };
        let footer_left = match (&self.error, self.updated) {
            (Some(_), Some(_)) if !rows.is_empty() => "Sin actualizar · reintentando".into(),
            (_, Some(u)) if (t - u) < 60 => "Actualizado ahora".into(),
            (_, Some(u)) => format!("Actualizado hace {} min", (t - u) / 60),
            _ => String::new(),
        };
        let footer_right = if cfg.show_ram { format!("{:.1} MB de RAM", ram_mb()).replace('.', ",") } else { String::new() };
        draw::View { rows, message, footer_left, footer_right }
    }

    fn refresh(&mut self) {
        if self.loading || now() < self.cfg.quiet_until { return; }
        self.loading = true;
        let (inbox, h) = (self.inbox.clone(), self.hwnd.0 as isize);
        std::thread::spawn(move || {
            let r = usage::fetch();
            *inbox.lock().unwrap() = Some(r);
            unsafe { let _ = PostMessageW(HWND(h as _), WM_DATA, WPARAM(0), LPARAM(0)); }
        });
    }

    fn on_data(&mut self) {
        self.loading = false;
        let Some(r) = self.inbox.lock().unwrap().take() else { return };
        match r {
            Ok(u) => {
                self.notify_crossings(&u);
                self.usage = Some(u);
                self.updated = Some(now());
                self.error = None;
            }
            Err(e) => {
                if let FetchError::RateLimited(s) = e {
                    self.cfg.quiet_until = now() + s;
                    config::save(&self.cfg);
                }
                self.error = Some(e);
            }
        }
        self.update_tray();
        self.redraw();
    }

    fn notify_crossings(&mut self, u: &Usage) {
        let new = limits(u).map(|(_, _, l, _)| l.map_or(0, |l| level(l.pct, &self.cfg)));
        if let (Some(old), true) = (self.levels, self.cfg.notify) {
            for (i, (label, _, l, _)) in limits(u).into_iter().enumerate() {
                let Some(l) = l else { continue };
                if new[i] > old[i] {
                    let mut n = nid(self.hwnd);
                    n.uFlags = NIF_INFO;
                    copy_w(&mut n.szInfoTitle, &format!("{label} al {:.0}%", l.pct));
                    let body = l.resets_at.map(|at| usage::reset_text(at, now())).unwrap_or_default();
                    copy_w(&mut n.szInfo, if body.is_empty() { "Cerca del límite" } else { &body });
                    n.dwInfoFlags = NIIF_INFO | NIIF_RESPECT_QUIET_TIME;
                    unsafe { let _ = Shell_NotifyIconW(NIM_MODIFY, &n); }
                    break;
                }
            }
        }
        self.levels = Some(new);
    }

    fn add_tray(&mut self) {
        let mut n = nid(self.hwnd);
        n.uFlags = NIF_MESSAGE;
        n.uCallbackMessage = WM_TRAY;
        unsafe {
            let _ = Shell_NotifyIconW(NIM_ADD, &n);
            n.Anonymous.uVersion = NOTIFYICON_VERSION_4;
            let _ = Shell_NotifyIconW(NIM_SETVERSION, &n);
        }
        self.update_tray();
    }

    fn update_tray(&mut self) {
        let light_bar = config::theme_dword(w!("SystemUsesLightTheme")) == Some(1);
        let p = draw::palette(!light_bar, self.cfg.accent);
        let pick = self.usage.as_ref().and_then(|u| match self.cfg.tray_metric {
            TrayMetric::Sesion => u.session,
            TrayMetric::Semana => u.week,
            TrayMetric::Mayor => [u.session, u.week, u.opus].into_iter().flatten().max_by(|a, b| a.pct.total_cmp(&b.pct)),
        });
        let (text, pct, color) = match pick {
            Some(l) => (format!("{:.0}", l.pct.min(100.0)), l.pct, [p.fg, p.warn, p.crit][level(l.pct, &self.cfg) as usize]),
            None => (String::new(), 0.0, p.fg),
        };
        let tip = match (&self.usage, &self.error) {
            (Some(u), _) => limits(u)
                .into_iter()
                .filter_map(|(label, _, l, _)| l.map(|l| format!("{label} {:.0}%", l.pct)))
                .collect::<Vec<_>>()
                .join(" · "),
            (None, Some(e)) => e.message().0,
            _ => "Tally · cargando".into(),
        };
        let size = unsafe { GetSystemMetrics(SM_CXSMICON) };
        let mut track = p.fg;
        track.a = 0.3;
        let Ok(icon) = self.r.tray_icon(size, &text, pct, &color, &track) else { return };
        let mut n = nid(self.hwnd);
        n.uFlags = NIF_ICON | NIF_TIP | NIF_SHOWTIP;
        n.hIcon = icon;
        copy_w(&mut n.szTip, &tip);
        unsafe {
            let _ = Shell_NotifyIconW(NIM_MODIFY, &n);
            if !self.icon.is_invalid() {
                let _ = DestroyIcon(self.icon);
            }
        }
        self.icon = icon;
    }

    fn visible(&self) -> bool { unsafe { IsWindowVisible(self.hwnd).as_bool() } }

    fn redraw(&mut self) {
        if self.visible() {
            self.position();
            unsafe { let _ = InvalidateRect(self.hwnd, None, false); }
        }
    }

    fn position(&mut self) {
        unsafe {
            let id = NOTIFYICONIDENTIFIER { cbSize: std::mem::size_of::<NOTIFYICONIDENTIFIER>() as u32, hWnd: self.hwnd, uID: 1, ..Default::default() };
            let icon = Shell_NotifyIconGetRect(&id).unwrap_or_else(|_| {
                let mut p = POINT::default();
                let _ = GetCursorPos(&mut p);
                RECT { left: p.x, top: p.y, right: p.x, bottom: p.y }
            });
            let c = POINT { x: (icon.left + icon.right) / 2, y: (icon.top + icon.bottom) / 2 };
            let mon = MonitorFromPoint(c, MONITOR_DEFAULTTONEAREST);
            let mut mi = MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, ..Default::default() };
            let _ = GetMonitorInfoW(mon, &mut mi);
            let (mut dx, mut dy) = (96u32, 96u32);
            let _ = GetDpiForMonitor(mon, MDT_EFFECTIVE_DPI, &mut dx, &mut dy);
            let k = dx as f32 / 96.0;
            let (w, h) = ((draw::WIDTH * k).round() as i32, (self.r.height(&self.cfg, &self.view()) * k).round() as i32);
            let m = (12.0 * k) as i32;
            let wa = mi.rcWork;
            let x = (c.x - w / 2).clamp(wa.left + m, (wa.right - w - m).max(wa.left + m));
            let y = if c.y >= wa.bottom {
                wa.bottom - h - m
            } else if c.y <= wa.top {
                wa.top + m
            } else {
                (c.y - h / 2).clamp(wa.top + m, (wa.bottom - h - m).max(wa.top + m))
            };
            let _ = SetWindowPos(self.hwnd, HWND_TOPMOST, x, y, w, h, SWP_NOACTIVATE);
        }
    }

    fn apply_theme(&self) {
        let p = draw::palette(is_dark(&self.cfg), self.cfg.accent);
        let rgb = |c: f32| (c * 255.0).round() as u32;
        let border = rgb(p.line.r) | rgb(p.line.g) << 8 | rgb(p.line.b) << 16;
        unsafe {
            let _ = DwmSetWindowAttribute(self.hwnd, DWMWA_BORDER_COLOR, &border as *const u32 as _, 4);
        }
    }

    fn show(&mut self) {
        if self.updated.is_none() {
            self.refresh();
        }
        self.apply_theme();
        self.position();
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_SHOW);
            let _ = SetForegroundWindow(self.hwnd);
            SetTimer(self.hwnd, LIVE, 1500, None);
        }
    }

    fn hide(&mut self) {
        if !self.visible() { return; }
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_HIDE);
            let _ = KillTimer(self.hwnd, LIVE);
            self.r.release();
            let _ = EmptyWorkingSet(GetCurrentProcess());
        }
        self.hidden_at = Instant::now();
    }

    fn toggle(&mut self) {
        if self.visible() {
            self.hide();
        } else if self.hidden_at.elapsed().as_millis() > 300 {
            self.show();
        }
    }

    fn paint(&mut self) {
        unsafe {
            let mut ps = PAINTSTRUCT::default();
            BeginPaint(self.hwnd, &mut ps);
            let dpi = GetDpiForWindow(self.hwnd) as f32;
            let p = draw::palette(is_dark(&self.cfg), self.cfg.accent);
            let v = self.view();
            let _ = self.r.draw(self.hwnd, dpi, &self.cfg, &p, &v);
            let _ = EndPaint(self.hwnd, &ps);
        }
    }

    fn click(&mut self, x: i32, y: i32) {
        let k = unsafe { GetDpiForWindow(self.hwnd) } as f32 / 96.0;
        let (x, y) = (x as f32 / k, y as f32 / k);
        let [refresh, settings] = draw::buttons(&self.cfg);
        let inside = |r: (f32, f32, f32, f32)| x >= r.0 && x < r.2 && y >= r.1 && y < r.3;
        if inside(refresh) {
            self.refresh();
        } else if inside(settings) {
            let mut p = POINT { x: (settings.0 * k) as i32, y: (settings.3 * k) as i32 };
            unsafe { let _ = ClientToScreen(self.hwnd, &mut p); }
            menu(self.hwnd, p);
        }
    }

    fn command(&mut self, id: u32) {
        let c = &mut self.cfg;
        match id {
            1 => self.refresh(),
            10..=12 => c.theme = [Theme::Sistema, Theme::Claro, Theme::Oscuro][(id - 10) as usize],
            20..=23 => c.accent = [Accent::Tinta, Accent::Arcilla, Accent::Azul, Accent::Salvia][(id - 20) as usize],
            30..=32 => c.shape = [Shape::Barras, Shape::Anillos, Shape::Numeros][(id - 30) as usize],
            40 | 41 => c.compact = id == 41,
            50 => c.show_session ^= true,
            51 => c.show_week ^= true,
            52 => c.show_opus ^= true,
            53 => c.show_reset ^= true,
            54 => c.show_pace ^= true,
            55 => c.show_ram ^= true,
            60..=63 => c.warn_at = WARN[(id - 60) as usize],
            70..=72 => c.crit_at = CRIT[(id - 70) as usize],
            75 => c.notify ^= true,
            80..=82 => c.tray_metric = [TrayMetric::Sesion, TrayMetric::Semana, TrayMetric::Mayor][(id - 80) as usize],
            90 => config::set_startup(!config::startup_enabled()),
            99 => unsafe {
                let _ = DestroyWindow(self.hwnd);
                return;
            },
            _ => return,
        }
        config::save(&self.cfg);
        self.apply_theme();
        self.update_tray();
        self.redraw();
    }
}

const WARN: [f32; 4] = [50.0, 60.0, 70.0, 80.0];
const CRIT: [f32; 3] = [80.0, 90.0, 95.0];

fn menu(hwnd: HWND, at: POINT) {
    let Some(a) = app() else { return };
    let c = a.cfg.clone();
    unsafe {
        let item = |m: HMENU, id: u32, text: &str, on: bool| {
            let _ = AppendMenuW(m, MF_STRING | if on { MF_CHECKED } else { MF_UNCHECKED }, id as usize, &HSTRING::from(text));
        };
        let sub = |m: HMENU, text: &str, items: &[(u32, &str, bool)]| {
            let s = CreatePopupMenu().unwrap();
            for &(id, t, on) in items {
                if id == 0 {
                    let _ = AppendMenuW(s, MF_SEPARATOR, 0, None);
                } else {
                    item(s, id, t, on);
                }
            }
            let _ = AppendMenuW(m, MF_POPUP, s.0 as usize, &HSTRING::from(text));
        };
        let m = CreatePopupMenu().unwrap();
        item(m, 1, "Actualizar ahora", false);
        let _ = AppendMenuW(m, MF_SEPARATOR, 0, None);
        sub(m, "Tema", &[(10, "Sistema", c.theme == Theme::Sistema), (11, "Claro", c.theme == Theme::Claro), (12, "Oscuro", c.theme == Theme::Oscuro)]);
        sub(m, "Acento", &[(20, "Tinta", c.accent == Accent::Tinta), (21, "Arcilla", c.accent == Accent::Arcilla), (22, "Azul", c.accent == Accent::Azul), (23, "Salvia", c.accent == Accent::Salvia)]);
        sub(m, "Forma", &[(30, "Barras", c.shape == Shape::Barras), (31, "Anillos", c.shape == Shape::Anillos), (32, "Números", c.shape == Shape::Numeros)]);
        sub(m, "Densidad", &[(40, "Cómoda", !c.compact), (41, "Compacta", c.compact)]);
        sub(m, "Mostrar", &[
            (50, "Sesión de 5 h", c.show_session),
            (51, "Semana, todos los modelos", c.show_week),
            (52, "Semana, Opus", c.show_opus),
            (53, "Hora de reinicio", c.show_reset),
            (54, "Marca de ritmo", c.show_pace),
            (55, "Memoria que usa el monitor", c.show_ram),
        ]);
        let wl: Vec<String> = WARN.iter().map(|v| format!("Cerca del límite: {v}%")).collect();
        let cl: Vec<String> = CRIT.iter().map(|v| format!("Casi agotado: {v}%")).collect();
        sub(m, "Alertas", &[
            (60, &wl[0], c.warn_at == WARN[0]), (61, &wl[1], c.warn_at == WARN[1]), (62, &wl[2], c.warn_at == WARN[2]), (63, &wl[3], c.warn_at == WARN[3]),
            (0, "", false),
            (70, &cl[0], c.crit_at == CRIT[0]), (71, &cl[1], c.crit_at == CRIT[1]), (72, &cl[2], c.crit_at == CRIT[2]),
            (0, "", false),
            (75, "Notificar al cruzar un umbral", c.notify),
        ]);
        sub(m, "El icono muestra", &[(80, "Sesión", c.tray_metric == TrayMetric::Sesion), (81, "Semana", c.tray_metric == TrayMetric::Semana), (82, "El más alto", c.tray_metric == TrayMetric::Mayor)]);
        let _ = AppendMenuW(m, MF_SEPARATOR, 0, None);
        item(m, 90, "Abrir al iniciar Windows", config::startup_enabled());
        item(m, 99, "Salir", false);
        let _ = SetForegroundWindow(hwnd);
        let id = TrackPopupMenu(m, TPM_RETURNCMD | TPM_RIGHTBUTTON | TPM_NONOTIFY, at.x, at.y, 0, hwnd, None).0 as u32;
        let _ = DestroyMenu(m);
        if let Some(a) = app() {
            a.command(id);
        }
    }
}

extern "system" fn wndproc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    let Some(a) = app() else { return unsafe { DefWindowProcW(hwnd, msg, wp, lp) } };
    match msg {
        WM_TRAY => match (lp.0 & 0xFFFF) as u32 {
            NIN_SELECT | NIN_KEYSELECT => a.toggle(),
            WM_CONTEXTMENU => {
                a.hide();
                let p = POINT { x: (wp.0 & 0xFFFF) as i16 as i32, y: (wp.0 >> 16 & 0xFFFF) as i16 as i32 };
                menu(hwnd, p);
            }
            _ => {}
        },
        WM_DATA => a.on_data(),
        WM_TIMER if wp.0 == POLL => a.refresh(),
        WM_TIMER if wp.0 == LIVE => unsafe { let _ = InvalidateRect(hwnd, None, false); },
        WM_ACTIVATE if (wp.0 & 0xFFFF) as u32 == WA_INACTIVE => a.hide(),
        WM_KEYDOWN if wp.0 == VK_ESCAPE.0 as usize => a.hide(),
        WM_LBUTTONUP => a.click((lp.0 & 0xFFFF) as i16 as i32, (lp.0 >> 16 & 0xFFFF) as i16 as i32),
        WM_PAINT => a.paint(),
        WM_DPICHANGED => a.redraw(),
        WM_SETTINGCHANGE => {
            a.apply_theme();
            a.update_tray();
            a.redraw();
        }
        WM_DESTROY => unsafe {
            let _ = Shell_NotifyIconW(NIM_DELETE, &nid(hwnd));
            PostQuitMessage(0);
        },
        m if m == a.taskbar_created => a.add_tray(),
        _ => return unsafe { DefWindowProcW(hwnd, msg, wp, lp) },
    }
    LRESULT(0)
}

fn main() -> Result<()> {
    unsafe {
        let _single = CreateMutexW(None, true, w!("Local\\TallyForClaude"))?;
        if GetLastError() == ERROR_ALREADY_EXISTS {
            return Ok(());
        }
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        let inst = GetModuleHandleW(None)?;
        let class = w!("TallyForClaude");
        RegisterClassW(&WNDCLASSW {
            style: CS_DROPSHADOW,
            lpfnWndProc: Some(wndproc),
            hInstance: inst.into(),
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            lpszClassName: class,
            ..Default::default()
        });
        let hwnd = CreateWindowExW(WS_EX_TOOLWINDOW | WS_EX_TOPMOST, class, w!("Uso de Claude"), WS_POPUP, 0, 0, 360, 300, None, None, inst, None)?;
        let round = DWMWCP_ROUND;
        let _ = DwmSetWindowAttribute(hwnd, DWMWA_WINDOW_CORNER_PREFERENCE, &round as *const _ as _, 4);

        let cfg = config::load();
        let wait = cfg.quiet_until - now();
        let app = Box::new(App {
            hwnd,
            cfg,
            r: draw::Renderer::new()?,
            usage: None,
            updated: None,
            error: (wait > 0).then_some(FetchError::RateLimited(wait)),
            loading: false,
            levels: None,
            icon: HICON::default(),
            hidden_at: Instant::now(),
            inbox: Arc::default(),
            taskbar_created: RegisterWindowMessageW(w!("TaskbarCreated")),
        });
        APP.with(|a| a.set(Box::into_raw(app)));
        let a = self::app().unwrap();
        a.add_tray();
        a.refresh();
        SetTimer(hwnd, POLL, POLL_MS, None);

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    Ok(())
}
