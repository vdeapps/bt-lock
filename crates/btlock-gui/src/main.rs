mod ui;

use gtk::prelude::*;
use libadwaita as adw;
use std::sync::{Arc, OnceLock};

use btlock_engine::new_engine;

static TOKIO: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

pub fn tokio_handle() -> &'static tokio::runtime::Runtime {
    TOKIO.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("tokio runtime")
    })
}

fn main() -> glib::ExitCode {
    tokio_handle();
    libadwaita::init().expect("libadwaita");

    let engine = new_engine();
    let engine_shutdown = Arc::clone(&engine);
    let app = adw::Application::builder()
        .application_id("com.btlock.app")
        .build();

    app.connect_activate({
        let engine = Arc::clone(&engine);
        move |app| {
            ui::build_main_window(app, engine.clone());
            let _hold = app.hold();
        }
    });

    app.connect_shutdown(move |_| {
        let _ = btlock_engine::engine_stop_monitoring(&engine_shutdown);
    });

    app.run()
}
