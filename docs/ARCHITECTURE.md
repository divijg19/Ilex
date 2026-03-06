# corefetch Foundation Architecture

This document defines the contract at `0.1.0`.

The goal is still narrow: preserve the one-way detector to module to renderer pipeline while extending the foundation with the first core hardware domains.

## Contract Version

- Foundation contract version: `foundation-v1`
- Release scope: `0.1.0`
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

## Snapshot Contract

Current snapshot fields:

```rust
pub struct SystemSnapshot {
    pub os: Option<OsInfo>,
    pub cpu: Option<CpuInfo>,
    pub memory: Option<MemoryInfo>,
}
```

Rules:

- Snapshot fields are optional so failed or unsupported detectors do not force a process crash.
- Snapshot expansion beyond `0.1.0` should add fields rather than replacing the top-level shape.
- The foundation phase supports operating system, CPU, and memory information.

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

## Failure Semantics

- Detection failures are recorded as issues instead of aborting the process.
- Missing snapshot data results in missing module entries, not a process panic.
- Readiness fails if any of the foundation domains are not renderable or if detection issues are present.

## Readiness Gate

`0.1.0` is considered complete only when all of the following are true:

- Canonical command remains `corefetch`
- Detector registry includes `os`, `cpu`, and `memory`
- Module registry includes `os`, `cpu`, and `memory`
- Renderer registry includes `bootstrap-text`
- The snapshot produces renderable `os`, `cpu`, and `memory` module entries
- Detection issue count is zero in the happy path

The runtime binary prints these checks so the foundation gate can be inspected without reading the source.

## Explicitly Deferred Beyond `0.1.0`

- Configuration parsing
- Structured JSON output
- Multiple renderers
- Plugin or extension loading
- Cross-platform support

Later releases should build on this contract instead of reopening the pipeline design.