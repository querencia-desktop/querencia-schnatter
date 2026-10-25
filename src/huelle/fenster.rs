//! Das Hauptfenster: Menüleiste, Hinweisleiste und die Webansicht, sonst
//! nichts um die Seite herum (labwc zeichnet die Titelleiste).
//!
//! Hier hängen die Signale der Webansicht an der Hülle: Navigation (E5),
//! Rechte (E3), Meldungen, Titelzähler (E10), Neuladen nach einem Absturz
//! des Web-Prozesses, Zoom und Fenstergröße (gemerkt).

use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use gtk::prelude::*;
use gtk::{gio, glib};
use webkit::prelude::*;

use super::dialoge::{self, Aenderung, Platz};
use super::einstellungen::{Einstellungen, Kennung};
use super::navigation::{self, Anfrage, Ziel};
use super::sitzung::Ordner;
use super::{meldungen, pruefung, rechte, zoom};
use crate::dienst::{Dienst, Start};
use crate::i18n::{gettext, gettext_f};

/// Höchstens so viele Abstürze in [`ABSTURZ_FENSTER`] werden von selbst neu
/// geladen; danach erst auf Knopfdruck (keine Schleife).
const ABSTUERZE: usize = 3;
const ABSTURZ_FENSTER: Duration = Duration::from_secs(60);

pub struct Fenster {
    pub fenster: gtk::ApplicationWindow,
    pub view: webkit::WebView,
    dienst: Rc<Dienst>,
    einstellungen: Rc<Einstellungen>,
    ordner: Ordner,
    leiste: gtk::Box,
    hinweis: gtk::Revealer,
    hinweis_text: gtk::Label,
    abstuerze: RefCell<Vec<Instant>>,
    /// Name des Programms für den Fenstertitel.
    titel: String,
    info: Platz,
    hilfe: Platz,
    einstellungsfenster: Platz,
}

impl Fenster {
    pub fn neu(
        app: &gtk::Application,
        dienst: Rc<Dienst>,
        einstellungen: Rc<Einstellungen>,
        sitzung: &webkit::NetworkSession,
        ordner: Ordner,
    ) -> Rc<Self> {
        let titel = if pruefung::aktiv() {
            gettext_f(
                "{program} – test page",
                &[("program", &gettext("Schnatter"))],
            )
        } else {
            gettext("Schnatter")
        };
        let fenster = gtk::ApplicationWindow::builder()
            .application(app)
            .title(&titel)
            .build();
        fenster.add_css_class("querencia-schnatter");
        let (b, h, maximiert) = einstellungen.fenster();
        fenster.set_default_size(b, h);
        if maximiert {
            fenster.maximize();
        }

        // Autoplay nur für diese Ansicht (E4): ohne Freigabe blieben
        // Meldungstöne und Sprachnachrichten stumm (gemessen).
        let regeln = webkit::WebsitePolicies::builder()
            .autoplay(webkit::AutoplayPolicy::Allow)
            .build();
        let inhalte = webkit::UserContentManager::new();
        if let Some(skript) = dienst.benutzerskript {
            inhalte.add_script(&webkit::UserScript::new(
                skript,
                webkit::UserContentInjectedFrames::TopFrame,
                webkit::UserScriptInjectionTime::Start,
                &[],
                &[],
            ));
        }
        let view = webkit::WebView::builder()
            .network_session(sitzung)
            .website_policies(&regeln)
            .user_content_manager(&inhalte)
            .vexpand(true)
            .hexpand(true)
            .build();

        let (hinweis, hinweis_text, hinweis_knopf, hinweis_zu) = hinweisleiste();
        let leiste = super::menue::leiste();
        let kasten = gtk::Box::new(gtk::Orientation::Vertical, 0);
        kasten.append(&leiste);
        kasten.append(&hinweis);
        kasten.append(&view);
        fenster.set_child(Some(&kasten));

        let f = Rc::new(Self {
            fenster,
            view,
            dienst,
            einstellungen,
            ordner,
            leiste,
            hinweis,
            hinweis_text,
            abstuerze: RefCell::default(),
            titel,
            info: Platz::default(),
            hilfe: Platz::default(),
            einstellungsfenster: Platz::default(),
        });
        let w = Rc::downgrade(&f);
        hinweis_knopf.connect_clicked(move |_| {
            if let Some(f) = w.upgrade() {
                f.hinweis.set_reveal_child(false);
                f.abstuerze.borrow_mut().clear();
                f.laden();
            }
        });
        let w = Rc::downgrade(&f);
        hinweis_zu.connect_clicked(move |_| {
            if let Some(f) = w.upgrade() {
                f.hinweis.set_reveal_child(false);
            }
        });
        f.einstellungen_anwenden();
        f.view.set_zoom_level(f.einstellungen.zoom());
        f.signale();
        f.aktionen();
        meldungen::verbinden(&f.view, &f.fenster);
        pruefung::zustand_beobachten(&f.view);
        f
    }

