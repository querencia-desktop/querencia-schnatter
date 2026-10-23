//! WhatsApp Web (S5): Adresse, erlaubte Hosts, Kennung, Titelzähler und
//! Benutzerskript. Alles Dienstbezogene steht hier, die Hülle bleibt
//! dienstneutral.
//!
//! Kennung (S4 a, Bericht der Machbarkeit §2.2): WebKit gibt sich für
//! web.whatsapp.com von sich aus als macOS-Safari aus (Seiten-Quirk,
//! Standard an); die Seite zeigt dann den QR-Code und wirbt für die
//! Mac-App. Die Linux-Kennung ist WebKits eigene Standardkennung mit
//! abgeschalteten Quirks (`huelle::einstellungen::Kennung::Linux`); sie
//! braucht keine von uns gepflegte Zeichenkette.

use super::{Dienst, Start};

/// Startadresse.
pub const ADRESSE: &str = "https://web.whatsapp.com/";
/// Einziger Host, auf dem das Hauptfenster bleibt (E5); alles andere geht in
/// den Standardbrowser. Bilder und Medien (`*.whatsapp.net`, `blob:`) sind
/// Unterressourcen und davon nicht betroffen.
pub const HOST: &str = "web.whatsapp.com";
/// Ursprung mit Benachrichtigungsrecht vorab (E3).
pub const URSPRUNG: &str = "https://web.whatsapp.com";

/// Benutzerskript: in 0.1 keins. Welle 0.2 ersetzt hier `window.Notification`,
/// um das Absenderbild zu erhalten (Bericht §2.3, „Benachrichtigung mit
/// Absenderbild“).
pub const BENUTZERSKRIPT: Option<&str> = None;

/// Der Dienst für die Hülle.
pub fn dienst() -> Dienst {
    Dienst {
        kurzname: "whatsapp",
        start: Start::Adresse(ADRESSE.to_owned()),
        hosts: vec![HOST.to_owned()],
        benachrichtigungen: URSPRUNG.to_owned(),
        zaehler: ungelesen,
        benutzerskript: BENUTZERSKRIPT,
        anzeigename: "WhatsApp Web",
    }
}

/// Ungelesene Nachrichten aus dem Seitentitel (E10): WhatsApp Web setzt
/// „(3) WhatsApp“, ohne ungelesene Nachrichten „WhatsApp“. `None`, wenn der
/// Titel keine Zahl in Klammern vorn trägt; eine Null zählt als keine.
pub fn ungelesen(titel: &str) -> Option<u32> {
    let rest = titel.trim_start().strip_prefix('(')?;
    let (zahl, _) = rest.split_once(')')?;
    let zahl = zahl.trim().trim_end_matches('+');
    if zahl.is_empty() || !zahl.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    zahl.parse::<u32>().ok().filter(|n| *n > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zaehler_kommt_aus_der_klammer_vorn() {
        assert_eq!(ungelesen("(3) WhatsApp"), Some(3));
        assert_eq!(ungelesen("(12) WhatsApp"), Some(12));
        assert_eq!(ungelesen(" (1) WhatsApp Business"), Some(1));
        assert_eq!(ungelesen("(99+) WhatsApp"), Some(99));
        assert_eq!(ungelesen("WhatsApp"), None);
        assert_eq!(ungelesen("(0) WhatsApp"), None);
        assert_eq!(ungelesen("() WhatsApp"), None);
        assert_eq!(ungelesen("(neu) WhatsApp"), None);
        assert_eq!(ungelesen("WhatsApp (3)"), None);
        assert_eq!(ungelesen("(99999999999) WhatsApp"), None);
        assert_eq!(ungelesen(""), None);
    }

    #[test]
    fn nur_web_whatsapp_com_bleibt_im_fenster() {
        let d = dienst();
        assert!(d.erlaubt("web.whatsapp.com"));
        assert!(d.erlaubt("WEB.WhatsApp.com"));
        assert!(!d.erlaubt("www.whatsapp.com"));
        assert!(!d.erlaubt("faq.whatsapp.com"));
        assert!(!d.erlaubt("web.whatsapp.com.example.org"));
        assert_eq!(d.benachrichtigungen, "https://web.whatsapp.com");
        assert_eq!(d.start, Start::Adresse("https://web.whatsapp.com/".into()));
    }
}
