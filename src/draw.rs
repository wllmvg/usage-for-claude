use crate::config::{Accent, Config, Shape};
use windows::core::{w, Result, PCWSTR};
use windows::Win32::Foundation::{HWND, RECT, TRUE};
use windows::Win32::Graphics::Direct2D::Common::*;
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::Graphics::DirectWrite::*;
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::WindowsAndMessaging::{CreateIconIndirect, GetClientRect, HICON, ICONINFO};

pub const WIDTH: f32 = 360.0;
const FONT: PCWSTR = w!("Segoe UI Variable Text");
const ICONS: PCWSTR = w!("Segoe MDL2 Assets");
const SOFTWARE: D2D1_RENDER_TARGET_TYPE = D2D1_RENDER_TARGET_TYPE_SOFTWARE;

type C = D2D1_COLOR_F;
type RT = ID2D1HwndRenderTarget;
type Brush = ID2D1SolidColorBrush;

pub struct Palette { pub bg: C, pub fg: C, pub mute: C, pub track: C, pub line: C, pub warn: C, pub crit: C, pub accent: C }

pub fn hex(v: u32) -> C {
    C { r: (v >> 16 & 255) as f32 / 255.0, g: (v >> 8 & 255) as f32 / 255.0, b: (v & 255) as f32 / 255.0, a: 1.0 }
}

pub fn palette(dark: bool, accent: Accent) -> Palette {
    let (ad, al) = match accent {
        Accent::Tinta => (0xE2E0DA, 0x2A2925),
        Accent::Arcilla => (0xD98B6C, 0xB0532E),
        Accent::Azul => (0x86A8F0, 0x2D5DC2),
        Accent::Salvia => (0x93BFA3, 0x3B7853),
    };
    let [bg, fg, mute, track, line, warn, crit, acc] = if dark {
        [0x171716, 0xEDECE8, 0x9E9D97, 0x2B2B29, 0x2D2D2B, 0xE5A83B, 0xF26A55, ad]
    } else {
        [0xFBFBFA, 0x181816, 0x62615C, 0xE6E5E0, 0xE2E1DC, 0x9A6200, 0xC13A22, al]
    };
    Palette { bg: hex(bg), fg: hex(fg), mute: hex(mute), track: hex(track), line: hex(line), warn: hex(warn), crit: hex(crit), accent: hex(acc) }
}

pub struct Row { pub label: &'static str, pub sub: &'static str, pub pct: f32, pub pace: Option<f32>, pub meta: String, pub level: u8 }

