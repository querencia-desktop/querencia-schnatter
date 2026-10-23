#!/usr/bin/env python3
"""Prüfhülle Querencia Nachrichten: Speicher (PSS) und Sandbox der Hülle und
aller ihrer Nachkommen (WebKit-Netzwerk- und Web-Prozesse, bwrap,
xdg-dbus-proxy). Liest nur /proc, beendet nichts.

Aufruf: messen.py <pid der hülle> <ausgabe.json>
Je Prozess: Name, PID, Eltern, PSS (KiB, smaps_rollup), Namensräume im
Vergleich zur Hülle (user, pid, net, mnt, ipc, uts), Seccomp, NoNewPrivs,
CapEff, NSpid. Lesefehler (z. B. ptrace-Schutz) stehen als „nicht lesbar“.
"""
import json
import os
import sys

NS = ("user", "pid", "net", "mnt", "ipc", "uts")


def lesen(pfad):
    try:
        with open(pfad) as f:
            return f.read()
    except OSError as e:
        return None if e.errno == 2 else f"nicht lesbar ({e.strerror})"


def status(pid):
    t = lesen(f"/proc/{pid}/status") or ""
    return {z.split(":", 1)[0]: z.split(":", 1)[1].strip() for z in t.splitlines() if ":" in z}


def eltern(pid):
    t = lesen(f"/proc/{pid}/stat")
    if not t or t.startswith("nicht"):
        return None
    return int(t.rsplit(")", 1)[1].split()[1])


def ns(pid):
    v = {}
    for n in NS:
        try:
            v[n] = os.readlink(f"/proc/{pid}/ns/{n}")
        except OSError as e:
            v[n] = f"nicht lesbar ({e.strerror})"
    return v


def pss(pid):
    t = lesen(f"/proc/{pid}/smaps_rollup")
    if t is None or t.startswith("nicht"):
        return t
    for z in t.splitlines():
        if z.startswith("Pss:"):
            return int(z.split()[1])
    return None


def main():
    wurzel, aus = int(sys.argv[1]), sys.argv[2]
    kinder = {}
    for name in os.listdir("/proc"):
        if name.isdigit():
            e = eltern(int(name))
            if e is not None:
                kinder.setdefault(e, []).append(int(name))
    alle, offen = [], [wurzel]
    while offen:
        p = offen.pop(0)
        alle.append(p)
        offen.extend(sorted(kinder.get(p, [])))
    ns_huelle = ns(wurzel)
    prozesse = []
    for p in alle:
        st = status(p)
        n = ns(p)
        prozesse.append({
            "pid": p, "eltern": eltern(p), "name": st.get("Name"),
            "pss_kib": pss(p),
            "eigene_namensraeume": [k for k in NS if n[k] != ns_huelle[k]],
            "seccomp": st.get("Seccomp"), "no_new_privs": st.get("NoNewPrivs"),
            "cap_eff": st.get("CapEff"), "nspid": st.get("NSpid"),
        })
    summe = sum(x["pss_kib"] for x in prozesse if isinstance(x["pss_kib"], int))
    ergebnis = {"prozesse": prozesse, "pss_summe_mib": round(summe / 1024, 1),
                "pss_lesbar": sum(isinstance(x["pss_kib"], int) for x in prozesse),
                "prozesse_gesamt": len(prozesse)}
    with open(aus, "w") as f:
        json.dump(ergebnis, f, indent=1, ensure_ascii=False)
    for x in prozesse:
        print("%-16s pid=%-7s pss=%-10s ns=%-24s seccomp=%s nnp=%s" % (
            x["name"], x["pid"], x["pss_kib"], ",".join(x["eigene_namensraeume"]) or "-",
            x["seccomp"], x["no_new_privs"]))
    print("PSS gesamt: %.1f MiB (%d von %d Prozessen lesbar)" % (
        summe / 1024, ergebnis["pss_lesbar"], len(prozesse)))


if __name__ == "__main__":
    main()
