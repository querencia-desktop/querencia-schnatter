//! Umzug der Ordner vom früheren Programmnamen (`querencia-nachrichten`, bis
//! 0.1.1) auf den heutigen (`querencia-webhuelle`, ab 0.2.0).
//!
//! Läuft in `main`, bevor die Einstellungen gelesen werden und bevor WebKit
//! einen Ordner öffnet. Für Daten, Cache und Einstellungen gilt je Ordner:
//!
//! - Gibt es unter dem neuen Namen nichts und unter dem alten einen Ordner,
//!   wird er umbenannt (`rename`; beide liegen im selben Elternordner, also
//!   auf demselben Dateisystem). Nichts wird kopiert oder gelöscht; WebKits
//!   Daten samt Anmeldung bei WhatsApp Web liegen danach unverändert unter
//!   dem neuen Namen.
//! - Gibt es den neuen schon, bleibt der alte unangetastet liegen (eine
//!   Protokollzeile sagt es).
//! - Ein symbolischer Link oder eine Datei unter dem alten Namen wird nicht
//!   angefasst.
//!
//! Steht ein Umzug an und läuft das alte Programm noch (ein Prozess dieses
//! Benutzers mit dem alten Binärnamen), zieht nichts um und das Programm
//! startet nicht: Der Netzwerkprozess des alten Programms schriebe sonst
//! weiter unter dem alten Pfad. Scheitert ein Umbenennen, werden die schon
//! umbenannten Ordner zurückbenannt und das Programm startet ebenfalls nicht;
//! es legt dann keinen neuen Ordner an, der den Umzug beim nächsten Start
//! verhindern würde.
//!
//! Dasselbe macht `integration/webhuelle-020/` beim Aktivieren (live), das
//! Programm findet danach nichts mehr zu tun.

use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use super::sitzung::Ordner;

/// Name der Ordner und der Binary bis 0.1.1.
pub const FRUEHERER_NAME: &str = "querencia-nachrichten";

/// Welcher der drei Ordner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Art {
    Daten,
    Cache,
    Konfig,
}

impl Art {
    fn schluessel(self) -> &'static str {
        match self {
            Art::Daten => "daten",
            Art::Cache => "cache",
            Art::Konfig => "konfig",
        }
    }
}

/// Ein Ordner unter altem und neuem Namen.
#[derive(Debug, Clone)]
pub struct Paar {
    pub art: Art,
    pub alt: PathBuf,
    pub neu: PathBuf,
}

/// Was mit einem Ordner geschah.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ergebnis {
    /// alt → neu umbenannt
    Verschoben,
    /// beide vorhanden: der alte bleibt liegen, der neue gilt
    AltBleibt,
    /// unter dem alten Namen liegt ein Link oder eine Datei: nicht angefasst
    AltFremd,
    /// kein alter Ordner
    Nichts,
}

impl Ergebnis {
    fn schluessel(self) -> &'static str {
        match self {
            Ergebnis::Verschoben => "verschoben",
            Ergebnis::AltBleibt => "alt-bleibt",
            Ergebnis::AltFremd => "alt-fremd",
            Ergebnis::Nichts => "nichts",
        }
    }
}

/// Ausgang des Umzugs beim Start.
#[derive(Debug)]
pub enum Stand {
    /// erledigt (auch: nichts zu tun); je Ordner das Ergebnis
    Fertig(Vec<(Art, Ergebnis)>),
    /// das alte Programm läuft noch (PID): nichts umgezogen, nicht starten
    AltesProgrammLaeuft(u32),
    /// ein Umbenennen scheiterte; zurückbenannt bzw. wie angegeben
    Gescheitert(String),
}

impl Stand {
    /// Ob das Programm starten darf.
    pub fn start_erlaubt(&self) -> bool {
        matches!(self, Stand::Fertig(_))
    }

