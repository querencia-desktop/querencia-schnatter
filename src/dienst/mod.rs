//! Beschreibung eines Web-Dienstes, den die Hülle zeigt (S5): Adresse,
//! erlaubte Hosts im Hauptfenster, Ursprung mit Benachrichtigungsrecht,
//! Zählerregel für den Seitentitel, Benutzerskript. Die Hülle
//! (`crate::huelle`) kennt nur diese Beschreibung, keinen Dienst selbst;
//! heute gibt es genau einen ([`whatsapp`]).

pub mod whatsapp;

/// Was die Hülle lädt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Start {
    /// Eine Adresse im Netz.
    Adresse(String),
    /// Eine lokale Seite (nur Prüfseite): HTML und Basisadresse.
    Html { inhalt: String, basis: String },
}

/// Ein Dienst für die Hülle.
#[derive(Debug, Clone)]
pub struct Dienst {
    /// Kurzname für Protokollzeilen.
    pub kurzname: &'static str,
    pub start: Start,
    /// Hosts, auf denen das Hauptfenster bleiben darf (genau, ohne Joker).
    pub hosts: Vec<String>,
    /// Ursprung (`https://host`), dem Benachrichtigungen vorab erlaubt sind.
    pub benachrichtigungen: String,
    /// Ungelesene Nachrichten aus dem Seitentitel.
    pub zaehler: fn(&str) -> Option<u32>,
    /// Skript, das vor der Seite läuft (Hauptrahmen); `None`: keins.
    pub benutzerskript: Option<&'static str>,
    /// Name des Dienstes in Texten der Hülle (Hilfe, Rückfragen).
    pub anzeigename: &'static str,
}

impl Dienst {
    /// Ob `host` im Hauptfenster bleiben darf.
    pub fn erlaubt(&self, host: &str) -> bool {
        self.hosts.iter().any(|h| h.eq_ignore_ascii_case(host))
    }
}
