//! Entry point for scx-synse-manager GUI.

mod app;
mod window;

fn main() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");
    let _enter = rt.enter();
    let app = app::SynseApplication::new();
    std::process::exit(app.run());
}
