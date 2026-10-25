//! Dauerhafte Netzwerksitzung (E2) und ihre Ordner.
//!
//! Ohne dauerhafte Sitzung gibt es keine Benachrichtigungen (gemessen:
//! `Notification.permission` bleibt `default`) und bei jedem Start eine neue
//! Anmeldung. Daten liegen unter `XDG_DATA_HOME/querencia-schnatter`, der
//! Cache unter `XDG_CACHE_HOME/querencia-schnatter`, die Einstellungen
//! unter `XDG_CONFIG_HOME/querencia-schnatter` (0.2.0 hießen sie
//! `querencia-webhuelle`, bis 0.1.1 `querencia-nachrichten`; den Umzug macht
//! [`super::umzug`]); alle drei Ordner 0700,
//! auch wenn sie schon mit anderen Rechten bestanden (R9: dort liegen die
//! Anmeldedaten).
//!
//! „Abmelden und Daten löschen“ (R9) leert die Daten über WebKit sofort
//! (Cookies, Speicher, IndexedDB, Service Worker, Caches) und merkt einen
//! vollständigen Rückbau beider Ordner für den nächsten Start vor
//! ([`loeschen_vormerken`]): Was WebKits Netzwerkprozess gerade offen hält,
//! verschwindet so spätestens dann.

use std::fs;
use std::io;
use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
use std::path::{Path, PathBuf};

use gtk::glib;

/// Name der Ordner unter den XDG-Basisordnern.
pub const NAME: &str = "querencia-schnatter";
/// Merkdatei im Einstellungsordner: beim nächsten Start Daten und Cache leeren.
const VORMERKUNG: &str = "daten-loeschen";

#[derive(Debug, Clone)]
pub struct Ordner {
    pub daten: PathBuf,
    pub cache: PathBuf,
    pub konfig: PathBuf,
}

impl Ordner {
    /// Die Ordner nach den XDG-Basisordnern der Sitzung.
    pub fn aus_xdg() -> Self {
        Self {
            daten: glib::user_data_dir().join(NAME),
            cache: glib::user_cache_dir().join(NAME),
            konfig: glib::user_config_dir().join(NAME),
        }
    }

    pub fn einstellungen(&self) -> PathBuf {
        self.konfig.join("einstellungen.ini")
    }

    /// Legt alle drei Ordner an (0700) und führt eine vorgemerkte Löschung
    /// aus, bevor WebKit die Ordner öffnet.
    pub fn vorbereiten(&self) -> io::Result<()> {
        privat(&self.konfig)?;
        let merk = self.konfig.join(VORMERKUNG);
        if merk.exists() {
            leeren(&self.daten)?;
            leeren(&self.cache)?;
            fs::remove_file(&merk)?;
            crate::log("vorgemerkte Löschung der Sitzungsdaten ausgeführt");
        }
        privat(&self.daten)?;
        privat(&self.cache)?;
        Ok(())
    }

    /// Merkt die vollständige Löschung für den nächsten Start vor.
    pub fn loeschen_vormerken(&self) -> io::Result<()> {
        privat(&self.konfig)?;
        fs::write(self.konfig.join(VORMERKUNG), b"")
    }
}

