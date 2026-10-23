//! Die einzigen unsicheren Stellen des Programms (QD-169): die EGL-Abfrage
//! im Probe-Kindprozess und das Setzen der Umgebung beim Start.
//!
//! Die Abfrage legt auf der surfaceless-Plattform von Mesa einen
//! OpenGL-ES-2-Kontext ohne Fläche an und liest `GL_VENDOR`,
//! `GL_RENDERER` und `GL_VERSION` – derselbe Treiber, den WebKits
//! Web-Prozess lädt. Sie läuft nur in einem eigenen Kindprozess
//! (`--grafik-probe`): Stürzt der Treiber dabei ab oder hängt er, bleibt die
//! Hülle davon unberührt, und Treiber und Vulkan-Gerät liegen danach nicht im
//! Speicher der Hülle.

#![allow(unsafe_code)]

use std::ffi::{CStr, c_char, c_void};
use std::ptr;

type EglDisplay = *mut c_void;
type EglConfig = *mut c_void;
type EglContext = *mut c_void;
type EglSurface = *mut c_void;
type EglBoolean = u32;
type EglInt = i32;
type EglEnum = u32;
type EglAttrib = isize;

const EGL_PLATFORM_SURFACELESS_MESA: EglEnum = 0x31DD;
const EGL_NONE: EglInt = 0x3038;
const EGL_RENDERABLE_TYPE: EglInt = 0x3040;
const EGL_OPENGL_ES2_BIT: EglInt = 0x0004;
const EGL_OPENGL_ES_API: EglEnum = 0x30A0;
const EGL_CONTEXT_CLIENT_VERSION: EglInt = 0x3098;
const GL_VENDOR: u32 = 0x1F00;
const GL_RENDERER: u32 = 0x1F01;
const GL_VERSION: u32 = 0x1F02;

#[link(name = "EGL")]
unsafe extern "C" {
    fn eglGetPlatformDisplay(
        platform: EglEnum,
        native: *mut c_void,
        attribs: *const EglAttrib,
    ) -> EglDisplay;
    fn eglInitialize(dpy: EglDisplay, major: *mut EglInt, minor: *mut EglInt) -> EglBoolean;
    fn eglBindAPI(api: EglEnum) -> EglBoolean;
    fn eglChooseConfig(
        dpy: EglDisplay,
        attribs: *const EglInt,
        configs: *mut EglConfig,
        size: EglInt,
        anzahl: *mut EglInt,
    ) -> EglBoolean;
    fn eglCreateContext(
        dpy: EglDisplay,
        config: EglConfig,
        share: EglContext,
        attribs: *const EglInt,
    ) -> EglContext;
    fn eglMakeCurrent(
        dpy: EglDisplay,
        draw: EglSurface,
        read: EglSurface,
        ctx: EglContext,
    ) -> EglBoolean;
    fn eglGetProcAddress(name: *const c_char) -> *mut c_void;
    fn eglDestroyContext(dpy: EglDisplay, ctx: EglContext) -> EglBoolean;
    fn eglTerminate(dpy: EglDisplay) -> EglBoolean;
}

type GlGetString = unsafe extern "C" fn(name: u32) -> *const u8;

/// Was die Abfrage liest.
pub struct Gelesen {
    pub anbieter: String,
    pub renderer: String,
    pub version: String,
}

/// Wo die Abfrage scheitert (Exit-Code des Kindprozesses).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fehler {
    Anzeige = 3,
    Kontext = 4,
    Aktivieren = 5,
    Funktion = 6,
}

/// Legt einen surfaceless GLES-2-Kontext an und liest die drei Zeichenketten.
pub fn abfragen() -> Result<Gelesen, Fehler> {
    // SAFETY: reine C-Aufrufe in die Mesa-EGL-Bibliothek mit gültigen,
    // mit EGL_NONE abgeschlossenen Attributlisten und Zeigern auf lokale
    // Variablen; jedes Ergebnis wird vor der Weiterverwendung geprüft.
    // glGetString kommt über eglGetProcAddress (EGL 1.5, Mesa liefert auch
    // Kernfunktionen) und wird nur mit einem aktiven Kontext gerufen; die
    // Zeichenketten gehören dem Treiber und werden vor eglTerminate kopiert.
    unsafe {
        let dpy =
            eglGetPlatformDisplay(EGL_PLATFORM_SURFACELESS_MESA, ptr::null_mut(), ptr::null());
        if dpy.is_null() {
            return Err(Fehler::Anzeige);
        }
        let (mut major, mut minor) = (0, 0);
        if eglInitialize(dpy, &mut major, &mut minor) == 0 {
            return Err(Fehler::Anzeige);
        }
        let ergebnis = (|| {
            if eglBindAPI(EGL_OPENGL_ES_API) == 0 {
                return Err(Fehler::Kontext);
            }
            let wahl = [EGL_RENDERABLE_TYPE, EGL_OPENGL_ES2_BIT, EGL_NONE];
            let mut config: EglConfig = ptr::null_mut();
            let mut anzahl = 0;
            if eglChooseConfig(dpy, wahl.as_ptr(), &mut config, 1, &mut anzahl) == 0 || anzahl < 1 {
                // EGL_KHR_no_config_context: Kontext ohne Konfiguration
                config = ptr::null_mut();
            }
            let attribute = [EGL_CONTEXT_CLIENT_VERSION, 2, EGL_NONE];
            let ctx = eglCreateContext(dpy, config, ptr::null_mut(), attribute.as_ptr());
            if ctx.is_null() {
                return Err(Fehler::Kontext);
            }
            if eglMakeCurrent(dpy, ptr::null_mut(), ptr::null_mut(), ctx) == 0 {
                eglDestroyContext(dpy, ctx);
                return Err(Fehler::Aktivieren);
            }
            let f = eglGetProcAddress(c"glGetString".as_ptr());
            let r = if f.is_null() {
                Err(Fehler::Funktion)
            } else {
                let get: GlGetString = std::mem::transmute::<*mut c_void, GlGetString>(f);
                let lesen = |name| {
                    let p = get(name);
                    if p.is_null() {
                        String::new()
                    } else {
                        CStr::from_ptr(p.cast()).to_string_lossy().into_owned()
                    }
                };
                Ok(Gelesen {
                    anbieter: lesen(GL_VENDOR),
                    renderer: lesen(GL_RENDERER),
                    version: lesen(GL_VERSION),
                })
            };
            eglMakeCurrent(dpy, ptr::null_mut(), ptr::null_mut(), ptr::null_mut());
            eglDestroyContext(dpy, ctx);
            r
        })();
        eglTerminate(dpy);
        ergebnis
    }
}

/// Setzt `name` auf `wert` in der eigenen Prozessumgebung.
///
/// Nur beim Start aus `main` rufen, bevor GTK, GLib oder WebKit einen
/// Thread anlegen: Die Web-Prozesse erben die Umgebung.
pub fn umgebung_setzen(name: &str, wert: &str) {
    // SAFETY: einziger Aufruf aus `grafik::vorbereiten`, das `main` als
    // Erstes ruft; zu diesem Zeitpunkt läuft nur der Hauptfaden (nur der
    // Probe-Kindprozess wurde gestartet und ist schon beendet), niemand
    // liest die Umgebung gleichzeitig.
    unsafe { std::env::set_var(name, wert) }
}
