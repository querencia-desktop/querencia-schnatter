#!/bin/bash
# Prüfhülle Schnatter, nach dem Prüfrahmen Querencia Dateien
# (querencia-runtime/werkzeug/dateien-pruefrahmen/huelle.sh, pruefrahmen/0.1.1,
# QD-142): gleicher Aufbau, eigene Stufen für eine Webansicht.
#
#   huelle.sh [Optionen] L|N|E -- BEFEHL [ARG …]
#
# Stufe L (lokal): bwrap mit eigenem Nutzer-, PID-, Netz- (nur lo), IPC- und
#   UTS-Namensraum; für die Prüfseite (kein Netz).
# Stufe N (Netz): wie L, aber im Netz-Namensraum des Rechners; /run bleibt
#   leer bis auf /run/systemd/resolve (Namensauflösung, nur lesend). Nur für
#   die Anmeldeseite von web.whatsapp.com.
# Stufe E (echt): ohne Namensräume, nur Umgebung, Busse, labwc und Ordner
#   isoliert (eigene Sitzung per setsid); so startet WebKit seine eigene
#   Sandbox (bwrap) wie im echten Betrieb, nicht verschachtelt. Mit Netz.
# Alle Stufen: env -i mit nur gesetzten Variablen, eigenes HOME und
#   XDG_*_HOME im Profil, XDG_RUNTIME_DIR=/tmp/qn-runtime-xxxxxx (nie
#   /tmp/qd-*: das sind die Hüllen anderer Worker), privater Session-Bus,
#   Attrappen-System-Bus. In L und N unsichtbar: das echte HOME (außer
#   --lesen/--schreiben), /run (Live-Bus, Wayland, Portale, PipeWire), /tmp.
#   Endet der Befehl, endet in L/N der PID-Namensraum mit jedem Prozess, in
#   E beendet die Hülle jeden Prozess ihrer Sitzung.
#
# Optionen:
#   --profil DIR         Profil (HOME, XDG-Ordner) hier anlegen und behalten
#   --lesen PFAD         Pfad nur lesend sichtbar machen (wiederholbar)
#   --schreiben PFAD     Pfad schreibbar sichtbar machen (wiederholbar)
#   --ordner PFAD        Arbeitsordner des Befehls (Vorgabe: HOME im Profil)
#   --setenv NAME=WERT   zusätzliche Variable (wiederholbar)
#   --sprache LOCALE     LANG (Vorgabe C.UTF-8); LANGUAGE über --setenv
#   --labwc              headless labwc 0.9.8 in der Hülle, WAYLAND_DISPLAY
#   --zeitlimit S        Abbruch nach S Sekunden (Vorgabe 600)
#   --protokoll DATEI    Hüllenprotokoll und Restbericht (JSON) dorthin
#
# Exit: Exit des Befehls; 90 Aufruffehler, 91 Aufbau gescheitert,
#   92 Isolation verletzt (Vorprüfung), 93 Zeitlimit, 95 Reste gefunden.

set -u
HIER=$(cd "$(dirname "$(readlink -f "$0")")" && pwd)
SELBST=$HIER/huelle.sh

melden() { echo "huelle: $*" >&2; }

