# corefetch Bootstrap Architecture

This document freezes the bootstrap contract at the end of `v0.0.4`.

The goal of this document is narrow: define the interfaces and behavioral expectations that must remain stable enough to begin `0.1.0` without redesigning the current pipeline.

## Contract Version

- Bootstrap contract version: `bootstrap-v1`
- Release scope: `0.0.4`
- Platform assumption: modern Linux only
- Command model: `corefetch` is the canonical command; `core`, `cf`, and `ilex` are aliases

## Pipeline Shape

The current bootstrap pipeline is strictly one way:

1. CLI invocation resolves the canonical command and alias metadata.
2. Detectors populate a typed `SystemSnapshot`.
3. Modules convert snapshot fields into renderable module entries.
4. Renderers consume the render view and produce terminal output.

The bootstrap releases intentionally avoid reverse dependencies between these layers.

## Detector Contract

Detector trait:

```rust
pub trait Detector {
    fn key(&self) -> &'static str;
    fn detect(&self, snapshot: &mut SystemSnapshot) -> Result<(), String>;
}
```

Rules:

- A detector mutates the shared typed snapshot.
- A detector identifies itself with a stable static key.
- A detector returns a string error instead of panicking on expected runtime failures.
- Detection timing is measured per detector and for the total detection pass.

Current bootstrap detector set:

- `os` via `/etc/os-release`

## Snapshot Contract

Current snapshot fields:

```rust
pub struct SystemSnapshot {
    pub os: Option<OsInfo>,
}
```

Rules:

- Snapshot fields are optional so failed or unsupported detectors do not force a process crash.
- Snapshot expansion in `0.1.0` should add fields rather than replacing the top-level shape.
- The bootstrap phase supports exactly one real detected domain: operating system information.

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

Current bootstrap module set:

- `os`

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

Current bootstrap renderer set:

- `bootstrap-text`

## Failure Semantics

- Detection failures are recorded as issues instead of aborting the process.
- Missing snapshot data results in missing module entries, not a process panic.
- Readiness fails if the OS flow is not renderable or if detection issues are present.

## Readiness Gate

`v0.0.4` is considered ready for `0.1.0` only when all of the following are true:

- Canonical command remains `corefetch`
- Detector registry includes `os`
- Module registry includes `os`
- Renderer registry includes `bootstrap-text`
- The snapshot produces at least one renderable OS module entry
- Detection issue count is zero in the happy path

The runtime binary prints these checks so the bootstrap gate can be inspected without reading the source.

## Explicitly Deferred Beyond `v0.0.4`

- CPU and memory detection
- Configuration parsing
- Structured JSON output
- Multiple renderers
- Plugin or extension loading
- Cross-platform support

`0.1.0` should build on this contract instead of reopening the pipeline design.