    /// Zeile für das Prüfereignis „umzug“.
    pub fn ereignis(&self) -> String {
        match self {
            Stand::Fertig(e) => e
                .iter()
                .map(|(a, r)| format!("{}={}", a.schluessel(), r.schluessel()))
                .collect::<Vec<_>>()
                .join(" "),
            Stand::AltesProgrammLaeuft(pid) => format!("blockiert pid={pid}"),
            Stand::Gescheitert(_) => "gescheitert".to_owned(),
        }
    }
}

/// Die drei Paare zu den heutigen Ordnern: derselbe Elternordner, der alte
/// Name.
pub fn paare(ordner: &Ordner) -> Vec<Paar> {
    let alt = |neu: &Path| neu.with_file_name(FRUEHERER_NAME);
    vec![
        Paar {
            art: Art::Daten,
            alt: alt(&ordner.daten),
            neu: ordner.daten.clone(),
        },
        Paar {
            art: Art::Cache,
            alt: alt(&ordner.cache),
            neu: ordner.cache.clone(),
        },
        Paar {
            art: Art::Konfig,
            alt: alt(&ordner.konfig),
            neu: ordner.konfig.clone(),
        },
    ]
}

/// Umzug beim Start mit den Ordnern der Sitzung; schreibt Protokollzeilen.
pub fn beim_start(ordner: &Ordner) -> Stand {
    let stand = umziehen(&paare(ordner), || altes_programm(FRUEHERER_NAME));
    match &stand {
        Stand::Fertig(e) => {
            for (paar, (_, r)) in paare(ordner).iter().zip(e) {
                match r {
                    Ergebnis::Verschoben => crate::log(&format!(
                        "Umzug: {} nach {} umbenannt",
                        paar.alt.display(),
                        paar.neu.display()
                    )),
                    Ergebnis::AltBleibt => crate::log(&format!(
                        "Umzug: {} bleibt liegen, {} besteht schon",
                        paar.alt.display(),
                        paar.neu.display()
                    )),
                    Ergebnis::AltFremd => crate::log(&format!(
                        "Umzug: {} ist kein Ordner, nicht angefasst",
                        paar.alt.display()
                    )),
                    Ergebnis::Nichts => {}
                }
            }
        }
        Stand::AltesProgrammLaeuft(pid) => crate::log(&format!(
            "Umzug steht an, aber {FRUEHERER_NAME} läuft noch (PID {pid}): nichts umgezogen, kein Start"
        )),
        Stand::Gescheitert(grund) => crate::log(&format!("Umzug gescheitert: {grund}")),
    }
    stand
}

/// Der Umzug selbst; `laeuft` sagt, ob das alte Programm läuft (PID).
pub fn umziehen(paare: &[Paar], laeuft: impl Fn() -> Option<u32>) -> Stand {
    let plan: Vec<Ergebnis> = paare.iter().map(planen).collect();
    if plan.contains(&Ergebnis::Verschoben)
        && let Some(pid) = laeuft()
    {
        return Stand::AltesProgrammLaeuft(pid);
    }
    let mut erledigt: Vec<&Paar> = Vec::new();
    for (paar, schritt) in paare.iter().zip(&plan) {
        if *schritt != Ergebnis::Verschoben {
            continue;
        }
        if let Err(e) = fs::rename(&paar.alt, &paar.neu) {
            let mut grund = format!("{} → {}: {e}", paar.alt.display(), paar.neu.display());
            for frueher in erledigt.iter().rev() {
                if let Err(e) = fs::rename(&frueher.neu, &frueher.alt) {
                    grund.push_str(&format!(
                        "; Zurückbenennen {} → {} scheiterte: {e}",
                        frueher.neu.display(),
                        frueher.alt.display()
                    ));
                }
            }
            return Stand::Gescheitert(grund);
        }
        erledigt.push(paar);
    }
    Stand::Fertig(paare.iter().map(|p| p.art).zip(plan).collect())
}

/// Was mit einem Paar zu tun ist (ohne etwas zu ändern).
fn planen(paar: &Paar) -> Ergebnis {
    let alt = match fs::symlink_metadata(&paar.alt) {
        Ok(m) => m,
        Err(_) => return Ergebnis::Nichts,
    };
    if !alt.is_dir() {
        return Ergebnis::AltFremd;
    }
    match fs::symlink_metadata(&paar.neu) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ergebnis::Verschoben,
        _ => Ergebnis::AltBleibt,
    }
}

