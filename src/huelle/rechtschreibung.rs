//! Rechtschreibprüfung (E7): die Sprachen der Oberfläche und Englisch (USA),
//! nur mit vorhandenem Wörterbuch.
//!
//! Gemessen in der Machbarkeit: WebKit (enchant-2 → hunspell) verwirft
//! fehlende Sprachen still. Die Sprachen kommen aus `g_get_language_names`
//! (`LANGUAGE`, `LC_ALL`, `LC_MESSAGES`, `LANG`), nicht nur aus `LANG`:
//! Die Querencia-Sitzung läuft mit `LANG=en_US.UTF-8` und `LANGUAGE=de`,
//! aus `LANG` allein käme kein Deutsch (Entscheidung im Bericht). Eine reine
//! Sprachangabe („de“) wird zum Wörterbuch des gleichnamigen Landes
//! („de_DE“), sonst zum ersten vorhandenen („de_AT“ …).

use std::path::PathBuf;

use gtk::glib;

/// Englisch kommt immer dazu (wenn vorhanden).
pub const ENGLISCH: &str = "en_US";

/// Ordner mit hunspell-Wörterbüchern (`<sprache>.dic` + `.aff`).
fn ordner() -> Vec<PathBuf> {
    let mut v = vec![glib::user_config_dir().join("enchant/hunspell")];
    for d in glib::system_data_dirs() {
        v.push(d.join("hunspell"));
        v.push(d.join("myspell"));
        v.push(d.join("myspell/dicts"));
    }
    v
}

/// Vorhandene Wörterbücher (Sprachkennungen wie „de_DE“), sortiert.
pub fn vorhanden() -> Vec<String> {
    let mut v: Vec<String> = Vec::new();
    for d in ordner() {
        let Ok(eintraege) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in eintraege.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if let Some(tag) = name.strip_suffix(".dic")
                && d.join(format!("{tag}.aff")).exists()
                && !v.iter().any(|t| t == tag)
            {
                v.push(tag.to_owned());
            }
        }
    }
    v.sort();
    v
}

/// Die Sprachen für WebKit aus den Namen der Oberflächensprachen (Reihenfolge
/// bleibt, keine doppelt) und den vorhandenen Wörterbüchern; Englisch (USA)
/// am Ende, falls vorhanden.
pub fn sprachen(namen: &[String], vorhanden: &[String]) -> Vec<String> {
    let mut v: Vec<String> = Vec::new();
    let nimm = |tag: &str, v: &mut Vec<String>| {
        if vorhanden.iter().any(|t| t == tag) && !v.iter().any(|t| t == tag) {
            v.push(tag.to_owned());
        }
    };
    for name in namen {
        // „de_DE.UTF-8@euro“ → „de_DE“
        let name = name.split(['.', '@']).next().unwrap_or_default();
        if name.is_empty() || name == "C" || name == "POSIX" {
            continue;
        }
        if name.contains('_') {
            nimm(name, &mut v);
            continue;
        }
        let eigen = format!("{name}_{}", name.to_ascii_uppercase());
        if vorhanden.contains(&eigen) {
            nimm(&eigen, &mut v);
        } else if let Some(erstes) = vorhanden
            .iter()
            .find(|t| t.starts_with(&format!("{name}_")))
        {
            nimm(erstes, &mut v);
        }
    }
    nimm(ENGLISCH, &mut v);
    v
}

/// Schaltet die Prüfung im WebKit-Kontext ein; meldet, was WebKit annimmt.
pub fn einrichten(ctx: &webkit::WebContext) {
    let namen: Vec<String> = glib::language_names()
        .iter()
        .map(|n| n.to_string())
        .collect();
    let liste = sprachen(&namen, &vorhanden());
    if liste.is_empty() {
        crate::log("Rechtschreibung aus: kein passendes Wörterbuch");
        ctx.set_spell_checking_enabled(false);
        return;
    }
    let refs: Vec<&str> = liste.iter().map(String::as_str).collect();
    ctx.set_spell_checking_languages(&refs);
    ctx.set_spell_checking_enabled(true);
    let angenommen: Vec<String> = ctx
        .spell_checking_languages()
        .iter()
        .map(|s| s.to_string())
        .collect();
    crate::ereignis(
        "rechtschreibung",
        &format!("angefragt={liste:?} angenommen={angenommen:?}"),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn sitzung_mit_language_de_bekommt_deutsch_und_englisch() {
        // LANGUAGE=de, LANG=en_US.UTF-8 (Querencia-Sitzung)
        let namen = s(&["de", "en_US.UTF-8", "en_US", "en", "C"]);
        let da = s(&["af_ZA", "de_BE", "de_DE", "de_LU", "en_US"]);
        assert_eq!(sprachen(&namen, &da), s(&["de_DE", "en_US"]));
    }

    #[test]
    fn fehlende_woerterbuecher_fallen_weg() {
        let namen = s(&["fr_FR.UTF-8", "fr_FR", "fr", "C"]);
        assert_eq!(sprachen(&namen, &s(&["en_US"])), s(&["en_US"]));
        assert!(sprachen(&namen, &s(&["af_ZA"])).is_empty());
    }

    #[test]
    fn reine_sprache_nimmt_das_erste_land_ohne_eigenes() {
        let namen = s(&["de", "C"]);
        assert_eq!(sprachen(&namen, &s(&["de_AT", "de_CH"])), s(&["de_AT"]));
        let namen = s(&["de_CH.UTF-8", "de_CH", "de", "C"]);
        assert_eq!(
            sprachen(&namen, &s(&["de_CH", "de_DE", "en_US"])),
            s(&["de_CH", "de_DE", "en_US"])
        );
    }
}