    /// Kennung und Ton aus den Einstellungen auf die Ansicht.
    fn einstellungen_anwenden(&self) {
        let s = webkit::prelude::WebViewExt::settings(&self.view).unwrap_or_default();
        let kennung = self.einstellungen.kennung();
        s.set_enable_site_specific_quirks(kennung == Kennung::Standard);
        // immer WebKits eigene Kennung, nie eine gepflegte Zeichenkette (S4)
        s.set_user_agent(None);
        s.set_enable_developer_extras(false);
        s.set_javascript_can_open_windows_automatically(false);
        // Konsole der Seite nur auf der Prüfseite nach stdout (Prüfergebnisse)
        s.set_enable_write_console_messages_to_stdout(pruefung::aktiv());
        // WebKits Attrappen-Mikrofon und -Kamera nur auf der Prüfseite (die
        // Prüfhülle hat keine Geräte; ohne Gerät fragt WebKit gar nicht)
        s.set_enable_mock_capture_devices(pruefung::aktiv());
        self.view.set_settings(&s);
        self.view.set_is_muted(!self.einstellungen.ton());
        crate::ereignis(
            "kennung",
            &format!(
                "{} quirks={} ton={} ua={}",
                kennung.schluessel(),
                s.enables_site_specific_quirks(),
                self.einstellungen.ton(),
                s.user_agent().unwrap_or_default()
            ),
        );
    }

    /// Lädt die Startseite des Dienstes.
    pub fn laden(&self) {
        match &self.dienst.start {
            Start::Adresse(a) => self.view.load_uri(a),
            Start::Html { inhalt, basis } => self.view.load_html(inhalt, Some(basis)),
        }
    }

    pub fn zeigen(&self) {
        self.fenster.present();
    }

    fn hinweis_zeigen(&self, text: &str) {
        self.hinweis_text.set_text(text);
        self.hinweis.set_reveal_child(true);
    }

