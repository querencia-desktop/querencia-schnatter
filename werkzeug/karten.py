#!/usr/bin/env python3
"""Prüfhülle Schnatter: welche Grafikbibliotheken die Hülle und
ihre Nachkommen geladen haben und mit welchen Grafik-Variablen sie laufen
(QD-169). Liest nur /proc/<pid>/maps und /proc/<pid>/environ, beendet
nichts.

Aufruf: karten.py <pid der hülle>
Je Prozess eine Zeile „karten <name> pid=<pid> gallium=ja|nein
vulkan_nouveau=ja|nein vulkan=<ICDs> egl_mesa=ja|nein umgebung=<die drei
Variablen des Grafik-Rückfalls aus /proc/<pid>/environ oder ->“; ein Prozess,
dessen maps nicht lesbar ist, steht mit „nicht lesbar“ da.
"""
import os
import re
import sys


def eltern(pid):
    try:
        with open(f"/proc/{pid}/stat") as f:
            return int(f.read().rsplit(")", 1)[1].split()[1])
    except (OSError, ValueError, IndexError):
        return None


def name(pid):
    try:
        with open(f"/proc/{pid}/comm") as f:
            return f.read().strip()
    except OSError:
        return "?"


VARIABLEN = ("LIBGL_ALWAYS_SOFTWARE", "MESA_LOADER_DRIVER_OVERRIDE", "GALLIUM_DRIVER")


def umgebung(pid):
    try:
        with open(f"/proc/{pid}/environ", "rb") as f:
            teile = f.read().split(b"\0")
    except OSError as e:
        return f"nicht-lesbar({e.strerror})"
    werte = [t.decode(errors="replace") for t in teile
             if t.split(b"=", 1)[0].decode(errors="replace") in VARIABLEN]
    return ",".join(sorted(werte)) or "-"


def main():
    wurzel = int(sys.argv[1])
    kinder = {}
    for n in os.listdir("/proc"):
        if n.isdigit():
            e = eltern(int(n))
            if e is not None:
                kinder.setdefault(e, []).append(int(n))
    alle, offen = [], [wurzel]
    while offen:
        p = offen.pop(0)
        alle.append(p)
        offen.extend(sorted(kinder.get(p, [])))
    for p in alle:
        try:
            with open(f"/proc/{p}/maps") as f:
                libs = {z.split()[-1] for z in f if "/" in z}
        except OSError as e:
            print(f"karten {name(p)} pid={p} nicht lesbar ({e.strerror})")
            continue
        basis = {os.path.basename(x) for x in libs}
        gallium = any(re.match(r"libgallium[-.]", b) for b in basis)
        nouveau = any(b.startswith("libvulkan_nouveau") for b in basis)
        icds = sorted({re.sub(r"\.so.*$", "", b) for b in basis
                       if b.startswith("libvulkan_") or b == "libvulkan.so.1"})
        egl = any(b.startswith("libEGL_mesa") for b in basis)
        print(f"karten {name(p)} pid={p} gallium={'ja' if gallium else 'nein'} "
              f"vulkan_nouveau={'ja' if nouveau else 'nein'} "
              f"vulkan={','.join(icds) or '-'} egl_mesa={'ja' if egl else 'nein'} "
              f"umgebung={umgebung(p)}")


if __name__ == "__main__":
    main()
