import { useRef, useState } from 'react';
import Popup from './Popup.jsx';
import { useInView, useProgress, useReveal, useScrolled } from './motion.js';

const RELEASE = {
  version: '0.1.0',
  size: '384 KB',
  sha256: '[HASH DEL RELEASE]',
  date: '[FECHA]',
  download: 'https://github.com/wllmvg/usage-for-claude/releases/latest',
  repo: 'https://github.com/wllmvg/usage-for-claude',
  issues: 'https://github.com/wllmvg/usage-for-claude/issues',
  license: 'https://github.com/wllmvg/usage-for-claude/blob/main/LICENSE',
};

const Logo = () => (
  <svg width="22" height="22" viewBox="0 0 22 22" aria-hidden="true">
    <rect x=".5" y=".5" width="21" height="21" rx="5" fill="#EDECE8" />
    <rect x="5" y="14" width="12" height="2" rx="1" fill="#B9B7B0" />
    <rect x="5" y="14" width="7" height="2" rx="1" fill="#D9774F" />
    <rect x="5" y="6" width="2" height="5" rx="1" fill="#0D0D0C" />
    <rect x="9" y="8" width="2" height="3" rx="1" fill="#0D0D0C" />
  </svg>
);

const DownloadIcon = () => (
  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
    <path d="M12 4v12" /><path d="M7 11l5 5 5-5" /><path d="M5 20h14" />
  </svg>
);

const Arrow = () => (
  <svg className="arrow" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
    <path d="M5 12h14" /><path d="M13 6l6 6-6 6" />
  </svg>
);

function DownloadButton() {
  return (
    <a className="btn btn-primary" href={RELEASE.download}>
      <DownloadIcon /> Descargar para Windows
    </a>
  );
}

function CodeBlock({ title, copy, children }) {
  const [done, setDone] = useState(false);
  const onCopy = () =>
    navigator.clipboard?.writeText(copy).then(() => {
      setDone(true);
      setTimeout(() => setDone(false), 1600);
    }, () => {});
  return (
    <figure className="code">
      <figcaption className="code-bar mono">
        <span>{title}</span>
        <button type="button" className="copy mono" onClick={onCopy} aria-label={`Copiar: ${title}`}>
          <span aria-live="polite">{done ? 'Copiado' : 'Copiar'}</span>
        </button>
      </figcaption>
      <pre className="mono">{children}</pre>
    </figure>
  );
}

function Hero() {
  const k = useProgress(true, 1800, 500);
  return (
    <section id="descargar" className="wrap hero">
      <div className="split">
        <div className="hero-copy">
          <h1 className="display">
            <span className="line"><span>Tu límite de Claude,</span></span>{' '}
            <span className="line"><span className="amber">a un vistazo.</span></span>
          </h1>
          <p className="lead rise" style={{ '--i': 2 }}>Un monitor para la bandeja de Windows con tu sesión de 5 horas y tu límite semanal. Sin instalador, sin navegador por dentro, sin ruido.</p>
          <div className="actions rise" style={{ '--i': 3 }}>
            <DownloadButton />
            <a className="btn btn-ghost" href={RELEASE.repo}>Ver el código</a>
          </div>
          <dl className="facts mono rise" style={{ '--i': 4 }}>
            <dt>versión</dt><dd>{RELEASE.version} · x64 · Windows 10 y 11</dd>
            <dt>archivo</dt><dd>tally.exe · {RELEASE.size}</dd>
            <dt>sha-256</dt><dd className="break">{RELEASE.sha256}</dd>
          </dl>
          <p className="note rise" style={{ '--i': 5 }}>Necesitas <a href="#preguntas">Claude Code con la sesión iniciada</a> en el mismo equipo.</p>
        </div>
        <div className="hero-shot">
          <div className="shot"><Popup accent="arcilla" delay={700} /></div>
        </div>
      </div>

      <div className="meter" role="img" aria-label="Medidor de ejemplo: sesión al 38 por ciento, por debajo del ritmo esperado del 55 por ciento">
        <div className="meter-label mono">
          <span>sesión · ventana de 5 h</span>
          <span><b>{Math.round(38 * k)}%</b> · ritmo 55%</span>
        </div>
        <div className="meter-track">
          <div className="meter-fill" style={{ width: `${38 * k}%` }} />
          <i className="meter-tick" style={{ opacity: k > 0.95 ? 1 : 0 }} />
        </div>
        <p className="stack mono">
          {['Rust 1.99', 'Win32', 'Direct2D', 'DirectWrite', 'WinHTTP', 'serde'].map((s) => <span key={s}>{s}</span>)}
        </p>
      </div>
    </section>
  );
}