# ---------------------------------------------------------------- innen
if [ "${1:-}" = "--_innen" ]; then
  # shellcheck disable=SC1090
  source "$2"
  P_LOG=$R/.huelle/protokoll
  status() { echo "$1" > "$R/.huelle/status"; }
  log() { echo "innen $(date +%T.%N | cut -c1-12) $*" >> "$P_LOG"; }

  # Vorprüfung: kein Live-Bus, kein Live-Wayland, kein echtes HOME; in L nur lo.
  netz=$(tail -n +3 /proc/net/dev | cut -d: -f1 | tr -d ' ' | sort | tr '\n' ' ')
  if [ "$STUFE" = L ] && [ "$netz" != "lo " ]; then status "isolation netz=$netz"; exit 92; fi
  if [ "$STUFE" != E ]; then
    for verboten in "/run/user/$HOST_UID/bus" "/run/user/$HOST_UID/wayland-0" \
                    "/run/user/$HOST_UID/pipewire-0" /run/dbus/system_bus_socket.live \
                    "$HEIM_ECHT/.ssh" "$HEIM_ECHT/.local/share/Trash" \
                    "$HEIM_ECHT/.local/share/querencia-nachrichten" "$HEIM_ECHT/.cache/querencia-nachrichten" \
                    "$HEIM_ECHT/.config/querencia-nachrichten" "$HEIM_ECHT/.local/share/querencia-webhuelle" \
                    "$HEIM_ECHT/.cache/querencia-webhuelle" "$HEIM_ECHT/.config/querencia-webhuelle" \
                    "$HEIM_ECHT/.local/share/querencia-schnatter" "$HEIM_ECHT/.cache/querencia-schnatter" \
                    "$HEIM_ECHT/.config/querencia-schnatter"; do
      if [ -e "$verboten" ]; then status "isolation sichtbar=$verboten"; exit 92; fi
    done
  fi
  case $XDG_RUNTIME_DIR:$DBUS_SESSION_BUS_ADDRESS in
    "$R":unix:path="$R"/bus) ;;
    *) status "isolation umgebung"; exit 92 ;;
  esac
  if [ "$(id -u)" != "$HOST_UID" ]; then status "isolation uid=$(id -u)"; exit 92; fi
  log "Vorprüfung ok (stufe=$STUFE netz=$netz uid=$(id -u) pid=$$ sid=$(ps -o sid= -p $$ | tr -d ' '))"

  # Startet einen Bus, wartet auf seine Adresse (Ereignis statt sleep).
  bus_starten() { # NAME KONFIG
    local fifo=$R/.huelle/$1.fifo addr
    mkfifo "$fifo"
    dbus-daemon --nofork --config-file="$2" --print-address=5 5>"$fifo" \
      </dev/null >>"$P_LOG" 2>&1 &
    if ! read -r -t 10 addr < "$fifo"; then status "aufbau bus $1"; exit 91; fi
    rm -f "$fifo"
    log "Bus $1: $addr (pid $!)"
  }
  bus_starten system "$R/.huelle/system.conf"
  bus_starten session "$R/.huelle/session.conf"

  if [ "$LABWC" = 1 ]; then
    mkdir -p "$P/config/labwc"
    [ -e "$P/config/labwc/rc.xml" ] || printf '%s\n' '<?xml version="1.0"?>' '<labwc_config>' \
      '  <core><decoration>server</decoration></core>' '</labwc_config>' > "$P/config/labwc/rc.xml"
    WLR_BACKENDS=headless WLR_RENDERER=pixman WLR_HEADLESS_OUTPUTS=1 WLR_LIBINPUT_NO_DEVICES=1 \
      XKB_DEFAULT_LAYOUT=de "$LABWC_BIN" -C "$P/config/labwc" </dev/null >>"$P/labwc.log" 2>&1 &
    lpid=$!
    for _ in $(seq 200); do
      [ -S "$R/wayland-0" ] && break
      kill -0 "$lpid" 2>/dev/null || break
      sleep 0.05
    done
    if [ ! -S "$R/wayland-0" ]; then status "aufbau labwc"; exit 91; fi
    export WAYLAND_DISPLAY=wayland-0
    log "labwc bereit (pid $lpid)"
  fi

  status "laeuft"
  log "Befehl: ${BEFEHL[*]}"
  cd "$ORDNER" || { status "aufbau ordner"; exit 91; }
  "${BEFEHL[@]}"
  rc=$?
  log "Befehl beendet: $rc"
  status "befehl $rc"

  # Geordnet beenden. L/N: alle Prozesse dieses PID-Namensraums außer 1 und
  # mir. E: alle Prozesse meiner Sitzung (setsid) und alle mit meinem
  # XDG_RUNTIME_DIR (WebKits bwrap-Kinder), außer mir.
  meine() {
    if [ "$STUFE" = E ]; then
      python3 -I "$HIER/eigene.py" "$(cut -d' ' -f6 /proc/$$/stat)" "$R"
      return
    fi
    local p
    for p in /proc/[0-9]*; do
      p=${p#/proc/}
      [ "$p" = 1 ] || [ "$p" = $$ ] || [ "$p" = "$BASHPID" ] && continue
      echo "$p"
    done
  }
  ende() { local n=0 p; for p in $(meine); do kill "-$1" "$p" 2>/dev/null && n=$((n + 1)); done; echo $n; }
  ende TERM >/dev/null
  for _ in $(seq 100); do
    [ "$(ende 0)" = 0 ] && break
    sleep 0.05
  done
  [ "$(ende 0)" = 0 ] || ende KILL >/dev/null
  log "beim Ende noch $(ende 0) Prozesse"
  exit 0
fi

# ---------------------------------------------------------------- mitte
# L/N: läuft in unshare -Urm (root im eigenen Nutzer-Namensraum, eigener
# Mount-Namensraum), dann bwrap.
if [ "${1:-}" = "--_mitte" ]; then
  # shellcheck disable=SC1090
  source "$2"
  B=(--ro-bind / / --dev /dev --proc /proc
     --tmpfs /tmp --bind "$R" "$R"
     --tmpfs /run --dir /run/dbus --dir /run/user
     --tmpfs /media --tmpfs /mnt
     --tmpfs "$HEIM_ECHT" --bind "$P" "$P" --ro-bind "$HIER" "$HIER")
  [ "$STUFE" = N ] && [ -d /run/systemd/resolve ] && B+=(--ro-bind /run/systemd/resolve /run/systemd/resolve)
  # Grafik (Mesa) wie im echten Betrieb, nur lesend; ohne Karte rendert WebKit in Software.
  [ -d /dev/dri ] && B+=(--dev-bind /dev/dri /dev/dri)
  for p in "${LESEN[@]}"; do B+=(--ro-bind "$p" "$p"); done
  for p in "${SCHREIBEN[@]}"; do B+=(--bind "$p" "$p"); done
  B+=(--unshare-user --uid "$HOST_UID" --gid "$HOST_GID"
      --unshare-pid --unshare-ipc --unshare-uts --unshare-cgroup-try
      --hostname qn-huelle --die-with-parent --new-session --chdir /)
  [ "$STUFE" = L ] && B+=(--unshare-net)
  exec /usr/bin/bwrap "${B[@]}" /usr/bin/env -i "${UMGEBUNG[@]}" /bin/bash "$SELBST" --_innen "$2"
fi

# ---------------------------------------------------------------- außen
fehler() { melden "$*"; exit 90; }
HOST_UID=$(id -u); HOST_GID=$(id -g)
HEIM_ECHT=$(getent passwd "$HOST_UID" | cut -d: -f6)
LABWC_BIN=$HEIM_ECHT/opt/labwc-0.9.8/bin/labwc
PROFIL="" ORDNER="" SPRACHE=C.UTF-8 LABWC=0 ZEITLIMIT=600 PROTOKOLL=""
LESEN=() SCHREIBEN=() SETENV=()

absolut() { realpath -e -- "$1" 2>/dev/null || fehler "Pfad fehlt: $1"; }

while [ $# -gt 0 ]; do
  case $1 in
    --profil) PROFIL=$(realpath -m -- "${2:?}"); shift 2 ;;
    --lesen) LESEN+=("$(absolut "${2:?}")"); shift 2 ;;
    --schreiben) SCHREIBEN+=("$(absolut "${2:?}")"); shift 2 ;;
    --ordner) ORDNER=$(realpath -m -- "${2:?}"); shift 2 ;;
    --setenv) [[ ${2:-} =~ ^[A-Za-z_][A-Za-z0-9_]*= ]] || fehler "--setenv NAME=WERT"; SETENV+=("$2"); shift 2 ;;
    --sprache) SPRACHE=${2:?}; shift 2 ;;
    --labwc) LABWC=1; shift ;;
    --zeitlimit) [[ ${2:-} =~ ^[0-9]+$ ]] || fehler "--zeitlimit SEKUNDEN"; ZEITLIMIT=$2; shift 2 ;;
    --protokoll) PROTOKOLL=$(realpath -m -- "${2:?}"); shift 2 ;;
    L|N|E) STUFE=$1; shift; break ;;
    -h|--help) sed -n '2,/^set -u/p' "$0" | sed '$d; s/^# \{0,1\}//'; exit 0 ;;
    *) fehler "unbekannt: $1 (Aufruf: huelle.sh [Optionen] L|N|E -- BEFEHL …)" ;;
  esac
