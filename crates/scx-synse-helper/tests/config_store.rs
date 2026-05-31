use scx_synse_helper::config_store::ConfigStore;

#[test]
fn load_returns_defaults_when_file_missing() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("scx_loader.toml");
    let store = ConfigStore::open(&path).unwrap();
    // Default config has no default_sched set.
    assert!(store.config().default_sched.is_none());
}

#[test]
fn save_then_reload_round_trips() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("scx_loader.toml");

    let mut store = ConfigStore::open(&path).unwrap();
    store.set_default_sched(Some(scx_loader::SupportedSched::Bpfland));
    store.save().unwrap();

    let reloaded = ConfigStore::open(&path).unwrap();
    assert_eq!(reloaded.config().default_sched, Some(scx_loader::SupportedSched::Bpfland));
}

#[test]
fn save_is_atomic_via_tmp_rename() {
    // Pre-create a poisoned config to make sure save replaces it cleanly.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("scx_loader.toml");
    std::fs::write(&path, "garbage = ").unwrap();

    let mut store = ConfigStore::open_or_default(&path);
    store.set_default_sched(Some(scx_loader::SupportedSched::Bpfland));
    store.save().unwrap();

    let reloaded = ConfigStore::open(&path).unwrap();
    assert_eq!(reloaded.config().default_sched, Some(scx_loader::SupportedSched::Bpfland));

    // No leftover .tmp file.
    let leftover = std::fs::read_dir(dir.path()).unwrap().any(|e| {
        let name = e.unwrap().file_name();
        name.to_string_lossy().ends_with(".tmp")
    });
    assert!(!leftover, "atomic rename should leave no .tmp file behind");
}
