# Tally

Monitor de la bandeja de Windows que muestra el límite de Claude: sesión de 5 h, semana (todos los modelos) y semana de Opus. Incluye la web de descarga. Repo: https://github.com/wllmvg/usage-for-claude · licencia MIT.

El usuario escribe en español; la interfaz, los textos y las respuestas van en español.

## Estructura

| Ruta | Qué es |
|---|---|
| `src/main.rs` | Ventana, icono de bandeja, menú contextual, temporizadores, notificaciones, estado (`App`) |
| `src/draw.rs` | Dibujo con Direct2D/DirectWrite: popup, icono de bandeja, paletas de colores |
| `src/usage.rs` | Lectura del token, consulta HTTP (WinHTTP), parseo del JSON, fechas ISO, textos de reinicio, tests |
| `src/config.rs` | Ajustes (`%APPDATA%\Tally\config.json`), inicio con Windows (registro `HKCU\...\Run`, valor `Tally`), tema del sistema |
| `installer/` | Instalador Inno Setup (`tally.iss`): carpeta elegible, tarea de inicio con Windows (mismo valor `Tally` del registro que usa la app), acceso en Inicio, página final con los pasos de `/login` (`despues.txt`). `assets.ps1` genera `tally.ico` y las imágenes del asistente |
| `web/` | Web de descarga: React 19 + Vite, sin librerías de animación |
| `web/src/App.jsx` | Todas las secciones; el bloque `RELEASE` arriba tiene versión, tamaño, hash, fecha y enlaces |
| `web/src/Popup.jsx` | Réplica del popup con los mismos tokens de color que `draw.rs` (mantenerlos sincronizados) |
| `web/src/motion.js` | `useProgress`, `useReveal`, `useInView`, `useScrolled` |

## Cómo funciona la app

1. Lee `accessToken` de `~/.claude/.credentials.json` (o `$CLAUDE_CONFIG_DIR/.credentials.json`), el archivo que mantiene Claude Code.
2. `GET https://api.anthropic.com/api/oauth/usage` con `Authorization: Bearer` y `anthropic-beta: oauth-2025-04-20`. Campos usados: `five_hour`, `seven_day`, `seven_day_opus`, cada uno con `utilization` (0–100) y `resets_at` (ISO 8601).
3. Consulta cada 5 min (`POLL_MS`). Ante un 429 respeta `Retry-After` (300 s por defecto, mínimo 60) con `quiet_until` (se guarda en `config.json`, sobrevive a reinicios), y no consulta antes, ni siquiera con el botón de actualizar.
4. Abrir el popup no consulta salvo que nunca haya habido datos.
5. Avisa con un globo de Windows cuando un límite sube de nivel (umbral de aviso o crítico). La primera lectura tras arrancar solo fija la línea base.

## Decisiones que no se ven en el código

- **Dibujo por software** (`D2D1_RENDER_TARGET_TYPE_SOFTWARE`) en el popup y en el icono. Con la GPU, Direct2D carga Direct3D y el driver: 36,5 MB en vez de 5,2 MB. No volver a hardware.
- Al ocultar el popup se liberan el render target y se llama a `EmptyWorkingSet`, para que la RAM en reposo quede baja.
- La RAM que muestra el pie es `PrivateUsage` del propio proceso; se refresca cada 1,5 s solo con el popup abierto.
- `NIN_KEYSELECT` se define a mano (`WM_USER + 1`) porque falta en el crate `windows` 0.58.
- Un clic en la bandeja justo después de que el popup se cierre por perder el foco no lo reabre (ventana de 300 ms en `toggle`).
- Se vuelve a añadir el icono al recibir `TaskbarCreated` (cuando se reinicia Explorer).
- Icono de bandeja sin datos: dibuja el logo (cuadrado redondeado, barras) en una cuadrícula de 22 unidades, igual que `web/public/favicon.svg`.
- Mutex de instancia única `Local\TallyForClaude` y clase de ventana `TallyForClaude`.

