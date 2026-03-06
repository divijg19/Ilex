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
| `core` | primary CLI command |
| `cf` | minimal shorthand |
| `ilex` | internal codename alias |

Example usage:

```bash
core
````

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

Example output may resemble:

```
OS: Fedora Linux 43
Kernel: 6.x
CPU: Ryzen 7
GPU: AMD Radeon
Memory: 6.2 GiB / 32 GiB
Disk: 140 GiB / 1 TB
Shell: zsh
Terminal: Ghostty
```

Future formats may include graphical bars:

```
CPU    ███████░░░ 68%
RAM    █████░░░░░ 42%
DISK   ████████░░ 81%
```

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

Early design phase.

Initial work focuses on:

* system detection primitives
* modular architecture
* CLI ergonomics
* baseline Linux hardware detection

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
