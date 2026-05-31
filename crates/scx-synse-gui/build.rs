fn main() {
    // Rebuild when the meson-provided helper path changes so the absolute
    // pkexec target baked into helper_client.rs stays in sync.
    println!("cargo:rerun-if-env-changed=SCX_SYNSE_HELPER_PATH");

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let resources_dir = format!("{manifest_dir}/../../data/resources");
    // Second source dir so the gresource can pull in the app icon, which lives
    // under data/icons (installed separately into hicolor by meson).
    let icons_dir = format!("{manifest_dir}/../../data/icons");
    glib_build_tools::compile_resources(
        &[&resources_dir, &icons_dir],
        &format!("{resources_dir}/resources.gresource.xml"),
        "scx-synse-manager.gresource",
    );
}
