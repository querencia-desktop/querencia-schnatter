#!/usr/bin/env python3
"""Attrappe für org.freedesktop.Notifications auf dem privaten Bus der
Prüfhülle (Muster: Machbarkeitsprobe Querencia Nachrichten).

Protokolliert jeden Notify-Aufruf als JSON-Zeile und meldet eine Sekunde
später einen Klick (ActionInvoked). Welche Aktion geklickt wird, sagen
Regeln „Teil von Überschrift oder Text=Beschriftung“ (erste passende gilt;
Beschriftung „default“ ist der Klick auf die Meldung selbst, „-“ heißt
nicht klicken); ohne passende Regel „default“.

Aufruf: attrappe_benachrichtigungen.py <protokoll> [REGEL …]
Läuft nur auf einem privaten Bus (verweigert /run/user/…).
"""
import json
import os
import sys

from gi.repository import Gio, GLib

if "/run/user/" in os.environ.get("DBUS_SESSION_BUS_ADDRESS", ""):
    sys.exit("verweigert: nur auf dem privaten Bus der Prüfhülle")

XML = """
<node><interface name="org.freedesktop.Notifications">
 <method name="Notify">
  <arg type="s" direction="in"/><arg type="u" direction="in"/><arg type="s" direction="in"/>
  <arg type="s" direction="in"/><arg type="s" direction="in"/><arg type="as" direction="in"/>
  <arg type="a{sv}" direction="in"/><arg type="i" direction="in"/><arg type="u" direction="out"/>
 </method>
 <method name="CloseNotification"><arg type="u" direction="in"/></method>
 <method name="GetCapabilities"><arg type="as" direction="out"/></method>
 <method name="GetServerInformation">
  <arg type="s" direction="out"/><arg type="s" direction="out"/>
  <arg type="s" direction="out"/><arg type="s" direction="out"/>
 </method>
 <signal name="ActionInvoked"><arg type="u"/><arg type="s"/></signal>
 <signal name="NotificationClosed"><arg type="u"/><arg type="u"/></signal>
</interface></node>
"""

protokoll = open(sys.argv[1], "a", buffering=1)
regeln = [r.split("=", 1) for r in sys.argv[2:] if "=" in r]
zaehler = [0]


def schreibe(d):
    protokoll.write(json.dumps(d, ensure_ascii=False, default=str) + "\n")


def klick_fuer(summary, body, actions):
    """Schlüssel der zu klickenden Aktion oder None."""
    for teil, beschriftung in regeln:
        if teil in summary + "\n" + body:
            if beschriftung == "-":
                return None
            if beschriftung == "default":
                return "default"
            for i in range(0, len(actions) - 1, 2):
                if actions[i + 1] == beschriftung:
                    return actions[i]
            return None
    return "default"


def aufruf(conn, sender, path, iface, method, params, inv):
    if method == "Notify":
        zaehler[0] += 1
        nid = zaehler[0]
        p = params.unpack()
        hints = {k: (v if not isinstance(v, (bytes, bytearray)) else "<%d Bytes>" % len(v))
                 for k, v in p[6].items()}
        if "image-data" in hints:
            hints["image-data"] = "<Bild %dx%d>" % (p[6]["image-data"][0], p[6]["image-data"][1])
        schreibe({"methode": "Notify", "id": nid, "app_name": p[0], "replaces": p[1],
                  "icon": p[2], "summary": p[3], "body": p[4], "actions": p[5],
                  "hints": hints, "timeout": p[7], "sender": sender})
        inv.return_value(GLib.Variant("(u)", (nid,)))
        schluessel = klick_fuer(p[3], p[4], p[5])
        if schluessel is None:
            return

        def klick():
            schreibe({"signal": "ActionInvoked", "id": nid, "action": schluessel})
            conn.emit_signal(sender, "/org/freedesktop/Notifications",
                             "org.freedesktop.Notifications", "ActionInvoked",
                             GLib.Variant("(us)", (nid, schluessel)))
            return False
        GLib.timeout_add(1000, klick)
    elif method == "GetCapabilities":
        schreibe({"methode": method})
        inv.return_value(GLib.Variant("(as)", (["body", "actions", "icon-static",
                                                "persistence"],)))
    elif method == "GetServerInformation":
        schreibe({"methode": method})
        inv.return_value(GLib.Variant("(ssss)", ("attrappe", "querencia-pruefhuelle", "0", "1.2")))
    else:
        schreibe({"methode": method, "params": str(params)})
        inv.return_value(None)


def bus_da(conn, _name):
    info = Gio.DBusNodeInfo.new_for_xml(XML).interfaces[0]
    conn.register_object("/org/freedesktop/Notifications", info, aufruf, None, None)


Gio.bus_own_name(Gio.BusType.SESSION, "org.freedesktop.Notifications",
                 Gio.BusNameOwnerFlags.NONE, bus_da, lambda *a: schreibe({"name": "erhalten"}),
                 lambda *a: schreibe({"name": "verloren"}))
loop = GLib.MainLoop()
GLib.timeout_add_seconds(600, lambda: loop.quit() or False)
loop.run()
