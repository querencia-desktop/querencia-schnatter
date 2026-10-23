//! Downloads (E6): ohne Rückfrage in den Download-Ordner der Sitzung, nie
//! über eine vorhandene Datei („foto.jpg“, dann „foto 2.jpg“ …, dieselbe
//! Regel wie Desktop und Dateimanager: `querencia_dateiarbeit::namen::
//! unique_name`), danach eine Meldung mit „Öffnen“ und „Ordner öffnen“.
//!
//! Gewählt wurde `unique_name` statt `copy_name`: Ein zweiter Download ist
//! keine Kopie einer vorhandenen Datei, „foto (Kopie).jpg“ wäre falsch
//! (Bericht, Entscheidung zu E6).

use std::cell::RefCell;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gio, glib};

use crate::i18n::{gettext, gettext_f};

/// Endung, unter der WebKit einen laufenden Download schreibt.
const TEIL: &str = ".wkdownload";

/// Der Download-Ordner der Sitzung (`XDG_DOWNLOAD_DIR`), sonst `~/Downloads`.
pub fn ordner() -> PathBuf {
    glib::user_special_dir(glib::UserDirectory::Downloads)
        .unwrap_or_else(|| glib::home_dir().join("Downloads"))
}

/// Ein Dateiname ohne Pfadteile und Steuerzeichen aus dem Vorschlag der
/// Seite; leer, „.“ oder „..“ wird „download“. Ein führender Punkt fällt weg
/// (keine versteckten Dateien aus dem Netz).
pub fn dateiname(vorschlag: &str) -> String {
    let n: String = vorschlag
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or_default()
        .chars()
        .filter(|c| !c.is_control())
        .collect();
    let n = n.trim().trim_start_matches('.').trim();
    if n.is_empty() {
        return gettext("download");
    }
    // höchstens 200 Bytes, an einer Zeichengrenze, Endung bleibt
    if n.len() <= 200 {
        return n.to_owned();
    }
    let (stamm, endung) = querencia_dateiarbeit::namen::split_extension(n);
    let endung = if endung.len() <= 16 { endung } else { "" };
    let mut s = String::new();
    for c in stamm.chars() {
        if s.len() + c.len_utf8() + endung.len() > 200 {
            break;
        }
        s.push(c);
    }
    s + endung
}

/// Der Zielpfad im Ordner `ordner` für `vorschlag`, nie ein belegter Name
/// (auch kein laufender Download und keiner aus `reserviert`).
pub fn ziel(ordner: &Path, vorschlag: &str, reserviert: &HashSet<String>) -> PathBuf {
    let name = querencia_dateiarbeit::namen::unique_name(&dateiname(vorschlag), |n| {
        reserviert.contains(n)
            || ordner.join(n).symlink_metadata().is_ok()
            || ordner.join(format!("{n}{TEIL}")).symlink_metadata().is_ok()
    });
    ordner.join(name)
}

/// Verbindet die Downloads der Sitzung mit der Hülle.
pub fn einrichten(sitzung: &webkit::NetworkSession, app: &gtk::Application) {
    let reserviert: Rc<RefCell<HashSet<String>>> = Rc::default();
    let app = app.downgrade();
    sitzung.connect_download_started(move |_, dl| {
        crate::ereignis("download-started", "");
        let r = reserviert.clone();
        let gemerkt: Rc<RefCell<Option<String>>> = Rc::default();
        let g = gemerkt.clone();
        dl.connect_decide_destination(move |dl, vorschlag| {
            let o = ordner();
            if let Err(e) = std::fs::create_dir_all(&o) {
                crate::log(&format!("Download-Ordner {}: {e}", o.display()));
            }
            let pfad = ziel(&o, vorschlag, &r.borrow());
            let name = pfad
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            r.borrow_mut().insert(name.clone());
            g.replace(Some(name));
            crate::ereignis("download-ziel", &pfad.display().to_string());
            dl.set_allow_overwrite(false);
            dl.set_destination(&pfad.to_string_lossy());
            true
        });
        let fehler: Rc<RefCell<Option<String>>> = Rc::default();
        let f = fehler.clone();
        dl.connect_failed(move |_, e| {
            crate::ereignis("download-failed", e.message());
            f.replace(Some(e.message().to_owned()));
        });
        let r = reserviert.clone();
        let app = app.clone();
        dl.connect_finished(move |dl| {
            if let Some(n) = gemerkt.borrow().as_ref() {
                r.borrow_mut().remove(n);
            }
            let Some(app) = app.upgrade() else { return };
            let pfad = dl.destination().map(|d| PathBuf::from(d.as_str()));
            if let Some(f) = fehler.borrow().as_ref() {
                // vom Nutzer abgebrochen: keine Meldung
                let abbruch = dl.destination().is_none() || f.to_lowercase().contains("cancel");
                if !abbruch {
                    melden_fehler(&app, pfad.as_deref());
                }
                return;
            }
            if let Some(p) = pfad {
                crate::ereignis("download-finished", &p.display().to_string());
                melden_fertig(&app, &p);
            }
        });
    });
}

