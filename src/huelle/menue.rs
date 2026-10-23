//! Die Menüleiste über `querencia_oberflaeche::classic_menu` (wie Querencia
//! Dateien): Datei, Bearbeiten, Ansicht, Hilfe.
//!
//! Die Kürzel in den Menüs sind nur Anzeige (`accel`-Attribut). Echte
//! Anwendungskürzel ([`KUERZEL`]) gibt es nur für das, was die Webansicht
//! nicht selbst kann; Ausschneiden, Kopieren, Einfügen und Alles auswählen
//! behandelt WebKit mit den üblichen Tasten selbst, ein Anwendungskürzel
//! würde sie der Seite wegnehmen (GTK 4 prüft Anwendungskürzel vor dem
//! fokussierten Element).

use gtk::gio;
use gtk::prelude::*;
use querencia_oberflaeche::classic_menu;

use crate::i18n::gettext;

/// Anwendungskürzel (Aktion mit Präfix, Tasten).
pub const KUERZEL: &[(&str, &[&str])] = &[
    ("win.neu-laden", &["<Control>r", "F5"]),
    ("app.beenden", &["<Control>q"]),
    (
        "win.vergroessern",
        &["<Control>plus", "<Control>equal", "<Control>KP_Add"],
    ),
    (
        "win.verkleinern",
        &["<Control>minus", "<Control>KP_Subtract"],
    ),
    ("win.normalgroesse", &["<Control>0", "<Control>KP_0"]),
    ("win.vollbild", &["F11"]),
    ("win.hilfe", &["F1"]),
];

pub fn kuerzel_setzen(app: &gtk::Application) {
    for (aktion, tasten) in KUERZEL {
        app.set_accels_for_action(aktion, tasten);
    }
}

/// Nimmt die Zugriffstaste aus einer Beschriftung („_Beenden“ → „Beenden“);
/// die klassischen Menüs zeigen Beschriftungen, wie sie sind.
pub fn ohne_zugriffstaste(text: &str) -> String {
    let mut aus = String::with_capacity(text.len());
    let mut z = text.chars().peekable();
    while let Some(c) = z.next() {
        if c == '_' {
            if z.peek() == Some(&'_') {
                aus.push('_');
                z.next();
            }
            continue;
        }
        aus.push(c);
    }
    aus
}

fn eintrag(text: &str, aktion: &str, kuerzel: Option<&str>, symbol: Option<&str>) -> gio::MenuItem {
    let item = gio::MenuItem::new(Some(&ohne_zugriffstaste(text)), Some(aktion));
    if let Some(k) = kuerzel {
        item.set_attribute_value("accel", Some(&k.to_variant()));
    }
    if let Some(s) = symbol {
        item.set_icon(&gio::ThemedIcon::new(s));
    }
    item
}

fn abschnitt(items: &[gio::MenuItem]) -> gio::MenuItem {
    let m = gio::Menu::new();
    for i in items {
        m.append_item(i);
    }
    gio::MenuItem::new_section(None, &m)
}

fn menue(abschnitte: Vec<Vec<gio::MenuItem>>) -> gio::Menu {
    let m = gio::Menu::new();
    for a in abschnitte {
        m.append_item(&abschnitt(&a));
    }
    m
}

pub fn datei() -> gio::Menu {
    menue(vec![
        vec![eintrag(
            &gettext("_Reload"),
            "win.neu-laden",
            Some("<Control>r"),
            Some("view-refresh"),
        )],
        vec![eintrag(
            &gettext("_Log Out and Delete Data…"),
            "win.abmelden",
            None,
            Some("system-log-out"),
        )],
        vec![eintrag(
            &gettext("_Quit"),
            "app.beenden",
            Some("<Control>q"),
            Some("application-exit"),
        )],
    ])
}

