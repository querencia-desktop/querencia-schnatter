//! Rechte der Seite (E3).
//!
//! - Benachrichtigungen: für den Ursprung des Dienstes vorab erlaubt, im
//!   Signal `initialize-notification-permissions` vor dem Start jedes
//!   Web-Prozesses; nie eine Rückfrage.
//! - Mikrofon, Kamera, Zwischenablage lesen: je Art einmal fragen, die
//!   Antwort speichern (`einstellungen::Recht`); Bearbeiten → Einstellungen
//!   vergisst sie wieder.
//! - Geräteliste (Namen der Mikrofone/Kameras): erlaubt, sobald Mikrofon
//!   oder Kamera erlaubt sind, sonst abgelehnt, ohne Rückfrage.
//! - Alles andere (Standort, Bildschirm teilen, Mauszeiger fangen, DRM,
//!   Speicherzugriff für Dritte, XR) wird abgelehnt.
//!
//! Jede Anfrage einer Seite außerhalb der Hosts des Dienstes wird
//! abgelehnt, ohne zu fragen.

use std::cell::RefCell;
use std::rc::Rc;

use gtk::glib;
use gtk::prelude::*;
use webkit::prelude::*;

use super::dialoge;
use super::einstellungen::{Antwort, Einstellungen, Recht};
use crate::dienst::Dienst;
use crate::i18n::{gettext, gettext_f};

/// Was eine Anfrage braucht.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Bedarf {
    /// Diese Rechte (alle müssen erlaubt sein).
    Rechte(Vec<Recht>),
    Benachrichtigung,
    Geraeteliste,
    /// Immer ablehnen (Name für das Protokoll).
    Nie(&'static str),
}

/// Bedarf einer WebKit-Anfrage.
pub fn bedarf(anfrage: &webkit::PermissionRequest) -> Bedarf {
    if let Some(m) = anfrage.downcast_ref::<webkit::UserMediaPermissionRequest>() {
        if webkit::functions::user_media_permission_is_for_display_device(m) {
            return Bedarf::Nie("bildschirm");
        }
        let mut v = Vec::new();
        if webkit::functions::user_media_permission_is_for_audio_device(m) {
            v.push(Recht::Mikrofon);
        }
        if webkit::functions::user_media_permission_is_for_video_device(m) {
            v.push(Recht::Kamera);
        }
        if v.is_empty() {
            return Bedarf::Nie("medien-ohne-art");
        }
        return Bedarf::Rechte(v);
    }
    if anfrage.is::<webkit::ClipboardPermissionRequest>() {
        return Bedarf::Rechte(vec![Recht::Zwischenablage]);
    }
    if anfrage.is::<webkit::NotificationPermissionRequest>() {
        return Bedarf::Benachrichtigung;
    }
    if anfrage.is::<webkit::DeviceInfoPermissionRequest>() {
        return Bedarf::Geraeteliste;
    }
    if anfrage.is::<webkit::GeolocationPermissionRequest>() {
        return Bedarf::Nie("standort");
    }
    if anfrage.is::<webkit::PointerLockPermissionRequest>() {
        return Bedarf::Nie("zeiger");
    }
    if anfrage.is::<webkit::MediaKeySystemPermissionRequest>() {
        return Bedarf::Nie("drm");
    }
    if anfrage.is::<webkit::WebsiteDataAccessPermissionRequest>() {
        return Bedarf::Nie("speicher-dritter");
    }
    Bedarf::Nie("sonstiges")
}

/// Ob Gerätenamen gezeigt werden dürfen.
pub fn geraeteliste_erlaubt(e: &Einstellungen) -> bool {
    [Recht::Mikrofon, Recht::Kamera]
        .iter()
        .any(|r| e.recht(*r) == Some(Antwort::Erlaubt))
}

/// Die Frage zu `recht` (Hauptzeile, Erklärung) für den Dienst `name`.
pub fn frage_text(recht: Recht, name: &str) -> (String, String) {
    let nachsatz = gettext("Your answer is remembered. Edit → Preferences can forget it again.");
    match recht {
        Recht::Mikrofon => (
            gettext_f(
                "Allow {service} to use the microphone?",
                &[("service", name)],
            ),
            format!(
                "{}\n{nachsatz}",
                gettext("The microphone is needed to record voice messages.")
            ),
        ),
        Recht::Kamera => (
            gettext_f("Allow {service} to use the camera?", &[("service", name)]),
            format!(
                "{}\n{nachsatz}",
                gettext("The camera is needed to take photos. Calls are not possible.")
            ),
        ),
        Recht::Zwischenablage => (
            gettext_f(
                "Allow {service} to read the clipboard?",
                &[("service", name)],
            ),
            format!(
                "{}\n{nachsatz}",
                gettext(
                    "The page can then read what you copied, also from other programs. \
                     Pasting with Ctrl+V works without this."
                )
            ),
        ),
    }
}

