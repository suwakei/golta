## v0.1.0

### Highlights
- First public release of `golta` CLI and shim.
- Cross-platform builds for Linux, macOS, and Windows.

### CLI
- Install Go versions by `golta install go` or `golta install go@<version>`.
- List available remote Go versions.
- Uninstall Go versions, clearing global default when applicable and warning if pinned.
- Fetches artifacts with progress reporting and validates latest stable resolution.

### Shim (`go` launcher)
- Delegates `go` execution to the version set in project `.golta.json` or global default.
- Provides clear errors when no version is pinned or configured.
- Includes tests for pin resolution, default fallback, and override behavior.

### Tooling
- CI: fmt, clippy, build, and test on pushes/PRs to `main`.
- Release workflow builds and packages binaries for all supported OS targets and publishes GitHub Releases from tags.

### Roadmap
- Add support for additional Go toolchain utilities (e.g., `gopls` and related tools).
