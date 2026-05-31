# scx-synse-manager

A friendly GTK4 / libadwaita app for managing `sched-ext` schedulers via
`scx_loader`.

## Highlights

- Plain-language descriptions for every supported scheduler.
- "Recommended" badge based on detected hardware (battery, GPU, core count).
- **One password prompt per session**: a long-lived privileged helper does all
  the work so you authenticate once, not once per change.
- Unsupported-kernel guard rail explains how to install a kernel with
  `CONFIG_SCHED_CLASS_EXT=y`.

## Requirements

- Linux kernel with sched_ext support (`CONFIG_SCHED_CLASS_EXT=y`).
- `scx_loader` (>= 1.1.1) and `scx-scheds` installed and running.
- `polkit`, `libadwaita >= 1.5`, `gtk4 >= 4.12`.

## Build from source

```sh
meson setup builddir --prefix=/usr/local
meson compile -C builddir
sudo meson install -C builddir
```

## Run

```sh
scx-synse-manager
```

## License

GPL-2.0-or-later.