pub fn bearbeiten() -> gio::Menu {
    menue(vec![
        vec![
            eintrag(
                &gettext("Cu_t"),
                "win.ausschneiden",
                Some("<Control>x"),
                Some("edit-cut"),
            ),
            eintrag(
                &gettext("_Copy"),
                "win.kopieren",
                Some("<Control>c"),
                Some("edit-copy"),
            ),
            eintrag(
                &gettext("_Paste"),
                "win.einfuegen",
                Some("<Control>v"),
                Some("edit-paste"),
            ),
        ],
        vec![eintrag(
            &gettext("Select _All"),
            "win.alles",
            Some("<Control>a"),
            Some("edit-select-all"),
        )],
        vec![eintrag(
            &gettext("Pr_eferences"),
            "win.einstellungen",
            None,
            Some("preferences-system"),
        )],
    ])
}

pub fn ansicht() -> gio::Menu {
    menue(vec![
        vec![
            eintrag(
                &gettext("Zoom _In"),
                "win.vergroessern",
                Some("<Control>plus"),
                Some("zoom-in"),
            ),
            eintrag(
                &gettext("Zoom _Out"),
                "win.verkleinern",
                Some("<Control>minus"),
                Some("zoom-out"),
            ),
            eintrag(
                &gettext("_Normal Size"),
                "win.normalgroesse",
                Some("<Control>0"),
                Some("zoom-original"),
            ),
        ],
        vec![eintrag(
            &gettext("_Full Screen"),
            "win.vollbild",
            Some("F11"),
            Some("view-fullscreen"),
        )],
    ])
}

pub fn hilfe() -> gio::Menu {
    menue(vec![vec![
        eintrag(
            &gettext("_Contents"),
            "win.hilfe",
            Some("F1"),
            Some("help-browser"),
        ),
        eintrag(&gettext("_About"), "win.info", None, Some("help-about")),
    ]])
}

/// Die Menüleiste eines Fensters.
pub fn leiste() -> gtk::Box {
    let leiste = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    leiste.add_css_class("querencia-menuleiste");
    let menues: [(String, gio::Menu); 4] = [
        (gettext("_File"), datei()),
        (gettext("_Edit"), bearbeiten()),
        (gettext("_View"), ansicht()),
        (gettext("_Help"), hilfe()),
    ];
    for (titel, modell) in menues {
        let knopf = gtk::MenuButton::new();
        knopf.set_label(&titel);
        knopf.set_use_underline(true);
        knopf.set_always_show_arrow(false);
        knopf.set_has_frame(false);
        knopf.set_can_focus(false);
        knopf.add_css_class("querencia-menuknopf");
        classic_menu::attach(&knopf, modell.upcast_ref());
        leiste.append(&knopf);
    }
    leiste
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zugriffstasten_fallen_aus_den_eintraegen() {
        assert_eq!(ohne_zugriffstaste("_Beenden"), "Beenden");
        assert_eq!(ohne_zugriffstaste("Aus_schneiden"), "Ausschneiden");
        assert_eq!(ohne_zugriffstaste("a__b"), "a_b");
    }

    #[test]
    fn menues_haben_die_eintraege_des_auftrags() {
        let aktionen = |m: &gio::Menu| -> Vec<String> {
            classic_menu::rows(m.upcast_ref())
                .into_iter()
                .filter_map(|r| match r {
                    classic_menu::Row::Entry { action, .. } => action,
                    _ => None,
                })
                .collect()
        };
        assert_eq!(
            aktionen(&datei()),
            ["win.neu-laden", "win.abmelden", "app.beenden"]
        );
        assert_eq!(
            aktionen(&bearbeiten()),
            [
                "win.ausschneiden",
                "win.kopieren",
                "win.einfuegen",
                "win.alles",
                "win.einstellungen"
            ]
        );
        assert_eq!(
            aktionen(&ansicht()),
            [
                "win.vergroessern",
                "win.verkleinern",
                "win.normalgroesse",
                "win.vollbild"
            ]
        );
        assert_eq!(aktionen(&hilfe()), ["win.hilfe", "win.info"]);
    }

    #[test]
    fn keine_anwendungskuerzel_fuer_die_bearbeiten_tasten() {
        // Strg+X/C/V/A gehören der Webansicht (Modulbeschreibung)
        for (_, tasten) in KUERZEL {
            for t in *tasten {
                for frei in ["<Control>x", "<Control>c", "<Control>v", "<Control>a"] {
                    assert_ne!(t, &frei);
                }
            }
        }
    }
}
