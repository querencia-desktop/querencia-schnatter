//! Grafik-Rückfall für die Web-Prozesse (QD-169).
//!
//! Auf nouveau läuft OpenGL über zink auf NVK (Mesa 26); damit stirbt
//! WebKits Web-Prozess beim Öffnen des Bildbetrachters von WhatsApp Web
//! („free(): corrupted unsorted chunks“). Mit Software-OpenGL (llvmpipe)
//! öffnet sich das Bild. Deshalb prüft die Hülle beim Start, welcher
//! OpenGL-Treiber die Web-Prozesse erwartet (Probe im Kindprozess, [`ffi`]),
//! und setzt bei zink auf nouveau in die eigene Umgebung
//! `LIBGL_ALWAYS_SOFTWARE=1`, `MESA_LOADER_DRIVER_OVERRIDE=llvmpipe` und
//! `GALLIUM_DRIVER=llvmpipe` – jede nur, wenn sie noch fehlt. Die
//! Web-Prozesse erben sie. Vulkan bleibt unberührt.
//!
//! Die Einstellung „Software-Grafik“ (automatisch, an, aus; Vorgabe
//! automatisch) gilt ab dem nächsten Start. Der Info-Dialog fragt den
//! Renderer mit der fertigen Umgebung erneut ab ([`renderer_abfragen`]):
//! Das ist der, den die Web-Prozesse haben.

mod ffi;

use std::cell::Cell;
use std::io::Read;
use std::process::{Command, Stdio};
use std::rc::Rc;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use gtk::{gio, glib};

/// Argument, mit dem sich das Programm selbst als Probe startet.
pub const PROBE_ARG: &str = "--grafik-probe";
/// Die Variablen für Software-OpenGL, in dieser Reihenfolge gesetzt.
pub const VARIABLEN: [(&str, &str); 3] = [
    ("LIBGL_ALWAYS_SOFTWARE", "1"),
    ("MESA_LOADER_DRIVER_OVERRIDE", "llvmpipe"),
    ("GALLIUM_DRIVER", "llvmpipe"),
];
/// Länger wartet der Start nicht auf die Probe.
const ZEITLIMIT: Duration = Duration::from_secs(5);

/// Einstellung „Software-Grafik“.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modus {
    /// Software nur, wenn die Probe zink auf nouveau findet (Vorgabe).
    Automatisch,
    /// Immer Software-OpenGL für die Web-Prozesse.
    An,
    /// Nie: der Treiber, den Mesa wählt.
    Aus,
}

impl Modus {
    pub const ALLE: [Modus; 3] = [Modus::Automatisch, Modus::An, Modus::Aus];

    pub fn schluessel(self) -> &'static str {
        match self {
            Modus::Automatisch => "automatisch",
            Modus::An => "an",
            Modus::Aus => "aus",
        }
    }

    pub fn aus(text: &str) -> Option<Self> {
        Modus::ALLE.into_iter().find(|m| m.schluessel() == text)
    }
}

/// Ergebnis einer Probe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Probe {
    Gelesen {
        anbieter: String,
        renderer: String,
        version: String,
    },
    /// Kein EGL, keine Anzeige oder kein Kontext (Exit-Code und Stelle).
    KeinGl(String),
    /// Die Probe ist abgestürzt (Signal) oder hing.
    Gescheitert(String),
}

/// Warum die Web-Prozesse den Treiber haben, den sie haben.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grund {
    /// Modus automatisch: zink auf nouveau erkannt.
    Erkannt,
    /// Modus automatisch: die Probe ist abgestürzt oder hing.
    ProbeGescheitert,
    /// Modus automatisch: ein anderer Treiber oder kein OpenGL.
    Hardware,
    /// Modus an.
    Einstellung,
    /// Modus aus.
    Aus,
    /// Alle drei Variablen standen schon in der Umgebung (z. B. von der
    /// Sitzung); keine Probe.
    Umgebung,
}

/// Stand nach dem Start, für Info-Dialog und Prüfungen.
#[derive(Debug, Clone)]
pub struct Stand {
    pub modus: Modus,
    pub grund: Grund,
    /// Was die Hülle selbst gesetzt hat.
    pub gesetzt: Vec<&'static str>,
}

static STAND: OnceLock<Stand> = OnceLock::new();

