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

┌ Disk ──────────────────────────────────────────────────────────────────────────────────┐
│Filesystem             Type       Size   Used  Avail                    Use% Mounted on │
│/dev/nvme0n1p2         ext4       468G   292G   152G  [████████░░░░]     62%  /         │
│  r:0B/s     w:1.2M/s   riops:0   wiops:12                                              │
│/dev/nvme0n1p1         vfat       1.1G   6.4M   1.1G  [░░░░░░░░░░░░]      1%  /boot/efi│
│tmpfs                  tmpfs       16G   2.3G    14G  [██░░░░░░░░░░]     14%  /dev/shm  │
│tmpfs                  tmpfs      1.5G   2.9M   1.5G  [░░░░░░░░░░░░]      1%  /run      │
│tmpfs                  tmpfs      7.5G   9.1M   7.5G  [░░░░░░░░░░░░]      1%  /tmp      │
└────────────────────────────────────────────────────────────────────────────────────────┘
 [q] Quit  [Tab] Switch tab  [↑↓] Scroll
```

Mount points are discovered automatically via `/proc/mounts`. The I/O sub-row (`r:` / `w:` / `riops:` / `wiops:`) only appears when a device has active throughput.

---

**Processes tab (F2)**

```
 ◈ SysWatch | F1 Overview  [F2 Processes]  F3 Network   F4 Disk

┌ Processes (312) ──────────────────────────────────────────────────────────────────────┐
│Tasks: 312  ●  Run:3  Sleep:289  Idle:18  Stop:0  Zombie:0                             │
│Sort: [CPU▼] [MEM] [PID] [NAME]  f=cycle  /=filter  k=kill(arm)  K=SIGKILL  Esc=cancel│
│    PID  NAME                  USER        CPU%     MEM%  ST   THR STATUS              │
│──────────────────────────────────────────────────────────────────────────────────────  │
│ 246131  code                  chetan       14.30     2.10  S     35 Sleeping           │
│ 246089  chrome                chetan        8.12     3.45  S     42 Sleeping           │
│   1823  Xorg                  root          3.90     0.88  S     11 Sleeping           │
│ 246401  syswatch              chetan        2.10     0.12  R      4 Running            │
│   9871  pulseaudio            chetan        0.80     0.20  S      5 Sleeping           │
│    912  systemd-journald      root          0.30     0.15  S      1 Sleeping           │
└───────────────────────────────────────────────────────────────────────────────────────┘
```

> Type `/` to filter by name in real time · `f` cycles sort: CPU→MEM→PID→Name · `k` to arm kill

---

**Network tab (F3)**

```
 ◈ SysWatch | F1 Overview   F2 Processes  [F3 Network]  F4 Disk

┌ Network Deep Dive ────────────────────────────────────────────────────────────────────┐
│▼ Download   4.7 KB/s  Total:  71.3 KiB                                                │
│▲ Upload     1.2 KB/s  Total: 118.6 KiB                                                │
│                                                                                        │
│RX History                              TX History                                      │
│▁▁▂▁▂▃▄▅▆▇█▇▆▅▄▃▂▃▄▅▆▇██               ▁▁▁▁▂▃▂▁▁▂▃▄▃▂▃▄▅▄▃▂▁▁▁▁▁                    │
│                                                                                        │
│  wlp2s0      RX      4.7 KB/s  TX      1.2 KB/s                                       │
│  lo          RX      0.0 B/s   TX      0.0 B/s                                        │
└───────────────────────────────────────────────────────────────────────────────────────┘
```

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
wget https://github.com/gurjarchetan/syswatch/releases/latest/download/syswatch_0.3.0_amd64.deb

# Install
sudo dpkg -i syswatch_0.3.0_amd64.deb

# Run
syswatch
```

### Option 2 — Snap package

```bash
sudo snap install syswatch
```

### Option 3 — Pre-built `.rpm` (RHEL / CentOS / Amazon Linux / Fedora)

```bash
# Download the latest release
wget https://github.com/gurjarchetan/syswatch/releases/latest/download/syswatch-0.3.0-1.x86_64.rpm

# Install (Amazon Linux / RHEL / CentOS / Fedora)
sudo rpm -i syswatch-0.3.0-1.x86_64.rpm
# or with dnf (Fedora / RHEL 8+ / Amazon Linux 2023)
sudo dnf install ./syswatch-0.3.0-1.x86_64.rpm
# or with yum (CentOS 7 / Amazon Linux 2)
sudo yum install ./syswatch-0.3.0-1.x86_64.rpm

# Run
syswatch
```

### Option 4 — Install script (any Linux distro)

Works on **all** distributions — Debian, Ubuntu, RHEL, CentOS, Amazon Linux, Fedora, Arch, Alpine, openSUSE, and more. No root required.

```bash
curl -fsSL https://raw.githubusercontent.com/gurjarchetan/syswatch/main/install.sh | bash
```

This downloads the correct binary for your architecture, places it in `~/.local/bin`, and adds it to your `$PATH`.

### Option 5 — Build from source

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

### Option 6 — Arch Linux (AUR)

```bash
yay -S syswatch-bin
# or
paru -S syswatch-bin
```

---

## CLI Usage

```
USAGE:
    syswatch [OPTIONS]

OPTIONS:
    -h, --help              Show this help message and exit
    -V, --version           Show version and exit
    -t, --tab <TAB>         Start on a specific tab
                              overview   (default)
                              processes
                              network
                              disk
    -i, --interval <MS>     Data-collection interval in ms (default: 500, min: 100)
```

**Examples**

```bash
syswatch                         # Start on Overview tab
syswatch --tab processes         # Jump straight to Processes
syswatch --tab disk              # Jump straight to Disk
syswatch --interval 250          # Refresh every 250 ms
syswatch -t network -i 1000      # Network tab, 1 s interval
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

**Debian / Ubuntu (`.deb`)**
```bash
cargo install cargo-deb
cargo deb
# Output: target/debian/syswatch_0.3.0_amd64.deb
```

**RHEL / CentOS / Amazon Linux / Fedora (`.rpm`)**
```bash
cargo install cargo-generate-rpm
cargo build --release
cargo generate-rpm
# Output: target/generate-rpm/syswatch-0.3.0-1.x86_64.rpm
```

---

## License

MIT — see [LICENSE](LICENSE)
