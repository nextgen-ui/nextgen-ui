#!/usr/bin/env python3
"""Measure idle CPU and RSS of the running app.

Idle cost is this project's whole thesis, so any change to code that runs every
frame or every tick should be measured before and after.

    python3 tools/measure_idle.py --seconds 30
    python3 tools/measure_idle.py --match edex-ui,eDEX-UI-Linux-x   # compare

Two traps this encodes, both of which produced wrong numbers during development:

1. Match on /proc/<pid>/comm, never the command line. The project path contains
   "nextgen-ui", so matching cmdline sweeps in vite, esbuild and your own shell.
   comm is also truncated to 15 characters — "WebKitWebProcess" appears as
   "WebKitWebProces".

2. Record which window has focus. WebKit throttles heavily when its window is
   not focused, and the app's own animations throttle too, so a focused and an
   unfocused sample are not comparable. Numbers quoted without a focus state
   are meaningless.
"""
import argparse, glob, os, time

HZ = os.sysconf("SC_CLK_TCK")
DEFAULT_MATCH = "nextgen-ui,WebKitWebProces,WebKitNetworkPr"


def find(patterns):
    out = {}
    for d in glob.glob("/proc/[0-9]*"):
        pid = d.rsplit("/", 1)[-1]
        try:
            comm = open(f"{d}/comm").read().strip()
        except OSError:
            continue
        if comm in patterns:
            out[pid] = comm
    return out


def cpu_ticks(pid):
    try:
        s = open(f"/proc/{pid}/stat").read()
        parts = s[s.rindex(")") + 2:].split()
        return int(parts[11]) + int(parts[12])      # utime + stime
    except (OSError, ValueError, IndexError):
        return None


def rss_kb(pid):
    try:
        for line in open(f"/proc/{pid}/status"):
            if line.startswith("VmRSS:"):
                return int(line.split()[1])
    except OSError:
        pass
    return 0


def focused_window():
    try:
        from Xlib import display, X
        d = display.Display()
        prop = d.screen().root.get_full_property(
            d.intern_atom("_NET_ACTIVE_WINDOW"), X.AnyPropertyType)
        if not prop or not prop.value[0]:
            return "?"
        w = d.create_resource_object("window", prop.value[0])
        # get_wm_name() often returns '' for modern clients; _NET_WM_NAME is set.
        name = w.get_full_property(d.intern_atom("_NET_WM_NAME"), X.AnyPropertyType)
        if name and name.value:
            return name.value.decode("utf8", "replace")
        return w.get_wm_name() or "?"
    except Exception as exc:
        return f"?({exc})"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--seconds", type=int, default=30)
    ap.add_argument("--match", default=DEFAULT_MATCH,
                    help="comma-separated /proc comm names (15 chars max each)")
    ap.add_argument("--label", default="idle")
    args = ap.parse_args()

    patterns = set(args.match.split(","))
    procs = find(patterns)
    if not procs:
        raise SystemExit(f"no processes matching {sorted(patterns)}")

    focus_start = focused_window()
    t0 = time.time()
    start = {p: cpu_ticks(p) for p in procs}
    samples = []
    while time.time() - t0 < args.seconds:
        time.sleep(2.0)
        samples.append(sum(rss_kb(p) for p in procs))
    elapsed = time.time() - t0

    per, total = {}, 0.0
    for pid, comm in procs.items():
        a, b = start[pid], cpu_ticks(pid)
        if a is None or b is None:
            continue
        pct = (b - a) / HZ / elapsed * 100
        per[comm] = per.get(comm, 0.0) + pct
        total += pct

    rss = sum(samples) / len(samples) / 1024 if samples else 0
    print(f"\n=== {args.label} ({elapsed:.0f}s) ===")
    print(f"  focus: {focus_start} -> {focused_window()}")
    for comm, pct in sorted(per.items(), key=lambda kv: -kv[1]):
        print(f"  {comm:<22} {pct:6.2f}% CPU")
    print(f"  {'-' * 22} {total:6.2f}% CPU total")
    print(f"  RSS mean over {len(samples)} samples: {rss:.0f} MB")


if __name__ == "__main__":
    main()