/// Der Stand nach [`vorbereiten`].
pub fn stand() -> Option<&'static Stand> {
    STAND.get()
}

/// Ob der Renderer zink auf nouveau ist: „zink“ und „NVK“ oder „nouveau“
/// im Renderer (Mesa: „zink Vulkan 1.4(NVIDIA GeForce RTX 3060 (NVK GA106)
/// (MESA_NVK))“).
pub fn zink_auf_nouveau(renderer: &str) -> bool {
    let r = renderer.to_ascii_lowercase();
    r.contains("zink") && (r.contains("nvk") || r.contains("nouveau"))
}

/// Ob die Web-Prozesse Software-OpenGL bekommen, und warum.
pub fn entscheiden(modus: Modus, probe: Option<&Probe>) -> (bool, Grund) {
    match modus {
        Modus::An => (true, Grund::Einstellung),
        Modus::Aus => (false, Grund::Aus),
        Modus::Automatisch => match probe {
            Some(Probe::Gelesen { renderer, .. }) if zink_auf_nouveau(renderer) => {
                (true, Grund::Erkannt)
            }
            // Eine Probe, die im Treiber stirbt oder hängt, sagt dasselbe
            // über den Web-Prozess voraus.
            Some(Probe::Gescheitert(_)) => (true, Grund::ProbeGescheitert),
            _ => (false, Grund::Hardware),
        },
    }
}

/// Wertet die Ausgabe der Probe aus (`vendor=`, `renderer=`, `version=`).
fn lesen(ausgabe: &str) -> Option<Probe> {
    let wert = |k: &str| {
        ausgabe
            .lines()
            .find_map(|z| z.strip_prefix(k)?.strip_prefix('='))
            .map(str::to_owned)
    };
    Some(Probe::Gelesen {
        anbieter: wert("vendor")?,
        renderer: wert("renderer")?,
        version: wert("version")?,
    })
}

fn deutung(code: Option<i32>, signal: Option<i32>) -> Probe {
    match (code, signal) {
        (_, Some(s)) => Probe::Gescheitert(format!("Signal {s}")),
        (Some(3), _) => Probe::KeinGl("keine EGL-Anzeige (surfaceless)".into()),
        (Some(4), _) => Probe::KeinGl("kein GLES-Kontext".into()),
        (Some(5), _) => Probe::KeinGl("Kontext nicht aktivierbar".into()),
        (Some(6), _) => Probe::KeinGl("glGetString fehlt".into()),
        (c, _) => Probe::KeinGl(format!("Exit {c:?}")),
    }
}

/// Der Probe-Kindprozess (`querencia-nachrichten --grafik-probe`): drei
/// Zeilen auf stdout, Exit 0; sonst Exit 3–6 ([`ffi::Fehler`]).
pub fn probe_im_kind() -> glib::ExitCode {
    match ffi::abfragen() {
        Ok(g) => {
            let ein = |s: &str| s.replace(['\n', '\r'], " ");
            println!("vendor={}", ein(&g.anbieter));
            println!("renderer={}", ein(&g.renderer));
            println!("version={}", ein(&g.version));
            glib::ExitCode::SUCCESS
        }
        Err(f) => glib::ExitCode::from(f as u8),
    }
}

/// Startet die Probe mit der aktuellen Umgebung und wartet höchstens
/// [`ZEITLIMIT`].
fn probe_starten() -> Probe {
    let exe = match std::env::current_exe() {
        Ok(e) => e,
        Err(e) => return Probe::KeinGl(format!("eigenes Programm nicht gefunden: {e}")),
    };
    let mut kind = match Command::new(exe)
        .arg(PROBE_ARG)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(k) => k,
        Err(e) => return Probe::KeinGl(format!("Probe nicht startbar: {e}")),
    };
    let beginn = Instant::now();
    let status = loop {
        match kind.try_wait() {
            Ok(Some(s)) => break s,
            Ok(None) if beginn.elapsed() < ZEITLIMIT => {
                std::thread::sleep(Duration::from_millis(5))
            }
            Ok(None) => {
                let _ = kind.kill();
                let _ = kind.wait();
                return Probe::Gescheitert(format!("keine Antwort in {} s", ZEITLIMIT.as_secs()));
            }
            Err(e) => return Probe::KeinGl(format!("Probe nicht abwartbar: {e}")),
        }
    };
    let mut ausgabe = String::new();
    if let Some(mut o) = kind.stdout.take() {
        let _ = o.read_to_string(&mut ausgabe);
    }
    use std::os::unix::process::ExitStatusExt;
    match (status.success(), lesen(&ausgabe)) {
        (true, Some(p)) => p,
        _ => deutung(status.code(), status.signal()),
    }
}