/// Legt `ordner` an (Eltern wie üblich, der Ordner selbst 0700) und setzt
/// einen schon bestehenden auf 0700. Ein symbolischer Link an dieser Stelle
/// wird nicht verfolgt, sondern abgelehnt.
pub fn privat(ordner: &Path) -> io::Result<()> {
    match fs::symlink_metadata(ordner) {
        Ok(m) if m.file_type().is_symlink() => {
            return Err(io::Error::other(format!(
                "{} ist ein symbolischer Link",
                ordner.display()
            )));
        }
        Ok(m) if !m.is_dir() => {
            return Err(io::Error::other(format!(
                "{} ist kein Ordner",
                ordner.display()
            )));
        }
        Ok(m) => {
            if m.permissions().mode() & 0o777 != 0o700 {
                fs::set_permissions(ordner, fs::Permissions::from_mode(0o700))?;
            }
            return Ok(());
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        Err(e) => return Err(e),
    }
    if let Some(eltern) = ordner.parent() {
        fs::create_dir_all(eltern)?;
    }
    // die umask nimmt nur Rechte weg: aus 0700 wird nie mehr
    fs::DirBuilder::new().mode(0o700).create(ordner)
}

/// Entfernt `ordner` samt Inhalt (fehlt er, ist nichts zu tun).
fn leeren(ordner: &Path) -> io::Result<()> {
    match fs::symlink_metadata(ordner) {
        Ok(m) if m.file_type().is_symlink() => fs::remove_file(ordner),
        Ok(_) => fs::remove_dir_all(ordner),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}

/// Die dauerhafte Netzwerksitzung auf den Ordnern.
pub fn netzwerksitzung(ordner: &Ordner) -> webkit::NetworkSession {
    let s = webkit::NetworkSession::new(
        Some(&ordner.daten.to_string_lossy()),
        Some(&ordner.cache.to_string_lossy()),
    );
    // Falscher Zertifikatsfehler bleibt ein Fehler (Standard, ausdrücklich).
    s.set_tls_errors_policy(webkit::TLSErrorsPolicy::Fail);
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("qn-sitzung-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    fn modus(p: &Path) -> u32 {
        fs::metadata(p).unwrap().permissions().mode() & 0o777
    }

    #[test]
    fn ordner_sind_0700_auch_wenn_sie_offen_bestanden() {
        let t = tmp("rechte");
        let o = Ordner {
            daten: t.join("data/querencia-schnatter"),
            cache: t.join("cache/querencia-schnatter"),
            konfig: t.join("config/querencia-schnatter"),
        };
        fs::create_dir_all(&o.cache).unwrap();
        fs::set_permissions(&o.cache, fs::Permissions::from_mode(0o755)).unwrap();
        o.vorbereiten().unwrap();
        for p in [&o.daten, &o.cache, &o.konfig] {
            assert_eq!(modus(p), 0o700, "{}", p.display());
        }
        fs::remove_dir_all(&t).unwrap();
    }

    #[test]
    fn link_statt_ordner_wird_abgelehnt() {
        let t = tmp("link");
        fs::create_dir(t.join("ziel")).unwrap();
        std::os::unix::fs::symlink(t.join("ziel"), t.join("link")).unwrap();
        assert!(privat(&t.join("link")).is_err());
        fs::write(t.join("datei"), b"x").unwrap();
        assert!(privat(&t.join("datei")).is_err());
        fs::remove_dir_all(&t).unwrap();
    }

    #[test]
    fn vorgemerkte_loeschung_leert_daten_und_cache_beim_start() {
        let t = tmp("loeschen");
        let o = Ordner {
            daten: t.join("d"),
            cache: t.join("c"),
            konfig: t.join("k"),
        };
        o.vorbereiten().unwrap();
        fs::create_dir_all(o.daten.join("storage/x")).unwrap();
        fs::write(o.daten.join("storage/x/cookies.db"), b"geheim").unwrap();
        fs::write(o.cache.join("cache.bin"), b"x").unwrap();
        fs::write(o.einstellungen(), b"[Seite]\nzoom=1.5\n").unwrap();
        o.loeschen_vormerken().unwrap();
        o.vorbereiten().unwrap();
        assert_eq!(fs::read_dir(&o.daten).unwrap().count(), 0);
        assert_eq!(fs::read_dir(&o.cache).unwrap().count(), 0);
        assert_eq!(modus(&o.daten), 0o700);
        // Einstellungen bleiben, die Merkdatei ist weg
        assert!(o.einstellungen().exists());
        assert!(!o.konfig.join(VORMERKUNG).exists());
        fs::remove_dir_all(&t).unwrap();
    }
}
