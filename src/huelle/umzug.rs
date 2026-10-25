//! Umzug der Ordner von den früheren Programmnamen auf den heutigen
//! (`querencia-schnatter`, ab 0.3.0): `querencia-webhuelle` (0.2.0) und
//! `querencia-nachrichten` (bis 0.1.1, für alle, die 0.2.0 übersprungen
//! haben).
//!
//! Läuft in `main`, bevor die Einstellungen gelesen werden und bevor WebKit
//! einen Ordner öffnet. Für Daten, Cache und Einstellungen gilt je Ordner:
//!
//! - Gibt es unter dem neuen Namen nichts, zählt der neueste frühere Name,
//!   unter dem etwas liegt. Ist das ein Ordner, wird er umbenannt (`rename`;
//!   alle Namen liegen im selben Elternordner, also auf demselben
//!   Dateisystem). Nichts wird kopiert oder gelöscht; WebKits Daten samt
//!   Anmeldung bei WhatsApp Web liegen danach unverändert unter dem neuen
//!   Namen. Ein älterer Ordner daneben bleibt liegen (eine Protokollzeile
//!   sagt es).
//! - Gibt es den neuen schon, bleiben die alten unangetastet liegen.
//! - Ein symbolischer Link oder eine Datei unter dem zählenden früheren
//!   Namen wird nicht angefasst, und auch kein älterer Ordner dahinter.
//!
//! Steht ein Umzug an und läuft eines der früheren Programme noch (ein
//! Prozess dieses Benutzers mit einem früheren Binärnamen), zieht nichts um
//! und das Programm startet nicht: Der Netzwerkprozess des alten Programms
//! schriebe sonst weiter unter dem alten Pfad. Scheitert ein Umbenennen,
//! werden die schon umbenannten Ordner zurückbenannt und das Programm
//! startet ebenfalls nicht; es legt dann keinen neuen Ordner an, der den
//! Umzug beim nächsten Start verhindern würde.
//!
//! Dasselbe macht `integration/schnatter-030/` beim Aktivieren (live), das
//! Programm findet danach nichts mehr zu tun.

use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use super::sitzung::Ordner;
use crate::i18n::gettext;

/// Frühere Namen der Ordner und der Binary, der neueste zuerst.
pub const FRUEHERE_NAMEN: [&str; 2] = ["querencia-webhuelle", "querencia-nachrichten"];

/// Anzeigename eines früheren Programms (für den Hinweis beim Start).
pub fn anzeigename(name: &str) -> String {
    match name {
        "querencia-webhuelle" => gettext("Querencia Web Wrapper"),
        "querencia-nachrichten" => gettext("Querencia Messages"),
        _ => name.to_owned(),
    }
}

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

/// Ein Ordner unter den früheren Namen (Reihenfolge wie
/// [`FRUEHERE_NAMEN`]) und dem neuen.
#[derive(Debug, Clone)]
pub struct Paar {
    pub art: Art,
    pub alt: Vec<PathBuf>,
    pub neu: PathBuf,
}

/// Was mit einem Ordner geschah.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ergebnis {
    /// `alt[i]` → neu umbenannt
    Verschoben(usize),
    /// der neue besteht schon: die alten bleiben liegen
    AltBleibt,
    /// unter dem zählenden alten Namen liegt ein Link oder eine Datei:
    /// nicht angefasst
    AltFremd,
    /// kein alter Ordner
    Nichts,
}

impl Ergebnis {
    fn schluessel(self) -> String {
        match self {
            Ergebnis::Verschoben(i) => format!(
                "verschoben-{}",
                FRUEHERE_NAMEN[i].trim_start_matches("querencia-")
            ),
            Ergebnis::AltBleibt => "alt-bleibt".to_owned(),
            Ergebnis::AltFremd => "alt-fremd".to_owned(),
            Ergebnis::Nichts => "nichts".to_owned(),
        }
    }
}

/// Ausgang des Umzugs beim Start.
#[derive(Debug)]
pub enum Stand {
    /// erledigt (auch: nichts zu tun); je Ordner das Ergebnis
    Fertig(Vec<(Art, Ergebnis)>),
    /// ein früheres Programm läuft noch: nichts umgezogen, nicht starten
    AltesProgrammLaeuft { name: String, pid: u32 },
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
            Stand::AltesProgrammLaeuft { name, pid } => {
                format!("blockiert name={name} pid={pid}")
            }
            Stand::Gescheitert(_) => "gescheitert".to_owned(),
        }
    }
}

