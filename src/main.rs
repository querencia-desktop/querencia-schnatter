//! Querencia Nachrichten – WhatsApp Web im eigenen Fenster für Querencia
//! Desktop.
//!
//! Eine schlanke GTK-4-Hülle um eine WebKitGTK-6.0-Ansicht: Menüleiste aus
//! `querencia-oberflaeche`, sonst nur die Seite. Dauerhafte Sitzung,
//! Meldungen über WebKits Standardweg an Querencia Benachrichtigungen,
//! Downloads in den Download-Ordner, Rechte je Art. Keine Anrufe (Ubuntus
//! WebKitGTK hat kein WebRTC). Nicht mit WhatsApp verbunden.
//!
//! Aufbau (S5): `huelle` ist dienstneutral, `dienst::whatsapp` beschreibt
//! den einen Dienst. Oberflächentexte englisch über gettext, Deutsch aus
//! `po/de.po` (i18n.rs).

mod dienst;
mod huelle;
mod i18n;

use std::sync::OnceLock;
use std::time::Instant;

use gtk::prelude::*;
use gtk::{gio, glib};
use querencia_oberflaeche::{farbschema, icon_theme, portal};

pub const APP_ID: &str = "org.querencia.Nachrichten";
const PROGRAMM: &str = "querencia-nachrichten";
const STIL: &str = include_str!("style.css");

/// Eine Protokollzeile auf stderr („querencia-nachrichten: …“).
pub fn log(text: &str) {
    eprintln!("{PROGRAMM}: {text}");
}

fn debug() -> Option<Instant> {
    static BEGINN: OnceLock<Option<Instant>> = OnceLock::new();
    *BEGINN.get_or_init(|| {
        (std::env::var("QUERENCIA_NACHRICHTEN_DEBUG").as_deref() == Ok("1")).then(Instant::now)
    })
}

/// Eine Ereigniszeile für Prüfungen, nur mit `QUERENCIA_NACHRICHTEN_DEBUG=1`
/// (`huelle::pruefung`): „querencia-nachrichten: E 12.345 art text“.
pub fn ereignis(art: &str, text: &str) {
    if let Some(b) = debug() {
        eprintln!(
            "{PROGRAMM}: E {:.3} {art} {text}",
            b.elapsed().as_secs_f64()
        );
    }
}

fn stil_laden() {
    let css = gtk::CssProvider::new();
    farbschema::folgen(&css, PROGRAMM);
    css.load_from_string(STIL);
    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &css,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
    farbschema::startfehler_beheben();
}

fn main() -> glib::ExitCode {
    i18n::init();
    debug();
    let modus = portal::prepare(PROGRAMM);
    glib::set_application_name(&i18n::gettext("Querencia Messages"));
    let app = gtk::Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::empty())
        .build();
    app.connect_startup(move |app| {
        portal::follow_gsettings(PROGRAMM, modus);
        gtk::Window::set_default_icon_name(huelle::SYMBOL);
        icon_theme::follow(PROGRAMM);
        stil_laden();
        // QD-081: GTK 4.22 lädt bei „prefer-dark“ das dunkle Thema-CSS nur
        // bei einem Wechsel im laufenden Programm; die Meldung stößt es an.
        if let Some(s) = gtk::Settings::default() {
            s.notify("gtk-interface-color-scheme");
        }
        huelle::starten(app, huelle::pruefung::dienst(dienst::whatsapp::dienst()));
    });
    app.run()
}