pub struct Setting { pub label: &'static str, pub options: Vec<(String, u32, bool)> }

pub struct View { pub rows: Vec<Row>, pub message: Option<(String, String)>, pub footer_left: String, pub footer_right: String, pub settings: Option<Vec<Setting>> }

const SET_LABEL: f32 = 84.0;
const SET_ROW: f32 = 28.0;
const SET_GAP: f32 = 6.0;

struct S { pad: f32, gap: f32, inner: f32, num: f32, label: f32, ring: f32 }

fn sizes(cfg: &Config) -> S {
    if cfg.compact {
        S { pad: 14.0, gap: 12.0, inner: 6.0, num: 20.0, label: 13.0, ring: 34.0 }
    } else {
        S { pad: 20.0, gap: 20.0, inner: 8.0, num: 28.0, label: 14.0, ring: 44.0 }
    }
}

pub fn buttons(cfg: &Config) -> [(f32, f32, f32, f32); 2] {
    let s = sizes(cfg);
    let x1 = WIDTH - s.pad;
    [(x1 - 64.0, s.pad, x1 - 32.0, s.pad + 32.0), (x1 - 32.0, s.pad, x1, s.pad + 32.0)]
}

pub fn chips(cfg: &Config, settings: &[Setting]) -> Vec<((f32, f32, f32, f32), u32)> {
    let s = sizes(cfg);
    let (x0, x1) = (s.pad + SET_LABEL, WIDTH - s.pad);
    let mut out = Vec::new();
    for (i, row) in settings.iter().enumerate() {
        let y = s.pad + 32.0 + s.gap + i as f32 * (SET_ROW + SET_GAP);
        let w = (x1 - x0) / row.options.len() as f32;
        for (j, (_, id, _)) in row.options.iter().enumerate() {
            let x = x0 + j as f32 * w;
            out.push(((x, y, x + w, y + SET_ROW), *id));
        }
    }
    out
}

pub struct Renderer {
    d2d: ID2D1Factory,
    dw: IDWriteFactory,
    round: ID2D1StrokeStyle,
    target: Option<(RT, Brush)>,
}

fn pt(x: f32, y: f32) -> D2D_POINT_2F { D2D_POINT_2F { x, y } }
fn rect(x: f32, y: f32, w: f32, h: f32) -> D2D_RECT_F { D2D_RECT_F { left: x, top: y, right: x + w, bottom: y + h } }

impl Renderer {
    pub fn new() -> Result<Self> {
        unsafe {
            let d2d: ID2D1Factory = D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)?;
            let dw: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)?;
            let round = d2d.CreateStrokeStyle(
                &D2D1_STROKE_STYLE_PROPERTIES { startCap: D2D1_CAP_STYLE_ROUND, endCap: D2D1_CAP_STYLE_ROUND, ..Default::default() },
                None,
            )?;
            Ok(Renderer { d2d, dw, round, target: None })
        }
    }

    pub fn release(&mut self) { self.target = None; }

    pub fn height(&self, cfg: &Config, v: &View) -> f32 { self.layout(None, cfg, &palette(true, cfg.accent), v) }

    pub fn draw(&mut self, hwnd: HWND, dpi: f32, cfg: &Config, p: &Palette, v: &View) -> Result<()> {
        unsafe {
            let mut rc = RECT::default();
            GetClientRect(hwnd, &mut rc)?;
            let size = D2D_SIZE_U { width: rc.right as u32, height: rc.bottom as u32 };
            if self.target.is_none() {
                let props = D2D1_RENDER_TARGET_PROPERTIES { r#type: SOFTWARE, dpiX: dpi, dpiY: dpi, ..Default::default() };
                let hp = D2D1_HWND_RENDER_TARGET_PROPERTIES { hwnd, pixelSize: size, presentOptions: D2D1_PRESENT_OPTIONS_NONE };
                let rt = self.d2d.CreateHwndRenderTarget(&props, &hp)?;
                let b = rt.CreateSolidColorBrush(&p.fg, None)?;
                self.target = Some((rt, b));
            }
            let (rt, b) = self.target.as_ref().unwrap();
            rt.Resize(&size)?;
            rt.SetDpi(dpi, dpi);
            rt.BeginDraw();
            rt.Clear(Some(&p.bg));
            self.layout(Some((rt, b)), cfg, p, v);
            if rt.EndDraw(None, None).is_err() {
                self.target = None;
            }
        }
        Ok(())
    }

    fn layout(&self, d: Option<(&RT, &Brush)>, cfg: &Config, p: &Palette, v: &View) -> f32 {
        let s = sizes(cfg);
        let (x0, x1) = (s.pad, WIDTH - s.pad);
        let mut y = s.pad;
        if let Some((rt, b)) = d {
            let title = if v.settings.is_some() { "Ajustes" } else { "Uso" };
            self.text(rt, b, title, 13.0, DWRITE_FONT_WEIGHT_MEDIUM, &p.mute, rect(x0, y, 100.0, 32.0), DWRITE_TEXT_ALIGNMENT_LEADING, DWRITE_PARAGRAPH_ALIGNMENT_CENTER);
            let [r1, r2] = buttons(cfg);
            self.icon(rt, b, "\u{E72C}", &p.mute, r1.0, r1.1);
            self.icon(rt, b, "\u{E713}", if v.settings.is_some() { &p.fg } else { &p.mute }, r2.0, r2.1);
        }
        y += 32.0 + s.gap;

        if let Some(set) = &v.settings {
            if let Some((rt, b)) = d {
                for (i, row) in set.iter().enumerate() {
                    let ry = y + i as f32 * (SET_ROW + SET_GAP);
                    self.text(rt, b, row.label, 12.0, DWRITE_FONT_WEIGHT_NORMAL, &p.mute, rect(x0, ry, SET_LABEL, SET_ROW), DWRITE_TEXT_ALIGNMENT_LEADING, DWRITE_PARAGRAPH_ALIGNMENT_CENTER);
                }
                for ((r, _), (text, _, on)) in chips(cfg, set).into_iter().zip(set.iter().flat_map(|row| &row.options)) {
                    let box_ = rect(r.0 + 2.0, r.1, r.2 - r.0 - 4.0, r.3 - r.1);
                    if *on {
                        self.fill(rt, b, &p.track, box_, 6.0);
                    }
                    let (c, wt) = if *on { (&p.fg, DWRITE_FONT_WEIGHT_MEDIUM) } else { (&p.mute, DWRITE_FONT_WEIGHT_NORMAL) };
                    self.text(rt, b, text, 12.0, wt, c, box_, DWRITE_TEXT_ALIGNMENT_CENTER, DWRITE_PARAGRAPH_ALIGNMENT_CENTER);
                }
            }
            y += set.len() as f32 * (SET_ROW + SET_GAP) - SET_GAP + s.gap;
        } else if let Some((title, detail)) = &v.message {
            if let Some((rt, b)) = d {
                self.text(rt, b, title, 14.0, DWRITE_FONT_WEIGHT_MEDIUM, &p.fg, rect(x0, y, x1 - x0, 20.0), DWRITE_TEXT_ALIGNMENT_LEADING, DWRITE_PARAGRAPH_ALIGNMENT_NEAR);
                self.wrapped(rt, b, detail, &p.mute, rect(x0, y + 22.0, x1 - x0, 34.0));
            }
            y += 56.0 + s.gap;
        }

        for r in &v.rows {
            let c = [&p.accent, &p.warn, &p.crit][r.level as usize];
            let numc = if r.level == 0 { &p.fg } else { c };
            if cfg.shape == Shape::Anillos {
                if let Some((rt, b)) = d {
                    let mid = y + s.ring / 2.0;
                    self.ring(rt, b, x0 + s.ring / 2.0, mid, s.ring / 2.0 - 2.0, r.pct, &p.track, c);
                    let tx = x0 + s.ring + 14.0;
                    self.text(rt, b, r.label, s.label, DWRITE_FONT_WEIGHT_MEDIUM, &p.fg, rect(tx, mid - 20.0, x1 - tx - 70.0, 19.0), DWRITE_TEXT_ALIGNMENT_LEADING, DWRITE_PARAGRAPH_ALIGNMENT_FAR);
                    self.text(rt, b, &r.meta, 12.0, DWRITE_FONT_WEIGHT_NORMAL, &p.mute, rect(tx, mid + 1.0, x1 - tx - 70.0, 16.0), DWRITE_TEXT_ALIGNMENT_LEADING, DWRITE_PARAGRAPH_ALIGNMENT_NEAR);
                    self.pct(rt, b, r.pct, s.num, numc, &p.mute, x1, mid - s.num * 0.8, s.num * 1.2);
                }
                y += s.ring;
            } else {
                let lh = (s.num * 1.2).round();
                if let Some((rt, b)) = d {
                    let lift = (s.num - s.label) * 0.22;
                    let w = self.text(rt, b, r.label, s.label, DWRITE_FONT_WEIGHT_MEDIUM, &p.fg, rect(x0, y, 200.0, lh - lift), DWRITE_TEXT_ALIGNMENT_LEADING, DWRITE_PARAGRAPH_ALIGNMENT_FAR);
                    self.text(rt, b, r.sub, 12.0, DWRITE_FONT_WEIGHT_NORMAL, &p.mute, rect(x0 + w + 8.0, y, 200.0, lh - lift), DWRITE_TEXT_ALIGNMENT_LEADING, DWRITE_PARAGRAPH_ALIGNMENT_FAR);
                    self.pct(rt, b, r.pct, s.num, numc, &p.mute, x1, y, lh);
                }
                y += lh + s.inner;
                if cfg.shape == Shape::Barras {
                    if let Some((rt, b)) = d {
                        let bw = x1 - x0;
                        self.fill(rt, b, &p.track, rect(x0, y, bw, 4.0), 2.0);
                        if r.pct > 0.0 {
                            self.fill(rt, b, c, rect(x0, y, bw * r.pct.min(100.0) / 100.0, 4.0), 2.0);
                        }
                        if let Some(pace) = r.pace {
                            self.fill(rt, b, &C { a: 0.45, ..p.fg }, rect(x0 + bw * pace / 100.0, y - 3.0, 1.0, 10.0), 0.0);
                        }
                    }
                    y += 4.0 + s.inner;
                }
                if let Some((rt, b)) = d {
                    self.text(rt, b, &r.meta, 12.0, DWRITE_FONT_WEIGHT_NORMAL, &p.mute, rect(x0, y, x1 - x0, 16.0), DWRITE_TEXT_ALIGNMENT_LEADING, DWRITE_PARAGRAPH_ALIGNMENT_NEAR);
                }
                y += 16.0;
            }
            y += s.gap;
        }

        if let Some((rt, b)) = d {
            self.fill(rt, b, &p.line, rect(x0, y, x1 - x0, 1.0), 0.0);
            let fy = y + 1.0 + s.inner;
            self.text(rt, b, &v.footer_left, 12.0, DWRITE_FONT_WEIGHT_NORMAL, &p.mute, rect(x0, fy, x1 - x0, 16.0), DWRITE_TEXT_ALIGNMENT_LEADING, DWRITE_PARAGRAPH_ALIGNMENT_NEAR);
            self.text(rt, b, &v.footer_right, 12.0, DWRITE_FONT_WEIGHT_NORMAL, &p.mute, rect(x0, fy, x1 - x0, 16.0), DWRITE_TEXT_ALIGNMENT_TRAILING, DWRITE_PARAGRAPH_ALIGNMENT_NEAR);
        }
        y + 1.0 + s.inner + 16.0 + s.pad
    }

    fn fill(&self, rt: &RT, b: &Brush, c: &C, r: D2D_RECT_F, radius: f32) {
        unsafe {
            b.SetColor(c);
            rt.FillRoundedRectangle(&D2D1_ROUNDED_RECT { rect: r, radiusX: radius, radiusY: radius }, b);
        }
    }

    fn layout_for(&self, s: &str, family: PCWSTR, size: f32, weight: DWRITE_FONT_WEIGHT, r: &D2D_RECT_F, wrap: bool) -> Option<IDWriteTextLayout> {
        unsafe {
            let f = self.dw.CreateTextFormat(family, None, weight, DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_STRETCH_NORMAL, size, w!("es-es")).ok()?;
            if !wrap {
                f.SetWordWrapping(DWRITE_WORD_WRAPPING_NO_WRAP).ok()?;
            }
            let s16: Vec<u16> = s.encode_utf16().collect();
            self.dw.CreateTextLayout(&s16, &f, r.right - r.left, r.bottom - r.top).ok()
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn text(&self, rt: &RT, b: &Brush, s: &str, size: f32, weight: DWRITE_FONT_WEIGHT, c: &C, r: D2D_RECT_F, align: DWRITE_TEXT_ALIGNMENT, para: DWRITE_PARAGRAPH_ALIGNMENT) -> f32 {
        let Some(l) = self.layout_for(s, FONT, size, weight, &r, false) else { return 0.0 };
        unsafe {
            let _ = l.SetTextAlignment(align);
            let _ = l.SetParagraphAlignment(para);
            b.SetColor(c);
            rt.DrawTextLayout(pt(r.left, r.top), &l, b, D2D1_DRAW_TEXT_OPTIONS_NONE);
            let mut m = DWRITE_TEXT_METRICS::default();
            let _ = l.GetMetrics(&mut m);
            m.width
        }
    }

    fn wrapped(&self, rt: &RT, b: &Brush, s: &str, c: &C, r: D2D_RECT_F) {
        let Some(l) = self.layout_for(s, FONT, 12.0, DWRITE_FONT_WEIGHT_NORMAL, &r, true) else { return };
        unsafe {
            b.SetColor(c);
            rt.DrawTextLayout(pt(r.left, r.top), &l, b, D2D1_DRAW_TEXT_OPTIONS_CLIP);
        }
    }

    fn icon(&self, rt: &RT, b: &Brush, glyph: &str, c: &C, x: f32, y: f32) {
        let r = rect(x, y, 32.0, 32.0);
        let Some(l) = self.layout_for(glyph, ICONS, 15.0, DWRITE_FONT_WEIGHT_NORMAL, &r, false) else { return };
        unsafe {
            let _ = l.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER);
            let _ = l.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);
            b.SetColor(c);
            rt.DrawTextLayout(pt(x, y), &l, b, D2D1_DRAW_TEXT_OPTIONS_NONE);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn pct(&self, rt: &RT, b: &Brush, pct: f32, size: f32, c: &C, mute: &C, right: f32, y: f32, h: f32) {
        let w = self.text(rt, b, "%", size * 0.55, DWRITE_FONT_WEIGHT_MEDIUM, mute, rect(right - 40.0, y, 40.0, h - size * 0.1), DWRITE_TEXT_ALIGNMENT_TRAILING, DWRITE_PARAGRAPH_ALIGNMENT_FAR);
        self.text(rt, b, &format!("{:.0}", pct), size, DWRITE_FONT_WEIGHT_MEDIUM, c, rect(right - w - 1.0 - 120.0, y, 120.0, h), DWRITE_TEXT_ALIGNMENT_TRAILING, DWRITE_PARAGRAPH_ALIGNMENT_FAR);
    }

    #[allow(clippy::too_many_arguments)]
    fn ring(&self, rt: &RT, b: &Brush, cx: f32, cy: f32, r: f32, pct: f32, track: &C, c: &C) {
        unsafe {
            let e = D2D1_ELLIPSE { point: pt(cx, cy), radiusX: r, radiusY: r };
            b.SetColor(track);
            rt.DrawEllipse(&e, b, 3.0, None);
            b.SetColor(c);
            if pct >= 99.9 {
                rt.DrawEllipse(&e, b, 3.0, None);
                return;
            }
            if pct <= 0.0 { return; }
            let a = pct / 100.0 * std::f32::consts::TAU;
            let Ok(geo) = self.d2d.CreatePathGeometry() else { return };
            let Ok(sink) = geo.Open() else { return };
            sink.BeginFigure(pt(cx, cy - r), D2D1_FIGURE_BEGIN_HOLLOW);
            sink.AddArc(&D2D1_ARC_SEGMENT {
                point: pt(cx + r * a.sin(), cy - r * a.cos()),
                size: D2D_SIZE_F { width: r, height: r },
                rotationAngle: 0.0,
                sweepDirection: D2D1_SWEEP_DIRECTION_CLOCKWISE,
                arcSize: if a > std::f32::consts::PI { D2D1_ARC_SIZE_LARGE } else { D2D1_ARC_SIZE_SMALL },
            });
            sink.EndFigure(D2D1_FIGURE_END_OPEN);
            if sink.Close().is_ok() {
                rt.DrawGeometry(&geo, b, 3.0, &self.round);
            }
        }
    }

    pub fn tray_icon(&self, size: i32, text: &str, pct: f32, fg: &C, track: &C) -> Result<HICON> {
        unsafe {
            let props = D2D1_RENDER_TARGET_PROPERTIES {
                r#type: SOFTWARE,
                pixelFormat: D2D1_PIXEL_FORMAT { format: DXGI_FORMAT_B8G8R8A8_UNORM, alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED },
                ..Default::default()
            };
            let rt = self.d2d.CreateDCRenderTarget(&props)?;
            let dc = CreateCompatibleDC(None);
            let bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: size,
                    biHeight: -size,
                    biPlanes: 1,
                    biBitCount: 32,
                    ..Default::default()
                },
                ..Default::default()
            };
            let mut bits = std::ptr::null_mut();
            let dib = CreateDIBSection(dc, &bmi, DIB_RGB_COLORS, &mut bits, None, 0)?;
            let old = SelectObject(dc, dib);
            rt.BindDC(dc, &RECT { left: 0, top: 0, right: size, bottom: size })?;
            rt.SetTextAntialiasMode(D2D1_TEXT_ANTIALIAS_MODE_GRAYSCALE);
            let b = rt.CreateSolidColorBrush(fg, None)?;
            let sz = size as f32;
            let font = if text.chars().count() > 2 { sz * 0.5 } else { sz * 0.66 };
            rt.BeginDraw();
            rt.Clear(Some(&C::default()));
            if text.is_empty() {
                let u = sz / 22.0;
                let cut = if fg.r + fg.g + fg.b > 1.5 { hex(0x171716) } else { hex(0xFBFBFA) };
                let r = |x: f32, y: f32, w: f32, h: f32, rad: f32, c: &C| {
                    b.SetColor(c);
                    rt.FillRoundedRectangle(&D2D1_ROUNDED_RECT { rect: rect(x * u, y * u, w * u, h * u), radiusX: rad * u, radiusY: rad * u }, &b);
                };
                r(0.5, 0.5, 21.0, 21.0, 5.0, fg);
                r(5.0, 14.0, 12.0, 2.0, 1.0, &C { a: 0.4, ..cut });
                r(5.0, 14.0, 7.0, 2.0, 1.0, &hex(0xD9774F));
                r(5.0, 6.0, 2.0, 5.0, 1.0, &cut);
                r(9.0, 8.0, 2.0, 3.0, 1.0, &cut);
            } else {
                if let Some(l) = self.layout_for(text, w!("Segoe UI"), font, DWRITE_FONT_WEIGHT_BOLD, &rect(0.0, 0.0, sz, sz * 0.82), false) {
                    let _ = l.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER);
                    let _ = l.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);
                    rt.DrawTextLayout(pt(0.0, 0.0), &l, &b, D2D1_DRAW_TEXT_OPTIONS_NONE);
                }
                let bh = (sz / 8.0).max(2.0).round();
                let bar = rect(sz * 0.1, sz - bh, sz * 0.8, bh);
                b.SetColor(track);
                rt.FillRectangle(&bar, &b);
                b.SetColor(fg);
                rt.FillRectangle(&rect(bar.left, bar.top, (bar.right - bar.left) * pct.clamp(0.0, 100.0) / 100.0, bh), &b);
            }
            rt.EndDraw(None, None)?;
            SelectObject(dc, old);
            let mask = CreateBitmap(size, size, 1, 1, None);
            let icon = CreateIconIndirect(&ICONINFO { fIcon: TRUE, hbmColor: dib, hbmMask: mask, ..Default::default() });
            let _ = DeleteObject(dib);
            let _ = DeleteObject(mask);
            let _ = DeleteDC(dc);
            icon
        }
    }
}
