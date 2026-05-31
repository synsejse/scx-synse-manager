# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

A GTK4 / libadwaita desktop app for managing `sched-ext` schedulers through
`scx_loader`. The defining design constraint is **one password prompt per
session**: a single long-lived privileged helper performs all root operations,
so the user authenticates once via `pkexec` rather than once per change.

## Build, test, lint

This is a Cargo workspace (edition 2024, MSRV 1.85). Cargo is the primary
driver; Meson only wraps it for install-time asset placement.

```sh
cargo build                          # debug build of all crates
cargo build --release                # release (LTO, panic=abort, stripped)
cargo test                           # run all workspace tests
cargo test -p scx-synse-helper       # one crate
cargo test -p scx-synse-helper --test idle_timeout   # one integration test file
cargo test round_trip_ping           # tests matching a name substring
cargo clippy --all-targets
cargo fmt
```

The GUI crate links GTK4 (≥ 4.12) and libadwaita (≥ 1.5); their `-devel`
packages must be installed to compile `scx-synse-gui`. `scx-synse-ipc` and
`scx-synse-helper` have no GUI dependencies and build/test in isolation.

Meson is used for packaging only — it shells out to `cargo build --release`
and copies the two binaries into the layout it expects (see `meson.build`):

```sh
meson setup builddir --prefix=/usr/local
meson compile -C builddir
sudo meson install -C builddir
```

## Architecture

Three crates split along a **privilege boundary**:

- **`scx-synse-ipc`** — pure serde types (`Request`, `Response`, `SchedMode`),
  no I/O. Shared by both binaries. The wire format is **newline-delimited JSON
  (NDJSON)**: one `Request` per line GUI→helper, one `Response` per line back.
  `SchedMode::as_raw()` numbering (Auto=0…Server=4) must stay aligned with
  upstream `scx_loader`.

- **`scx-synse-helper`** — the privileged binary (`scx-synse-helper`,
  installed to `libexecdir`, i.e. `/usr/libexec/`). Runs as root via `pkexec`.
  Reads `Request` lines from stdin, writes `Response` lines to stdout. It is
  the *only* component that performs mutations: calling `scx_loader` over the
  system D-Bus bus (`switch_scheduler` / `stop_scheduler`) and writing
  `/etc/scx_loader.toml`. Has a 5-minute idle watchdog so an orphaned root
  process exits if the GUI dies without closing the pipe.

- **`scx-synse-gui`** — the user-facing binary (`scx-synse-manager`, installed
  to `bindir`). Runs unprivileged. Performs **read-only** work directly as the
  user: checking sched_ext kernel support via `/sys/kernel/sched_ext`
  (`system_info.rs`), and querying the running scheduler+mode and the supported
  list over D-Bus (`loader_query.rs`). All *writes* are delegated to the helper.

### The single-prompt helper flow

`helper_client.rs` is the heart of the single-prompt design. `HelperClient`
spawns `pkexec scx-synse-helper` lazily on the **first** `send()` and then
reuses that same child process for every subsequent request — one pkexec
prompt covers the whole session. The child is killed on drop (closing stdin →
helper sees EOF → exits). A pkexec exit code of 126/127 before any response is
mapped to `HelperError::AuthCanceled` (surfaced as a toast, never fatal).

### Testability via the `Executor` trait

The helper's protocol loop (`protocol.rs`) is generic over an `Executor`
trait (`executor.rs`). `RealExecutor` (`real_executor.rs`) is the production
impl that hits D-Bus and the filesystem; tests inject fake/noop executors so
the protocol, idle-timeout, and config logic can be exercised **without root,
D-Bus, or a real `/etc/scx_loader.toml`**. Similarly, `system_info::is_supported`
takes an injectable `SysfsRoot`, and `helper_client` tests drive a
`fake_helper.sh` fixture instead of pkexec. Preserve these seams when editing.

### Config persistence

`config_store.rs` owns `/etc/scx_loader.toml` and writes it atomically
(write-to-`.tmp` then `rename`). `open_or_default` deliberately swallows a
malformed file and falls back to upstream defaults so a corrupt config can
never wedge the app. `Apply` persists `default_sched`/`default_mode` so
scx_loader restores the choice on next boot; `Disable` just stops the
running scheduler and leaves the stored default in place.

### GUI composition

- `app.rs` builds the `adw::Application`, registers the embedded gresource
  bundle, and loads `style.css`.
- `window.rs` is the controller: it branches to an "unsupported kernel" view
  when `/sys/kernel/sched_ext` is absent, otherwise populates the scheduler
  list and wires the single contextual action button — "Apply" normally, or
  "Turn off" when the selection is already the running scheduler+mode. A
  1-second `glib` timer polls scx_loader to refresh the status banner.
- `scheduler_picker.rs` / `profile_picker.rs` are reusable widgets exposing a
  `selected()` getter and an `on_change` listener hook.
- `catalog.rs` maps scheduler names → human descriptions, icons, and a
  `supports_profiles` flag, with a graceful fallback for unknown schedulers so
  the UI stays useful when `scx_loader` reports something new.
- `recommend.rs` picks a "Recommended" scheduler from detected hardware
  (battery > dGPU > core-count, in priority order).

GTK layout lives in `.ui` XML templates under `data/resources/ui/`, compiled
into a gresource bundle by `crates/scx-synse-gui/build.rs` and loaded at
runtime via `Builder::from_resource`. **To change layout, edit the `.ui`
files**, not Rust widget-construction code. Widgets are pulled out by ID with
the local `object()` helper, which panics if an ID is missing from the
template — so IDs in the `.ui` and the Rust lookups must stay in sync.

## Conventions

- The privilege split is the core invariant: never add a mutating syscall,
  D-Bus write, or filesystem write to the GUI crate — route it through a new
  `Request` variant + `Executor` method instead.
- The app ID is `com.synsenetwork.scx-synse-manager` (gresource prefix
  `/com/synsenetwork/scx-synse-manager`); the binary stays `scx-synse-manager`.
- The polkit policy and the GUI's pkexec target are both generated from Meson's
  `libexecdir` (`helper_path`), so they never drift — `data/polkit/*.policy.in`
  is templated, and `SCX_SYNSE_HELPER_PATH` is baked into the GUI at build time.