function Weight() {
  const ref = useRef(null);
  const seen = useInView(ref);
  const bars = [
    ['Electron', 100, '100–200 MB'],
    ['Python + Qt', 37, '40–70 MB'],
    ['Tauri', 30, '30–60 MB'],
  ];
  return (
    <section className="band alt">
      <div className="wrap split top reveal">
        <div className="prose">
          <h2>Cinco megas. <span className="sage">Medidos, no prometidos.</span></h2>
          <p>Escrito en Rust directamente sobre la API de Windows. No arrastra un navegador ni un runtime: es un solo ejecutable que dibuja su ventana con Direct2D.</p>
          <p>Dibujar en la GPU costaba 30 MB solo por cargar Direct3D. Para unas líneas de texto no vale la pena, así que dibuja por software y devuelve la memoria a Windows al cerrar el popup.</p>
        </div>
        <figure ref={ref} className={`weights ${seen ? 'in' : ''}`}>
          <figcaption>Memoria en reposo de una app de bandeja, por tecnología</figcaption>
          <div className="weights-grid">
            {bars.map(([name, w, label], i) => (
              <div key={name} className="weights-row">
                <span>{name}</span>
                <span className="weights-bar" style={{ '--w': `${w}%`, '--i': i }} />
                <span className="mono dim">{label}</span>
              </div>
            ))}
            <div className="weights-row ours">
              <span>Tally</span>
              <span className="weights-bar" style={{ '--w': '3.5%', '--i': 3 }} />
              <span className="mono">5,2 MB</span>
            </div>
          </div>
          <p className="fine">Las otras cifras son rangos típicos aproximados. La nuestra es memoria privada medida en Windows 11.</p>
        </figure>
      </div>
    </section>
  );
}

function Flow() {
  const ref = useRef(null);
  const seen = useInView(ref);
  const nodes = [
    ['#93BFA3', '~/.claude/.credentials.json', 'La sesión de Claude Code. Solo se lee, nunca se modifica.'],
    ['#F2B04A', 'tally.exe', 'Consulta cada 5 minutos y espera lo que pida el servidor si responde 429.'],
    ['#86A8F0', 'api.anthropic.com', 'Por HTTPS con WinHTTP, el cliente de red de Windows. Sin librerías TLS añadidas.'],
    ['#D98B6C', 'bandeja de Windows', 'Icono con tu porcentaje, popup al hacer clic y un aviso al cruzar un umbral.'],
  ];
  return (
    <section id="como-funciona" className="band">
      <div className="wrap stack-lg">
        <div className="prose reveal">
          <h2>Un solo camino <span className="blue">para tus datos.</span></h2>
          <p>Lee tu sesión, pregunta a un único servidor y te lo muestra. Sin servidores intermedios ni copias en la nube.</p>
        </div>
        <div ref={ref} className={`flow-wrap ${seen ? 'in' : ''}`}>
          <ol className="flow" aria-label="Recorrido de los datos">
            {nodes.map(([c, title, text], i) => (
              <li key={title} className="node" style={{ '--i': i, '--c': c }}>
                <span className="dot" />
                <span className="mono node-title">{title}</span>
                <span className="dim">{text}</span>
                {i < nodes.length - 1 && <Arrow />}
              </li>
            ))}
          </ol>
          <div className="packet-track" aria-hidden="true"><span className="packet" /></div>
        </div>
      </div>
    </section>
  );
}

