import { useProgress } from './motion.js';

const THEMES = {
  oscuro: { desk: '#0E0E0E', bg: '#171716', fg: '#EDECE8', mute: '#9E9D97', track: '#2B2B29', line: '#2D2D2B', bar: '#1C1C1B', hover: '#2A2A28', warn: '#E5A83B', crit: '#F26A55' },
  claro: { desk: '#D8D7D2', bg: '#FBFBFA', fg: '#181816', mute: '#62615C', track: '#E6E5E0', line: '#E2E1DC', bar: '#EFEEEA', hover: '#E2E1DC', warn: '#9A6200', crit: '#C13A22' },
};
const ACCENTS = { tinta: ['#E2E0DA', '#2A2925'], arcilla: ['#D98B6C', '#B0532E'], azul: ['#86A8F0', '#2D5DC2'], salvia: ['#93BFA3', '#3B7853'] };
const DENSITY = {
  cómodo: { pad: 20, gap: 20, inner: 8, num: 28, label: 14, ring: 44 },
  compacto: { pad: 14, gap: 12, inner: 6, num: 20, label: 13, ring: 34 },
};

const Icon = ({ d }) => (
  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.75" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">{d}</svg>
);

export default function Popup({ theme = 'oscuro', accent = 'tinta', shape = 'barras', density = 'cómodo', sesion = 38, semanal = 61, opus = 22, showOpus = true, warnAt = 70, critAt = 90, run = true, delay = 0 }) {
  const t = THEMES[theme];
  const d = DENSITY[density];
  const acc = ACCENTS[accent][theme === 'oscuro' ? 0 : 1];
  const k = useProgress(run, 1600, delay);
  const level = (p) => (p >= critAt ? 'crit' : p >= warnAt ? 'warn' : 'ok');
  const rows = [
    { label: 'Sesión', sub: 'ventana de 5 h', pct: sesion, pace: 55, reset: 'Se reinicia en 2 h 14 min' },
    { label: 'Semana', sub: 'todos los modelos', pct: semanal, pace: 43, reset: 'Se reinicia el jue 09:00' },
    showOpus && { label: 'Opus', sub: 'semanal', pct: opus, pace: 43, reset: 'Se reinicia el jue 09:00' },
  ].filter(Boolean);

  return (
    <div className="popup-stage" style={{ background: t.desk, color: t.fg }}>
      <section aria-label="Ejemplo del popup de uso" className="popup" style={{ background: t.bg, borderColor: t.line, padding: d.pad, gap: d.gap, boxShadow: theme === 'oscuro' ? '0 18px 44px rgba(0,0,0,.55), 0 2px 6px rgba(0,0,0,.4)' : '0 18px 44px rgba(24,24,20,.16), 0 2px 6px rgba(24,24,20,.08)' }}>
        <header className="popup-head" style={{ color: t.mute }}>
          <span>Uso</span>
          <span className="popup-icons">
            <Icon d={<><path d="M20 11a8 8 0 1 0-2.3 5.7" /><path d="M20 4v7h-7" /></>} />
            <Icon d={<><path d="M4 7h10" /><path d="M18 7h2" /><circle cx="16" cy="7" r="2" /><path d="M4 17h4" /><path d="M12 17h8" /><circle cx="10" cy="17" r="2" /></>} />
          </span>
        </header>

        {rows.map((r) => {
          const lv = level(r.pct);
          const color = lv === 'ok' ? acc : t[lv];
          const numColor = lv === 'ok' ? t.fg : t[lv];
          const status = lv === 'crit' ? 'Casi agotado' : lv === 'warn' ? 'Cerca del límite' : '';
          const shown = Math.round(r.pct * k);
          const pct = <span className="popup-num" style={{ fontSize: d.num, color: numColor }}>{shown}<small style={{ color: t.mute }}>%</small></span>;
          const meta = <span className="popup-meta" style={{ color: t.mute }}>{[r.reset, status].filter(Boolean).join(' · ')}</span>;

          if (shape === 'anillos') {
            return (
              <div key={r.label} className="popup-ring-row">
                <svg width={d.ring} height={d.ring} viewBox="0 0 40 40" aria-hidden="true">
                  <circle cx="20" cy="20" r="17" fill="none" stroke={t.track} strokeWidth="3" />
                  <circle cx="20" cy="20" r="17" fill="none" stroke={color} strokeWidth="3" strokeLinecap="round" transform="rotate(-90 20 20)" strokeDasharray={`${(r.pct * k * 1.068).toFixed(1)} 107`} />
                </svg>
                <span className="popup-ring-text"><span style={{ fontSize: d.label, fontWeight: 500 }}>{r.label}</span>{meta}</span>
                {pct}
              </div>
            );
          }
          return (
            <div key={r.label} className="popup-row" style={{ gap: d.inner }}>
              <div className="popup-row-head">
                <span><span style={{ fontSize: d.label, fontWeight: 500 }}>{r.label}</span> <span className="popup-meta" style={{ color: t.mute }}>{r.sub}</span></span>
                {pct}
              </div>
              {shape === 'barras' && (
                <div className="popup-bar" style={{ background: t.track }}>
                  <div style={{ width: `${r.pct * k}%`, background: color }} />
                  <i style={{ left: `${r.pace}%`, background: t.fg, opacity: 0.45 * k }} />
                </div>
              )}
              {meta}
            </div>
          );
        })}

        <footer className="popup-foot" style={{ borderColor: t.line, color: t.mute, paddingTop: d.inner }}>
          <span>Actualizado ahora</span>
          <span>5,2 MB de RAM</span>
        </footer>
      </section>

      <div className="taskbar" style={{ background: t.bar, borderColor: t.line }} aria-hidden="true">
        <span className="tray-icon" style={{ background: t.hover }}>
          <b style={{ color: level(sesion) === 'ok' ? t.fg : t[level(sesion)] }}>{Math.round(sesion * k)}</b>
          <span style={{ background: t.track }}><span style={{ width: `${sesion * k}%`, background: level(sesion) === 'ok' ? t.fg : t[level(sesion)] }} /></span>
        </span>
        <span className="clock"><span>14:26</span><span>21/10/2026</span></span>
      </div>
    </div>
  );
}