done
[ -n "${STUFE:-}" ] || fehler "Stufe L, N oder E fehlt"
[ "${1:-}" = "--" ] || fehler "nach der Stufe folgt -- BEFEHL"
shift
[ $# -gt 0 ] || fehler "Befehl fehlt"
BEFEHL=("$@")
[ "$LABWC" = 0 ] || [ -x "$LABWC_BIN" ] || fehler "labwc fehlt: $LABWC_BIN"
for p in "${SCHREIBEN[@]}"; do
  case $p in /|/usr|/usr/*|/etc|/etc/*|"$HEIM_ECHT") fehler "--schreiben $p ist zu weit" ;; esac
done

# Profil: eigenes oder Wegwerfprofil.
if [ -n "$PROFIL" ]; then
  if [ -e "$PROFIL" ] && [ -n "$(ls -A "$PROFIL")" ] && [ ! -e "$PROFIL/.qn-profil" ]; then
    fehler "Profil $PROFIL ist nicht leer und kein Prüfhüllen-Profil"
  fi
  mkdir -p "$PROFIL"; P=$PROFIL; BEHALTEN=1
else
  mkdir -p "$HEIM_ECHT/.cache/qn-pruefhuelle"
  P=$(mktemp -d "$HEIM_ECHT/.cache/qn-pruefhuelle/p-XXXXXX"); BEHALTEN=0
fi
: > "$P/.qn-profil"
mkdir -p "$P/heim" "$P/config/glib-2.0/settings" "$P/data" "$P/cache" "$P/state"
chmod 700 "$P"
[ -n "$ORDNER" ] || ORDNER=$P/heim

# Kurzes Laufzeitverzeichnis (Unix-Sockets ≤ 108 Byte); Name „qn-runtime-“
# (Schutz in qeingabe/qausgabe), nie /tmp/qd-*.
R=$(mktemp -d /tmp/qn-runtime-XXXXXX)
mkdir -m 700 "$R/.huelle"
chmod 700 "$R"

# gsettings im Profil (keyfile): feste Schrift, Thema; wsdd aus (QD-041).
KEYFILE=$P/config/glib-2.0/settings/keyfile
if [ ! -e "$KEYFILE" ]; then
  printf '[org/gnome/system/wsdd]\ndisplay-mode=%s\n\n' "'disabled'" > "$KEYFILE"
  printf '[org/gnome/desktop/interface]\nfont-name=%s\n' "'DejaVu Sans 12'" >> "$KEYFILE"
fi

# D-Bus: Session-Bus ohne Standard-Dienstordner, Attrappen-System-Bus ohne Dienste.
mkdir -p "$R/.huelle/dienste"
bus_konfig() { # TYP LISTEN DIENSTORDNER
  cat <<EOF
<!DOCTYPE busconfig PUBLIC "-//freedesktop//DTD D-Bus Bus Configuration 1.0//EN"
 "http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd">
<busconfig>
  <type>$1</type>
  <keep_umask/>
  <listen>$2</listen>
  <auth>EXTERNAL</auth>
  <apparmor mode="disabled"/>
  ${3:+<servicedir>$3</servicedir>}
  <policy context="default">
    <allow send_destination="*" eavesdrop="true"/>
    <allow eavesdrop="true"/>
    <allow own="*"/>
  </policy>
</busconfig>
EOF
}
if [ "$STUFE" = E ]; then SYSBUS=$R/system_bus_socket; else SYSBUS=/run/dbus/system_bus_socket; fi
bus_konfig session "unix:path=$R/bus" "$R/.huelle/dienste" > "$R/.huelle/session.conf"
bus_konfig system "unix:path=$SYSBUS" "" > "$R/.huelle/system.conf"

# Umgebung: nur diese Variablen (env -i).
PFAD=/usr/local/bin:/usr/bin:/bin
UMGEBUNG=(
  "HOME=$P/heim" "USER=$(id -un)" "LOGNAME=$(id -un)" "LANG=$SPRACHE" "TERM=dumb"
  "XDG_CONFIG_HOME=$P/config" "XDG_DATA_HOME=$P/data" "XDG_CACHE_HOME=$P/cache" "XDG_STATE_HOME=$P/state"
  "XDG_RUNTIME_DIR=$R" "XDG_DATA_DIRS=$P/data:/usr/local/share:/usr/share" "XDG_CONFIG_DIRS=/etc/xdg"
  "DBUS_SESSION_BUS_ADDRESS=unix:path=$R/bus" "DBUS_SYSTEM_BUS_ADDRESS=unix:path=$SYSBUS"
  "GSETTINGS_BACKEND=keyfile" "GIO_USE_VOLUME_MONITOR=unix" "GVFS_REMOTE_VOLUME_MONITOR_IGNORE=1" "GIO_USE_VFS=local"
  "GTK_USE_PORTAL=0" "GDK_DEBUG=no-portals" "NO_AT_BRIDGE=1" "GTK_A11Y=none"
  "QN_HUELLE=$STUFE" "QN_PROFIL=$P" "TMPDIR=/tmp"
)
UMGEBUNG+=("PATH=$PFAD")
for e in "${SETENV[@]}"; do UMGEBUNG+=("$e"); done
[ "$LABWC" = 1 ] && LESEN+=("$(dirname "$(dirname "$LABWC_BIN")")")

# Sichtbarkeit des Arbeitsordners und des Befehls prüfen (L/N).
sichtbar() {
  [ "$STUFE" = E ] && return 0
  case $1 in "$HEIM_ECHT"/*|/tmp/*|/run/*) ;; *) return 0 ;; esac
  local w
  for w in "$P" "$R" "$HIER" "${LESEN[@]}" "${SCHREIBEN[@]}"; do
    case $1 in "$w"|"$w"/*) return 0 ;; esac
  done
  return 1
}
sichtbar "$ORDNER" || fehler "Arbeitsordner $ORDNER ist in der Hülle unsichtbar (--lesen/--schreiben)"
case ${BEFEHL[0]} in /*) sichtbar "${BEFEHL[0]}" || fehler "Befehl ${BEFEHL[0]} ist in der Hülle unsichtbar (--lesen)" ;; esac

{
  declare -p STUFE P R HIER SELBST HOST_UID HOST_GID HEIM_ECHT ORDNER LABWC LABWC_BIN
  declare -p LESEN SCHREIBEN UMGEBUNG BEFEHL
} > "$R/.huelle/konfig"
KONFIG=$R/.huelle/konfig
echo "aussen $(date +%T) Stufe $STUFE, Laufzeit $R, Profil $P, Befehl: ${BEFEHL[*]}" > "$R/.huelle/protokoll"

python3 -I "$HIER/reste.py" vorher > "$R/.huelle/vorher.json"
start=${EPOCHREALTIME/,/.}
if [ "$STUFE" = E ]; then
  env -i PATH=/usr/bin:/bin LANG=C.UTF-8 \
    timeout --kill-after=10 "$ZEITLIMIT" \
    /usr/bin/env -i "${UMGEBUNG[@]}" /usr/bin/setsid --wait /bin/bash "$SELBST" --_innen "$KONFIG"
else
  env -i PATH=/usr/bin:/bin LANG=C.UTF-8 \
    timeout --kill-after=10 "$ZEITLIMIT" \
    /usr/bin/unshare --user --map-root-user --mount --propagation private \
    /bin/bash "$SELBST" --_mitte "$KONFIG"
fi
rc_lauf=$?
if [ "$STUFE" = E ]; then
  # Nach Zeitlimit oder Abbruch: übrige Prozesse mit genau diesem
  # Laufzeitordner (eindeutig, nur dieser Lauf) beenden.
  for _ in 1 2 3; do
    mapfile -t uebrig < <(python3 -I "$HIER/eigene.py" - "$R")
    [ ${#uebrig[@]} = 0 ] && break
    melden "beende ${#uebrig[@]} übrige Prozesse des Laufs"
    kill -TERM "${uebrig[@]}" 2>/dev/null; sleep 1
  done
fi
dauer=$(awk -v a="$start" -v b="${EPOCHREALTIME/,/.}" 'BEGIN { printf "%.1f", b - a }')
status=$(cat "$R/.huelle/status" 2>/dev/null || echo "kein-status")

case $status in
  "befehl "*) rc=${status#befehl } ;;
  isolation*) melden "Isolation verletzt: $status"; rc=92 ;;
  *) if [ "$rc_lauf" = 124 ] || [ "$rc_lauf" = 137 ]; then melden "Zeitlimit $ZEITLIMIT s"; rc=93
     else melden "Aufbau gescheitert: $status (Exit $rc_lauf)"; rc=91; fi ;;
esac

# Aufräumen und Restprüfung (nur lesend über /proc und mountinfo).
cp "$R/.huelle/protokoll" "$P/.huelle-protokoll" 2>/dev/null
python3 -I "$HIER/reste.py" nachher --laufzeit "$R" --profil "$P" --vorher "$R/.huelle/vorher.json" \
  --json "$P/.huelle-reste.json"
rc_reste=$?
if [ -n "$PROTOKOLL" ]; then
  mkdir -p "$(dirname "$PROTOKOLL")"
  {
    cat "$R/.huelle/protokoll"
    echo "aussen Status: $status; Exit Lauf $rc_lauf; Dauer ${dauer}s"
    echo "aussen Restbericht:"; cat "$P/.huelle-reste.json"; echo
  } > "$PROTOKOLL"
fi
if [ "$rc_reste" = 0 ]; then
  chmod -R u+w -- "$R" 2>/dev/null; rm -rf -- "$R"
  if [ "$BEHALTEN" = 0 ]; then chmod -R u+w -- "$P" 2>/dev/null; rm -rf -- "$P"; fi
else
  rc=95
  melden "Reste gefunden; Laufzeit $R und Profil $P bleiben zur Untersuchung"
fi
melden "Stufe $STUFE, Befehl-Status: $status, Reste: $([ "$rc_reste" = 0 ] && echo keine || echo JA), Dauer ${dauer}s, Exit $rc"
exit "$rc"