function Mini({ i, caption, ...props }) {
  const ref = useRef(null);
  const seen = useInView(ref);
  return (
    <figure ref={ref} className="mini-fig reveal" style={{ '--i': i }}>
      <div className="mini"><Popup {...props} run={seen} delay={200 + i * 140} /></div>
      <figcaption className="mono">{caption}</figcaption>
    </figure>
  );
}

function Customize() {
  const levels = [['#E2E0DA', 'bajo el umbral'], ['#E5A83B', 'cerca · 70%'], ['#F26A55', 'casi agotado · 90%']];
  const features = [
    ['Forma', 'Barras, anillos o solo números.'],
    ['Tema y acento', 'Claro, oscuro o el del sistema; cuatro acentos: tinta, arcilla, azul y salvia.'],
    ['Marca de ritmo', 'Dónde deberías ir si repartes el uso parejo.'],
    ['Umbrales', 'Tú decides cuándo es “cerca” y cuándo es “casi agotado”.'],
    ['Qué se ve', 'Sesión, semana, Opus, reinicio y su propia RAM, por separado.'],
    ['Al iniciar Windows', 'Un clic, sin permisos de administrador.'],
  ];
  return (
    <section id="personaliza" className="band alt">
      <div className="wrap stack-lg">
        <div className="prose reveal">
          <h2>Callado hasta <span className="red">que importa.</span></h2>
          <p>Casi monocromo mientras vas bien. El color aparece cuando cruzas el umbral que tú eliges, y Windows te avisa una sola vez.</p>
          <ul className="levels mono" aria-label="Niveles">
            {levels.map(([c, t]) => <li key={t}><span style={{ background: c }} />{t}</li>)}
          </ul>
        </div>
        <div className="minis">
          <Mini i={0} caption="por defecto · todo en orden" />
          <Mini i={1} caption="anillos · compacto · azul" accent="azul" shape="anillos" density="compacto" />
          <Mini i={2} caption="números · cerca del límite" accent="salvia" shape="números" sesion={76} semanal={64} showOpus={false} />
          <Mini i={3} caption="claro · casi agotado" theme="claro" accent="arcilla" sesion={94} semanal={88} opus={41} />
        </div>
        <dl className="features">
          {features.map(([t, d], i) => (
            <div key={t} className="reveal" style={{ '--i': i % 3 }}>
              <dt>{t}</dt>
              <dd>{d}</dd>
            </div>
          ))}
        </dl>
      </div>
    </section>
  );
}

function Specs() {
  const rows = [
    ['Ejecutable', `${RELEASE.size}, sin instalador`],
    ['Memoria en reposo', '≈ 5 MB', 'sage'],
    ['Dependencias', 'ninguna'],
    ['Sistema', 'Windows 10 y 11, x64'],
    ['Conexiones', 'solo api.anthropic.com'],
    ['Frecuencia', 'cada 5 min; respeta los límites'],
    ['Telemetría', 'ninguna', 'sage'],
    ['Ajustes', '%APPDATA%\\Tally'],
    ['Licencia', 'MIT · código abierto'],
  ];
  return (
    <section id="especificaciones" className="band">
      <div className="wrap split top">
        <div className="prose reveal">
          <h2>Especificaciones</h2>
          <p>Todo lo que hace, sin letra pequeña.</p>
          <h3>Instalación</h3>
          <ol className="steps">
            <li>Inicia sesión en Claude Code con tu cuenta de Claude: abre una terminal, escribe <span className="mono amber">claude</span> y dentro <span className="mono amber">/login</span>. Solo hace falta una vez.</li>
            <li>Descarga <span className="mono amber">tally.exe</span>, muévelo a una carpeta fija (no lleva instalador) y ábrelo con doble clic.</li>
            <li>Si Windows muestra “Windows protegió su PC”, pulsa <em>Más información</em> y luego <em>Ejecutar de todas formas</em>.</li>
            <li>Aparece en la bandeja. Clic para ver tu uso; el engranaje del popup abre los ajustes.</li>
            <li>Para que se abra solo, activa <em>Con Windows</em> en los ajustes. Si lo quieres en el menú Inicio, clic derecho en <span className="mono">tally.exe</span> y <em>Anclar a Inicio</em>.</li>
          </ol>
          <h3>Atajos</h3>
          <dl className="keys">
            <dt><kbd>Clic</kbd></dt><dd>Abre o cierra el popup</dd>
            <dt><kbd>Clic derecho</kbd></dt><dd>Ajustes, umbrales y salir</dd>
            <dt><kbd>Esc</kbd></dt><dd>Cierra el popup</dd>
            <dt><kbd>Win</kbd> <kbd>B</kbd></dt><dd>Lleva el foco a la bandeja; con flechas y <kbd>Enter</kbd> lo abres sin ratón</dd>
          </dl>
        </div>
        <table className="spec reveal" style={{ '--i': 1 }}>
          <tbody>
            {rows.map(([k, v, tone]) => (
              <tr key={k}><th scope="row">{k}</th><td className={`mono ${tone ?? ''}`}>{v}</td></tr>
            ))}
          </tbody>
        </table>
      </div>
    </section>
  );
}

