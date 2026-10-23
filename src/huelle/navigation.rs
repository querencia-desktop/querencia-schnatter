//! Wohin eine Navigation geht (E5): Das Hauptfenster bleibt auf den Hosts
//! des Dienstes, fremde Links öffnet der Standardbrowser, neue Fenster gibt
//! es nicht.
//!
//! WebKit sagt bei einer Navigationsanfrage nicht, ob sie den Hauptrahmen
//! oder einen eingebetteten Rahmen betrifft. Deshalb:
//! - eine Navigation mit Nutzergeste (Klick auf einen Link) zu einem fremden
//!   Ziel geht hinaus und wird hier verworfen: http, https und mailto ohne
//!   Rückfrage an Standardbrowser bzw. Mailprogramm, jedes andere Schema
//!   (`smb:`, `ssh:`, eigene Schemata) erst nach der Rückfrage „Link mit
//!   anderem Programm öffnen?“ mit dem Ziel ([`Ziel::Fragen`], QD-162 a);
//! - eine Navigation ohne Geste bleibt erlaubt (Rahmen in der Seite); für den
//!   Hauptrahmen fängt sie die Antwortprüfung ab
//!   (`is_main_frame_main_resource`): fremde Hauptseiten werden verworfen,
//!   ohne Geste öffnet sich auch kein Browser (keine Seite kann so
//!   Browserfenster aufmachen);
//! - die Hauptseite wechselt nie auf `data:`, `file:` und `blob:` mit
//!   fremdem Ursprung (QD-162 b): `data:` und fremde `blob:` lehnt die
//!   Antwortprüfung des Hauptrahmens ab (Rahmen in der Seite dürfen sie
//!   laden, auch nach einem Klick: gemessen kommt ein per Klick angelegter
//!   `data:`-Rahmen mit Geste an), `file:` schon die Navigationsprüfung;
//!   jede Ablehnung steht ohne Inhalt im Protokoll ([`Ziel::Sperren`]);
//!   `about:` und `blob:` des Dienstes bleiben erlaubt. `javascript:` lehnen
//!   beide Prüfungen ab, doch WebKit 2.52 fragt dafür gar nicht (gemessen):
//!   Es führt eine solche Adresse nur aus, wenn sie aus demselben Ursprung
//!   kommt, und dann wie ein Skript der Seite selbst, ohne Seitenwechsel;
//! - neue Fenster (`target=_blank`, `window.open`) gehen mit Geste hinaus
//!   (wie Links), sonst nirgendwohin.

use gtk::glib;
use gtk::prelude::*;

use crate::dienst::Dienst;
use crate::i18n::{gettext, gettext_f};

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
    /// Ohne Rückfrage im Standardbrowser bzw. Mailprogramm öffnen, hier
    /// verwerfen (http, https, mailto).
    Extern(String),
    /// Erst nach Rückfrage im Standardprogramm des Schemas öffnen, hier
    /// verwerfen.
    Fragen(String),
    /// Verwerfen.
    Verwerfen,
    /// Verwerfen und protokollieren; der Text nennt nur die Art des Ziels,
    /// nie Inhalt oder Adresse.
    Sperren(&'static str),
}

/// Schemata, die ohne Rückfrage hinausgehen.
pub const OHNE_FRAGE: [&str; 3] = ["http", "https", "mailto"];
/// Schemata, die WebKit in der Seite selbst behandelt; nie hinaus.
const SEITE: [&str; 5] = ["about", "blob", "data", "javascript", "file"];

/// Ob `u` (`blob:<ursprung>/<id>`) zum Dienst gehört.
fn blob_eigen(dienst: &Dienst, u: &glib::Uri) -> bool {
    let Ok(innen) = glib::Uri::parse(&u.path(), glib::UriFlags::NONE) else {
        return false;
    };
    let schema = innen.scheme().to_ascii_lowercase();
    (schema == "http" || schema == "https")
        && dienst.erlaubt(&innen.host().map(|h| h.to_string()).unwrap_or_default())
}

/// Hinaus: ohne Rückfrage nur [`OHNE_FRAGE`].
fn hinaus(schema: &str, uri: &str) -> Ziel {
    if OHNE_FRAGE.contains(&schema) {
        Ziel::Extern(uri.to_owned())
    } else {
        Ziel::Fragen(uri.to_owned())
    }
}

