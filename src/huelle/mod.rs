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

/// Programmsymbol, vorerst aus dem Symbolthema (Chat-Symbol aus GNOME 2;
/// das eigene Symbol `org.querencia.Nachrichten` liefert QD-166).
pub const SYMBOL: &str = "empathy";

/// Startet die Hülle in `app` für `dienst`: Ordner, Sitzung, Rechte vorab,
/// Rechtschreibung, Downloads, ein Fenster. Ein zweiter Start holt nur das
/// vorhandene Fenster nach vorn.
pub fn starten(app: &gtk::Application, dienst: Dienst) {
    let dienst = Rc::new(dienst);
    let haupt: Rc<RefCell<Option<Rc<Fenster>>>> = Rc::default();
    let h = haupt.clone();
    app.connect_activate(move |app| {
        if let Some(f) = h.borrow().as_ref() {
            f.zeigen();
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
            fehler_und_ende(app, &e.to_string());
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
    w.set_title(Some(&gettext("Querencia Messages")));
    let l = gtk::Label::new(Some(&gettext_f(
        "The data folder cannot be used: {error}",
        &[("error", text)],
    )));
    l.set_margin_top(24);
    l.set_margin_bottom(24);
    l.set_margin_start(24);
    l.set_margin_end(24);
    l.set_wrap(true);
    w.set_child(Some(&l));
    w.present();
}
