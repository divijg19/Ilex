# corefetch Roadmap

This document defines the incremental implementation plan for corefetch.

The goal is to keep each release narrow, measurable, and intentionally scoped so the project can reach a stable 1.0 without collapsing under premature plugin, telemetry, or cross-platform complexity.

## Versioning Model

corefetch should use scoped pre-1.0 semantic versioning.

- `0.y.0`: a new implementation scope with new user-visible capability or internal contract expansion
- `0.y.z`: stabilization release for the current scope, limited to fixes, documentation, performance, and packaging improvements
- `1.0.0`: first release with stable core contracts, stable JSON output, stable config behavior, and a supportable Linux baseline

Rules for version increments:

- Increase the minor version only when the project adds a new planned scope.
- Increase the patch version only for bug fixes, parser hardening, performance work, test improvements, and packaging work that does not expand the planned feature surface.
- Do not introduce experimental subsystems in patch releases.
- Do not promise runtime plugins, cross-platform support, or live telemetry before the core snapshot pipeline is stable.

## Release Principles

- Linux-first, modern Linux only
- Rust-first implementation until benchmarks justify Zig
- No subprocess-heavy detection in the baseline path
- Typed detector output before renderer or plugin expansion
- Structured JSON output treated as a first-class product feature
- Each release must define explicit out-of-scope items

## Scope Summary

| Version | Theme | Primary Outcome |
| --- | --- | --- |
| `0.0.1` | Bootstrap | Compiling Rust scaffold with alias-aware entrypoint wiring |
| `0.0.2` | First Pipeline | Typed snapshot with one real OS detection flow |
| `0.0.3` | Hardening | Fixture-backed tests, CI checks, and timing hooks |
| `0.0.4` | Contract Freeze | Documented interfaces and a readiness gate for 0.1.0 |
| `0.1.0` | Foundations | Core architecture, CLI entrypoints, typed snapshot model |
| `0.1.1` | Foundation Stabilization | CPU and memory parser fallback hardening |
| `0.1.2` | Output Consistency | Shared formatting utilities for module output |
| `0.1.3` | Parser Hardening | Split parser module and malformed fixture coverage |
| `0.1.4` | Detector Structure | Per-domain detector implementation files |
| `0.2.0` | Baseline Fetch | Useful default fetch output for daily use on Linux |
| `0.2.1` | Environment Context | Shell and terminal reporting on top of the baseline fetch path |
| `0.2.2` | Config Slice | Initial TOML config loading, module toggles, and ordering |
| `0.2.3` | Baseline Closeout | Integration coverage, CI expansion, and documentation alignment |
| `0.3.0` | Hardware Expansion | Broader Linux detection coverage and stronger renderer behavior |
| `0.4.0` | Stable Structured Output | Versioned JSON schema and config validation |
| `0.5.0` | Layout and UX | Higher-quality terminal presentation without changing core architecture |
| `0.6.0` | Extension Surface | Safe extensibility model with strict boundaries |
| `0.7.0` | Packaging and Hardening | Distribution readiness, benchmarks, and operational polish |
| `1.0.0` | Stable Core | Locked baseline contracts and production-ready Linux release |

## `0.0.1` Bootstrap

Objective: create a compiling project scaffold that proves the repository layout and command entrypoint wiring.

Planned scope:

- Initialize the Rust package and binary target.
- Create the top-level `cli`, `config`, `detectors`, `modules`, and `render` modules.
- Support alias-aware invocation for `corefetch`, `core`, `cf`, and `ilex` when the binary is invoked through those names.
- Add placeholder registries so the binary exercises the architecture shape without implementing real system detection.
- Emit a bootstrap status output suitable for manual verification.

Out of scope:

- Real detector implementations
- Configuration file parsing
- JSON output
- Terminal UX work
- Performance instrumentation
- CI and packaging

Exit criteria:

- `cargo build` succeeds.
- Running the binary prints bootstrap metadata instead of panicking.
- Invoking the binary through alias symlinks reports the alias name correctly.

## `0.0.2` First Pipeline

Objective: replace the placeholder detection path with one real typed detector to module to renderer flow.

Planned scope:

- Add the initial typed system snapshot model.
- Implement Linux OS detection using `/etc/os-release`.
- Add an OS module that formats the detected operating system value.
- Render real module output from the snapshot instead of placeholder-only metadata.
- Add unit tests for OS release parsing and module consumption.

Out of scope:

- CPU and memory detection
- Config file parsing
- JSON output
- Performance instrumentation
- CI and packaging

Exit criteria:

- `cargo test` passes with detector and module tests.
- `cargo run` prints a real OS line from the detected snapshot.
- The binary still reports the invocation alias correctly.

## `0.0.3` Hardening

Objective: harden the first real pipeline with fixture-driven tests, CI checks, and basic timing visibility.

Planned scope:

- Add fixture files for os-release parsing coverage.
- Expand detector tests to validate fixture parsing, fallback behavior, and failure reporting.
- Record total detection timing and per-detector timing.
- Render timing information to make the baseline measurable during development.
- Add a CI workflow that runs formatting, tests, and build checks.