function Developers() {
  return (
    <section id="desarrolladores" className="band alt">
      <div className="wrap stack-lg">
        <div className="prose reveal">
          <h2>No confíes, <span className="sage">compruébalo.</span></h2>
          <p>El código es abierto, con licencia MIT. Verifica que el archivo que bajaste es el publicado, o compílalo tú mismo en un par de minutos.</p>
        </div>
        <div className="codes">
          <div className="reveal" style={{ '--i': 0 }}>
            <CodeBlock title="PowerShell · verificar la descarga" copy={'Get-FileHash .\\tally.exe -Algorithm SHA256'}>
              <span className="dim"># debe coincidir con el SHA-256 de esta página</span>{'\n'}
              <span className="amber">Get-FileHash</span> .\tally.exe -Algorithm SHA256
            </CodeBlock>
          </div>
          <div className="reveal" style={{ '--i': 1 }}>
            <CodeBlock title="Terminal · compilar desde el código" copy={`git clone ${RELEASE.repo}\ncd usage-for-claude\ncargo build --release`}>
              <span className="amber">git</span> clone {RELEASE.repo}{'\n'}
              <span className="amber">cd</span> usage-for-claude{'\n'}
              <span className="amber">cargo</span> build --release   <span className="dim"># requiere Rust</span>
            </CodeBlock>
          </div>
        </div>
      </div>
    </section>
  );
}

function Changelog() {
  const items = [
    ['+', 'Sesión de 5 h, límite semanal y semanal de Opus, con hora de reinicio y marca de ritmo.'],
    ['+', 'Tres formas, dos densidades, tema claro u oscuro y cuatro acentos.'],
    ['+', 'Avisos de Windows al cruzar el umbral que elijas.'],
    ['+', 'Opción de abrirse al iniciar Windows, sin permisos de administrador.'],
    ['~', 'Dibuja por software en lugar de con la GPU: de 36,5 MB a 5,2 MB de memoria.'],
    ['~', 'Consulta cada 5 minutos y respeta la espera que pide el servidor.'],
  ];
  return (
    <section id="novedades" className="band">
      <div className="wrap split top">
        <div className="prose reveal">
          <h2>Novedades</h2>
          <p>Cada versión, con lo que cambió y por qué.</p>
        </div>
        <article className="release reveal" style={{ '--i': 1 }}>
          <header>
            <h3 className="mono">v{RELEASE.version}</h3>
            <span className="mono dim">{RELEASE.date} · primera versión</span>
          </header>
          <ul className="log">
            {items.map(([m, t]) => (
              <li key={t}><span className={`mono ${m === '+' ? 'sage' : 'blue'}`} aria-label={m === '+' ? 'Nuevo' : 'Cambio'}>{m}</span><span>{t}</span></li>
            ))}
          </ul>
        </article>
      </div>
    </section>
  );
}

