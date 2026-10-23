//! Einstellungen der Hülle in `XDG_CONFIG_HOME/querencia-nachrichten/
//! einstellungen.ini` (GLib-Schlüsseldatei, Ordner 0700): Fenstergröße,
//! Zoom, Kennung (S4), Ton (E4) und die gespeicherten Rechte je Art (E3).
//!
//! Jede Änderung wird sofort geschrieben (`g_file_set_contents`, atomar).
//! Fehlt die Datei oder ist sie unlesbar, gelten die Vorgaben.

use std::path::{Path, PathBuf};

use gtk::glib;

const FENSTER: &str = "Fenster";
const SEITE: &str = "Seite";
const RECHTE: &str = "Rechte";

/// Vorgabegröße des Fensters (passt auch auf 1024×768).
pub const BREITE: i32 = 1000;
pub const HOEHE: i32 = 700;

/// Wie sich die Webansicht ausgibt (S4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kennung {
    /// WebKit-Standard mit Seiten-Quirks (für WhatsApp: macOS-Safari).
    Standard,
    /// WebKits Linux-Kennung, Seiten-Quirks aus.
    Linux,
}

impl Kennung {
    pub fn schluessel(self) -> &'static str {
        match self {
            Kennung::Standard => "standard",
            Kennung::Linux => "linux",
        }
    }

    pub fn aus(text: &str) -> Option<Self> {
        match text {
            "standard" => Some(Kennung::Standard),
            "linux" => Some(Kennung::Linux),
            _ => None,
        }
    }
}

/// Ein Recht, das einmal erfragt und dann gespeichert wird (E3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Recht {
    Mikrofon,
    Kamera,
    Zwischenablage,
}

impl Recht {
    pub const ALLE: [Recht; 3] = [Recht::Mikrofon, Recht::Kamera, Recht::Zwischenablage];

    pub fn schluessel(self) -> &'static str {
        match self {
            Recht::Mikrofon => "mikrofon",
            Recht::Kamera => "kamera",
            Recht::Zwischenablage => "zwischenablage-lesen",
        }
    }
}

/// Die gespeicherte Antwort auf eine Rechte-Frage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Antwort {
    Erlaubt,
    Abgelehnt,
}

pub struct Einstellungen {
    datei: PathBuf,
    werte: glib::KeyFile,
}

impl Einstellungen {
    /// Liest `datei`; fehlt sie, sind alle Werte Vorgaben.
    pub fn laden(datei: &Path) -> Self {
        let werte = glib::KeyFile::new();
        if datei.exists()
            && let Err(e) = werte.load_from_file(datei, glib::KeyFileFlags::KEEP_COMMENTS)
        {
            crate::log(&format!(
                "Einstellungen unlesbar, Vorgaben gelten ({}): {e}",
                datei.display()
            ));
        }
        Self {
            datei: datei.to_path_buf(),
            werte,
        }
    }

    fn speichern(&self) {
        if let Err(e) = self.werte.save_to_file(&self.datei) {
            crate::log(&format!(
                "Einstellungen nicht gespeichert ({}): {e}",
                self.datei.display()
            ));
        }
    }

    /// Breite, Höhe, maximiert.
    pub fn fenster(&self) -> (i32, i32, bool) {
        let b = self
            .werte
            .integer(FENSTER, "breite")
            .ok()
            .filter(|b| (320..=16384).contains(b))
            .unwrap_or(BREITE);
        let h = self
            .werte
            .integer(FENSTER, "hoehe")
            .ok()
            .filter(|h| (240..=16384).contains(h))
            .unwrap_or(HOEHE);
        let m = self.werte.boolean(FENSTER, "maximiert").unwrap_or(false);
        (b, h, m)
    }

    pub fn fenster_setzen(&self, breite: i32, hoehe: i32, maximiert: bool) {
        if self.fenster() == (breite, hoehe, maximiert) {
            return;
        }
        self.werte.set_integer(FENSTER, "breite", breite);
        self.werte.set_integer(FENSTER, "hoehe", hoehe);
        self.werte.set_boolean(FENSTER, "maximiert", maximiert);
        self.speichern();
    }

    pub fn zoom(&self) -> f64 {
        self.werte
            .double(SEITE, "zoom")
            .ok()
            .map(super::zoom::begrenzen)
            .unwrap_or(1.0)
    }

