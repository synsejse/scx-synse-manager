project('scx-synse-manager', 'rust',
  version: '0.2.0',
  license: 'GPL-2.0-or-later',
  meson_version: '>= 1.2.0',
)

cargo = find_program('cargo', required: true)
bindir = get_option('prefix') / get_option('bindir')
libexecdir = get_option('prefix') / get_option('libexecdir')
datadir = get_option('prefix') / get_option('datadir')

# Absolute install path of the privileged helper. Single source of truth: the
# GUI bakes this in (so it can pkexec the helper by absolute path) and the
# polkit policy annotates the same path so its action matches.
helper_path = libexecdir / 'scx-synse-helper'

# Build via cargo, then move outputs to meson's expected layout.
# Cargo writes to <build_root>/cargo-target/release/, but meson's custom_target
# expects outputs at <build_root>/. We chain cargo build + cp via sh -c.
cargo_target = custom_target('scx-synse-build',
  output: ['scx-synse-manager', 'scx-synse-helper'],
  command: ['sh', '-c',
            'SCX_SYNSE_HELPER_PATH=' + helper_path + ' ' +
            cargo.full_path() + ' build --release' +
            ' --manifest-path ' + meson.project_source_root() / 'Cargo.toml' +
            ' --target-dir ' + meson.project_build_root() / 'cargo-target' +
            ' && cp ' + meson.project_build_root() / 'cargo-target/release/scx-synse-manager' +
            ' ' + meson.project_build_root() / 'scx-synse-manager' +
            ' && cp ' + meson.project_build_root() / 'cargo-target/release/scx-synse-helper' +
            ' ' + meson.project_build_root() / 'scx-synse-helper'],
  build_by_default: true,
  install: true,
  install_dir: [bindir, libexecdir],
  install_mode: 'rwxr-xr-x',
)

# Desktop file
configure_file(
  input: 'data/com.synsenetwork.scx-synse-manager.desktop.in',
  output: 'com.synsenetwork.scx-synse-manager.desktop',
  configuration: configuration_data(),
  install_dir: datadir / 'applications',
)

# AppStream metainfo
configure_file(
  input: 'data/com.synsenetwork.scx-synse-manager.metainfo.xml.in',
  output: 'com.synsenetwork.scx-synse-manager.metainfo.xml',
  configuration: configuration_data(),
  install_dir: datadir / 'metainfo',
)

# Icon
install_data(
  'data/icons/hicolor/scalable/apps/com.synsenetwork.scx-synse-manager.svg',
  install_dir: datadir / 'icons' / 'hicolor' / 'scalable' / 'apps',
)

# Polkit policy — annotate the same helper_path the GUI invokes.
polkit_conf = configuration_data()
polkit_conf.set('helper_path', helper_path)
configure_file(
  input: 'data/polkit/com.synsenetwork.scx-synse-manager.policy.in',
  output: 'com.synsenetwork.scx-synse-manager.policy',
  configuration: polkit_conf,
  install_dir: datadir / 'polkit-1' / 'actions',
)
