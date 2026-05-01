<div align="center">

```
 ◈ SysWatch
```

**A high-performance, near-zero-footprint Linux system monitor TUI — written in Rust.**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![Release](https://img.shields.io/github/v/release/gurjarchetan/syswatch)](https://github.com/gurjarchetan/syswatch/releases)
[![Stars](https://img.shields.io/github/stars/gurjarchetan/syswatch?style=social)](https://github.com/gurjarchetan/syswatch/stargazers)

</div>

---

```
 ◈ SysWatch | chetan-pc | Ubuntu 26.04  up 02:14:07 | 14:32:01
 F1 Overview   F2 Processes   F3 Network   F4 Disk

┌ CPU ──────────────────────────────────────────────────────────────┐┌ Memory ─────────────────────────────┐
│CPU×16 ▕████████████░░░░░░░░░░░░░░░░░░▏  41.2%  13th Gen i5-13500H││RAM  [█████████████░░░░░░░]  66.5%    │
│Load avg 0.42 1m  0.51 5m  0.38 15m                                ││     9.9 GiB / 14.9 GiB (Free: 1.3G) │
│────────────────────────────────────────────────────────────────── ││Swap [░░░░░░░░░░░░░░░░░░░░]   0.0%    │
│C0  ▕████████░░░░▏  55.1%  2.4GHz │C1  ▕█████████░░░░▏  62.3%  2.4GHz│└─────────────────────────────────────┘
│C2  ▕███████░░░░░▏  48.0%  2.1GHz │C3  ▕██████░░░░░░▏   41.2%  1.9GHz│┌ Network ────────────────────────────┐
│────────────────────────────────────────────────────────────────── ││▲ TX  1.2 KB/s  Total: 118.6 KiB     │
│100%│                                                 ▄▅▆▇         ││▼ RX  4.7 KB/s  Total:  71.3 KiB     │
│ 50%│                                   ▂▃  ▂▁▃▄▅▅▆▇████         │└─────────────────────────────────────┘
│  0%│▁▁▁▁▁▁▁▁▁▁▁▁▁▂▁▂▁▂▃▃▃▄▃▃▄▄▄▄▅▅▅▅▅▆████████████████         │┌ Disk ───────────────────────────────┐
└───────────────────────────────────────────────────────────────────┘│[██████░░░░]  62%  468.0G  /          │
                                                                      │[░░░░░░░░░░]   1%   1.1G  /boot/efi  │
                                                                      └─────────────────────────────────────┘
 [q] Quit  [Tab] Switch tab  [↑↓] Scroll  [F4] Full disk details
```

---

## Features

| Panel | What you get |
|---|---|
| **CPU** | Per-core bars with clock frequency (`2.4GHz` · `800MHz`), global gauge, load average (1m/5m/15m), rolling history graph |
| **Memory** | Physical RAM — used / cached / free breakdown + Swap, colour-coded progress bars |
| **Disk** | `df -hT` style table — Filesystem, Type, Size, Used, Avail, Use% bar, Mounted on; real-time IOPS + throughput per mount point |
| **Network** | Per-interface RX/TX bandwidth, cumulative totals since launch, sparklines |
| **Processes** | Searchable, sortable table — PID, Name, User, CPU%, MEM%, Threads, Status (Running/Sleeping/Zombie/…) |

**Disk tab — mount-point overview**

```
 ◈ SysWatch | F1 Overview   F2 Processes   F3 Network  [F4 Disk]

┌ Disk ──────────────────────────────────────────────────────────────────────────┐
│Filesystem             Type       Size   Used  Avail                    Use% Mounted on │
│/dev/sda2              ext4       468G   134G   310G  [████████░░░░]     29%  /         │
│  r:0B/s     w:1.2M/s   riops:0   wiops:12                                              │
│/dev/sda1              vfat       511M   6.1M   505M  [░░░░░░░░░░░░]      1%  /boot/efi │
│/dev/sdb1              ext4       931G   520G   364G  [██████░░░░░░]     56%  /home     │
│  r:4.5M/s   w:0B/s     riops:8   wiops:0                                               │
│tmpfs                  tmpfs       16G   2.3G    14G  [██░░░░░░░░░░]     14%  /dev/shm  │
│/dev/nvme0n1p3         btrfs      200G    48G   152G  [███░░░░░░░░░]     24%  /var      │
└────────────────────────────────────────────────────────────────────────────────┘
 [q] Quit  [Tab] Switch tab  [↑↓] Scroll
```

Mount points are discovered automatically. The I/O sub-row (`r:` / `w:` / `riops:` / `wiops:`) only appears when a device has active throughput.

---

**Visual highlights**
- Block-character history graph (`▁▂▃▄▅▆▇█`) with Y-axis labels — instantly readable CPU trend
- Gradient colour per bar: cyan (idle) → green → yellow → red (hot)
- Load average colour-coded vs core count — turns red when system is overloaded
- Process state summary bar: **Run / Sleep / Idle / Stop / Zombie** counts at a glance
- Two-step kill: `k` arms (row highlights red) → `k` sends SIGTERM · `K` sends SIGKILL
- Mouse scroll support
- Responsive layout — adapts to any terminal width/height

---

## Installation

### Option 1 — Pre-built `.deb` (Debian / Ubuntu)

```bash
# Download the latest release
wget https://github.com/gurjarchetan/syswatch/releases/latest/download/syswatch_0.2.0_amd64.deb

# Install
sudo dpkg -i syswatch_0.2.0_amd64.deb

# Run
syswatch
```

### Option 2 — Snap package

```bash
sudo snap install syswatch
```

### Option 3 — Install script (any Linux distro)

```bash
curl -fsSL https://raw.githubusercontent.com/gurjarchetan/syswatch/main/install.sh | bash
```

This downloads the correct binary for your architecture, places it in `~/.local/bin`, and adds it to your `$PATH`.

### Option 4 — Build from source

**Prerequisites:** Rust 1.75+ ([install via rustup](https://rustup.rs))

```bash
# Clone
git clone https://github.com/gurjarchetan/syswatch.git
cd syswatch

# Run directly
cargo run --release

# Or install to ~/.cargo/bin/
cargo install --path .

# Then run from anywhere
syswatch
```

### Option 5 — Arch Linux (AUR)

```bash
yay -S syswatch-bin
# or
paru -S syswatch-bin
```

---

## Keyboard Shortcuts

| Key | Action |
|---|---|
| `F1` / `F2` / `F3` / `F4` | Switch to Overview / Processes / Network / Disk tab |
| `Tab` | Cycle through tabs |
| `↑` `↓` | Scroll / select process |
| `j` | Scroll down (vim-style) |
| `/` | Enter filter mode — type to search processes by name |
| `Esc` / `Enter` | Exit filter or cancel kill confirmation |
| `f` | Cycle sort column: CPU% → MEM% → PID → Name |
| `k` | **Arm** kill — row turns red; press `k` again to send `SIGTERM` |
| `K` | Send `SIGKILL` immediately to selected process |
| `q` | Quit |
| `Ctrl-C` | Force quit |

> Mouse scroll is supported on the process list in all terminals with mouse reporting.

---

## Uninstall

```bash
# If installed via .deb
sudo dpkg -r syswatch

# If installed via snap
sudo snap remove syswatch

# If installed via cargo
cargo uninstall syswatch

# If installed via install.sh
rm ~/.local/bin/syswatch
```

---

## Architecture

```
src/
├── main.rs                  ← tokio entry point, terminal setup, render loop
├── app.rs                   ← shared UI state (tab, sort, filter, scroll)
├── collector/               ← DATA LAYER — runs every 500 ms
│   ├── mod.rs               ← Arc<RwLock<SystemState>>, spawn_collector()
│   ├── cpu.rs               ← per-core %, global usage, 60-sample history
│   ├── memory.rs            ← RAM / Swap via sysinfo
│   ├── disk.rs              ← mount points, space, I/O
│   ├── network.rs           ← per-interface RX/TX, cumulative totals
│   └── process.rs           ← process list, sort by CPU
└── ui/                      ← RENDER LAYER — runs at ≤ 30 fps
    ├── mod.rs               ← top-level draw() dispatcher
    ├── braille.rs           ← Braille sparkline engine (U+2800 block)
    ├── theme.rs             ← colour-coded status styles
    ├── widgets/             ← reusable components
    │   ├── cpu_widget.rs
    │   ├── mem_widget.rs
    │   ├── gauge.rs
    │   ├── title_bar.rs
    │   ├── tab_bar.rs
    │   └── status_bar.rs
    └── layout/              ← per-tab responsive grid layouts
        ├── overview.rs
        ├── processes.rs
        ├── network.rs
        └── disk.rs
input/
└── mod.rs                   ← async crossterm keyboard + mouse event loop
```

### Design principles

| Principle | Implementation |
|---|---|
| **Near-zero CPU overhead** | 500 ms data sampling · ≤ 30 fps render · non-blocking 50 ms event poll |
| **Strict layer separation** | `collector` and `ui` share state only via `Arc<RwLock<SystemState>>` |
| **No blocking** | All I/O and event polling is async via `tokio` |
| **Memory safe** | Written in Rust — no GC pauses, no segfaults |

---

## Building packages locally

```bash
# Install cargo-deb
cargo install cargo-deb

# Build .deb
cargo deb

# Output: target/debian/syswatch_0.2.0_amd64.deb
```

---

## License

MIT — see [LICENSE](LICENSE)
