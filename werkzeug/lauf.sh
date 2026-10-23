#!/bin/bash
# Prüfhülle Querencia Nachrichten: ein Lauf.
#   lauf.sh funktionen <de|en> <BxH> <hell|dunkel>
#   lauf.sh grafik <de|en> <BxH> <hell|dunkel>        (0.1.1, QD-169)
#   lauf.sh anmeldeseite <N|E> <de|en> <BxH> <hell|dunkel> <standard|linux>
# Gestartet wird die Ablage-Kopie des Release-Baus (BIN=… überschreibt).
# Ergebnis unter nachweis/<art>/<name>/. Sprache wie live: LANG=en_US.UTF-8,
# LANGUAGE=de bzw. en.
set -euo pipefail
HIER=$(cd "$(dirname "$0")" && pwd)
R=$(cd "$HIER/.." && pwd)
ART=${1:?funktionen|anmeldeseite}; shift
case $ART in
  funktionen|grafik) STUFE=L; SPR=${1:?}; RES=${2:?}; SCHEMA=${3:?}; KENNUNG=standard; NAME=$SPR-$RES-$SCHEMA ;;
  anmeldeseite) STUFE=${1:?}; SPR=${2:?}; RES=${3:?}; SCHEMA=${4:?}; KENNUNG=${5:?}
                NAME=$STUFE-$KENNUNG-$SPR-$RES-$SCHEMA${NAMENSZUSATZ:+-$NAMENSZUSATZ} ;;
  *) echo "lauf.sh funktionen|grafik|anmeldeseite …" >&2; exit 2 ;;
esac
BIN=${BIN:-$R/target/remap/release/querencia-nachrichten}
[ -x "$BIN" ] || { echo "Release-Bau fehlt: $BIN (werkzeug/bauen.sh release)" >&2; exit 2; }
[ -x "$HIER/bin/qeingabe" ] || bash "$HIER/werkzeug-bauen.sh" >/dev/null
OUT=$R/nachweis/$ART/$NAME
P=$R/gui-test/$ART-$NAME
[ -e "$P" ] && { chmod -R u+rwX "$P"; rm -rf -- "$P"; }
rm -rf -- "$OUT"; mkdir -p "$OUT" "$P"
# Fenster: auf 1920×1080 groß, auf 1024×768 die Vorgabegröße 1000×700
case $RES in 1920x1080) FENSTER=1400x900 ;; *) FENSTER=1000x700 ;; esac
bash "$HIER/profil.sh" "$P" "$BIN" "$SCHEMA" "$KENNUNG" "$FENSTER" >/dev/null
cat >"$P/env.sh" <<X
BIN=\$(ls "$P"/opt/bin/querencia-nachrichten)
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
LESEN=(--lesen "$(readlink -f "$HOME/.local/share/themes/Querencia-Klassisch")/.."
       --lesen "$HOME/.local/share/icons/Querencia")
bash "$HIER/huelle.sh" --profil "$P" --labwc --sprache en_US.UTF-8 --setenv "LANGUAGE=$SPR" \
  "${LESEN[@]}" --schreiben "$OUT" --zeitlimit "${ZEIT:-600}" --protokoll "$OUT/huelle.txt" \
  "$STUFE" -- /bin/bash "$HIER/szenario-$ART.sh"