/// Die Entscheidung für `uri`.
pub fn entscheiden(dienst: &Dienst, uri: &str, anfrage: Anfrage) -> Ziel {
    let Ok(u) = glib::Uri::parse(uri, glib::UriFlags::NONE) else {
        return Ziel::Verwerfen;
    };
    let schema = u.scheme().to_ascii_lowercase();
    let host = u.host().map(|h| h.to_string()).unwrap_or_default();
    let web = schema == "http" || schema == "https";
    let eigen = (web && dienst.erlaubt(&host))
        || schema == "about"
        || (schema == "blob" && blob_eigen(dienst, &u));
    match anfrage {
        Anfrage::Navigation { geste } => match schema.as_str() {
            _ if eigen => Ziel::Weiter,
            "javascript" => Ziel::Sperren("javascript:"),
            "file" => Ziel::Sperren("file:"),
            // Rahmen in der Seite (auch nach einem Klick); der Hauptrahmen
            // wird an der Antwort geprüft
            "data" | "blob" => Ziel::Weiter,
            s if geste => hinaus(s, uri),
            // Rahmen in der Seite; der Hauptrahmen wird an der Antwort geprüft
            _ if web => Ziel::Weiter,
            _ => Ziel::Verwerfen,
        },
        Anfrage::NeuesFenster { geste } => {
            if geste && !eigen && !SEITE.contains(&schema.as_str()) {
                hinaus(&schema, uri)
            } else {
                Ziel::Verwerfen
            }
        }
        Anfrage::Antwort { hauptseite } => match schema.as_str() {
            _ if !hauptseite || eigen => Ziel::Weiter,
            "data" => Ziel::Sperren("data:"),
            "javascript" => Ziel::Sperren("javascript:"),
            "file" => Ziel::Sperren("file:"),
            "blob" => Ziel::Sperren("blob: mit fremdem Ursprung"),
            _ => Ziel::Verwerfen,
        },
    }
}

/// Öffnet `uri` im Standardprogramm der Sitzung (Browser für http/https;
/// über das Portal, sonst über GIO).
pub fn extern_oeffnen(fenster: &impl IsA<gtk::Window>, uri: &str) {
    crate::ereignis("extern", &super::pruefung::adresse(Some(uri)));
    let launcher = gtk::UriLauncher::new(uri);
    let schema = uri.split(':').next().unwrap_or_default().to_owned();
    launcher.launch(
        Some(fenster.as_ref()),
        None::<&gtk::gio::Cancellable>,
        move |r| {
            if let Err(e) = r {
                crate::log(&format!("Link ({schema}:) nicht geöffnet: {e}"));
            }
        },
    );
}

/// Höchstens so viele Zeichen des Ziels zeigt die Rückfrage.
const ZIEL_ZEICHEN: usize = 300;