    fn signale(self: &Rc<Self>) {
        let w = Rc::downgrade(self);
        self.view
            .connect_decide_policy(move |_, entscheidung, art| {
                let Some(f) = w.upgrade() else { return false };
                f.entscheiden(entscheidung, art)
            });
        // „Link in neuem Fenster öffnen“ aus dem Kontextmenü kommt ohne
        // Entscheidung hier an: nie ein zweites Fenster.
        let w = Rc::downgrade(self);
        self.view.connect_create(move |_, aktion| {
            let f = w.upgrade()?;
            let uri = aktion.request().and_then(|r| r.uri()).unwrap_or_default();
            let ziel = navigation::entscheiden(
                &f.dienst,
                &uri,
                Anfrage::NeuesFenster {
                    geste: aktion.is_user_gesture(),
                },
            );
            crate::ereignis(
                "create",
                &format!("{} -> {}", pruefung::adresse(Some(&uri)), ziel_text(&ziel)),
            );
            f.ziel_ausfuehren(&ziel);
            None
        });
        let w = Rc::downgrade(self);
        self.view.connect_permission_request(move |view, req| {
            let Some(f) = w.upgrade() else { return false };
            rechte::anfrage(
                f.fenster.upcast_ref(),
                view,
                &f.dienst,
                &f.einstellungen,
                req,
            )
        });
        let w = Rc::downgrade(self);
        self.view.connect_query_permission_state(move |_, q| {
            let Some(f) = w.upgrade() else { return false };
            rechte::abfrage(&f.dienst, &f.einstellungen, q)
        });
        let w = Rc::downgrade(self);
        self.view.connect_title_notify(move |view| {
            if let Some(f) = w.upgrade() {
                f.titel_setzen(view.title().as_deref().unwrap_or_default());
            }
        });
        let w = Rc::downgrade(self);
        self.view.connect_web_process_terminated(move |_, grund| {
            if let Some(f) = w.upgrade() {
                f.abgestuerzt(grund);
            }
        });
        let w = Rc::downgrade(self);
        self.view.connect_load_changed(move |view, ereignis| {
            crate::ereignis(
                "load-changed",
                &format!("{ereignis:?} {}", pruefung::adresse(view.uri().as_deref())),
            );
            if ereignis == webkit::LoadEvent::Finished
                && let Some(f) = w.upgrade()
            {
                f.geladen();
            }
        });
        let w = Rc::downgrade(self);
        self.view.connect_load_failed(move |_, _, uri, fehler| {
            if fehler.matches(webkit::NetworkError::Cancelled)
                || fehler.matches(webkit::PolicyError::FrameLoadInterruptedByPolicyChange)
            {
                return false;
            }
            crate::ereignis(
                "load-failed",
                &format!("{} {}", pruefung::adresse(Some(uri)), fehler.message()),
            );
            if let Some(f) = w.upgrade() {
                f.hinweis_zeigen(&gettext_f(
                    "{service} could not be loaded: {error}",
                    &[
                        ("service", f.dienst.anzeigename),
                        ("error", fehler.message()),
                    ],
                ));
            }
            false
        });
        // Dateiwahl (Bericht §2.3): WebKits Standardweg, der GTK-Dateidialog
        // über das Portal; hier nur protokolliert.
        self.view.connect_run_file_chooser(|_, req| {
            crate::ereignis(
                "run-file-chooser",
                &format!(
                    "mehrfach={} mime={:?}",
                    req.selects_multiple(),
                    req.mime_types()
                ),
            );
            false
        });
        self.view.connect_is_playing_audio_notify(|v| {
            crate::ereignis("is-playing-audio", &v.is_playing_audio().to_string());
        });
        let w = Rc::downgrade(self);
        self.view.connect_zoom_level_notify(move |v| {
            if let Some(f) = w.upgrade() {
                f.einstellungen.zoom_setzen(v.zoom_level());
            }
        });
        // Vollbild (F11 oder ein Video): ohne Menüleiste.
        let w = Rc::downgrade(self);
        self.fenster.connect_fullscreened_notify(move |fenster| {
            if let Some(f) = w.upgrade() {
                f.leiste.set_visible(!fenster.is_fullscreen());
            }
        });
        let w = Rc::downgrade(self);
        self.fenster.connect_close_request(move |fenster| {
            if let Some(f) = w.upgrade()
                && !fenster.is_fullscreen()
            {
                let (b, h) = fenster.default_size();
                crate::ereignis(
                    "fenster-gemerkt",
                    &format!("{b}x{h} maximiert={}", fenster.is_maximized()),
                );
                f.einstellungen.fenster_setzen(b, h, fenster.is_maximized());
            }
            glib::Propagation::Proceed
        });
    }

    fn entscheiden(
        &self,
        entscheidung: &webkit::PolicyDecision,
        art: webkit::PolicyDecisionType,
    ) -> bool {
        let (uri, anfrage) = match art {
            webkit::PolicyDecisionType::NavigationAction
            | webkit::PolicyDecisionType::NewWindowAction => {
                let Some(aktion) = entscheidung
                    .downcast_ref::<webkit::NavigationPolicyDecision>()
                    .and_then(|d| d.navigation_action())
                else {
                    return false;
                };
                let uri = aktion.request().and_then(|r| r.uri()).unwrap_or_default();
                let geste = aktion.is_user_gesture();
                let a = if art == webkit::PolicyDecisionType::NewWindowAction {
                    Anfrage::NeuesFenster { geste }
                } else {
                    Anfrage::Navigation { geste }
                };
                (uri.to_string(), a)
            }
            webkit::PolicyDecisionType::Response => {
                let Some(d) = entscheidung.downcast_ref::<webkit::ResponsePolicyDecision>() else {
                    return false;
                };
                let uri = d.request().and_then(|r| r.uri()).unwrap_or_default();
                (
                    uri.to_string(),
                    Anfrage::Antwort {
                        hauptseite: d.is_main_frame_main_resource(),
                    },
                )
            }
            _ => return false,
        };
        let ziel = navigation::entscheiden(&self.dienst, &uri, anfrage);
        if ziel != Ziel::Weiter || !matches!(anfrage, Anfrage::Antwort { hauptseite: false }) {
            crate::ereignis(
                "decide-policy",
                &format!(
                    "{anfrage:?} {} -> {}",
                    pruefung::adresse(Some(&uri)),
                    ziel_text(&ziel)
                ),
            );
        }
        if ziel == Ziel::Weiter {
            return false;
        }
        entscheidung.ignore();
        self.ziel_ausfuehren(&ziel);
        true
    }