/// PID eines laufenden Prozesses dieses Benutzers, dessen Binary `name`
/// heißt (`/proc/<pid>/exe`; Prozesse anderer Benutzer sind ohnehin nicht
/// lesbar).
pub fn altes_programm(name: &str) -> Option<u32> {
    let eigene = fs::metadata("/proc/self").ok()?.uid();
    let geloescht = format!("{name} (deleted)");
    for eintrag in fs::read_dir("/proc").ok()?.flatten() {
        let Some(pid) = eintrag
            .file_name()
            .to_str()
            .and_then(|s| s.parse::<u32>().ok())
        else {
            continue;
        };
        if eintrag.metadata().map(|m| m.uid()).ok() != Some(eigene) {
            continue;
        }
        let Ok(exe) = fs::read_link(eintrag.path().join("exe")) else {
            continue;
        };
        let datei = exe.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if datei == name || datei == geloescht {
            return Some(pid);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("qw-umzug-{}-{name}", std::process::id()));
        if d.exists() {
            let _ = fs::set_permissions(&d, fs::Permissions::from_mode(0o700));
            for e in fs::read_dir(&d).unwrap().flatten() {
                let _ = fs::set_permissions(e.path(), fs::Permissions::from_mode(0o700));
            }
            fs::remove_dir_all(&d).unwrap();
        }
        fs::create_dir_all(&d).unwrap();
        d
    }

    /// XDG-Basisordner unter `t` mit den heutigen Namen.
    fn ordner(t: &Path) -> Ordner {
        Ordner {
            daten: t.join("data/querencia-webhuelle"),
            cache: t.join("cache/querencia-webhuelle"),
            konfig: t.join("config/querencia-webhuelle"),
        }
    }

    /// Ein alter Ordner mit einer Attrappe der WebKit-Daten.
    fn alt_anlegen(alt: &Path) {
        fs::create_dir_all(alt.join("storage/abc")).unwrap();
        fs::write(alt.join("storage/abc/attrappe.db"), b"anmeldung-attrappe").unwrap();
        fs::set_permissions(alt, fs::Permissions::from_mode(0o700)).unwrap();
    }

    fn nie() -> Option<u32> {
        None
    }

    #[test]
    fn alte_ordner_werden_umbenannt_inhalt_und_rechte_bleiben() {
        let t = tmp("alle");
        let o = ordner(&t);
        let p = paare(&o);
        for paar in &p {
            alt_anlegen(&paar.alt);
        }
        fs::write(p[2].alt.join("einstellungen.ini"), b"[Seite]\nzoom=1.25\n").unwrap();
        let s = umziehen(&p, nie);
        assert!(s.start_erlaubt());
        assert_eq!(
            s.ereignis(),
            "daten=verschoben cache=verschoben konfig=verschoben"
        );
        for paar in &p {
            assert!(!paar.alt.exists(), "{}", paar.alt.display());
            assert_eq!(
                fs::read(paar.neu.join("storage/abc/attrappe.db")).unwrap(),
                b"anmeldung-attrappe"
            );
            let m = fs::metadata(&paar.neu).unwrap().permissions().mode() & 0o777;
            assert_eq!(m, 0o700);
        }
        assert_eq!(
            fs::read(o.einstellungen()).unwrap(),
            b"[Seite]\nzoom=1.25\n"
        );
        // ein zweiter Start findet nichts mehr
        assert_eq!(
            umziehen(&p, nie).ereignis(),
            "daten=nichts cache=nichts konfig=nichts"
        );
        fs::remove_dir_all(&t).unwrap();
    }

    #[test]
    fn besteht_der_neue_ordner_bleibt_der_alte_unangetastet() {
        let t = tmp("beide");
        let p = paare(&ordner(&t));
        alt_anlegen(&p[0].alt);
        fs::create_dir_all(&p[0].neu).unwrap();
        fs::write(p[0].neu.join("neu.txt"), b"neu").unwrap();
        let s = umziehen(&p, nie);
        assert_eq!(s.ereignis(), "daten=alt-bleibt cache=nichts konfig=nichts");
        assert_eq!(
            fs::read(p[0].alt.join("storage/abc/attrappe.db")).unwrap(),
            b"anmeldung-attrappe"
        );
        assert_eq!(fs::read(p[0].neu.join("neu.txt")).unwrap(), b"neu");
        fs::remove_dir_all(&t).unwrap();
    }

    #[test]
    fn link_oder_datei_unter_dem_alten_namen_wird_nicht_angefasst() {
        let t = tmp("fremd");
        let p = paare(&ordner(&t));
        fs::create_dir_all(t.join("woanders")).unwrap();
        fs::create_dir_all(p[0].alt.parent().unwrap()).unwrap();
        std::os::unix::fs::symlink(t.join("woanders"), &p[0].alt).unwrap();
        fs::create_dir_all(p[1].alt.parent().unwrap()).unwrap();
        fs::write(&p[1].alt, b"x").unwrap();
        let s = umziehen(&p, nie);
        assert_eq!(
            s.ereignis(),
            "daten=alt-fremd cache=alt-fremd konfig=nichts"
        );
        assert!(
            fs::symlink_metadata(&p[0].alt)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert!(!p[0].neu.exists() && !p[1].neu.exists());
        fs::remove_dir_all(&t).unwrap();
    }

    #[test]
    fn laeuft_das_alte_programm_zieht_nichts_um() {
        let t = tmp("laeuft");
        let p = paare(&ordner(&t));
        alt_anlegen(&p[0].alt);
        let s = umziehen(&p, || Some(4242));
        assert!(!s.start_erlaubt());
        assert_eq!(s.ereignis(), "blockiert pid=4242");
        assert!(p[0].alt.exists() && !p[0].neu.exists());
        // ohne anstehenden Umzug spielt das alte Programm keine Rolle
        let t2 = tmp("laeuft-leer");
        assert!(umziehen(&paare(&ordner(&t2)), || Some(4242)).start_erlaubt());
        fs::remove_dir_all(&t).unwrap();
        fs::remove_dir_all(&t2).unwrap();
    }

    #[test]
    fn scheitert_ein_umbenennen_wird_zurueckbenannt() {
        // als root greifen die Rechte nicht (Paketbau unter fakeroot o. ä.)
        if fs::metadata("/proc/self").map(|m| m.uid()).unwrap_or(0) == 0 {
            return;
        }
        let t = tmp("scheitert");
        let p = paare(&ordner(&t));
        for paar in &p {
            alt_anlegen(&paar.alt);
        }
        // der Elternordner des Cache ist schreibgeschützt: rename scheitert
        let eltern = p[1].alt.parent().unwrap().to_path_buf();
        fs::set_permissions(&eltern, fs::Permissions::from_mode(0o555)).unwrap();
        let s = umziehen(&p, nie);
        fs::set_permissions(&eltern, fs::Permissions::from_mode(0o755)).unwrap();
        assert!(!s.start_erlaubt());
        assert_eq!(s.ereignis(), "gescheitert");
        for paar in &p {
            assert!(paar.alt.exists(), "{}", paar.alt.display());
            assert!(!paar.neu.exists(), "{}", paar.neu.display());
        }
        fs::remove_dir_all(&t).unwrap();
    }

    #[test]
    fn eigener_prozess_wird_unter_seinem_namen_gefunden() {
        let exe = std::env::current_exe().unwrap();
        let name = exe.file_name().unwrap().to_str().unwrap().to_owned();
        assert!(altes_programm(&name).is_some());
        assert_eq!(altes_programm("kein-solches-programm-qw"), None);
    }

    #[test]
    fn paare_liegen_im_selben_elternordner() {
        for p in paare(&ordner(Path::new("/x"))) {
            assert_eq!(p.alt.parent(), p.neu.parent());
            assert_eq!(p.alt.file_name().unwrap(), FRUEHERER_NAME);
        }
    }
}
