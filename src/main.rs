//! Querencia Webhülle (englisch „Querencia Web Wrapper“) – eine Hülle für
//! WhatsApp Web im eigenen Fenster für Querencia Desktop. Bis 0.1.1 hieß
//! das Programm „Querencia Nachrichten“ (`querencia-nachrichten`,
//! `org.querencia.Nachrichten`); die Ordner zieht `huelle::umzug` um.
//!
//! Eine schlanke GTK-4-Hülle um eine WebKitGTK-6.0-Ansicht: Menüleiste aus
//! `querencia-oberflaeche`, sonst nur die Seite. Dauerhafte Sitzung,
//! Meldungen über WebKits Standardweg an Querencia Benachrichtigungen,
//! Downloads in den Download-Ordner, Rechte je Art. Keine Anrufe (Ubuntus
//! WebKitGTK hat kein WebRTC). Kein Produkt von WhatsApp oder Meta und nicht
//! mit ihnen verbunden.
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

pub const APP_ID: &str = "org.querencia.Webhuelle";
const PROGRAMM: &str = "querencia-webhuelle";
const STIL: &str = include_str!("style.css");

/// Eine Protokollzeile auf stderr („querencia-webhuelle: …“).
pub fn log(text: &str) {
    eprintln!("{PROGRAMM}: {text}");
}

fn debug() -> Option<Instant> {
    static BEGINN: OnceLock<Option<Instant>> = OnceLock::new();
    *BEGINN.get_or_init(|| {
        (std::env::var("QUERENCIA_WEBHUELLE_DEBUG").as_deref() == Ok("1")).then(Instant::now)
    })
}

/// Eine Ereigniszeile für Prüfungen, nur mit `QUERENCIA_WEBHUELLE_DEBUG=1`
/// (`huelle::pruefung`): „querencia-webhuelle: E 12.345 art text“.
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
    if std::env::args_os().nth(1).as_deref()
        == Some(std::ffi::OsStr::new(huelle::grafik::PROBE_ARG))
    {
        return huelle::grafik::probe_im_kind();
    }
    debug();
    // Ordner des früheren Namens übernehmen, bevor die Einstellungen
    // gelesen werden (nur Dateisystem, kein Faden)
    let ordner = huelle::sitzung::Ordner::aus_xdg();
    let umzug = huelle::umzug::beim_start(&ordner);
    // QD-169: vor allem anderen (nur der Hauptfaden läuft), damit die
    // Web-Prozesse die Umgebung erben
    let einstellungen = huelle::einstellungen::Einstellungen::laden(&ordner.einstellungen());
    huelle::grafik::vorbereiten(einstellungen.grafik());
    i18n::init();
    let modus = portal::prepare(PROGRAMM);
    glib::set_application_name(&i18n::gettext("Querencia Web Wrapper"));
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
        huelle::starten(
            app,
            huelle::pruefung::dienst(dienst::whatsapp::dienst()),
            &umzug,
        );
    });
    app.run()
}
