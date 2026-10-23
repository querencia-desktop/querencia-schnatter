//! Wohin eine Navigation geht (E5): Das Hauptfenster bleibt auf den Hosts
//! des Dienstes, fremde Links öffnet der Standardbrowser, neue Fenster gibt
//! es nicht.
//!
//! WebKit sagt bei einer Navigationsanfrage nicht, ob sie den Hauptrahmen
//! oder einen eingebetteten Rahmen betrifft. Deshalb:
//! - eine Navigation mit Nutzergeste (Klick auf einen Link) zu einem fremden
//!   Ziel geht in den Browser und wird hier verworfen;
//! - eine Navigation ohne Geste bleibt erlaubt (Rahmen in der Seite); für den
//!   Hauptrahmen fängt sie die Antwortprüfung ab
//!   (`is_main_frame_main_resource`): fremde Hauptseiten werden verworfen,
//!   ohne Geste öffnet sich auch kein Browser (keine Seite kann so
//!   Browserfenster aufmachen);
//! - neue Fenster (`target=_blank`, `window.open`) gehen mit Geste in den
//!   Browser, sonst nirgendwohin.

use gtk::glib;
use gtk::prelude::*;

use crate::dienst::Dienst;

/// Art der Anfrage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anfrage {
    Navigation {
        geste: bool,
    },
    NeuesFenster {
        geste: bool,
    },
    /// Antwort auf eine Anfrage; `hauptseite`: Hauptressource des Hauptrahmens.
    Antwort {
        hauptseite: bool,
    },
}

/// Was mit der Anfrage geschieht.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ziel {
    /// WebKit entscheidet wie üblich (laden, herunterladen).
    Weiter,
    /// Im Standardbrowser bzw. Standardprogramm öffnen, hier verwerfen.
    Extern(String),
    /// Verwerfen.
    Verwerfen,
}

/// Schemata, die WebKit in der Seite selbst behandelt.
const INTERN: [&str; 4] = ["about", "blob", "data", "javascript"];

/// Die Entscheidung für `uri`.
pub fn entscheiden(dienst: &Dienst, uri: &str, anfrage: Anfrage) -> Ziel {
    let Ok(u) = glib::Uri::parse(uri, glib::UriFlags::NONE) else {
        return Ziel::Verwerfen;
    };
    let schema = u.scheme().to_ascii_lowercase();
    let host = u.host().map(|h| h.to_string()).unwrap_or_default();
    let web = schema == "http" || schema == "https";
    let eigen = web && dienst.erlaubt(&host);
    match anfrage {
        Anfrage::Navigation { geste } => {
            if eigen || INTERN.contains(&schema.as_str()) {
                Ziel::Weiter
            } else if schema == "file" {
                Ziel::Verwerfen
            } else if geste {
                Ziel::Extern(uri.to_owned())
            } else if web {
                // Rahmen in der Seite; der Hauptrahmen wird an der Antwort geprüft
                Ziel::Weiter
            } else {
                Ziel::Verwerfen
            }
        }
        Anfrage::NeuesFenster { geste } => {
            if geste && !eigen && schema != "file" && !INTERN.contains(&schema.as_str()) {
                Ziel::Extern(uri.to_owned())
            } else {
                Ziel::Verwerfen
            }
        }
        Anfrage::Antwort { hauptseite } => {
            if !hauptseite || eigen || INTERN.contains(&schema.as_str()) {
                Ziel::Weiter
            } else {
                Ziel::Verwerfen
            }
        }
    }
}

/// Öffnet `uri` im Standardprogramm der Sitzung (Browser für http/https;
/// über das Portal, sonst über GIO).
pub fn extern_oeffnen(fenster: &impl IsA<gtk::Window>, uri: &str) {
    crate::ereignis("extern", uri);
    let launcher = gtk::UriLauncher::new(uri);
    let u = uri.to_owned();
    launcher.launch(
        Some(fenster.as_ref()),
        None::<&gtk::gio::Cancellable>,
        move |r| {
            if let Err(e) = r {
                crate::log(&format!("{u} nicht geöffnet: {e}"));
            }
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dienst::whatsapp;

    fn nav(uri: &str, geste: bool) -> Ziel {
        entscheiden(&whatsapp::dienst(), uri, Anfrage::Navigation { geste })
    }

    #[test]
    fn eigener_host_bleibt_im_fenster() {
        assert_eq!(nav("https://web.whatsapp.com/", false), Ziel::Weiter);
        assert_eq!(
            nav("https://web.whatsapp.com/send?phone=1", true),
            Ziel::Weiter
        );
        assert_eq!(nav("blob:https://web.whatsapp.com/abc", true), Ziel::Weiter);
        assert_eq!(nav("about:blank", false), Ziel::Weiter);
    }

    #[test]
    fn fremder_link_mit_klick_geht_in_den_browser() {
        assert_eq!(
            nav("https://example.org/a?b=c", true),
            Ziel::Extern("https://example.org/a?b=c".into())
        );
        assert_eq!(
            nav("https://faq.whatsapp.com/", true),
            Ziel::Extern("https://faq.whatsapp.com/".into())
        );
        assert_eq!(
            nav("mailto:jemand@example.org", true),
            Ziel::Extern("mailto:jemand@example.org".into())
        );
        // ohne Klick: Rahmen erlaubt (Hauptrahmen prüft die Antwort), mailto nie
        assert_eq!(nav("https://example.org/", false), Ziel::Weiter);
        assert_eq!(nav("mailto:jemand@example.org", false), Ziel::Verwerfen);
        // lokale Dateien nie, auch nicht mit Klick
        assert_eq!(nav("file:///etc/passwd", true), Ziel::Verwerfen);
        assert_eq!(nav("kein uri", true), Ziel::Verwerfen);
    }

    #[test]
    fn neue_fenster_nur_mit_klick_und_nur_fremd() {
        let d = whatsapp::dienst();
        let neu = |u: &str, geste| entscheiden(&d, u, Anfrage::NeuesFenster { geste });
        assert_eq!(
            neu("https://example.org/", true),
            Ziel::Extern("https://example.org/".into())
        );
        assert_eq!(neu("https://example.org/", false), Ziel::Verwerfen);
        assert_eq!(neu("https://web.whatsapp.com/", true), Ziel::Verwerfen);
        assert_eq!(
            neu("blob:https://web.whatsapp.com/x", true),
            Ziel::Verwerfen
        );
        assert_eq!(neu("file:///tmp/x", true), Ziel::Verwerfen);
    }

    #[test]
    fn fremde_hauptseite_wird_an_der_antwort_verworfen() {
        let d = whatsapp::dienst();
        let antwort = |u: &str, hauptseite| entscheiden(&d, u, Anfrage::Antwort { hauptseite });
        assert_eq!(antwort("https://example.org/", true), Ziel::Verwerfen);
        assert_eq!(antwort("https://example.org/", false), Ziel::Weiter);
        assert_eq!(antwort("https://web.whatsapp.com/", true), Ziel::Weiter);
        assert_eq!(antwort("http://web.whatsapp.com/", true), Ziel::Weiter);
    }
}
