use serde_json::Value;
use std::{fs, path::PathBuf};
use windows::core::{w, HSTRING, PCWSTR};
use windows::Win32::Foundation::{FILETIME, SYSTEMTIME};
use windows::Win32::Networking::WinHttp::*;
use windows::Win32::Storage::FileSystem::FileTimeToLocalFileTime;
use windows::Win32::System::Time::FileTimeToSystemTime;

#[derive(Clone, Copy)]
pub struct Limit {
    pub pct: f32,
    pub resets_at: Option<i64>,
}

#[derive(Clone, Copy)]
pub struct Usage {
    pub session: Option<Limit>,
    pub week: Option<Limit>,
    pub opus: Option<Limit>,
}

pub enum FetchError {
    NoLogin,
    Expired,
    Network,
    RateLimited(i64),
    Bad(String),
}

impl FetchError {
    pub fn message(&self) -> (String, String) {
        let (t, d) = match self {
            FetchError::NoLogin => ("Inicia sesión en Claude Code", "Abre una terminal, escribe claude y luego /login. Tally lo detectará en unos minutos.".into()),
            FetchError::Expired => ("Tu sesión de Claude expiró", "Abre una terminal, escribe claude y luego /login. Tally lo detectará en unos minutos.".into()),
            FetchError::Network => ("Sin conexión", "Reintentaré en 5 minutos.".into()),
            FetchError::RateLimited(s) => ("Demasiadas consultas", format!("Anthropic pidió esperar; reintentaré en {} min.", (s + 59) / 60)),
            FetchError::Bad(s) => ("No pude leer el uso", s.clone()),
        };
        (t.into(), d)
    }
}

pub fn fetch() -> Result<Usage, FetchError> {
    let token = read_token().ok_or(FetchError::NoLogin)?;
    let headers = format!("Authorization: Bearer {token}\r\nanthropic-beta: oauth-2025-04-20\r\n");
    let (status, body, retry) = get("api.anthropic.com", "/api/oauth/usage", &headers).map_err(|_| FetchError::Network)?;
    match status {
        200 => parse(&body).ok_or_else(|| FetchError::Bad("La respuesta no tiene el formato esperado.".into())),
        401 | 403 => Err(FetchError::Expired),
        429 => Err(FetchError::RateLimited(retry.unwrap_or(300).max(60))),
        s => Err(FetchError::Bad(format!("El servidor respondió {s}."))),
    }
}

fn read_token() -> Option<String> {
    let dir = std::env::var_os("CLAUDE_CONFIG_DIR")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("USERPROFILE").map(|h| PathBuf::from(h).join(".claude")))?;
    let v: Value = serde_json::from_str(&fs::read_to_string(dir.join(".credentials.json")).ok()?).ok()?;
    v["claudeAiOauth"]["accessToken"].as_str().filter(|t| !t.is_empty()).map(String::from)
}

fn parse(body: &str) -> Option<Usage> {
    let v: Value = serde_json::from_str(body).ok()?;
    let lim = |k: &str| {
        let x = &v[k];
        x["utilization"].as_f64().map(|p| Limit { pct: p as f32, resets_at: x["resets_at"].as_str().and_then(parse_iso) })
    };
    let u = Usage { session: lim("five_hour"), week: lim("seven_day"), opus: lim("seven_day_opus") };
    (u.session.is_some() || u.week.is_some()).then_some(u)
}

struct Handle(*mut core::ffi::c_void);
impl Drop for Handle {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { let _ = WinHttpCloseHandle(self.0); }
        }
    }
}

