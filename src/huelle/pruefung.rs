//! Prüfhilfen im Programm, nur über Umgebungsvariablen (docs/testing.md):
//!
//! - `QUERENCIA_WEBHUELLE_DEBUG=1`: Ereigniszeilen auf stderr
//!   (`crate::ereignis`); Adressen ohne Abfrage und Fragment, Inhalte von
//!   Meldungen nur auf der Prüfseite.
//! - `QUERENCIA_WEBHUELLE_PRUEFSEITE=<datei.html>`: statt des Dienstes die
//!   lokale Prüfseite unter `http://localhost/` (kein Netz); Hosts und
//!   Benachrichtigungsrecht gelten dann für `localhost`, Titelzähler und
//!   Benutzerskript bleiben die des Dienstes. Der Fenstertitel sagt
//!   „Prüfseite“.
//! - `QUERENCIA_WEBHUELLE_ZUSTAND=1`: alle 2 s (höchstens 3 Minuten) eine
//!   Zeile mit Merkmalen der Seite (Kennung, QR-Code vorhanden, dunkles
//!   Schema, Rechte), ohne Seitentext.

use std::sync::OnceLock;
use std::time::{Duration, Instant};

use gtk::glib;
use webkit::prelude::*;

use crate::dienst::{Dienst, Start};

const VAR_SEITE: &str = "QUERENCIA_WEBHUELLE_PRUEFSEITE";
const VAR_ZUSTAND: &str = "QUERENCIA_WEBHUELLE_ZUSTAND";
const BASIS: &str = "http://localhost/";

fn seite() -> &'static Option<String> {
    static SEITE: OnceLock<Option<String>> = OnceLock::new();
    SEITE.get_or_init(|| {
        let pfad = std::env::var_os(VAR_SEITE).filter(|p| !p.is_empty())?;
        match std::fs::read_to_string(&pfad) {
            Ok(s) => Some(s),
            Err(e) => {
                crate::log(&format!(
                    "Prüfseite {} unlesbar, Dienst gilt: {e}",
                    std::path::Path::new(&pfad).display()
                ));
                None
            }
        }
    })
}

/// Ob die Prüfseite statt des Dienstes läuft.
pub fn aktiv() -> bool {
    seite().is_some()
}

/// Der Dienst, auf der Prüfseite mit lokaler Seite und `localhost`.
pub fn dienst(echt: Dienst) -> Dienst {
    let Some(inhalt) = seite().clone() else {
        return echt;
    };
    Dienst {
        kurzname: "pruefseite",
        start: Start::Html {
            inhalt,
            basis: BASIS.to_owned(),
        },
        hosts: vec!["localhost".to_owned()],
        benachrichtigungen: "http://localhost".to_owned(),
        anzeigename: "Prüfseite",
        ..echt
    }
}

/// `uri` fürs Protokoll: auf der Prüfseite ganz, sonst bei http(s) ohne
/// Abfrage und Fragment (dort könnten Telefonnummern stehen), bei jedem
/// anderen Schema nur das Schema (`mailto:`, `data:` und Co. tragen den
/// Inhalt im Pfad).
pub fn adresse(uri: Option<&str>) -> String {
    let uri = uri.unwrap_or_default();
    if aktiv() {
        return uri.to_owned();
    }
    match glib::Uri::parse(uri, glib::UriFlags::NONE) {
        Ok(u) if matches!(u.scheme().to_ascii_lowercase().as_str(), "http" | "https") => format!(
            "{}://{}{}",
            u.scheme(),
            u.host().unwrap_or_default(),
            u.path()
        ),
        Ok(u) => format!("{}:", u.scheme()),
        Err(_) => "(keine Adresse)".to_owned(),
    }
}

/// Merkmale der Seite, ohne Seitentext.
const ZUSTAND_JS: &str = r#"(() => {
  const t = document.body ? document.body.innerText : '';
  const qr = document.querySelector('canvas');
  return JSON.stringify({
    titel: document.title,
    host: location.host,
    ua: navigator.userAgent,
    qr: !!qr,
    qr_groesse: qr ? [qr.width, qr.height] : null,
    dunkel: matchMedia('(prefers-color-scheme: dark)').matches,
    notification: typeof Notification !== 'undefined' ? Notification.permission : 'fehlt',
    sicher: window.isSecureContext,
    sprache: navigator.language,
    rtc: typeof RTCPeerConnection !== 'undefined',
    mac_hinweis: /\bMac\b/.test(t),
    angemeldet_bleiben: /angemeldet bleiben|stay (logged|signed) in|keep me (logged|signed) in/i.test(t),
    nicht_unterstuetzt: /not supported|nicht unterst|update (your )?browser/i.test(t),
  });
})()"#;

/// Beobachtet die Seite, wenn `QUERENCIA_WEBHUELLE_ZUSTAND=1`.
pub fn zustand_beobachten(view: &webkit::WebView) {
    if std::env::var(VAR_ZUSTAND).as_deref() != Ok("1") {
        return;
    }
    let beginn = Instant::now();
    let v = view.downgrade();
    glib::timeout_add_local(Duration::from_secs(2), move || {
        let Some(view) = v.upgrade() else {
            return glib::ControlFlow::Break;
        };
        if beginn.elapsed() > Duration::from_secs(180) {
            return glib::ControlFlow::Break;
        }
        view.evaluate_javascript(
            ZUSTAND_JS,
            None,
            None,
            None::<&gtk::gio::Cancellable>,
            |r| match r {
                Ok(wert) => crate::ereignis("zustand", &wert.to_str()),
                Err(e) => crate::ereignis("zustand-fehler", e.message()),
            },
        );
        glib::ControlFlow::Continue
    });
}
