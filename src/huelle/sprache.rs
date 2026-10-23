//! Sprache der Seite: WebKit nimmt für `navigator.language` und
//! `Accept-Language` sonst die Locale von `LC_MESSAGES` und übergeht
//! `LANGUAGE`. Die Querencia-Sitzung läuft mit `LANG=en_US.UTF-8` und
//! `LANGUAGE=de`; WhatsApp Web käme dann englisch (gemessen 23.10.2026,
//! Anmeldeseiten-Wächter). Die Hülle gibt WebKit deshalb die Sprachen der
//! Oberfläche (`g_get_language_names`) als BCP-47-Kennungen.

use gtk::glib;

/// „de“, „en_US.UTF-8“, „en_US“, „en“, „C“ → „de“, „en-US“, „en“.
pub fn kennungen(namen: &[String]) -> Vec<String> {
    let mut v: Vec<String> = Vec::new();
    for n in namen {
        let n = n.split(['.', '@']).next().unwrap_or_default();
        if n.is_empty() || n == "C" || n == "POSIX" {
            continue;
        }
        let k = n.replace('_', "-");
        if !v.contains(&k) {
            v.push(k);
        }
    }
    v
}

pub fn einrichten(ctx: &webkit::WebContext) {
    let namen: Vec<String> = glib::language_names()
        .iter()
        .map(|n| n.to_string())
        .collect();
    let liste = kennungen(&namen);
    if liste.is_empty() {
        return;
    }
    let refs: Vec<&str> = liste.iter().map(String::as_str).collect();
    ctx.set_preferred_languages(&refs);
    crate::ereignis("sprachen", &format!("{liste:?}"));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn oberflaechensprachen_werden_bcp47() {
        assert_eq!(
            kennungen(&s(&["de", "en_US.UTF-8", "en_US", "en", "C"])),
            s(&["de", "en-US", "en"])
        );
        assert_eq!(
            kennungen(&s(&["de_DE.UTF-8@euro", "de_DE", "de", "C"])),
            s(&["de-DE", "de"])
        );
        assert!(kennungen(&s(&["C", "POSIX"])).is_empty());
    }
}