/// Liest die Einstellung, prüft den Treiber und setzt die Variablen; nur
/// aus `main`, bevor GTK startet (siehe [`ffi::umgebung_setzen`]).
pub fn vorbereiten(modus: Modus) {
    let vorhanden = |n: &str| std::env::var_os(n).is_some();
    let stand = if VARIABLEN.iter().all(|(n, _)| vorhanden(n)) {
        let werte: Vec<String> = VARIABLEN
            .iter()
            .map(|(n, _)| format!("{n}={}", std::env::var(n).unwrap_or_default()))
            .collect();
        crate::log(&format!(
            "Grafik: Umgebung bringt {} mit, keine Probe (Einstellung {})",
            werte.join(" "),
            modus.schluessel()
        ));
        Stand {
            modus,
            grund: Grund::Umgebung,
            gesetzt: Vec::new(),
        }
    } else {
        let beginn = Instant::now();
        let probe = (modus == Modus::Automatisch).then(probe_starten);
        let dauer = beginn.elapsed().as_millis();
        let (software, grund) = entscheiden(modus, probe.as_ref());
        let mut gesetzt = Vec::new();
        if software {
            for (n, w) in VARIABLEN {
                if !vorhanden(n) {
                    ffi::umgebung_setzen(n, w);
                    gesetzt.push(n);
                }
            }
        }
        let befund = match &probe {
            Some(Probe::Gelesen {
                anbieter, renderer, ..
            }) => format!("OpenGL „{renderer}“ ({anbieter}) in {dauer} ms"),
            Some(Probe::KeinGl(w)) => format!("kein OpenGL ({w}) in {dauer} ms"),
            Some(Probe::Gescheitert(w)) => format!("Probe gescheitert ({w}) nach {dauer} ms"),
            None => "keine Probe".to_owned(),
        };
        let folge = if software {
            let liste: Vec<String> = VARIABLEN
                .iter()
                .filter(|(n, _)| gesetzt.contains(n))
                .map(|(n, w)| format!("{n}={w}"))
                .collect();
            format!(
                "Web-Prozesse mit Software-OpenGL (gesetzt: {})",
                if liste.is_empty() {
                    "nichts, alles schon da".to_owned()
                } else {
                    liste.join(" ")
                }
            )
        } else {
            "Web-Prozesse mit dem Treiber von Mesa".to_owned()
        };
        crate::log(&format!(
            "Grafik: Einstellung {}, {befund} – {grund:?}: {folge}",
            modus.schluessel()
        ));
        Stand {
            modus,
            grund,
            gesetzt,
        }
    };
    crate::ereignis(
        "grafik",
        &format!(
            "modus={} grund={:?} gesetzt={:?}",
            stand.modus.schluessel(),
            stand.grund,
            stand.gesetzt
        ),
    );
    let _ = STAND.set(stand);
}

