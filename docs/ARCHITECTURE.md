# corefetch Foundation Architecture

This document defines the current stable internal contracts through `0.2.2`.

The goal remains narrow: preserve the one-way detector to module to renderer pipeline while adding the first user-facing baseline fetch output on top of the original foundation.

## Contract Version

- Foundation contract version: `foundation-v1`
- Baseline contract version: `baseline-v1`
- Environment contract version: `environment-v1`
- Release scope: `0.2.2`
- Platform assumption: modern Linux only
- Command model: `corefetch` is the canonical command; `core`, `cf`, and `ilex` are aliases

## Pipeline Shape

The current bootstrap pipeline is strictly one way:

1. CLI invocation resolves the canonical command and alias metadata.
2. Detectors populate a typed `SystemSnapshot`.
3. Modules convert snapshot fields into renderable module entries.
4. Renderers consume the render view and produce terminal output.

The foundation release keeps the same dependency rule: later layers consume earlier layers, but not the reverse.

## Detector Contract

Detector trait:

```rust
pub trait Detector {
    fn key(&self) -> &'static str;
    fn detect(&self, snapshot: &mut SystemSnapshot) -> Result<(), DetectionError>;
}
```

Rules:

- A detector mutates the shared typed snapshot.
- A detector identifies itself with a stable static key.
- A detector returns a typed `DetectionError` instead of panicking on expected runtime failures.
- Detection timing is measured per detector and for the total detection pass.

Current foundation detector set:

- `os` via `/etc/os-release`
- `cpu` via `/proc/cpuinfo`
- `memory` via `/proc/meminfo`

Current baseline detector additions:

- `disk` via `/proc/mounts` plus filesystem statistics on the primary mount point

Current environment detector additions:

- `shell` from `$SHELL` with `/etc/passwd` fallback
- `terminal` from deterministic environment sources such as `TERM_PROGRAM` and `TERM`

## Snapshot Contract

Current snapshot fields:

```rust
pub struct SystemSnapshot {
    pub os: Option<OsInfo>,
    pub cpu: Option<CpuInfo>,
    pub memory: Option<MemoryInfo>,
    pub disk: Option<DiskInfo>,
    pub shell: Option<ShellInfo>,
    pub terminal: Option<TerminalInfo>,
}
```

Rules:

- Snapshot fields are optional so failed or unsupported detectors do not force a process crash.
- Snapshot expansion beyond `0.1.0` should add fields rather than replacing the top-level shape.
- The current environment phase supports operating system, CPU, memory, primary-filesystem disk, shell, and terminal information.

## Module Contract

Module trait:

```rust
pub trait Module {
    fn key(&self) -> &'static str;
    fn collect(&self, snapshot: &SystemSnapshot) -> Option<ModuleEntry>;
}
```

Rules:

- Modules consume typed snapshot data only.
- Modules do not perform direct probing in the bootstrap contract.
- A module may return `None` if its required snapshot field is missing.

Current foundation module set:

- `os`
- `cpu`
- `memory`

Current baseline module additions:

- `disk`

Current environment module additions:

- `shell`
- `terminal`

## Renderer Contract

Renderer trait:

```rust
pub trait Renderer {
    fn key(&self) -> &'static str;
    fn render(&self, view: &RenderView) -> String;
}
```

Rules:

- Renderers consume a render view only.
- Renderers do not probe the system or mutate detector state.
- Renderers may show bootstrap metadata, timings, issues, and readiness state.

Current foundation renderer set:

- `bootstrap-text`

Current baseline renderer additions:

- `fetch-text`
- `minimal-text`
- `json`

## Configuration Contract

Current `0.2.2` config behavior:

- Config is loaded from `~/.config/corefetch/config.toml` when present.
- Missing config falls back to defaults without changing startup behavior.
- Invalid config values fail startup with an actionable error message.
- Config affects renderer selection defaults and rendered module presentation only.
- Config does not change detector execution policy in `0.2.x`.

Currently supported config keys:

- `output.default_mode`
- `modules.order`
- `modules.enabled.<key>`

## Failure Semantics

- Detection failures are recorded as issues instead of aborting the process.
- Missing snapshot data results in missing module entries, not a process panic.
- Readiness fails if any of the foundation domains are not renderable or if detection issues are present.

## Readiness Gate

`foundation-v1` is considered complete only when all of the following are true:

- Canonical command remains `corefetch`
- Detector registry includes `os`, `cpu`, and `memory`
- Module registry includes `os`, `cpu`, and `memory`
- Renderer registry includes `bootstrap-text`
- The snapshot produces renderable `os`, `cpu`, and `memory` module entries
- Detection issue count is zero in the happy path

The runtime binary prints these checks so the foundation gate can be inspected without reading the source.

`baseline-v1` is considered complete only when all of the following are true:

- Canonical command remains `corefetch`
- Detector registry includes `os`, `cpu`, `memory`, and `disk`
- Module registry includes `os`, `cpu`, `memory`, and `disk`
- Renderer registry includes `fetch-text`, `minimal-text`, and `json`
- The snapshot produces renderable `os`, `cpu`, `memory`, and `disk` module entries
- Detection issue count is zero in the happy path

`environment-v1` is considered complete only when all of the following are true:

- Canonical command remains `corefetch`
- Detector registry includes `os`, `cpu`, `memory`, `disk`, `shell`, and `terminal`
- Module registry includes `os`, `cpu`, `memory`, `disk`, `shell`, and `terminal`
- Renderer registry includes `fetch-text`, `minimal-text`, and `json`
- The snapshot produces renderable `os`, `cpu`, `memory`, `disk`, `shell`, and `terminal` module entries
- Detection issue count is zero in the happy path

## Explicitly Deferred Beyond `0.2.2`

- Configuration parsing beyond initial `0.2.x` defaults
- Detector execution policy changes driven by config
- Plugin or extension loading
- Cross-platform support

Later releases should build on this contract instead of reopening the pipeline design.