function Faq() {
  const qs = [
    ['¿Necesito Claude Code?', 'Sí. El monitor lee la sesión que Claude Code guarda en tu equipo; no tiene inicio de sesión propio. Inicia sesión una vez: escribe claude en una terminal y luego /login. No hace falta tenerlo abierto, pero sí usarlo de vez en cuando para que la sesión siga vigente; si caduca, repite /login. Claude Desktop y la web no sirven.'],
    ['¿Mi sesión sale de mi equipo?', 'No. /login la guarda en tu equipo, en ~/.claude/.credentials.json. Tally solo la lee de ahí para preguntar tu uso a api.anthropic.com, el mismo servidor con el que ya habla Claude Code. No la copia, no la guarda en otro sitio y no la envía a nadie más.'],
    ['¿Qué datos envía y a dónde?', 'Una consulta de uso a api.anthropic.com cada 5 minutos, con tu sesión de Claude Code. Nada más: sin analíticas, sin servidores propios. Tu sesión nunca sale de tu equipo hacia otro sitio.'],
    ['¿Por qué Windows me avisa al abrirlo?', 'El ejecutable aún no está firmado con un certificado de pago, y SmartScreen avisa ante cualquier programa nuevo sin firma. Puedes comprobar que el archivo es el original con el SHA-256 de esta página.'],
    ['¿Es oficial? ¿Lo permite Anthropic?', 'No es oficial. Anthropic limita el uso de las sesiones de suscripción a Claude Code y Claude.ai, y la consulta de uso no está documentada: puede cambiar o dejar de funcionar. Úsalo con tu propio criterio.'],
  ];
  return (
    <section id="preguntas" className="band alt">
      <div className="wrap narrow stack-md">
        <h2 className="reveal">Preguntas</h2>
        <div className="reveal" style={{ '--i': 1 }}>
          {qs.map(([q, a], i) => (
            <details key={q} open={i === 0}>
              <summary>{q}</summary>
              <p>{a}</p>
            </details>
          ))}
        </div>
      </div>
    </section>
  );
}

export default function App() {
  useReveal();
  const scrolled = useScrolled();
  return (
    <>
      <a className="skip" href="#descargar">Saltar al contenido</a>
      <header className={`top ${scrolled ? 'scrolled' : ''}`}>
        <div className="wrap top-inner">
          <a href="#" className="brand"><Logo /> Tally</a>
          <a href="#novedades" className="pill mono">v{RELEASE.version} · novedades</a>
          <nav aria-label="Principal">
            <a href="#personaliza">Personalización</a>
            <a href="#especificaciones">Especificaciones</a>
            <a href="#preguntas">Preguntas</a>
            <a href={RELEASE.repo}>Código fuente</a>
          </nav>
        </div>
      </header>

      <main>
        <Hero />
        <Weight />
        <Flow />
        <Customize />
        <Specs />
        <Developers />
        <Changelog />
        <Faq />
        <section className="band">
          <div className="wrap cta reveal">
            <h2>{RELEASE.size} entre tú y <span className="amber">saber cuánto te queda.</span></h2>
            <DownloadButton />
          </div>
        </section>
      </main>

      <footer className="foot">
        <div className="wrap foot-grid">
          <div className="foot-brand">
            <span className="brand"><Logo /> Tally</span>
            <p>Tu límite de Claude en la bandeja de Windows. Hecho para gastar menos memoria que la pestaña donde lo descargaste.</p>
          </div>
          <nav aria-label="Producto">
            <h3>Producto</h3>
            <ul>
              <li><a href="#descargar">Descargar</a></li>
              <li><a href="#personaliza">Personalización</a></li>
              <li><a href="#como-funciona">Cómo funciona</a></li>
              <li><a href="#especificaciones">Especificaciones</a></li>
            </ul>
          </nav>
          <nav aria-label="Proyecto">
            <h3>Proyecto</h3>
            <ul>
              <li><a href={RELEASE.repo}>Código fuente</a></li>
              <li><a href="#novedades">Novedades</a></li>
              <li><a href={RELEASE.issues}>Reportar un problema</a></li>
              <li><a href={RELEASE.license}>Licencia MIT</a></li>
            </ul>
          </nav>
        </div>
        <div className="wrap foot-legal">
          <p>Tally no está afiliado a Anthropic ni cuenta con su respaldo. Claude es una marca de Anthropic.</p>
          <p className="mono">v{RELEASE.version} · licencia MIT</p>
        </div>
      </footer>
    </>
  );
}