/// Meldung „Download abgeschlossen“ mit „Öffnen“ und „Ordner öffnen“.
fn melden_fertig(app: &gtk::Application, pfad: &Path) {
    let name = pfad
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let n = gio::Notification::new(&gettext("Download finished"));
    n.set_body(Some(&name));
    let ziel = pfad.to_string_lossy().to_variant();
    n.add_button_with_target_value(&gettext("Open"), "app.download-oeffnen", Some(&ziel));
    n.add_button_with_target_value(&gettext("Open Folder"), "app.download-ordner", Some(&ziel));
    n.set_default_action_and_target_value("app.download-ordner", Some(&ziel));
    n.set_category(Some("transfer.complete"));
    app.send_notification(Some(&format!("download-{name}")), &n);
}

fn melden_fehler(app: &gtk::Application, pfad: Option<&Path>) {
    let name = pfad
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let n = gio::Notification::new(&gettext("Download failed"));
    n.set_body(Some(&gettext_f(
        "“{name}” could not be downloaded.",
        &[("name", &name)],
    )));
    n.set_category(Some("transfer.error"));
    app.send_notification(Some(&format!("download-{name}")), &n);
}

/// Die Aktionen der Meldung: `app.download-oeffnen(s)` öffnet die Datei mit
/// dem Standardprogramm, `app.download-ordner(s)` zeigt sie im Dateimanager.
pub fn aktionen(app: &gtk::Application) {
    let oeffnen = gio::SimpleAction::new("download-oeffnen", Some(glib::VariantTy::STRING));
    let a = app.downgrade();
    oeffnen.connect_activate(move |_, ziel| {
        let (Some(app), Some(p)) = (a.upgrade(), ziel.and_then(|z| z.get::<String>())) else {
            return;
        };
        crate::ereignis("download-oeffnen", &p);
        let l = gtk::FileLauncher::new(Some(&gio::File::for_path(&p)));
        l.launch(
            app.active_window().as_ref(),
            None::<&gio::Cancellable>,
            move |r| {
                if let Err(e) = r {
                    crate::log(&format!("{p} nicht geöffnet: {e}"));
                }
            },
        );
    });
    app.add_action(&oeffnen);
    let ordner = gio::SimpleAction::new("download-ordner", Some(glib::VariantTy::STRING));
    let a = app.downgrade();
    ordner.connect_activate(move |_, ziel| {
        let (Some(app), Some(p)) = (a.upgrade(), ziel.and_then(|z| z.get::<String>())) else {
            return;
        };
        crate::ereignis("download-ordner", &p);
        let l = gtk::FileLauncher::new(Some(&gio::File::for_path(&p)));
        let fenster = app.active_window();
        let kontext = fenster
            .as_ref()
            .map(|w| WidgetExt::display(w).app_launch_context());
        l.open_containing_folder(fenster.as_ref(), None::<&gio::Cancellable>, move |r| {
            let Err(e) = r else { return };
            // Ohne Portal und ohne org.freedesktop.FileManager1 öffnet GTK
            // keinen Ordner; dann den Ordner selbst mit dem Standardprogramm.
            let ordner = Path::new(&p).parent().map(gio::File::for_path);
            let ergebnis =
                ordner.map(|o| gio::AppInfo::launch_default_for_uri(&o.uri(), kontext.as_ref()));
            if !matches!(ergebnis, Some(Ok(()))) {
                crate::log(&format!("Ordner von {p} nicht geöffnet: {e}"));
            }
        });
    });
    app.add_action(&ordner);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dateinamen_ohne_pfad_und_versteckte_punkte() {
        assert_eq!(dateiname("foto.jpg"), "foto.jpg");
        assert_eq!(dateiname("../../etc/passwd"), "passwd");
        assert_eq!(dateiname("a\\b\\c.txt"), "c.txt");
        assert_eq!(dateiname(".bashrc"), "bashrc");
        assert_eq!(dateiname(" \u{7}bild.png "), "bild.png");
        assert_eq!(dateiname(""), "download");
        assert_eq!(dateiname(".."), "download");
        assert_eq!(dateiname("ordner/"), "download");
        let lang = format!("{}.pdf", "ä".repeat(150));
        let kurz = dateiname(&lang);
        assert!(kurz.len() <= 200 && kurz.ends_with(".pdf"), "{kurz}");
    }

    #[test]
    fn belegte_namen_bekommen_eine_nummer() {
        let d = std::env::temp_dir().join(format!("qn-dl-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        let mut r = HashSet::new();
        assert_eq!(ziel(&d, "foto.jpg", &r), d.join("foto.jpg"));
        std::fs::write(d.join("foto.jpg"), b"1").unwrap();
        assert_eq!(ziel(&d, "foto.jpg", &r), d.join("foto 2.jpg"));
        // laufender Download und reservierter Name zählen als belegt
        std::fs::write(d.join("foto 2.jpg.wkdownload"), b"").unwrap();
        r.insert("foto 3.jpg".to_owned());
        assert_eq!(ziel(&d, "foto.jpg", &r), d.join("foto 4.jpg"));
        // ein toter Link ist auch belegt
        std::os::unix::fs::symlink(d.join("fehlt"), d.join("x.txt")).unwrap();
        assert_eq!(ziel(&d, "x.txt", &r), d.join("x 2.txt"));
        std::fs::remove_dir_all(&d).unwrap();
    }
}
