#!/usr/bin/env python3
"""Plot WBLNA sweep results from JSON output."""
import json
import sys
import matplotlib.pyplot as plt
import numpy as np


def main():
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} results.json", file=sys.stderr)
        sys.exit(1)
    with open(sys.argv[1]) as f:
        results = json.load(f)

    n = len(results)
    colors = plt.cm.tab10.colors if n <= 10 else plt.cm.tab20.colors

    fig, (ax_prof, ax_noise) = plt.subplots(1, 2, figsize=(16, 6))

    all_te = [v for r in results for v in r["te_k"]]
    noise_ylim = (0, max(all_te) * 1.1)

    for i, r in enumerate(results):
        color = colors[i % len(colors)]
        tline = r["tline"].split("/")[-1].replace("_fit.json", "")
        length_mm = r["length_m"] * 1e3
        label = f"{tline}  L={length_mm:.0f}mm  (obj={r['obj']:.3f})"

        # Profile
        pos = np.array(r["positions_mm"])
        widths = np.array(r["widths_mm"])
        delta = pos[1] - pos[0] if len(pos) > 1 else 1.0
        xs, y_top, y_bot = [], [], []
        for p, w in zip(pos, widths):
            xs.extend([p, p + delta])
            y_top.extend([w / 2, w / 2])
            y_bot.extend([-w / 2, -w / 2])
        xs, y_top, y_bot = np.array(xs), np.array(y_top), np.array(y_bot)
        ax_prof.fill_between(xs, y_bot, y_top, color=color, alpha=0.4, lw=0)
        ax_prof.plot(xs, y_top, color=color, lw=1.0, label=label)
        ax_prof.plot(xs, y_bot, color=color, lw=1.0)

        # Noise
        freqs = np.array(r["freqs_ghz"])
        te = np.array(r["te_k"])
        ax_noise.plot(freqs, te, color=color, lw=1.2, label=label)

    ax_prof.set_xlabel("Position (mm)")
    ax_prof.set_ylabel("Width (mm)")
    ax_prof.set_title("Strip Profiles")
    ax_prof.set_ylim(-7, 7)
    ax_prof.set_aspect('equal')
    ax_prof.legend(fontsize=7, loc="upper right")

    ax_noise.set_xlabel("Frequency (GHz)")
    ax_noise.set_ylabel("T_e (K)")
    ax_noise.set_title("Noise Temperature")
    ax_noise.set_ylim(noise_ylim)
    ax_noise.legend(fontsize=7, loc="upper right")

    plt.tight_layout()
    out = sys.argv[1].replace(".json", ".png")
    plt.savefig(out, dpi=120, bbox_inches="tight")
    print(f"saved {out}")
    plt.show()


if __name__ == "__main__":
    main()