## Limitaciones conocidas (deuda)

- **Depende de Claude Code**: solo lee el token, no lo renueva. Renovarlo nosotros rotaría el refresh token de Claude Code. Si caduca, la UI pide abrir Claude Code. No añadir un login OAuth propio: Anthropic restringe el uso de los tokens de suscripción en apps de terceros (documentado en la FAQ de la web).
- **Menús claros en modo oscuro**: los menús nativos no siguen el tema; harían falta llamadas no documentadas de `uxtheme`.
- Los ajustes se editan dentro del mismo popup (engranaje → vista "Ajustes" con chips, `settings_rows` en `main.rs`, `chips` en `draw.rs`), sin ventana aparte. El menú contextual del icono solo tiene "Salir". Los ajustes no se pueden usar con teclado.
- Se distribuye con instalador por usuario (`PrivilegesRequired=lowest`, por defecto en `%LOCALAPPDATA%\Programs\Tally`). El exe no lleva icono propio (la toolchain GNU no trae `windres`); los accesos directos usan `tally.ico` instalado al lado.
- Un token vacío en `.credentials.json` (Claude Code cerró la sesión) se trata como `NoLogin` sin llamar a la API: llamar con token vacío devolvía 429 con una hora de espera.
- Ejecutable sin firmar: SmartScreen avisa. Opciones gratuitas: Microsoft Store (MSIX) o SignPath Foundation.

## Compilar y probar

La ruta del proyecto tiene "ó" y el enlazador de la toolchain GNU falla con ella: compilar siempre con `CARGO_TARGET_DIR` fuera del proyecto.

```powershell
$env:CARGO_TARGET_DIR = "$env:LOCALAPPDATA\monitor-uso-target"
cargo test
cargo build --release     # -> $env:LOCALAPPDATA\monitor-uso-target\release\tally.exe (~384 KB)
```

Toolchain: `stable-x86_64-pc-windows-gnu` (no requiere Visual Studio). Crate `windows` 0.58.

Instalador (requiere Inno Setup 6: `winget install JRSoftware.InnoSetup`):

```powershell
powershell -File installer\assets.ps1   # solo si cambias el logo
& "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe" installer\tally.iss   # -> dist\Tally-Setup-0.1.0.exe
```

La versión está en `Cargo.toml`, `installer/tally.iss` (`AppVersion`) y `RELEASE` en `web/src/App.jsx`.

Web:

```powershell
npm --prefix web install
npm --prefix web run dev      # http://localhost:5173
npm --prefix web run build    # web/dist
```

## Convenciones

- Sin comentarios en el código: el contexto vive en este archivo.
- Dependencias mínimas: Rust solo `windows`, `serde`, `serde_json`; web solo `react`, `react-dom`, `vite`, `@vitejs/plugin-react`.
- Diseño "instrumento silencioso": casi monocromo; el color aparece solo al cruzar umbrales. Web oscura con acentos ámbar `#F2B04A`, salvia `#93BFA3`, azul `#86A8F0`, arcilla `#D98B6C`, rojo `#F26A55`. Tipografías: Schibsted Grotesk y Fragment Mono (Geist se descartó por estar demasiado vista).
- Animaciones: curva `cubic-bezier(.16,1,.3,1)`, contenido visible sin JS (clase `.js` en `<html>`), todo desactivado con `prefers-reduced-motion`.
- La web nunca menciona proyectos de terceros en los que se inspiró, y siempre lleva el aviso de no afiliación con Anthropic.
- No hacer commits: el usuario los hace.

## Pendiente para la primera release

- Publicar `v0.1.0` en GitHub Releases con `Tally-Setup-0.1.0.exe` (`sha256` y `date` ya están en `RELEASE`; si se recompila el instalador, el hash cambia y hay que actualizarlo).
- Activar GitHub Pages (Settings → Pages → Source: GitHub Actions). El workflow `.github/workflows/pages.yml` publica `web/` en cada push a `main`; `base` solo se aplica en `build`.