/// Bearbeitet eine Anfrage: gespeicherte Antworten gelten, fehlende werden
/// nacheinander erfragt. `true`: Anfrage übernommen (WebKit wartet).
pub fn anfrage(
    fenster: &gtk::Window,
    view: &webkit::WebView,
    dienst: &Rc<Dienst>,
    einstellungen: &Rc<Einstellungen>,
    req: &webkit::PermissionRequest,
) -> bool {
    let host = view
        .uri()
        .and_then(|u| glib::Uri::parse(&u, glib::UriFlags::NONE).ok())
        .and_then(|u| u.host().map(|h| h.to_string()))
        .unwrap_or_default();
    let b = bedarf(req);
    if !dienst.erlaubt(&host) {
        crate::ereignis(
            "recht",
            &format!("{b:?} von fremdem Host {host} -> abgelehnt"),
        );
        req.deny();
        return true;
    }
    match b {
        Bedarf::Benachrichtigung => {
            crate::ereignis("recht", "benachrichtigung -> erlaubt");
            req.allow();
        }
        Bedarf::Geraeteliste => {
            let ok = geraeteliste_erlaubt(einstellungen);
            crate::ereignis("recht", &format!("geraeteliste -> {}", erlaubt_text(ok)));
            if ok { req.allow() } else { req.deny() }
        }
        Bedarf::Nie(art) => {
            crate::ereignis("recht", &format!("{art} -> abgelehnt"));
            req.deny();
        }
        Bedarf::Rechte(rechte) => {
            nacheinander(
                fenster.clone(),
                dienst.clone(),
                einstellungen.clone(),
                req.clone(),
                Rc::new(RefCell::new(rechte)),
            );
        }
    }
    true
}

fn erlaubt_text(ok: bool) -> &'static str {
    if ok { "erlaubt" } else { "abgelehnt" }
}

/// Erfragt das erste noch offene Recht aus `offen`, dann das nächste; am
/// Ende erlaubt oder lehnt die Anfrage ab.
fn nacheinander(
    fenster: gtk::Window,
    dienst: Rc<Dienst>,
    einstellungen: Rc<Einstellungen>,
    req: webkit::PermissionRequest,
    offen: Rc<RefCell<Vec<Recht>>>,
) {
    loop {
        let Some(recht) = offen.borrow().first().copied() else {
            crate::ereignis("recht", "anfrage -> erlaubt");
            req.allow();
            return;
        };
        match einstellungen.recht(recht) {
            Some(Antwort::Erlaubt) => {
                crate::ereignis(
                    "recht",
                    &format!("{} gespeichert erlaubt", recht.schluessel()),
                );
                offen.borrow_mut().remove(0);
            }
            Some(Antwort::Abgelehnt) => {
                crate::ereignis(
                    "recht",
                    &format!("{} gespeichert abgelehnt", recht.schluessel()),
                );
                req.deny();
                return;
            }
            None => break,
        }
    }
    let recht = offen.borrow()[0];
    let (haupt, text) = frage_text(recht, dienst.anzeigename);
    crate::ereignis("recht", &format!("{} frage", recht.schluessel()));
    let f = fenster.clone();
    dialoge::frage(
        &fenster,
        "dialog-question",
        &haupt,
        &text,
        &gettext("_Deny"),
        &gettext("_Allow"),
        move |ja| {
            let antwort = if ja {
                Antwort::Erlaubt
            } else {
                Antwort::Abgelehnt
            };
            einstellungen.recht_setzen(recht, antwort);
            crate::ereignis(
                "recht",
                &format!("{} beantwortet {}", recht.schluessel(), erlaubt_text(ja)),
            );
            nacheinander(f, dienst, einstellungen, req, offen);
        },
    );
}

/// Antwort auf `navigator.permissions.query` aus den gespeicherten Rechten.
/// `true`: beantwortet.
pub fn abfrage(
    dienst: &Dienst,
    einstellungen: &Einstellungen,
    q: &webkit::PermissionStateQuery,
) -> bool {
    let name = q.name().map(|n| n.to_string()).unwrap_or_default();
    let ursprung = q
        .security_origin()
        .map(|o| o.to_str().to_string())
        .unwrap_or_default();
    let eigen = ursprung == dienst.benachrichtigungen;
    let recht = match name.as_str() {
        "microphone" => Some(Recht::Mikrofon),
        "camera" => Some(Recht::Kamera),
        "clipboard-read" => Some(Recht::Zwischenablage),
        _ => None,
    };
    let zustand = match (name.as_str(), recht) {
        ("notifications", _) if eigen => webkit::PermissionState::Granted,
        ("notifications", _) => webkit::PermissionState::Denied,
        (_, Some(r)) if eigen => match einstellungen.recht(r) {
            Some(Antwort::Erlaubt) => webkit::PermissionState::Granted,
            Some(Antwort::Abgelehnt) => webkit::PermissionState::Denied,
            None => webkit::PermissionState::Prompt,
        },
        (_, Some(_)) => webkit::PermissionState::Denied,
        _ => return false,
    };
    crate::ereignis(
        "rechte-abfrage",
        &format!("{name} {ursprung} -> {zustand:?}"),
    );
    q.finish(zustand);
    true
}