/// Fragt den Renderer mit der jetzigen Umgebung ab (wie ihn die
/// Web-Prozesse sehen), ohne den Hauptfaden zu blockieren.
pub fn renderer_abfragen(fertig: impl FnOnce(Probe) + 'static) {
    let exe = match std::env::current_exe() {
        Ok(e) => e,
        Err(e) => return fertig(Probe::KeinGl(e.to_string())),
    };
    let kind = match gio::Subprocess::newv(
        &[exe.as_os_str(), std::ffi::OsStr::new(PROBE_ARG)],
        gio::SubprocessFlags::STDOUT_PIPE | gio::SubprocessFlags::STDERR_SILENCE,
    ) {
        Ok(k) => k,
        Err(e) => return fertig(Probe::KeinGl(e.to_string())),
    };
    // Hängt die Probe, beendet die Wache sie; sonst entfernt die Antwort die Wache.
    let wache: Rc<Cell<Option<glib::SourceId>>> = Rc::default();
    let (w, k) = (wache.clone(), kind.clone());
    wache.set(Some(glib::timeout_add_local_once(ZEITLIMIT, move || {
        w.take();
        k.force_exit();
    })));
    let k = kind.clone();
    kind.communicate_utf8_async(None, None::<&gio::Cancellable>, move |r| {
        if let Some(w) = wache.take() {
            w.remove();
        }
        let ausgabe = r.ok().and_then(|(o, _)| o).unwrap_or_default();
        let probe = match (k.is_successful(), lesen(&ausgabe)) {
            (true, Some(p)) => p,
            _ if k.has_signaled() => Probe::Gescheitert(format!("Signal {}", k.term_sig())),
            _ => deutung(Some(k.exit_status()), None),
        };
        fertig(probe);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    const NVK: &str = "zink Vulkan 1.4(NVIDIA GeForce RTX 3060 (NVK GA106) (MESA_NVK))";

    fn gelesen(renderer: &str) -> Probe {
        Probe::Gelesen {
            anbieter: "Mesa".into(),
            renderer: renderer.into(),
            version: "OpenGL ES 3.2 Mesa 26.0.8".into(),
        }
    }

    #[test]
    fn zink_auf_nouveau_wird_erkannt() {
        assert!(zink_auf_nouveau(NVK));
        assert!(zink_auf_nouveau("zink (nouveau NV176)"));
        assert!(zink_auf_nouveau("ZINK Vulkan 1.3(NVK TU117)"));
        assert!(!zink_auf_nouveau("NV176"));
        assert!(!zink_auf_nouveau("NVIDIA GeForce RTX 3060/PCIe/SSE2"));
        assert!(!zink_auf_nouveau(
            "zink Vulkan 1.3(AMD Radeon RX 6600 (RADV NAVI23))"
        ));
        assert!(!zink_auf_nouveau(
            "AMD Radeon RX 6600 (radeonsi, navi23, LLVM 20.1.8)"
        ));
        assert!(!zink_auf_nouveau("llvmpipe (LLVM 21.1.2, 256 bits)"));
        assert!(!zink_auf_nouveau(
            "Mesa Intel(R) UHD Graphics 620 (KBL GT2)"
        ));
        assert!(!zink_auf_nouveau(""));
    }

    #[test]
    fn entscheidung_je_modus() {
        let nvk = gelesen(NVK);
        let radv = gelesen("AMD Radeon RX 6600 (radeonsi, navi23)");
        let a = Modus::Automatisch;
        assert_eq!(entscheiden(a, Some(&nvk)), (true, Grund::Erkannt));
        assert_eq!(entscheiden(a, Some(&radv)), (false, Grund::Hardware));
        assert_eq!(
            entscheiden(a, Some(&Probe::Gescheitert("Signal 6".into()))),
            (true, Grund::ProbeGescheitert)
        );
        assert_eq!(
            entscheiden(a, Some(&Probe::KeinGl("Exit 3".into()))),
            (false, Grund::Hardware)
        );
        assert_eq!(entscheiden(a, None), (false, Grund::Hardware));
        assert_eq!(
            entscheiden(Modus::An, Some(&radv)),
            (true, Grund::Einstellung)
        );
        assert_eq!(entscheiden(Modus::An, None), (true, Grund::Einstellung));
        assert_eq!(entscheiden(Modus::Aus, Some(&nvk)), (false, Grund::Aus));
    }

    #[test]
    fn ausgabe_der_probe() {
        let p = lesen(
            "vendor=Mesa\nrenderer=llvmpipe (LLVM 21.1.2, 256 bits)\nversion=OpenGL ES 3.2\n",
        );
        assert_eq!(
            p,
            Some(Probe::Gelesen {
                anbieter: "Mesa".into(),
                renderer: "llvmpipe (LLVM 21.1.2, 256 bits)".into(),
                version: "OpenGL ES 3.2".into(),
            })
        );
        assert_eq!(lesen("vendor=Mesa\n"), None);
        assert_eq!(lesen(""), None);
        assert_eq!(
            deutung(None, Some(11)),
            Probe::Gescheitert("Signal 11".into())
        );
        assert!(matches!(deutung(Some(3), None), Probe::KeinGl(_)));
    }

    #[test]
    fn modus_schluessel() {
        for m in Modus::ALLE {
            assert_eq!(Modus::aus(m.schluessel()), Some(m));
        }
        assert_eq!(Modus::aus("vielleicht"), None);
    }
}
