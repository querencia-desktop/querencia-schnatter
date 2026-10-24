#!/bin/bash
# Prüfhülle Querencia Webhülle: ein Lauf.
#   lauf.sh funktionen <de|en> <BxH> <hell|dunkel>
#   [VERGLEICH_BIN=…] lauf.sh grafik <de|en> <BxH> <hell|dunkel>   (0.1.1, QD-169)
#   lauf.sh anmeldeseite <N|E> <de|en> <BxH> <hell|dunkel> <standard|linux>
#   ALT_ABLAGE=<Ablage 0.1.1> lauf.sh umzug <de|en> <BxH> <hell|dunkel>   (0.2.0)
# Gestartet wird die Ablage-Kopie des Release-Baus (BIN=… überschreibt).
# Ergebnis unter nachweis/<art>/<name>/. Sprache wie live: LANG=en_US.UTF-8,
# LANGUAGE=de bzw. en.
set -euo pipefail
HIER=$(cd "$(dirname "$0")" && pwd)
R=$(cd "$HIER/.." && pwd)
ART=${1:?funktionen|anmeldeseite}; shift
case $ART in
  funktionen|grafik|umzug) STUFE=L; SPR=${1:?}; RES=${2:?}; SCHEMA=${3:?}; KENNUNG=standard
                     NAME=$SPR-$RES-$SCHEMA${NAMENSZUSATZ:+-$NAMENSZUSATZ} ;;
  anmeldeseite) STUFE=${1:?}; SPR=${2:?}; RES=${3:?}; SCHEMA=${4:?}; KENNUNG=${5:?}
                NAME=$STUFE-$KENNUNG-$SPR-$RES-$SCHEMA${NAMENSZUSATZ:+-$NAMENSZUSATZ} ;;
  *) echo "lauf.sh funktionen|grafik|umzug|anmeldeseite …" >&2; exit 2 ;;
esac
BIN=${BIN:-$R/target/remap/release/querencia-webhuelle}
[ -x "$BIN" ] || { echo "Release-Bau fehlt: $BIN (werkzeug/bauen.sh release)" >&2; exit 2; }
[ -x "$HIER/bin/qeingabe" ] || bash "$HIER/werkzeug-bauen.sh" >/dev/null
OUT=$R/nachweis/$ART/$NAME
P=$R/gui-test/$ART-$NAME
[ -e "$P" ] && { chmod -R u+rwX "$P"; rm -rf -- "$P"; }
rm -rf -- "$OUT"; mkdir -p "$OUT" "$P"
# Fenster: auf 1920×1080 groß, auf 1024×768 die Vorgabegröße 1000×700
case $RES in 1920x1080) FENSTER=1400x900 ;; *) FENSTER=1000x700 ;; esac
bash "$HIER/profil.sh" "$P" "$BIN" "$SCHEMA" "$KENNUNG" "$FENSTER" >/dev/null
# Grafik-Szenario: eine Vergleichs-Binary (z. B. 0.1.0 aus der Ablage, nur gelesen) ins Profil
if [ -n "${VERGLEICH_BIN:-}" ]; then
  mkdir -p "$P/opt/vergleich"; cp "$VERGLEICH_BIN" "$P/opt/vergleich/querencia-webhuelle"
  (cd "$R" && python3 werkzeug/i18n.py mo "$P/opt/vergleich/share/locale" >/dev/null)
fi
# Umzug (0.2.0): die Ablage 0.1.1 (Binary querencia-nachrichten und ihre
# Kataloge, nur gelesen) ins Profil; Einstellungen unter dem alten Namen
if [ "$ART" = umzug ]; then
  [ -x "${ALT_ABLAGE:-}/querencia-nachrichten" ] || { echo "ALT_ABLAGE=<Ablage 0.1.1> fehlt" >&2; exit 2; }
  mkdir -p "$P/opt/alt"; cp -r "$ALT_ABLAGE/querencia-nachrichten" "$ALT_ABLAGE/share" "$P/opt/alt/"
  mv -T "$P/config/querencia-webhuelle" "$P/config/querencia-nachrichten"
fi
cat >"$P/env.sh" <<X
BIN=\$(ls "$P"/opt/bin/querencia-webhuelle)
OUT=$OUT
W=$HIER
SPR=$SPR
RES=$RES
SCHEMA=$SCHEMA
KENNUNG=$KENNUNG
STUFE=$STUFE
SANDBOX=${SANDBOX:-aus}
FENSTER=$FENSTER
export LANGUAGE=$SPR
X
[ -n "${GSK_RENDERER:-}" ] && echo "export GSK_RENDERER=$GSK_RENDERER" >>"$P/env.sh"
# Grafik-Szenario: Fälle und Reihenfolge (Vorgabe: automatisch an aus umgebung)
[ -n "${GRAFIK_FAELLE:-}" ] && echo "GRAFIK_FAELLE=\"$GRAFIK_FAELLE\"" >>"$P/env.sh"
LESEN=(--lesen "$(readlink -f "$HOME/.local/share/themes/Querencia-Klassisch")/.."
       --lesen "$HOME/.local/share/icons/Querencia")
bash "$HIER/huelle.sh" --profil "$P" --labwc --sprache en_US.UTF-8 --setenv "LANGUAGE=$SPR" \
  "${LESEN[@]}" --schreiben "$OUT" --zeitlimit "${ZEIT:-600}" --protokoll "$OUT/huelle.txt" \
  "$STUFE" -- /bin/bash "$HIER/szenario-$ART.sh"
