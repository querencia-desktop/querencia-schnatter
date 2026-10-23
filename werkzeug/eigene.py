#!/usr/bin/env python3
"""Prüfhülle Querencia Nachrichten, Stufe E: PIDs der Prozesse dieses Laufs.

Aufruf: eigene.py <sitzungs-id> <laufzeitordner>
Gibt jede PID aus, deren Sitzung <sitzungs-id> ist oder deren Umgebung genau
XDG_RUNTIME_DIR=<laufzeitordner> enthält (WebKits bwrap-Kinder legen eine
eigene Sitzung an). Ausgenommen: dieses Skript und seine Vorfahren (die
Hülle). Liest nur /proc, beendet nichts.
"""
import os
import sys


def vorfahren():
    pids, pid = set(), os.getpid()
    while pid > 1 and pid not in pids:
        pids.add(pid)
        try:
            with open(f"/proc/{pid}/stat") as f:
                pid = int(f.read().rsplit(")", 1)[1].split()[1])
        except OSError:
            break
    return pids


def main():
    sid, laufzeit = sys.argv[1], sys.argv[2].encode()
    marke = b"XDG_RUNTIME_DIR=" + laufzeit
    aus = vorfahren()
    for name in os.listdir("/proc"):
        if not name.isdigit() or int(name) in aus or name == "1":
            continue
        try:
            with open(f"/proc/{name}/stat") as f:
                felder = f.read().rsplit(")", 1)[1].split()
            if felder[3] == sid:
                print(name)
                continue
            with open(f"/proc/{name}/environ", "rb") as f:
                if marke in f.read().split(b"\0"):
                    print(name)
        except OSError:
            continue


if __name__ == "__main__":
    main()