    /// Öffnet ein Ziel außerhalb (sofort oder nach Rückfrage) bzw.
    /// protokolliert eine Sperre; das Verwerfen in WebKit macht der Aufrufer.
    fn ziel_ausfuehren(&self, ziel: &Ziel) {
        match ziel {
            Ziel::Weiter | Ziel::Verwerfen => {}
            Ziel::Extern(u) => navigation::extern_oeffnen(&self.fenster, u),
            Ziel::Fragen(u) => navigation::fragen_und_oeffnen(self.fenster.upcast_ref(), u),
            Ziel::Sperren(was) => crate::log(&format!("Wechsel auf {was} abgelehnt")),
        }
    }

    /// Fenstertitel mit der Zahl der ungelesenen Nachrichten (E10).
    fn titel_setzen(&self, seitentitel: &str) {
        let zahl = (self.dienst.zaehler)(seitentitel);
        let t = match zahl {
            Some(n) => format!("({n}) {}", self.titel),
            None => self.titel.clone(),
        };
        crate::ereignis("titel", &format!("ungelesen={zahl:?} fenster={t:?}"));
        self.fenster.set_title(Some(&t));
    }

    fn geladen(&self) {
        // Ein Ladefehler ist behoben; ein Absturzhinweis bleibt stehen, bis
        // er geschlossen wird oder 20 s vergangen sind.
        if self.abstuerze.borrow().is_empty() {
            self.hinweis.set_reveal_child(false);
        } else {
            let h = self.hinweis.downgrade();
            glib::timeout_add_local_once(Duration::from_secs(20), move || {
                if let Some(h) = h.upgrade() {
                    h.set_reveal_child(false);
                }
            });
        }
    }

    /// Der Web-Prozess ist weg: neu laden, mit Hinweis; nach mehreren
    /// Abstürzen kurz hintereinander nur noch auf Knopfdruck.
    fn abgestuerzt(&self, grund: webkit::WebProcessTerminationReason) {
        crate::ereignis("web-process-terminated", &format!("{grund:?}"));
        if grund == webkit::WebProcessTerminationReason::TerminatedByApi {
            return;
        }
        let jetzt = Instant::now();
        let mut a = self.abstuerze.borrow_mut();
        a.retain(|t| jetzt.duration_since(*t) < ABSTURZ_FENSTER);
        a.push(jetzt);
        let zahl = a.len();
        drop(a);
        if zahl <= ABSTUERZE {
            self.hinweis_zeigen(&gettext("The page stopped unexpectedly and was reloaded."));
            crate::ereignis("neu-laden", &format!("nach Absturz {zahl}"));
            self.laden();
        } else {
            self.hinweis_zeigen(&gettext(
                "The page stopped unexpectedly several times. Reload it when you are ready.",
            ));
        }
    }

    fn aktionen(self: &Rc<Self>) {
        type Tun = fn(&Rc<Fenster>);
        let liste: [(&str, Tun); 13] = [
            ("neu-laden", |f| {
                f.hinweis.set_reveal_child(false);
                f.view.reload();
            }),
            ("abmelden", |f| f.abmelden_fragen()),
            ("ausschneiden", |f| {
                f.view.execute_editing_command(webkit::EDITING_COMMAND_CUT)
            }),
            ("kopieren", |f| {
                f.view.execute_editing_command(webkit::EDITING_COMMAND_COPY)
            }),
            ("einfuegen", |f| {
                f.view
                    .execute_editing_command(webkit::EDITING_COMMAND_PASTE)
            }),
            ("alles", |f| {
                f.view
                    .execute_editing_command(webkit::EDITING_COMMAND_SELECT_ALL)
            }),
            ("einstellungen", |f| f.einstellungen_zeigen()),
            ("vergroessern", |f| {
                f.view.set_zoom_level(zoom::groesser(f.view.zoom_level()))
            }),
            ("verkleinern", |f| {
                f.view.set_zoom_level(zoom::kleiner(f.view.zoom_level()))
            }),
            ("normalgroesse", |f| f.view.set_zoom_level(1.0)),
            ("vollbild", |f| {
                if f.fenster.is_fullscreen() {
                    f.fenster.unfullscreen();
                } else {
                    f.fenster.fullscreen();
                }
            }),
            ("hilfe", |f| {
                dialoge::hilfe(f.fenster.upcast_ref(), &f.hilfe)
            }),
            ("info", |f| {
                dialoge::info(f.fenster.upcast_ref(), &f.info, super::SYMBOL)
            }),
        ];
        for (name, tun) in liste {
            let a = gio::SimpleAction::new(name, None);
            let w = Rc::downgrade(self);
            a.connect_activate(move |_, _| {
                if let Some(f) = w.upgrade() {
                    crate::ereignis("aktion", name);
                    tun(&f);
                }
            });
            self.fenster.add_action(&a);
        }
    }