Out of scope:

- New user-facing detectors
- Config file parsing
- JSON output
- Release packaging
- Benchmark comparison infrastructure

Exit criteria:

- `cargo fmt --check`, `cargo test`, and `cargo build` pass locally and in CI.
- `cargo run` prints timing lines in addition to the OS module output.
- Fixture-backed tests cover both the happy path and a fallback parsing case.

## `0.0.4` Contract Freeze

Objective: freeze the bootstrap interfaces and make readiness for `0.1.0` explicit.

Planned scope:

- Document the detector, module, and renderer contracts as they exist at the end of `v0.0.x`.
- Add a machine-visible bootstrap contract version.
- Compute a readiness report from the live bootstrap pipeline.
- Render readiness checks so the binary exposes whether the bootstrap gate is satisfied.
- Verify the readiness gate in CI.

Out of scope:

- New detectors or renderers
- Config file parsing
- JSON output
- Packaging or release automation
- Expansion into CPU or memory detection

Exit criteria:

- The architecture contract is documented in the repository.
- `cargo run` prints both the contract version and a readiness result.
- CI verifies the primary entrypoint and readiness gate.

## `0.1.0` Foundations

Objective: establish the internal architecture and minimal executable path.

Planned scope:

- Create the Rust project structure for `cli`, `config`, `detectors`, `modules`, and `render`.
- Support the `core`, `cf`, and `ilex` command entrypoints.
- Define the typed system snapshot model consumed by modules.
- Implement detector interfaces and internal error types.
- Add initial detectors for `os`, `cpu`, and `memory`.
- Add a minimal text renderer for debugging and early testing.
- Add fixture-based tests for `/proc` and `/sys` parsing where practical.
- Establish performance measurement hooks for startup and per-detector timing.

Out of scope:

- GPU detection
- Runtime plugins
- Zig integration
- Bar graphs
- Watch mode
- Broad configuration surface

Exit criteria:

- The binary runs successfully on a modern Linux system.
- The detector to module to renderer pipeline is implemented end to end.
- Core internal types are documented well enough to support the next scope.

Implementation status:

- `os`, `cpu`, and `memory` detectors are implemented for Linux.
- Matching modules are rendered through the existing bootstrap-text renderer.
- Foundation readiness is exposed at runtime through the `foundation-v1` contract.
- CPU parsing includes fallback handling for non-x86 model keys and `cpu cores` fallback.
- Memory parsing falls back from `MemAvailable` to `MemFree` when needed.
- Module output formatting now uses shared utilities for memory and core-count display.
- Detector parser logic is separated into a dedicated parser submodule.
- Malformed fixture coverage now asserts missing-field vs parse error behavior.
- Detector implementations are now split by domain file for easier maintenance.

## `0.2.0` Baseline Fetch

Objective: ship the first genuinely useful daily-driver release.

Planned scope:

- Add a primary-filesystem `disk` detector.
- Add a default text renderer for standard fetch output.
- Add a `--minimal` mode.
- Add a `--json` mode backed by the same snapshot pipeline.
- Add a machine-visible baseline readiness gate while preserving `foundation-v1`.
- Add graceful fallback behavior for missing detector data.

Out of scope:

- `shell` and `terminal` detection
- TOML config loading
- Module enable and disable toggles
- Simple module ordering
- Runtime plugin loading
- Advanced layout presets
- Network telemetry
- Cross-platform support

Exit criteria:

- A user can install the binary, run `core`, and get useful system output without extra configuration.
- JSON and human-readable output are both generated from the same internal snapshot.
- Performance remains within the target budget for a baseline Linux run.

Implementation status:

- Default fetch rendering is now the standard output path.
- `--minimal` and `--json` are now parsed and routed through the same snapshot pipeline.
- The Linux detector set now includes a primary-filesystem `disk` detector.
- Baseline readiness is exposed through the new `baseline-v1` contract while `foundation-v1` remains intact.

## `0.2.1` Environment Context

Objective: widen the baseline fetch output with immediate user environment context.

Planned scope:

- Add `shell` detection without introducing subprocesses.
- Add `terminal` detection from deterministic environment-first sources.
- Add matching modules and stable renderer ordering in text and JSON output.
- Add fixture or deterministic env-driven coverage for both domains.

Out of scope:

- Config-driven module ordering
- Terminal capability probing beyond basic identity
- Advanced renderer layouts

Exit criteria:

- Default and minimal output include `shell` and `terminal` when available.
- JSON output includes the same fields without special-case logic.

## `0.2.2` Config Slice

Objective: add the first real configuration surface without reopening the core probe pipeline.

Planned scope:

- Load `~/.config/corefetch/config.toml`.
- Support module enable and disable toggles.
- Support simple module ordering.
- Support output-mode defaults.
- Return actionable errors for invalid config values.

Out of scope:

- Detector execution policy changes
- Plugin loading
- Cross-platform config behavior

Exit criteria:

- Missing config files fall back cleanly to defaults.
- Config overrides output ordering and module visibility predictably.

## `0.2.3` Baseline Closeout

Objective: harden the complete `0.2.x` baseline series before moving on to broader hardware work.

