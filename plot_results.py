#!/usr/bin/env python3
"""Plot WBLNA optimizer results from JSON output."""

import json
import sys

import matplotlib.pyplot as plt
import numpy as np


def main():
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} results.json", file=sys.stderr)
        sys.exit(1)

    with open(sys.argv[1]) as f:
        data = json.load(f)

    params = data["params"]
    pos = np.array(data["positions_mm"])
    widths = np.array(data["widths_mm"])
    freqs = np.array(data["freqs_ghz"])
    rl = np.array(data["return_loss_db"])
    te = np.array(data["te_k"])
    te_min = np.array(data["te_amp_min_k"])
    te_imn = np.array(data["te_imn_k"])

    fig = plt.figure(figsize=(12, 7))
    gs = fig.add_gridspec(2, 2, height_ratios=[1, 0.6], hspace=0.35, wspace=0.3)

    # Top-left: Return loss
    ax = fig.add_subplot(gs[0, 0])
    ax.plot(freqs, rl, linewidth=1.2)
    ax.axhline(params["gamma_max_db"], color="r", linestyle="--", linewidth=0.8,
               label=f"Constraint ({params['gamma_max_db']} dB)")
    ax.set_xlabel("Frequency (GHz)")
    ax.set_ylabel("Return Loss (dB)")
    ax.set_title("Return Loss")
    ax.legend(fontsize=8)

    # Top-right: Noise temperature
    ax = fig.add_subplot(gs[0, 1])
    ax.plot(freqs, te, linewidth=1.2, label=f"Total (mean={data['mean_te_k']:.1f} K)")
    ax.plot(freqs, te_min, linewidth=1.0, linestyle="--", label="T_min (amplifier)")
    ax.plot(freqs, te_imn, linewidth=1.0, linestyle=":", label="T_e IMN")
    ax.set_xlabel("Frequency (GHz)")
    ax.set_ylabel("Noise Temperature (K)")
    ax.set_title("Noise Temperature")
    ax.legend(fontsize=8)

    # Bottom: Physical strip shape spanning full width
    ax = fig.add_subplot(gs[1, :])
    delta = pos[1] - pos[0]
    xs = []
    y_top = []
    y_bot = []
    for p, w in zip(pos, widths):
        xs.extend([p, p + delta])
        y_top.extend([w / 2, w / 2])
        y_bot.extend([-w / 2, -w / 2])
    xs = np.array(xs)
    y_top = np.array(y_top)
    y_bot = np.array(y_bot)
    ax.fill_between(xs, y_bot, y_top, color="C0", alpha=0.85, edgecolor="C0", linewidth=0.5)
    ax.set_xlabel("Position (mm)")
    ax.set_ylabel("Width (mm)")
    ax.set_title("Strip Profile")
    ax.set_aspect("equal")
    ax.set_xlim(xs[0], xs[-1])

    plt.show()


if __name__ == "__main__":
    main()
