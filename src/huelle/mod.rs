//! Die dienstneutrale Hülle (S5): Fenster, Sitzung, Rechte, Downloads,
//! Meldungen, Zoom, Neuladen. Was sie zeigt, beschreibt ein
//! [`crate::dienst::Dienst`].

pub mod dialoge;
pub mod downloads;
pub mod einstellungen;
pub mod fenster;
pub mod grafik;
pub mod meldungen;
pub mod menue;
pub mod navigation;
pub mod pruefung;
pub mod rechte;
pub mod rechtschreibung;
pub mod sitzung;
pub mod sprache;
pub mod umzug;
pub mod zoom;

use std::cell::RefCell;
use std::rc::Rc;

use gtk::gio;
use gtk::prelude::*;

use crate::dienst::Dienst;
use crate::i18n::{gettext, gettext_f};
use einstellungen::Einstellungen;
use fenster::Fenster;
use sitzung::Ordner;

/// Programmsymbol (`data/icons/org.querencia.Schnatter.svg`, hicolor):
/// grüne Sprechblase mit Schnabel, kein Logo eines Dienstes.
pub const SYMBOL: &str = "org.querencia.Schnatter";

/// Startet die Hülle in `app` für `dienst`: Ordner, Sitzung, Rechte vorab,
/// Rechtschreibung, Downloads, ein Fenster. Ein zweiter Start holt nur das
/// vorhandene Fenster nach vorn. Ließ der Umzug (`umzug`) keinen Start zu,
/// zeigt das Fenster nur den Grund.
pub fn starten(app: &gtk::Application, dienst: Dienst, umzug: &umzug::Stand) {
    crate::ereignis("umzug", &umzug.ereignis());
    let sperre = match umzug {
        s if s.start_erlaubt() => None,
        umzug::Stand::Fertig(_) => None,
        umzug::Stand::AltesProgrammLaeuft { name, .. } => Some(gettext_f(
            "{program} is still running. Close it and start Schnatter again: it \
             then takes over the sign-in and the settings.",
            &[("program", &umzug::anzeigename(name))],
        )),
        umzug::Stand::Gescheitert(grund) => Some(gettext_f(
            "The folders of the previous version could not be taken over: {error}. \
             Nothing was deleted.",
            &[("error", grund)],
        )),
    };
    let dienst = Rc::new(dienst);
    let haupt: Rc<RefCell<Option<Rc<Fenster>>>> = Rc::default();
    let h = haupt.clone();
    app.connect_activate(move |app| {
        if let Some(f) = h.borrow().as_ref() {
            f.zeigen();
            return;
        }
        if let Some(text) = &sperre {
            fehler_und_ende(app, text);
            return;
        }
        crate::ereignis(
            "start",
            &format!(
                "dienst={} version={}",
                dienst.kurzname,
                env!("CARGO_PKG_VERSION")
            ),
        );
        let ordner = Ordner::aus_xdg();
        if let Err(e) = ordner.vorbereiten() {
            crate::log(&format!("Sitzungsordner nicht nutzbar: {e}"));
            fehler_und_ende(
                app,
                &gettext_f(
                    "The data folder cannot be used: {error}",
                    &[("error", &e.to_string())],
                ),
            );
            return;
        }
        let einstellungen = Rc::new(Einstellungen::laden(&ordner.einstellungen()));
        let sitzung = sitzung::netzwerksitzung(&ordner);
        let ctx = webkit::WebContext::default().expect("WebKit-Kontext");
        // Benachrichtigungen vorab erlaubt (E3), vor jedem Web-Prozess
        let ursprung = dienst.benachrichtigungen.clone();
        ctx.connect_initialize_notification_permissions(move |ctx| {
            crate::ereignis("initialize-notification-permissions", &ursprung);
            let o = webkit::SecurityOrigin::for_uri(&ursprung);
            ctx.initialize_notification_permissions(&[&o], &[]);
        });
        rechtschreibung::einrichten(&ctx);
        sprache::einrichten(&ctx);
        downloads::einrichten(&sitzung, app);
        let f = Fenster::neu(app, dienst.clone(), einstellungen, &sitzung, ordner);
        f.laden();
        f.zeigen();
        h.replace(Some(f));
    });
    let a = gio::SimpleAction::new("beenden", None);
    let w = app.downgrade();
    a.connect_activate(move |_, _| {
        if let Some(app) = w.upgrade() {
            // Fenster schließen (merkt die Größe), dann beenden
            for f in app.windows() {
                f.close();
            }
            app.quit();
        }
    });
    app.add_action(&a);
    downloads::aktionen(app);
    menue::kuerzel_setzen(app);
    let _ = haupt;
}

fn fehler_und_ende(app: &gtk::Application, text: &str) {
    let w = gtk::ApplicationWindow::new(app);
    w.set_title(Some(&gettext("Schnatter")));
    let l = gtk::Label::new(Some(text));
    l.set_margin_top(24);
    l.set_margin_bottom(24);
    l.set_margin_start(24);
    l.set_margin_end(24);
    l.set_wrap(true);
    l.set_max_width_chars(60);
    w.set_child(Some(&l));
    w.present();
}
