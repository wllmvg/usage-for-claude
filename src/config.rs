use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use windows::core::{w, PCWSTR};
use windows::Win32::System::Registry::*;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum Theme { Sistema, Claro, Oscuro }

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum Accent { Tinta, Arcilla, Azul, Salvia }

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum Shape { Barras, Anillos, Numeros }

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum TrayMetric { Sesion, Semana, Mayor }

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Config {
    pub theme: Theme,
    pub accent: Accent,
    pub shape: Shape,
    pub compact: bool,
    pub show_session: bool,
    pub show_week: bool,
    pub show_opus: bool,
    pub show_reset: bool,
    pub show_pace: bool,
    pub show_ram: bool,
    pub warn_at: f32,
    pub crit_at: f32,
    pub notify: bool,
    pub tray_metric: TrayMetric,
    pub quiet_until: i64,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            theme: Theme::Sistema,
            accent: Accent::Tinta,
            shape: Shape::Barras,
            compact: false,
            show_session: true,
            show_week: true,
            show_opus: true,
            show_reset: true,
            show_pace: true,
            show_ram: true,
            warn_at: 70.0,
            crit_at: 90.0,
            notify: true,
            tray_metric: TrayMetric::Sesion,
            quiet_until: 0,
        }
    }
}

fn path() -> PathBuf {
    PathBuf::from(std::env::var_os("APPDATA").unwrap_or_default()).join("Tally").join("config.json")
}

pub fn load() -> Config {
    fs::read_to_string(path()).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
}

pub fn save(c: &Config) {
    let p = path();
    let _ = fs::create_dir_all(p.parent().unwrap());
    let tmp = p.with_extension("tmp");
    if fs::write(&tmp, serde_json::to_string_pretty(c).unwrap()).is_ok() {
        let _ = fs::rename(&tmp, &p);
    }
}

const RUN: PCWSTR = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
const NAME: PCWSTR = w!("Tally");

pub fn startup_enabled() -> bool {
    unsafe { RegGetValueW(HKEY_CURRENT_USER, RUN, NAME, RRF_RT_REG_SZ, None, None, None).is_ok() }
}

pub fn set_startup(on: bool) {
    unsafe {
        if on {
            let exe = std::env::current_exe().unwrap_or_default();
            let v: Vec<u16> = format!("\"{}\"", exe.display()).encode_utf16().chain([0]).collect();
            let _ = RegSetKeyValueW(HKEY_CURRENT_USER, RUN, NAME, REG_SZ.0, Some(v.as_ptr() as _), (v.len() * 2) as u32);
        } else {
            let _ = RegDeleteKeyValueW(HKEY_CURRENT_USER, RUN, NAME);
        }
    }
}

pub fn theme_dword(name: PCWSTR) -> Option<u32> {
    let mut v = 0u32;
    let mut n = 4u32;
    unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"),
            name,
            RRF_RT_REG_DWORD,
            None,
            Some(&mut v as *mut u32 as _),
            Some(&mut n),
        )
        .ok()
        .ok()?;
    }
    Some(v)
}