Planned scope:

- Add integration-style coverage for app bootstrapping and renderer mode selection.
- Expand CI verification for default, minimal, and JSON output.
- Align README and architecture docs with the implemented baseline feature set.

Out of scope:

- New hardware domains beyond the `0.2.x` plan
- Layout-heavy renderer work

Exit criteria:

- The `0.2.x` series is covered by both unit and integration-style validation.
- Public documentation describes the current product state accurately.

## `0.3.0` Hardware Expansion

Objective: widen Linux detection coverage without breaking the architecture.

Planned scope:

- Add detectors for `gpu` and `network`.
- Improve disk and memory reporting accuracy.
- Add optional bar rendering for percentage-capable modules.
- Add terminal capability detection for color and Unicode behavior.
- Add more robust fixture coverage for hardware-dependent parsing paths.
- Introduce benchmark comparisons against representative fastfetch runs on the same machine class.

Out of scope:

- Live dashboards
- External module APIs
- Dynamic plugin ABI

Exit criteria:

- The default output covers the expected baseline fetch categories.
- Renderer fallbacks behave predictably across terminals with reduced capability.
- Detection accuracy is validated on more than one Linux distribution family.

## `0.4.0` Stable Structured Output

Objective: make corefetch reliable for automation, tooling, and downstream integrations.

Planned scope:

- Freeze the first versioned JSON schema.
- Document config keys, defaults, and validation behavior.
- Add config validation errors with actionable messages.
- Add schema compatibility tests for JSON output.
- Add explicit missing or unsupported field semantics.
- Add machine-readable metadata for module names and statuses.

Out of scope:

- Runtime code plugins
- Continuous telemetry mode
- User-defined command execution modules

Exit criteria:

- Consumers can depend on JSON output shape across patch releases.
- Config parsing failures are deterministic and documented.
- Internal modules use the same output contract instead of ad hoc formatting.

## `0.5.0` Layout and UX

Objective: improve presentation quality after the underlying contracts stabilize.

Planned scope:

- Add compact and wide layout presets.
- Improve spacing, alignment, and renderer formatting controls.
- Add optional logo placement support.
- Add adaptive layout behavior based on terminal width.
- Add richer bar styles where terminal capability permits.
- Add documentation examples for common configurations.

Out of scope:

- Real-time watch mode as a default feature
- Cross-platform renderer support guarantees
- Theme engines that bypass module contracts

Exit criteria:

- The rendered output is intentionally designed rather than just technically correct.
- Layout options do not require detector-specific branching in renderers.
- UX improvements do not regress startup cost materially.

## `0.6.0` Extension Surface

Objective: open a safe path for extensibility without destabilizing the main binary.

Planned scope:

- Define a versioned extension contract.
- Prefer a subprocess or data-interface model for first external extensions.
- Add support for a small set of optional modules such as `docker`, `git`, or developer environment inspection.
- Add extension discovery rules and failure reporting.
- Document extension compatibility and version negotiation.

Out of scope:

- In-process dynamic library plugins by default
- Unrestricted shell command execution inside config
- WASI or sandbox runtimes unless clearly justified

Exit criteria:

- External extension behavior is isolated from the stability of the main fetch path.
- Corefetch remains usable even when extensions fail or are missing.
- The extension contract is versioned independently from internal refactors.

## `0.7.0` Packaging and Hardening

Objective: make the project easy to test, package, and operate.

Planned scope:

- Add release packaging for major Linux distribution targets.
- Add shell completions and man page generation.
- Add a `--doctor` style diagnostics mode for environment and detector issues.
- Expand benchmark automation and regression thresholds.
- Add CI coverage for representative Linux environments.
- Add packaging and release documentation.

Out of scope:

- Major new detectors
- Major renderer rewrites
- Plugin model redesign

Exit criteria:

- The project can be packaged and installed cleanly.
- Release artifacts are reproducible and tested.
- Performance regressions can be detected before release.

## `1.0.0` Stable Core

Objective: declare the core Linux product stable.

Planned scope:

- Lock the baseline config behavior.
- Lock the JSON schema for the supported module set.
- Publish the supported Linux scope and known limitations.
- Finalize the default module set and default layout.
- Publish stability expectations for extensions, if included before 1.0.
- Publish performance targets and benchmark methodology.

Exit criteria:

- The core snapshot pipeline is stable and documented.
- Patch releases can preserve output and config expectations.
- The project has a clear basis for future 1.x work without reopening fundamental architecture decisions.

## Deferred Until After `1.0.0`

These items should remain explicitly deferred unless a later roadmap revision promotes them with a narrower design:

- Cross-platform support beyond Linux
- Live watch mode as a polished product feature
- Telemetry-oriented continuous monitoring
- In-process runtime plugin ABI
- Zig detector implementations without benchmark proof

## Planning Notes

- If Zig is introduced, it should happen only after a benchmark report shows a specific detector path that Rust does not handle well enough.
- If extensions are introduced, a safer first step is a versioned subprocess protocol instead of loading foreign code into the main process.
- If watch mode is revisited, it should be treated as a separate product surface with its own performance and UX requirements.