/// Die drei Paare zu den heutigen Ordnern: derselbe Elternordner, die
/// früheren Namen.
pub fn paare(ordner: &Ordner) -> Vec<Paar> {
    let paar = |art, neu: &Path| Paar {
        art,
        alt: FRUEHERE_NAMEN
            .iter()
            .map(|n| neu.with_file_name(n))
            .collect(),
        neu: neu.to_path_buf(),
    };
    vec![
        paar(Art::Daten, &ordner.daten),
        paar(Art::Cache, &ordner.cache),
        paar(Art::Konfig, &ordner.konfig),
    ]
}

/// Umzug beim Start mit den Ordnern der Sitzung; schreibt Protokollzeilen.
pub fn beim_start(ordner: &Ordner) -> Stand {
    let paare = paare(ordner);
    let stand = umziehen(&paare, || altes_programm(&FRUEHERE_NAMEN));
    match &stand {
        Stand::Fertig(e) => {
            for (paar, (_, r)) in paare.iter().zip(e) {
                let liegen = |ausser: Option<usize>| {
                    for (i, alt) in paar.alt.iter().enumerate() {
                        if Some(i) != ausser && fs::symlink_metadata(alt).is_ok() {
                            crate::log(&format!(
                                "Umzug: {} bleibt liegen, {} besteht schon",
                                alt.display(),
                                paar.neu.display()
                            ));
                        }
                    }
                };
                match r {
                    Ergebnis::Verschoben(i) => {
                        crate::log(&format!(
                            "Umzug: {} nach {} umbenannt",
                            paar.alt[*i].display(),
                            paar.neu.display()
                        ));
                        liegen(Some(*i));
                    }
                    Ergebnis::AltBleibt => liegen(None),
                    Ergebnis::AltFremd => crate::log(&format!(
                        "Umzug: {} ist kein Ordner, nicht angefasst",
                        paar.alt
                            .iter()
                            .find(|a| fs::symlink_metadata(a).is_ok())
                            .unwrap_or(&paar.alt[0])
                            .display()
                    )),
                    Ergebnis::Nichts => {}
                }
            }
        }
        Stand::AltesProgrammLaeuft { name, pid } => crate::log(&format!(
            "Umzug steht an, aber {name} läuft noch (PID {pid}): nichts umgezogen, kein Start"
        )),
        Stand::Gescheitert(grund) => crate::log(&format!("Umzug gescheitert: {grund}")),
    }
    stand
}

/// Der Umzug selbst; `laeuft` nennt ein laufendes früheres Programm (Name,
/// PID).
pub fn umziehen(paare: &[Paar], laeuft: impl Fn() -> Option<(String, u32)>) -> Stand {
    let plan: Vec<Ergebnis> = paare.iter().map(planen).collect();
    if plan.iter().any(|e| matches!(e, Ergebnis::Verschoben(_)))
        && let Some((name, pid)) = laeuft()
    {
        return Stand::AltesProgrammLaeuft { name, pid };
    }
    let mut erledigt: Vec<(&Path, &Path)> = Vec::new();
    for (paar, schritt) in paare.iter().zip(&plan) {
        let Ergebnis::Verschoben(i) = *schritt else {
            continue;
        };
        let alt = &paar.alt[i];
        if let Err(e) = fs::rename(alt, &paar.neu) {
            let mut grund = format!("{} → {}: {e}", alt.display(), paar.neu.display());
            for (frueher_alt, frueher_neu) in erledigt.iter().rev() {
                if let Err(e) = fs::rename(frueher_neu, frueher_alt) {
                    grund.push_str(&format!(
                        "; Zurückbenennen {} → {} scheiterte: {e}",
                        frueher_neu.display(),
                        frueher_alt.display()
                    ));
                }
            }
            return Stand::Gescheitert(grund);
        }
        erledigt.push((alt, &paar.neu));
    }
    Stand::Fertig(paare.iter().map(|p| p.art).zip(plan).collect())
}

/// Was mit einem Paar zu tun ist (ohne etwas zu ändern).
fn planen(paar: &Paar) -> Ergebnis {
    let vorhanden: Vec<(usize, fs::Metadata)> = paar
        .alt
        .iter()
        .enumerate()
        .filter_map(|(i, a)| fs::symlink_metadata(a).ok().map(|m| (i, m)))
        .collect();
    let Some((i, alt)) = vorhanden.first() else {
        return Ergebnis::Nichts;
    };
    match fs::symlink_metadata(&paar.neu) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        _ => return Ergebnis::AltBleibt,
    }
    if alt.is_dir() {
        Ergebnis::Verschoben(*i)
    } else {
        Ergebnis::AltFremd
    }
}