fn get(host: &str, path: &str, headers: &str) -> windows::core::Result<(u32, String, Option<i64>)> {
    unsafe {
        let s = Handle(WinHttpOpen(w!("Tally/0.1"), WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY, PCWSTR::null(), PCWSTR::null(), 0));
        if s.0.is_null() { return Err(windows::core::Error::from_win32()); }
        WinHttpSetTimeouts(s.0, 10_000, 10_000, 10_000, 10_000)?;
        let c = Handle(WinHttpConnect(s.0, &HSTRING::from(host), INTERNET_DEFAULT_HTTPS_PORT as u16, 0));
        if c.0.is_null() { return Err(windows::core::Error::from_win32()); }
        let r = Handle(WinHttpOpenRequest(c.0, w!("GET"), &HSTRING::from(path), PCWSTR::null(), PCWSTR::null(), std::ptr::null(), WINHTTP_FLAG_SECURE));
        if r.0.is_null() { return Err(windows::core::Error::from_win32()); }
        let h: Vec<u16> = headers.encode_utf16().collect();
        WinHttpSendRequest(r.0, Some(&h), None, 0, 0, 0)?;
        WinHttpReceiveResponse(r.0, std::ptr::null_mut())?;
        let mut status = 0u32;
        let mut len = 4u32;
        WinHttpQueryHeaders(r.0, WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER, PCWSTR::null(), Some(&mut status as *mut u32 as _), &mut len, std::ptr::null_mut())?;
        let mut ra = [0u16; 32];
        let mut ra_len = std::mem::size_of_val(&ra) as u32;
        let retry = WinHttpQueryHeaders(r.0, WINHTTP_QUERY_CUSTOM, w!("Retry-After"), Some(ra.as_mut_ptr() as _), &mut ra_len, std::ptr::null_mut())
            .ok()
            .and_then(|_| String::from_utf16_lossy(&ra[..ra_len as usize / 2]).trim().parse().ok());
        let mut body = Vec::new();
        let mut buf = [0u8; 8192];
        loop {
            let mut n = 0u32;
            WinHttpReadData(r.0, buf.as_mut_ptr() as _, buf.len() as u32, &mut n)?;
            if n == 0 { break; }
            body.extend_from_slice(&buf[..n as usize]);
        }
        Ok((status, String::from_utf8_lossy(&body).into_owned(), retry))
    }
}

pub fn parse_iso(s: &str) -> Option<i64> {
    let b = s.as_bytes();
    let n = |a: usize, l: usize| -> Option<i64> { s.get(a..a + l)?.parse().ok() };
    let (y, mo, d, h, mi, se) = (n(0, 4)?, n(5, 2)?, n(8, 2)?, n(11, 2)?, n(14, 2)?, n(17, 2)?);
    let mut i = 19;
    while i < b.len() && (b[i] == b'.' || b[i].is_ascii_digit()) { i += 1; }
    let off = match b.get(i) {
        Some(&c @ (b'+' | b'-')) => {
            let o = n(i + 1, 2)? * 3600 + n(i + 4, 2)? * 60;
            if c == b'-' { -o } else { o }
        }
        _ => 0,
    };
    Some(days_from_civil(y, mo, d) * 86400 + h * 3600 + mi * 60 + se - off)
}

fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    era * 146097 + yoe * 365 + yoe / 4 - yoe / 100 + doy - 719468
}

pub fn reset_text(at: i64, now: i64) -> String {
    let left = (at - now).max(0);
    if left < 86400 {
        let (h, m) = (left / 3600, left % 3600 / 60);
        return if h > 0 { format!("Se reinicia en {h} h {m} min") } else { format!("Se reinicia en {m} min") };
    }
    let t = (at * 10_000_000 + 116_444_736_000_000_000) as u64;
    let ft = FILETIME { dwLowDateTime: t as u32, dwHighDateTime: (t >> 32) as u32 };
    let (mut lf, mut st) = (FILETIME::default(), SYSTEMTIME::default());
    unsafe {
        let _ = FileTimeToLocalFileTime(&ft, &mut lf);
        let _ = FileTimeToSystemTime(&lf, &mut st);
    }
    let day = ["dom", "lun", "mar", "mié", "jue", "vie", "sáb"][st.wDayOfWeek as usize % 7];
    format!("Se reinicia el {day} {:02}:{:02}", st.wHour, st.wMinute)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso() {
        assert_eq!(parse_iso("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(parse_iso("2000-03-01T00:00:00Z"), Some(951_868_800));
        assert_eq!(parse_iso("2000-03-01T02:00:00.123456+02:00"), Some(951_868_800));
        assert_eq!(parse_iso("1999-12-31T22:00:00-02:00"), Some(946_684_800));
        assert_eq!(parse_iso("basura"), None);
    }

    #[test]
    fn response() {
        let u = parse(r#"{"five_hour":{"utilization":38.0,"resets_at":"2000-03-01T00:00:00Z"},"seven_day":{"utilization":61,"resets_at":null},"seven_day_opus":null}"#).unwrap();
        let s = u.session.unwrap();
        assert_eq!((s.pct, s.resets_at), (38.0, Some(951_868_800)));
        assert_eq!(u.week.unwrap().resets_at, None);
        assert!(u.opus.is_none());
        assert!(parse("{}").is_none());
    }

    #[test]
    fn reset_soon() {
        assert_eq!(reset_text(8040, 0), "Se reinicia en 2 h 14 min");
        assert_eq!(reset_text(300, 0), "Se reinicia en 5 min");
    }
}