    fn einstellungen_zeigen(self: &Rc<Self>) {
        let w = Rc::downgrade(self);
        dialoge::einstellungen(
            self.fenster.upcast_ref(),
            &self.einstellungsfenster,
            &self.einstellungen,
            Rc::new(move |aenderung| {
                let Some(f) = w.upgrade() else { return };
                crate::ereignis("einstellung", &format!("{aenderung:?}"));
                match aenderung {
                    Aenderung::Kennung => {
                        f.einstellungen_anwenden();
                        f.laden();
                    }
                    Aenderung::Ton => f.view.set_is_muted(!f.einstellungen.ton()),
                    // gilt ab dem nächsten Start (grafik::vorbereiten)
                    Aenderung::Rechte | Aenderung::Grafik => {}
                }
            }),
        );
    }

    /// „Abmelden und Daten löschen“ (R9): nach Rückfrage alle Daten der
    /// Sitzung über WebKit löschen, die gespeicherten Rechte vergessen, den
    /// vollständigen Rückbau der Ordner für den nächsten Start vormerken
    /// und die Startseite neu laden.
    fn abmelden_fragen(self: &Rc<Self>) {
        let w = Rc::downgrade(self);
        dialoge::frage(
            self.fenster.upcast_ref(),
            "dialog-warning",
            &gettext_f(
                "Log out of {service} and delete its data?",
                &[("service", self.dienst.anzeigename)],
            ),
            &gettext(
                "Everything this program stored on this computer is deleted: the \
                 sign-in, cached chats and media, and the remembered permissions. \
                 Your chats stay on your phone. Downloaded files are kept.",
            ),
            &gettext("_Cancel"),
            &gettext("_Log Out and Delete"),
            move |ja| {
                if ja && let Some(f) = w.upgrade() {
                    f.abmelden();
                }
            },
        );
    }

    fn abmelden(self: &Rc<Self>) {
        crate::ereignis("abmelden", "beginnt");
        self.view.stop_loading();
        self.einstellungen.rechte_vergessen();
        if let Err(e) = self.ordner.loeschen_vormerken() {
            crate::log(&format!("Löschung nicht vorgemerkt: {e}"));
        }
        let Some(daten) = self
            .view
            .network_session()
            .and_then(|s| s.website_data_manager())
        else {
            crate::log("keine Datenverwaltung der Sitzung – nur beim nächsten Start gelöscht");
            return;
        };
        let w = glib::thread_guard::ThreadGuard::new(Rc::downgrade(self));
        daten.clear(
            webkit::WebsiteDataTypes::ALL,
            glib::TimeSpan::from_seconds(0),
            None::<&gio::Cancellable>,
            move |r| {
                match &r {
                    Ok(()) => crate::ereignis("abmelden", "daten gelöscht"),
                    Err(e) => crate::log(&format!("Daten nicht gelöscht: {e}")),
                }
                if let Some(f) = w.get_ref().upgrade() {
                    f.laden();
                }
            },
        );
    }
}

/// Ein Ziel fürs Protokoll, ohne Adresse (die steht davor, gekürzt).
fn ziel_text(ziel: &Ziel) -> String {
    match ziel {
        Ziel::Weiter => "Weiter".into(),
        Ziel::Extern(_) => "Extern".into(),
        Ziel::Fragen(_) => "Fragen".into(),
        Ziel::Verwerfen => "Verwerfen".into(),
        Ziel::Sperren(was) => format!("Sperren({was})"),
    }
}

/// Die Hinweisleiste über der Seite: Symbol, Text, „Neu laden“, Schließen.
fn hinweisleiste() -> (gtk::Revealer, gtk::Label, gtk::Button, gtk::Button) {
    let r = gtk::Revealer::new();
    r.set_transition_type(gtk::RevealerTransitionType::SlideDown);
    let b = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    b.add_css_class("qn-hinweis");
    let bild = gtk::Image::from_icon_name("dialog-information");
    let text = gtk::Label::new(None);
    text.set_hexpand(true);
    text.set_xalign(0.0);
    text.set_wrap(true);
    let neu = gtk::Button::with_mnemonic(&gettext("_Reload"));
    let zu = gtk::Button::from_icon_name("window-close");
    zu.set_has_frame(false);
    zu.set_tooltip_text(Some(&gettext("Close")));
    b.append(&bild);
    b.append(&text);
    b.append(&neu);
    b.append(&zu);
    r.set_child(Some(&b));
    (r, text, neu, zu)
}