/// Name und PID eines laufenden Prozesses dieses Benutzers, dessen Binary
/// einen der `namen` trägt (`/proc/<pid>/exe`; Prozesse anderer Benutzer
/// sind ohnehin nicht lesbar).
pub fn altes_programm(namen: &[&str]) -> Option<(String, u32)> {
    let eigene = fs::metadata("/proc/self").ok()?.uid();
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
        let datei = datei.strip_suffix(" (deleted)").unwrap_or(datei);
        if let Some(name) = namen.iter().find(|n| **n == datei) {
            return Some(((*name).to_owned(), pid));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    const WEBH: usize = 0;
    const NACHR: usize = 1;

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("qs-umzug-{}-{name}", std::process::id()));
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
            daten: t.join("data/querencia-schnatter"),
            cache: t.join("cache/querencia-schnatter"),
            konfig: t.join("config/querencia-schnatter"),
        }
    }

    /// Ein alter Ordner mit einer Attrappe der WebKit-Daten (`wert` sagt,
    /// welcher es war).
    fn alt_anlegen(alt: &Path, wert: &[u8]) {
        fs::create_dir_all(alt.join("storage/abc")).unwrap();
        fs::write(alt.join("storage/abc/attrappe.db"), wert).unwrap();
        fs::set_permissions(alt, fs::Permissions::from_mode(0o700)).unwrap();
    }

    fn attrappe(ordner: &Path) -> Vec<u8> {
        fs::read(ordner.join("storage/abc/attrappe.db")).unwrap()
    }

    fn nie() -> Option<(String, u32)> {
        None
    }

    #[test]
    fn ordner_der_webhuelle_werden_umbenannt_inhalt_und_rechte_bleiben() {
        let t = tmp("webhuelle");
        let o = ordner(&t);
        let p = paare(&o);
        for paar in &p {
            alt_anlegen(&paar.alt[WEBH], b"anmeldung-webhuelle");
        }
        fs::write(
            p[2].alt[WEBH].join("einstellungen.ini"),
            b"[Seite]\nzoom=1.25\n",
        )
        .unwrap();
        let s = umziehen(&p, nie);
        assert!(s.start_erlaubt());
        assert_eq!(
            s.ereignis(),
            "daten=verschoben-webhuelle cache=verschoben-webhuelle konfig=verschoben-webhuelle"
        );
        for paar in &p {
            assert!(!paar.alt[WEBH].exists(), "{}", paar.alt[WEBH].display());
            assert_eq!(attrappe(&paar.neu), b"anmeldung-webhuelle");
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
    fn ordner_der_nachrichten_werden_umbenannt_wenn_0_2_0_fehlte() {
        let t = tmp("nachrichten");
        let p = paare(&ordner(&t));
        for paar in &p {
            alt_anlegen(&paar.alt[NACHR], b"anmeldung-nachrichten");
        }
        let s = umziehen(&p, nie);
        assert_eq!(
            s.ereignis(),
            "daten=verschoben-nachrichten cache=verschoben-nachrichten konfig=verschoben-nachrichten"
        );
        for paar in &p {
            assert!(!paar.alt[NACHR].exists());
            assert_eq!(attrappe(&paar.neu), b"anmeldung-nachrichten");
        }
        fs::remove_dir_all(&t).unwrap();
    }

    #[test]
    fn liegen_beide_alten_zieht_der_neueste_um_der_aeltere_bleibt() {
        let t = tmp("zwei-alte");
        let p = paare(&ordner(&t));
        alt_anlegen(&p[0].alt[WEBH], b"anmeldung-webhuelle");
        alt_anlegen(&p[0].alt[NACHR], b"anmeldung-nachrichten");
        // im Cache nur der älteste Name: je Ordner entscheidet der neueste
        // vorhandene
        alt_anlegen(&p[1].alt[NACHR], b"cache-nachrichten");
        let s = umziehen(&p, nie);
        assert_eq!(
            s.ereignis(),
            "daten=verschoben-webhuelle cache=verschoben-nachrichten konfig=nichts"
        );
        assert_eq!(attrappe(&p[0].neu), b"anmeldung-webhuelle");
        assert_eq!(attrappe(&p[0].alt[NACHR]), b"anmeldung-nachrichten");
        assert_eq!(attrappe(&p[1].neu), b"cache-nachrichten");
        fs::remove_dir_all(&t).unwrap();
    }

    #[test]
    fn besteht_der_neue_ordner_bleiben_die_alten_unangetastet() {
        let t = tmp("neu-da");
        let p = paare(&ordner(&t));
        alt_anlegen(&p[0].alt[WEBH], b"anmeldung-webhuelle");
        alt_anlegen(&p[0].alt[NACHR], b"anmeldung-nachrichten");
        fs::create_dir_all(&p[0].neu).unwrap();
        fs::write(p[0].neu.join("neu.txt"), b"neu").unwrap();
        let s = umziehen(&p, nie);
        assert_eq!(s.ereignis(), "daten=alt-bleibt cache=nichts konfig=nichts");
        assert_eq!(attrappe(&p[0].alt[WEBH]), b"anmeldung-webhuelle");
        assert_eq!(attrappe(&p[0].alt[NACHR]), b"anmeldung-nachrichten");
        assert_eq!(fs::read(p[0].neu.join("neu.txt")).unwrap(), b"neu");
        fs::remove_dir_all(&t).unwrap();
    }

    #[test]
    fn link_oder_datei_unter_einem_alten_namen_wird_nicht_angefasst() {
        let t = tmp("fremd");
        let p = paare(&ordner(&t));
        fs::create_dir_all(t.join("woanders")).unwrap();
        fs::create_dir_all(p[0].neu.parent().unwrap()).unwrap();
        std::os::unix::fs::symlink(t.join("woanders"), &p[0].alt[WEBH]).unwrap();
        fs::create_dir_all(p[1].neu.parent().unwrap()).unwrap();
        fs::write(&p[1].alt[NACHR], b"x").unwrap();
        // Link unter dem neueren Namen: auch der ältere Ordner bleibt
        fs::create_dir_all(p[2].neu.parent().unwrap()).unwrap();
        std::os::unix::fs::symlink(t.join("woanders"), &p[2].alt[WEBH]).unwrap();
        alt_anlegen(&p[2].alt[NACHR], b"konfig-nachrichten");
        let s = umziehen(&p, nie);
        assert_eq!(
            s.ereignis(),
            "daten=alt-fremd cache=alt-fremd konfig=alt-fremd"
        );
        assert!(
            fs::symlink_metadata(&p[0].alt[WEBH])
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert!(!p[0].neu.exists() && !p[1].neu.exists() && !p[2].neu.exists());
        assert_eq!(attrappe(&p[2].alt[NACHR]), b"konfig-nachrichten");
        fs::remove_dir_all(&t).unwrap();
    }

    #[test]
    fn laeuft_ein_frueheres_programm_zieht_nichts_um() {
        let t = tmp("laeuft");
        let p = paare(&ordner(&t));
        alt_anlegen(&p[0].alt[NACHR], b"anmeldung-nachrichten");
        let s = umziehen(&p, || Some(("querencia-webhuelle".to_owned(), 4242)));
        assert!(!s.start_erlaubt());
        assert_eq!(s.ereignis(), "blockiert name=querencia-webhuelle pid=4242");
        assert!(p[0].alt[NACHR].exists() && !p[0].neu.exists());
        // ohne anstehenden Umzug spielt ein altes Programm keine Rolle
        let t2 = tmp("laeuft-leer");
        assert!(
            umziehen(&paare(&ordner(&t2)), || Some((
                "querencia-nachrichten".to_owned(),
                4242
            )))
            .start_erlaubt()
        );
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
        alt_anlegen(&p[0].alt[WEBH], b"daten-webhuelle");
        alt_anlegen(&p[1].alt[NACHR], b"cache-nachrichten");
        alt_anlegen(&p[2].alt[WEBH], b"konfig-webhuelle");
        // der Elternordner der Einstellungen ist schreibgeschützt: das dritte
        // rename scheitert, die ersten beiden werden zurückbenannt
        let eltern = p[2].neu.parent().unwrap().to_path_buf();
        fs::set_permissions(&eltern, fs::Permissions::from_mode(0o555)).unwrap();
        let s = umziehen(&p, nie);
        fs::set_permissions(&eltern, fs::Permissions::from_mode(0o755)).unwrap();
        assert!(!s.start_erlaubt());
        assert_eq!(s.ereignis(), "gescheitert");
        assert_eq!(attrappe(&p[0].alt[WEBH]), b"daten-webhuelle");
        assert_eq!(attrappe(&p[1].alt[NACHR]), b"cache-nachrichten");
        assert_eq!(attrappe(&p[2].alt[WEBH]), b"konfig-webhuelle");
        for paar in &p {
            assert!(!paar.neu.exists(), "{}", paar.neu.display());
        }
        fs::remove_dir_all(&t).unwrap();
    }

    #[test]
    fn eigener_prozess_wird_unter_seinem_namen_gefunden() {
        let exe = std::env::current_exe().unwrap();
        let name = exe.file_name().unwrap().to_str().unwrap().to_owned();
        assert_eq!(
            altes_programm(&["kein-solches-programm-qs", &name]).map(|(n, _)| n),
            Some(name)
        );
        assert_eq!(altes_programm(&["kein-solches-programm-qs"]), None);
    }

    #[test]
    fn paare_liegen_im_selben_elternordner() {
        for p in paare(&ordner(Path::new("/x"))) {
            for (alt, name) in p.alt.iter().zip(FRUEHERE_NAMEN) {
                assert_eq!(alt.parent(), p.neu.parent());
                assert_eq!(alt.file_name().unwrap(), name);
            }
        }
    }
}
