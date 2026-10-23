#!/bin/bash
# Prüfhülle Querencia Nachrichten: legt ein Profil für einen Lauf an
# (Muster: Querencia Dokumente werkzeug/profil.sh).
#   profil.sh <profil> <binary> <hell|dunkel> <standard|linux> <BxH des Fensters>
# Inhalt: labwc rc.xml der Sitzung (Runtime-Repo, nur Kopie), gsettings
# (keyfile) mit Thema Querencia-Klassisch, Symbolthema Querencia, Schrift
# DejaVu Sans 12 und Farbschema, Thema und Symbole als Link (in der Hülle
# per --lesen sichtbar), Ablage-Kopie unter <profil>/opt/nachrichten/<sha16>/
# mit share/locale wie die spätere Live-Ablage, Download-Ordner mit einer
# schon vorhandenen probe-download.txt, Testöffner für https, Ordner und
# Textdateien (testoeffner.sh protokolliert statt zu öffnen), Einstellungen
# mit der gewählten Kennung.
set -euo pipefail
HIER=$(cd "$(dirname "$0")" && pwd)
R=$(cd "$HIER/.." && pwd)
P=${1:?profil}; BIN=${2:?binary}; SCHEMA=${3:?hell|dunkel}; KENNUNG=${4:?standard|linux}; FENSTER=${5:?BxH}
mkdir -p "$P"/{config/labwc,config/glib-2.0/settings,data/themes,data/icons,data/applications,heim/Downloads,logs,opt/bin}
: > "$P/.qn-profil"
L=$HOME/projects/querencia-runtime/session/querencia/labwc
cp "$L/rc.xml" "$P/config/labwc/"
for t in Querencia-Klassisch Querencia-Klassisch-Dunkel; do
  ln -sfn "$(readlink -f "$HOME/.local/share/themes/$t")" "$P/data/themes/$t"
done
ln -sfn "$HOME/.local/share/icons/Querencia" "$P/data/icons/Querencia"
case $SCHEMA in hell) CS=default ;; dunkel) CS=prefer-dark ;; *) echo "Schema hell|dunkel" >&2; exit 2 ;; esac
cat > "$P/config/glib-2.0/settings/keyfile" <<X
[org/gnome/system/wsdd]
display-mode='disabled'

[org/gnome/desktop/interface]
font-name='DejaVu Sans 12'
gtk-theme='Querencia-Klassisch'
icon-theme='Querencia'
color-scheme='$CS'
enable-animations=false
X
cat > "$P/config/user-dirs.dirs" <<'X'
XDG_DOWNLOAD_DIR="$HOME/Downloads"
X
printf 'schon da\n' > "$P/heim/Downloads/probe-download.txt"
# Ablage-Kopie (Binary + Kataloge)
S=$(sha256sum "$BIN" | cut -c1-16)
A=$P/opt/nachrichten/$S
mkdir -p "$A"
cp "$BIN" "$A/querencia-nachrichten"
(cd "$R" && python3 werkzeug/i18n.py mo "$A/share/locale" >/dev/null)
chmod -R a-w "$A"
ln -sfn "$A/querencia-nachrichten" "$P/opt/bin/querencia-nachrichten"
# Testöffner statt Browser, Dateimanager und Texteditor
for art in browser ordner datei; do
  case $art in
    browser) MT="x-scheme-handler/http;x-scheme-handler/https;x-scheme-handler/mailto;" ; ARG=%u ;;
    ordner) MT="inode/directory;" ; ARG=%u ;;
    datei) MT="text/plain;" ; ARG=%u ;;
  esac
  cat > "$P/data/applications/qn-test-$art.desktop" <<X
[Desktop Entry]
Type=Application
Name=Prüföffner $art
Exec=$HIER/testoeffner.sh $art $ARG
MimeType=$MT
NoDisplay=true
X
done
cat > "$P/config/mimeapps.list" <<'X'
[Default Applications]
x-scheme-handler/http=qn-test-browser.desktop
x-scheme-handler/https=qn-test-browser.desktop
x-scheme-handler/mailto=qn-test-browser.desktop
inode/directory=qn-test-ordner.desktop
text/plain=qn-test-datei.desktop
X
update-desktop-database "$P/data/applications" 2>/dev/null || true
mkdir -p -m 700 "$P/config/querencia-nachrichten"
printf '[Fenster]\nbreite=%s\nhoehe=%s\nmaximiert=false\n\n[Seite]\nkennung=%s\n' \
  "${FENSTER%x*}" "${FENSTER#*x}" "$KENNUNG" > "$P/config/querencia-nachrichten/einstellungen.ini"
echo "$A"
