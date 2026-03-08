# `corefetch`
#### The internal codename for `corefetch` is **Ilex**.

**`corefetch`** is a modern, fast, and modular system introspection tool for Linux.  
It provides rich system information with minimal latency, clean terminal rendering, and an architecture designed for extensibility.

`corefetch` aims to evolve the traditional "*fetch tool*" concept beyond static system summaries into a **modern system introspection layer** that is fast, composable, and developer-friendly.

---

## Overview

Fetch tools have historically been simple utilities that print system information in the terminal. While tools like `neofetch`, `fastfetch`, and `macchina` have improved performance and aesthetics, they remain largely **static system reporters**.

`corefetch` takes a different approach.

It is designed as a **modular system inspection framework** capable of:

- fast hardware detection
- structured system reporting
- extensible modules
- modern terminal UX
- developer environment introspection

The goal is not only to replicate existing fetch tools but to **expand what a fetch tool can be**.

---

## Commands

`corefetch` exposes multiple entrypoint/command aliases.

| Command | Description |
|-------|-------------|
| `corefetch` | primary CLI command |
| `core` | short alias |
| `cf` | minimal shorthand |
| `ilex` | internal codename alias |

`corefetch` remains the canonical entrypoint. `core`, `cf`, and `ilex` are aliases only.

Example usage:

```bash
corefetch
```

```bash
corefetch --minimal
```

```bash
corefetch --json
```

```bash
core
```

```bash
cf
```

```bash
ilex
```

All commands invoke `corefetch`.

---

## Philosophy

`corefetch` is designed around four principles:

### 1. Speed

Startup latency should remain in the **single-digit millisecond range**.

System probing avoids subprocesses and relies primarily on direct reads from:

```
/proc
/sys
/dev
```

### 2. Modularity

The architecture separates:

```
detectors
modules
renderers
plugins
```

This allows `corefetch` to scale without becoming monolithic.

### 3. Modern Terminal UX

`corefetch` supports modern terminal capabilities including:

* truecolor
* Unicode blocks
* compact layouts
* dynamic module formatting

Future rendering modes may include:

* bar graphs
* compact dashboards
* live watch mode

### 4. Extensibility

The long-term goal is a **plugin-friendly architecture** where additional modules can be installed without recompilation.

Possible modules include:

```
docker
git
developer environments
network telemetry
battery
sensors
```

---

## Example Output

Current baseline output resembles:

```
OS: Fedora Linux 43
CPU: Ryzen 7 7840U (16 cores)
Memory: 6.2 GiB / 32.0 GiB
Disk: 140.0 GiB / 512.0 GiB (/)
Shell: fish
Terminal: Ghostty (xterm-256color)
```

Current alternate modes:

- `--minimal` renders a compact single-line summary.
- `--json` renders machine-readable output from the same snapshot pipeline.

Future output expansion still includes:

- broader hardware coverage
- richer layout and graph modes

---

## Architecture

High-level structure:

```
corefetch
├─ cli
├─ config
├─ detectors
├─ modules
├─ render
└─ plugins
```

### Detectors

Low-level hardware probing.

Examples:

```
CPU
GPU
memory
disk
network
OS
```

### Modules

Presentation layer that formats detector results.

### Renderers

Terminal output engines.

Possible render targets:

```
ascii
unicode
json
minimal
```

### Plugins

Optional modules extending the system.

---

## Configuration

Configuration lives in:

```
~/.config/corefetch/config.toml
```

Example:

```toml
[layout]
logo = "left"

[module.cpu]
enabled = true

[module.memory]
bar = true
```

---

## Goals

`corefetch` aims to provide:

* the speed of modern fetch tools
* the clarity of a minimal CLI utility
* the extensibility of a modular system tool
* the ergonomics of modern developer tooling

---

## Status

Active implementation phase.

Current release status:

- Current version: `0.2.1`
- Canonical command: `corefetch` (`core`, `cf`, and `ilex` are aliases)
- Implemented Linux detectors: `os`, `cpu`, `memory`, `disk`, `shell`, `terminal`
- Implemented module pipeline: detector -> module -> renderer
- Output modes: default fetch, `--minimal`, `--json`
- Contracts and readiness gates: `foundation-v1`, `baseline-v1`, `environment-v1`
- Shared module formatting utilities are now in place for core-count, memory, disk, and terminal display
- Detector parsing is split into a dedicated submodule with malformed-input fixture coverage
- Detector implementations are split by domain file for maintainability
- CI checks include format, tests, build, default output, minimal output, and JSON verification

Near-term focus:

- `0.2.2` initial config loading and module ordering/toggles
- `0.2.3` baseline closeout and integration coverage

---

## Inspiration

`corefetch` draws inspiration from:

* `neofetch`
* `fastfetch`
* `macchina`

while attempting to push the concept further into **a modern system introspection platform**.

---

## License

TBD