    pub fn zoom_setzen(&self, zoom: f64) {
        if (self.zoom() - zoom).abs() < 1e-6 {
            return;
        }
        self.werte.set_double(SEITE, "zoom", zoom);
        self.speichern();
    }

    pub fn kennung(&self) -> Kennung {
        self.werte
            .string(SEITE, "kennung")
            .ok()
            .and_then(|k| Kennung::aus(k.as_str()))
            .unwrap_or(Kennung::Standard)
    }

    pub fn kennung_setzen(&self, kennung: Kennung) {
        self.werte
            .set_string(SEITE, "kennung", kennung.schluessel());
        self.speichern();
    }

    /// Töne der Seite (Meldungstöne, Sprachnachrichten); Vorgabe an.
    pub fn ton(&self) -> bool {
        self.werte.boolean(SEITE, "ton").unwrap_or(true)
    }

    pub fn ton_setzen(&self, an: bool) {
        self.werte.set_boolean(SEITE, "ton", an);
        self.speichern();
    }

    pub fn recht(&self, recht: Recht) -> Option<Antwort> {
        match self.werte.string(RECHTE, recht.schluessel()).ok()?.as_str() {
            "erlaubt" => Some(Antwort::Erlaubt),
            "abgelehnt" => Some(Antwort::Abgelehnt),
            _ => None,
        }
    }

    pub fn recht_setzen(&self, recht: Recht, antwort: Antwort) {
        let wert = match antwort {
            Antwort::Erlaubt => "erlaubt",
            Antwort::Abgelehnt => "abgelehnt",
        };
        self.werte.set_string(RECHTE, recht.schluessel(), wert);
        self.speichern();
    }

    /// Vergisst alle gespeicherten Rechte (nächste Anfrage fragt wieder).
    pub fn rechte_vergessen(&self) {
        if self.werte.has_group(RECHTE) {
            let _ = self.werte.remove_group(RECHTE);
            self.speichern();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("qn-einst-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d.join("einstellungen.ini")
    }

    #[test]
    fn vorgaben_ohne_datei() {
        let e = Einstellungen::laden(&tmp("leer"));
        assert_eq!(e.fenster(), (BREITE, HOEHE, false));
        assert_eq!(e.zoom(), 1.0);
        assert_eq!(e.kennung(), Kennung::Standard);
        assert!(e.ton());
        for r in Recht::ALLE {
            assert_eq!(e.recht(r), None);
        }
    }

    #[test]
    fn werte_ueberleben_neues_laden() {
        let f = tmp("werte");
        let e = Einstellungen::laden(&f);
        e.fenster_setzen(1200, 800, true);
        e.zoom_setzen(1.25);
        e.kennung_setzen(Kennung::Linux);
        e.ton_setzen(false);
        e.recht_setzen(Recht::Mikrofon, Antwort::Erlaubt);
        e.recht_setzen(Recht::Kamera, Antwort::Abgelehnt);
        let e = Einstellungen::laden(&f);
        assert_eq!(e.fenster(), (1200, 800, true));
        assert_eq!(e.zoom(), 1.25);
        assert_eq!(e.kennung(), Kennung::Linux);
        assert!(!e.ton());
        assert_eq!(e.recht(Recht::Mikrofon), Some(Antwort::Erlaubt));
        assert_eq!(e.recht(Recht::Kamera), Some(Antwort::Abgelehnt));
        assert_eq!(e.recht(Recht::Zwischenablage), None);
        e.rechte_vergessen();
        let e = Einstellungen::laden(&f);
        assert_eq!(e.recht(Recht::Mikrofon), None);
        assert_eq!(e.kennung(), Kennung::Linux);
    }

    #[test]
    fn unsinnige_werte_gelten_als_vorgabe() {
        let f = tmp("unsinn");
        std::fs::write(
            &f,
            "[Fenster]\nbreite=5\nhoehe=99999\n[Seite]\nzoom=40\nkennung=chrome\n\
             [Rechte]\nmikrofon=vielleicht\n",
        )
        .unwrap();
        let e = Einstellungen::laden(&f);
        assert_eq!(e.fenster(), (BREITE, HOEHE, false));
        assert_eq!(e.zoom(), super::super::zoom::GROESSTER);
        assert_eq!(e.kennung(), Kennung::Standard);
        assert_eq!(e.recht(Recht::Mikrofon), None);
        std::fs::write(&f, "kein ini\n[[").unwrap();
        assert_eq!(Einstellungen::laden(&f).kennung(), Kennung::Standard);
    }
}
