//! Nebenfenster: Rückfragen (Rechte, Abmelden), Einstellungen, Hilfe, Info.
//!
//! Jedes Nebenfenster wird bei jedem Öffnen neu gebaut und beim Schließen
//! zerstört, nie verborgen und wieder gezeigt (QD-076: labwc 0.9.8 dekoriert
//! ein wieder gezeigtes GTK-Fenster nicht mehr; Muster Querencia Dokumente).
//! Ein schon offenes Fenster derselben Art wird nach vorn geholt (`Platz`).

use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gdk, glib};

use super::einstellungen::{Antwort, Einstellungen, Kennung, Recht};
use crate::i18n::{gettext, gettext_f};

/// Platz für ein Nebenfenster; hält das offene Fenster nur schwach.
#[derive(Clone, Default)]
pub struct Platz(Rc<glib::WeakRef<gtk::Window>>);

impl Platz {
    /// Holt ein offenes Fenster nach vorn; `true`, wenn es eins gab.
    pub fn vorholen(&self) -> bool {
        match self.0.upgrade() {
            Some(w) => {
                w.present();
                true
            }
            None => false,
        }
    }

    pub fn merken(&self, w: &gtk::Window) {
        self.0.set(Some(w));
    }
}

/// Leeres Nebenfenster zu `eltern`, gehört zur Anwendung, Escape schließt.
pub fn fenster(eltern: &gtk::Window, titel: &str, modal: bool) -> gtk::Window {
    let w = gtk::Window::builder()
        .transient_for(eltern)
        .modal(modal)
        .destroy_with_parent(true)
        .resizable(false)
        .title(titel)
        .build();
    if let Some(app) = eltern.application() {
        w.set_application(Some(&app));
    }
    w.add_css_class("qn-dialog");
    let tasten = gtk::EventControllerKey::new();
    let ww = w.downgrade();
    tasten.connect_key_pressed(move |_, taste, _, _| {
        if taste == gdk::Key::Escape {
            if let Some(w) = ww.upgrade() {
                spaeter_schliessen(&w);
            }
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });
    w.add_controller(tasten);
    w
}

/// Schließt `w` im nächsten Leerlauf (nicht aus dem laufenden Handler).
pub fn spaeter_schliessen(w: &gtk::Window) {
    let w = w.clone();
    glib::idle_add_local_once(move || w.close());
}

/// Symbol links, fette Hauptzeile und Erklärung rechts (GNOME-2-Meldung).
fn meldung(symbol: &str, haupt: &str, text: &str) -> gtk::Box {
    let bild = gtk::Image::from_icon_name(symbol);
    bild.set_pixel_size(48);
    bild.set_valign(gtk::Align::Start);
    let h = gtk::Label::new(None);
    h.set_markup(&format!(
        "<b><big>{}</big></b>",
        glib::markup_escape_text(haupt)
    ));
    h.set_xalign(0.0);
    h.set_wrap(true);
    h.set_max_width_chars(50);
    let t = gtk::Label::new(Some(text));
    t.set_xalign(0.0);
    t.set_wrap(true);
    t.set_max_width_chars(50);
    let rechts = gtk::Box::new(gtk::Orientation::Vertical, 8);
    rechts.append(&h);
    rechts.append(&t);
    let b = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    b.append(&bild);
    b.append(&rechts);
    b
}

/// Knopfzeile rechts unten.
fn knopfzeile(knoepfe: &[&gtk::Button]) -> gtk::Box {
    let zeile = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    zeile.set_halign(gtk::Align::End);
    zeile.set_margin_top(12);
    for k in knoepfe {
        zeile.append(*k);
    }
    zeile
}

fn rahmen(inhalt: &impl IsA<gtk::Widget>, zeile: &gtk::Box) -> gtk::Box {
    let aussen = gtk::Box::new(gtk::Orientation::Vertical, 0);
    for (setze, wert) in [
        (gtk::Widget::set_margin_top as fn(&gtk::Widget, i32), 12),
        (gtk::Widget::set_margin_bottom, 12),
        (gtk::Widget::set_margin_start, 12),
        (gtk::Widget::set_margin_end, 12),
    ] {
        setze(aussen.upcast_ref(), wert);
    }
    aussen.append(inhalt);
    aussen.append(zeile);
    aussen
}

/// Wartende Antwort einer Rückfrage.
type Antwortfn = Box<dyn FnOnce(bool)>;

/// Rückfrage mit zwei Knöpfen; `antwort(true)` für den zweiten. Schließen
/// des Fensters (Escape, Titelleiste) zählt wie der erste Knopf. Die
/// Antwort kommt genau einmal.
pub fn frage(
    eltern: &gtk::Window,
    symbol: &str,
    haupt: &str,
    text: &str,
    nein: &str,
    ja: &str,
    antwort: impl FnOnce(bool) + 'static,
) {
    let w = fenster(eltern, &gettext("Querencia Messages"), true);
    let k_nein = gtk::Button::with_mnemonic(nein);
    let k_ja = gtk::Button::with_mnemonic(ja);
    k_ja.add_css_class("suggested-action");
    let rest: Rc<RefCell<Option<Antwortfn>>> = Rc::new(RefCell::new(Some(Box::new(antwort))));
    let geben = {
        let rest = rest.clone();
        move |wert: bool| {
            if let Some(f) = rest.borrow_mut().take() {
                f(wert);
            }
        }
    };
    let ww = w.downgrade();
    let g = geben.clone();
    k_nein.connect_clicked(move |_| {
        g(false);
        if let Some(w) = ww.upgrade() {
            spaeter_schliessen(&w);
        }
    });
    let ww = w.downgrade();
    let g = geben.clone();
    k_ja.connect_clicked(move |_| {
        g(true);
        if let Some(w) = ww.upgrade() {
            spaeter_schliessen(&w);
        }
    });
    w.connect_close_request(move |_| {
        geben(false);
        glib::Propagation::Proceed
    });
    w.set_child(Some(&rahmen(
        &meldung(symbol, haupt, text),
        &knopfzeile(&[&k_nein, &k_ja]),
    )));
    w.set_default_widget(Some(&k_nein));
    GtkWindowExt::set_focus(&w, Some(&k_nein));
    w.present();
}

/// Inhalt mit einem Knopf „Schließen“.
fn mit_schliessen(w: &gtk::Window, inhalt: &impl IsA<gtk::Widget>) {
    let k = gtk::Button::with_mnemonic(&gettext("_Close"));
    let ww = w.downgrade();
    k.connect_clicked(move |_| {
        if let Some(w) = ww.upgrade() {
            spaeter_schliessen(&w);
        }
    });
    w.set_child(Some(&rahmen(inhalt, &knopfzeile(&[&k]))));
    w.set_default_widget(Some(&k));
    GtkWindowExt::set_focus(w, Some(&k));
}

/// Website von Querencia Desktop (Info-Dialog).
pub const WEBSITE: &str = "https://querencialinux.org";

/// Info: Symbol, Name, Version, Hinweis auf die Alpha und die fehlende
/// Verbindung zu WhatsApp, Website, Copyright, Lizenz, WebKitGTK-Stand.
pub fn info(eltern: &gtk::Window, platz: &Platz, symbol: &str) {
    if platz.vorholen() {
        return;
    }
    // TRANSLATORS: Fenstertitel des Info-Dialogs
    let w = fenster(eltern, &gettext("About Querencia Messages"), false);
    let inhalt = gtk::Box::new(gtk::Orientation::Vertical, 8);
    let bild = gtk::Image::from_icon_name(symbol);
    bild.set_pixel_size(64);
    inhalt.append(&bild);
    let name = gtk::Label::new(None);
    let titel = gettext_f(
        "Querencia Messages {version}",
        &[("version", env!("CARGO_PKG_VERSION"))],
    );
    name.set_markup(&format!(
        "<b><big>{}</big></b>",
        glib::markup_escape_text(&titel)
    ));
    inhalt.append(&name);
    let text = gtk::Label::new(Some(&gettext(
        "WhatsApp Web in its own window for Querencia Desktop.\n\n\
         Not affiliated with WhatsApp or Meta. WhatsApp is a trademark\n\
         of its owner. Calls are not possible.",
    )));
    text.set_justify(gtk::Justification::Center);
    inhalt.append(&text);
    inhalt.append(&gtk::Label::new(Some(&gettext(
        "Alpha – not released for everyday use.",
    ))));
    let web = gtk::Label::new(None);
    web.set_markup(&format!("<a href=\"{WEBSITE}\">querencialinux.org</a>"));
    let ww = w.downgrade();
    web.connect_activate_link(move |_, uri| {
        if let Some(w) = ww.upgrade() {
            super::navigation::extern_oeffnen(&w, uri);
        }
        glib::Propagation::Stop
    });
    inhalt.append(&web);
    let webkit_stand = format!(
        "{}.{}.{}",
        webkit::functions::major_version(),
        webkit::functions::minor_version(),
        webkit::functions::micro_version()
    );
    for zeile in [
        "© 2026 Querencia Desktop contributors".to_owned(),
        gettext("License: GPL-3.0-or-later"),
        gettext_f(
            "Web engine: WebKitGTK {version} (LGPL)",
            &[("version", &webkit_stand)],
        ),
    ] {
        let l = gtk::Label::new(Some(&zeile));
        l.add_css_class("dim-label");
        inhalt.append(&l);
    }
    mit_schliessen(&w, &inhalt);
    platz.merken(&w);
    w.present();
}

/// Die Hilfe als kurze Seite: Anmelden, keine Anrufe, Meldungen, Downloads,
/// Rechte, Abmelden.
pub fn hilfe(eltern: &gtk::Window, platz: &Platz) {
    if platz.vorholen() {
        return;
    }
    let w = fenster(eltern, &gettext("Querencia Messages Help"), false);
    w.set_resizable(true);
    w.set_default_size(560, 520);
    let inhalt = gtk::Box::new(gtk::Orientation::Vertical, 6);
    let abschnitte: [(String, String); 7] = [
        (
            gettext("What this is"),
            gettext(
                "Querencia Messages shows WhatsApp Web in its own window. It is not \
                 affiliated with WhatsApp; your messages go directly between this \
                 window and WhatsApp.",
            ),
        ),
        (
            gettext("Signing in"),
            gettext(
                "Open WhatsApp on your phone, choose “Linked devices”, then “Link a \
                 device”, and scan the QR code in the window. You stay signed in \
                 until you log out.",
            ),
        ),
        (
            gettext("No calls"),
            gettext(
                "Voice and video calls are not possible: the web engine of Ubuntu \
                 is built without WebRTC. Take calls on your phone. Voice messages \
                 work.",
            ),
        ),
        (
            gettext("Notifications"),
            gettext(
                "New messages are announced by the notification area of the \
                 desktop while the window is open. Closing the window quits the \
                 program; then there are no notifications.",
            ),
        ),
        (
            gettext("Downloads"),
            gettext(
                "Downloaded files go to your Downloads folder without a question; \
                 a file never overwrites another one. A notification offers to \
                 open the file or its folder.",
            ),
        ),
        (
            gettext("Microphone, camera, clipboard"),
            gettext(
                "The page asks once for each; the answer is remembered. Edit → \
                 Preferences forgets the answers.",
            ),
        ),
        (
            gettext("Logging out"),
            gettext(
                "File → Log Out and Delete Data removes everything this program \
                 stored for WhatsApp Web on this computer. Your chats stay on \
                 your phone.",
            ),
        ),
    ];
    for (titel, text) in abschnitte {
        let t = gtk::Label::new(None);
        t.set_markup(&format!("<b>{}</b>", glib::markup_escape_text(&titel)));
        t.set_xalign(0.0);
        t.set_margin_top(6);
        let l = gtk::Label::new(Some(&text));
        l.set_xalign(0.0);
        l.set_wrap(true);
        l.set_max_width_chars(60);
        inhalt.append(&t);
        inhalt.append(&l);
    }
    let rollen = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vexpand(true)
        .child(&inhalt)
        .build();
    mit_schliessen(&w, &rollen);
    platz.merken(&w);
    w.present();
}

/// Die Einstellungen (Bearbeiten → Einstellungen): Kennung, Töne,
/// gespeicherte Rechte. Jede Änderung gilt sofort; `geaendert` erfährt sie.
pub fn einstellungen(
    eltern: &gtk::Window,
    platz: &Platz,
    e: &Rc<Einstellungen>,
    geaendert: Rc<dyn Fn(Aenderung)>,
) {
    if platz.vorholen() {
        return;
    }
    let w = fenster(eltern, &gettext("Querencia Messages Preferences"), false);
    let inhalt = gtk::Box::new(gtk::Orientation::Vertical, 6);
    let ueberschrift = |text: &str| {
        let l = gtk::Label::new(None);
        l.set_markup(&format!("<b>{}</b>", glib::markup_escape_text(text)));
        l.set_xalign(0.0);
        l.set_margin_top(6);
        l
    };
    inhalt.append(&ueberschrift(&gettext("Browser identification")));
    let standard = gtk::CheckButton::with_mnemonic(&gettext(
        "_Standard of the web engine (WhatsApp sees Safari on a Mac)",
    ));
    let linux =
        gtk::CheckButton::with_mnemonic(&gettext("_Linux identification without site exceptions"));
    linux.set_group(Some(&standard));
    match e.kennung() {
        Kennung::Standard => standard.set_active(true),
        Kennung::Linux => linux.set_active(true),
    }
    let hinweis = gtk::Label::new(Some(&gettext(
        "Changing it reloads the page. If WhatsApp Web stops working, switch back.",
    )));
    hinweis.set_xalign(0.0);
    hinweis.set_wrap(true);
    hinweis.set_max_width_chars(55);
    hinweis.add_css_class("dim-label");
    for k in [&standard, &linux] {
        k.set_margin_start(12);
        inhalt.append(k);
    }
    hinweis.set_margin_start(12);
    inhalt.append(&hinweis);
    let ee = e.clone();
    let g = geaendert.clone();
    linux.connect_toggled(move |k| {
        let neu = if k.is_active() {
            Kennung::Linux
        } else {
            Kennung::Standard
        };
        if ee.kennung() != neu {
            ee.kennung_setzen(neu);
            g(Aenderung::Kennung);
        }
    });

    inhalt.append(&ueberschrift(&gettext("Sound")));
    let ton = gtk::CheckButton::with_mnemonic(&gettext("_Play sounds and voice messages"));
    ton.set_active(e.ton());
    ton.set_margin_start(12);
    let ee = e.clone();
    let g = geaendert.clone();
    ton.connect_toggled(move |k| {
        ee.ton_setzen(k.is_active());
        g(Aenderung::Ton);
    });
    inhalt.append(&ton);

    inhalt.append(&ueberschrift(&gettext("Remembered permissions")));
    let liste = gtk::Grid::builder()
        .column_spacing(12)
        .row_spacing(2)
        .margin_start(12)
        .build();
    let zustaende: Vec<gtk::Label> = Recht::ALLE
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let name = gtk::Label::new(Some(&recht_name(*r)));
            name.set_xalign(0.0);
            let z = gtk::Label::new(Some(&antwort_text(e.recht(*r))));
            z.set_xalign(0.0);
            z.add_css_class("dim-label");
            liste.attach(&name, 0, i as i32, 1, 1);
            liste.attach(&z, 1, i as i32, 1, 1);
            z
        })
        .collect();
    inhalt.append(&liste);
    let vergessen = gtk::Button::with_mnemonic(&gettext("_Forget Answers"));
    vergessen.set_halign(gtk::Align::Start);
    vergessen.set_margin_start(12);
    vergessen.set_margin_top(4);
    let ee = e.clone();
    vergessen.connect_clicked(move |_| {
        ee.rechte_vergessen();
        for z in &zustaende {
            z.set_text(&antwort_text(None));
        }
        geaendert(Aenderung::Rechte);
    });
    inhalt.append(&vergessen);
    mit_schliessen(&w, &inhalt);
    platz.merken(&w);
    w.present();
}

/// Was in den Einstellungen geändert wurde.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Aenderung {
    Kennung,
    Ton,
    Rechte,
}

fn recht_name(r: Recht) -> String {
    match r {
        Recht::Mikrofon => gettext("Microphone"),
        Recht::Kamera => gettext("Camera"),
        Recht::Zwischenablage => gettext("Reading the clipboard"),
    }
}

fn antwort_text(a: Option<Antwort>) -> String {
    match a {
        Some(Antwort::Erlaubt) => gettext("allowed"),
        Some(Antwort::Abgelehnt) => gettext("denied"),
        None => gettext("not asked yet"),
    }
}
