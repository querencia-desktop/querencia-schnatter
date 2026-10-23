//! Meldungen der Seite (Web-Benachrichtigungen, E3).
//!
//! 0.1 nimmt WebKits Standardweg (gemessen): `show-notification` gibt
//! `false` zurück, WebKit schickt die Meldung selbst per D-Bus an
//! `org.freedesktop.Notifications` (Querencia Benachrichtigungen), mit
//! Programmname und `desktop-entry`. Ein Klick kommt als `clicked` zurück,
//! WebKit reicht ihn als `onclick` an die Seite weiter, die Hülle holt das
//! Fenster nach vorn. Ohne Aktivierungs-Token (R6, Welle 0.2) kann labwc das
//! Fenster womöglich nur markieren statt nach vorn holen.

use gtk::prelude::*;
use webkit::prelude::*;

use super::pruefung;

pub fn verbinden(view: &webkit::WebView, fenster: &gtk::ApplicationWindow) {
    let w = fenster.downgrade();
    view.connect_show_notification(move |_, n| {
        // Inhalte (Absender, Text) nur auf der Prüfseite ins Protokoll
        let inhalt = if pruefung::aktiv() {
            format!(
                "titel={:?} text={:?} tag={:?}",
                n.title().unwrap_or_default(),
                n.body().unwrap_or_default(),
                n.tag().unwrap_or_default()
            )
        } else {
            "inhalt verborgen".to_owned()
        };
        crate::ereignis("show-notification", &inhalt);
        let w = w.clone();
        n.connect_clicked(move |_| {
            crate::ereignis("notification-clicked", "");
            if let Some(f) = w.upgrade() {
                f.present();
            }
        });
        false
    });
}