/// Fragt, ob `uri` mit dem Standardprogramm seines Schemas geöffnet werden
/// soll, und öffnet es nur nach „Öffnen“. Vorgabe ist „Abbrechen“.
pub fn fragen_und_oeffnen(fenster: &gtk::Window, uri: &str) {
    let schema = uri
        .split(':')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    crate::ereignis("fremdes-schema", &format!("frage {schema}:"));
    let mut gezeigt: String = uri.chars().take(ZIEL_ZEICHEN).collect();
    if uri.chars().count() > ZIEL_ZEICHEN {
        gezeigt.push('…');
    }
    let w = fenster.downgrade();
    let u = uri.to_owned();
    super::dialoge::frage_mit_ziel(
        fenster,
        "dialog-warning",
        &gettext("Open the link with another program?"),
        &gettext_f(
            "The link is not a web or email address. Opening it starts the program \
             that is registered for “{scheme}:” addresses with this target:",
            &[("scheme", &schema)],
        ),
        &gezeigt,
        &gettext("_Cancel"),
        &gettext("_Open Link"),
        move |ja| {
            crate::ereignis(
                "fremdes-schema",
                &format!("{schema}: {}", if ja { "geoeffnet" } else { "abgebrochen" }),
            );
            if ja && let Some(w) = w.upgrade() {
                extern_oeffnen(&w, &u);
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

    fn antwort(uri: &str, hauptseite: bool) -> Ziel {
        entscheiden(&whatsapp::dienst(), uri, Anfrage::Antwort { hauptseite })
    }

    #[test]
    fn eigener_host_bleibt_im_fenster() {
        assert_eq!(nav("https://web.whatsapp.com/", false), Ziel::Weiter);
        assert_eq!(
            nav("https://web.whatsapp.com/send?phone=1", true),
            Ziel::Weiter
        );
        assert_eq!(nav("blob:https://web.whatsapp.com/abc", true), Ziel::Weiter);
        assert_eq!(
            nav("blob:https://web.whatsapp.com/abc", false),
            Ziel::Weiter
        );
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
        assert_eq!(nav("kein uri", true), Ziel::Verwerfen);
    }

    #[test]
    fn andere_schemata_nur_nach_rueckfrage() {
        for u in [
            "smb://server/freigabe",
            "ssh://nutzer@example.org",
            "tel:+491234",
            "sftp://example.org/x",
            "eigenes-schema:etwas",
            "MAILTOX:a@b",
        ] {
            assert_eq!(nav(u, true), Ziel::Fragen(u.into()), "{u}");
            // ohne Klick nie, weder fragen noch öffnen
            assert_eq!(nav(u, false), Ziel::Verwerfen, "{u}");
        }
        let d = whatsapp::dienst();
        let neu = |u: &str, geste| entscheiden(&d, u, Anfrage::NeuesFenster { geste });
        assert_eq!(
            neu("smb://server/x", true),
            Ziel::Fragen("smb://server/x".into())
        );
        assert_eq!(neu("smb://server/x", false), Ziel::Verwerfen);
        // Groß geschriebene Schemata gelten wie klein geschriebene
        assert_eq!(
            nav("HTTPS://example.org/", true),
            Ziel::Extern("HTTPS://example.org/".into())
        );
    }

    #[test]
    fn hauptseite_wechselt_nie_auf_seiteninterne_schemata() {
        // javascript: und file: schon an der Navigation, mit und ohne Klick
        for geste in [true, false] {
            assert_eq!(
                nav("javascript:alert(1)", geste),
                Ziel::Sperren("javascript:")
            );
            assert_eq!(nav("file:///etc/passwd", geste), Ziel::Sperren("file:"));
            // data: und fremde blob: dürfen Rahmen laden, nie hinaus …
            assert_eq!(nav("data:text/html,x", geste), Ziel::Weiter);
            assert_eq!(nav("blob:https://example.org/abc", geste), Ziel::Weiter);
        }
        // … die Hauptseite sperrt die Antwortprüfung
        assert_eq!(antwort("data:text/html,x", true), Ziel::Sperren("data:"));
        assert_eq!(
            antwort("blob:null/abc", true),
            Ziel::Sperren("blob: mit fremdem Ursprung")
        );
        assert_eq!(antwort("javascript:1", true), Ziel::Sperren("javascript:"));
        assert_eq!(antwort("data:image/png;base64,AA", false), Ziel::Weiter);
        assert_eq!(
            antwort("blob:https://example.org/abc", true),
            Ziel::Sperren("blob: mit fremdem Ursprung")
        );
        assert_eq!(
            antwort("blob:https://web.whatsapp.com/abc", true),
            Ziel::Weiter
        );
        assert_eq!(antwort("about:blank", true), Ziel::Weiter);
        assert_eq!(antwort("file:///etc/passwd", true), Ziel::Sperren("file:"));
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
        assert_eq!(neu("blob:https://example.org/x", true), Ziel::Verwerfen);
        assert_eq!(neu("data:text/html,x", true), Ziel::Verwerfen);
        assert_eq!(neu("javascript:1", true), Ziel::Verwerfen);
        assert_eq!(neu("file:///tmp/x", true), Ziel::Verwerfen);
    }

    #[test]
    fn fremde_hauptseite_wird_an_der_antwort_verworfen() {
        assert_eq!(antwort("https://example.org/", true), Ziel::Verwerfen);
        assert_eq!(antwort("https://example.org/", false), Ziel::Weiter);
        assert_eq!(antwort("https://web.whatsapp.com/", true), Ziel::Weiter);
        assert_eq!(antwort("http://web.whatsapp.com/", true), Ziel::Weiter);
    }
}
