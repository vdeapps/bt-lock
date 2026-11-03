use crate::tokio_handle;
use btlock_bt::BtDevice;
use btlock_core::{AppConfig, DeviceKind, ProximityMethod};
use btlock_engine::{
    engine_get_config, engine_get_status, engine_save_config, engine_scan as scan_bt_devices,
    engine_select_device, engine_start_monitoring, engine_stop_monitoring, SharedEngine,
};
use gtk::prelude::*;
use gtk::{glib, Align, Box as GtkBox, Button, Label, ListBox, ListBoxRow, Orientation, Scale, SpinButton};
use libadwaita as adw;
use libadwaita::prelude::*;
use std::rc::Rc;
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;

pub struct UiHandles {
    pub status_event: Label,
    pub status_rssi: Label,
    pub status_link: Label,
    pub status_streak: Label,
    pub status_device: Label,
    pub monitor_btn: Button,
    pub device_list: ListBox,
    pub message: Label,
    pub rssi_scale: Scale,
    pub rssi_box: GtkBox,
    pub mode_row: adw::ComboRow,
    pub delay_spin: SpinButton,
    pub attempts_spin: SpinButton,
}

fn kind_label(kind: DeviceKind) -> &'static str {
    match kind {
        DeviceKind::Phone => "Téléphone",
        DeviceKind::Watch => "Montre",
        DeviceKind::Unknown => "Autre",
    }
}

fn refresh_status(engine: &SharedEngine, ui: &UiHandles) {
    let cfg = engine_get_config(engine);
    let st = engine_get_status(engine);

    ui.status_device.set_text(cfg.device.as_ref().map(|d| d.name.as_str()).unwrap_or("—"));
    ui.status_rssi.set_text(
        &st
            .last_rssi
            .map(|v| format!("{v} dBm"))
            .unwrap_or_else(|| "—".into()),
    );
    ui.status_link.set_text(match st.link_connected {
        Some(true) => "Connecté",
        Some(false) => "Déconnecté",
        None => "—",
    });
    ui.status_streak.set_text(&st.weak_streak.to_string());
    ui.status_event.set_text(&st.last_event);

    ui.monitor_btn.set_label(if st.active {
        "Arrêter la surveillance"
    } else {
        "Démarrer la surveillance"
    });
    ui.monitor_btn.remove_css_class("destructive-action");
    ui.monitor_btn.remove_css_class("suggested-action");
    if st.active {
        ui.monitor_btn.add_css_class("destructive-action");
    } else {
        ui.monitor_btn.add_css_class("suggested-action");
    }
    ui.monitor_btn.set_sensitive(cfg.device.is_some());
}

fn load_settings_into_ui(cfg: &AppConfig, ui: &UiHandles) {
    ui.delay_spin.set_value(cfg.settings.delay_before_lock_secs as f64);
    ui.attempts_spin.set_value(cfg.settings.max_attempts as f64);
    ui.rssi_scale.set_value(cfg.settings.rssi_threshold_dbm as f64);
    let idx = match cfg.settings.proximity_method {
        ProximityMethod::LinkPresence => 0,
        ProximityMethod::Rssi => 1,
    };
    ui.mode_row.set_selected(idx);
    ui.rssi_box.set_visible(cfg.settings.proximity_method == ProximityMethod::Rssi);
}

fn persist_settings_from_ui(engine: &SharedEngine, ui: &UiHandles) {
    let mut cfg = engine_get_config(engine);
    cfg.settings.delay_before_lock_secs = ui.delay_spin.value() as u32;
    cfg.settings.max_attempts = ui.attempts_spin.value() as u32;
    cfg.settings.rssi_threshold_dbm = ui.rssi_scale.value() as i16;
    cfg.settings.proximity_method = if ui.mode_row.selected() == 0 {
        ProximityMethod::LinkPresence
    } else {
        ProximityMethod::Rssi
    };
    let _ = engine_save_config(engine, cfg);
}

fn set_message(ui: &UiHandles, text: &str) {
    ui.message.set_text(text);
    ui.message.set_visible(!text.is_empty());
}

fn populate_devices(ui: &UiHandles, devices: &[BtDevice], selected: Option<&str>) {
    while let Some(row) = ui.device_list.row_at_index(0) {
        ui.device_list.remove(&row);
    }
    for dev in devices {
        let title = dev.name.clone();
        let subtitle = format!(
            "{} · {}{}",
            kind_label(dev.kind),
            dev.address,
            dev.rssi
                .map(|r| format!(" · {r} dBm"))
                .unwrap_or_default()
        );
        let row = adw::ActionRow::new();
        row.set_title(&title);
        row.set_subtitle(&subtitle);
        if selected == Some(dev.address.as_str()) {
            row.add_prefix(&gtk::Image::from_icon_name("emblem-ok-symbolic"));
        }
        let list_row = ListBoxRow::new();
        list_row.set_child(Some(&row));
        list_row.set_widget_name(&dev.address);
        ui.device_list.append(&list_row);
    }
}

fn run_on_main_when_ready<T, F>(rx: mpsc::Receiver<T>, on_ready: F)
where
    T: Send + 'static,
    F: FnOnce(T) + 'static,
{
    let mut on_ready = Some(on_ready);
    glib::timeout_add_local(Duration::from_millis(50), move || {
        match rx.try_recv() {
            Ok(value) => {
                if let Some(f) = on_ready.take() {
                    f(value);
                }
                glib::ControlFlow::Break
            }
            Err(mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
            Err(mpsc::TryRecvError::Disconnected) => glib::ControlFlow::Break,
        }
    });
}

fn spawn_bt_task<T, Fut, F>(future: Fut, on_ready: F)
where
    T: Send + 'static,
    Fut: std::future::Future<Output = T> + Send + 'static,
    F: FnOnce(T) + 'static,
{
    let (tx, rx) = mpsc::channel();
    tokio_handle().spawn(async move {
        let _ = tx.send(future.await);
    });
    run_on_main_when_ready(rx, on_ready);
}

pub fn build_main_window(app: &adw::Application, engine: SharedEngine) {
    let _runtime = tokio_handle();

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("BTlock")
        .default_width(920)
        .default_height(640)
        .build();

    window.connect_close_request(|win| {
        win.set_visible(false);
        glib::Propagation::Stop
    });

    let header = adw::HeaderBar::new();
    let scan_btn = Button::builder()
        .label("Rechercher")
        .icon_name("system-search-symbolic")
        .build();
    header.pack_end(&scan_btn);

    let ui = UiHandles {
        status_event: Label::new(None),
        status_rssi: Label::new(None),
        status_link: Label::new(None),
        status_streak: Label::new(None),
        status_device: Label::new(None),
        monitor_btn: Button::new(),
        device_list: ListBox::new(),
        message: Label::new(None),
        rssi_scale: Scale::with_range(Orientation::Horizontal, -95.0, -50.0, 1.0),
        rssi_box: GtkBox::new(Orientation::Vertical, 6),
        mode_row: adw::ComboRow::new(),
        delay_spin: SpinButton::with_range(1.0, 60.0, 1.0),
        attempts_spin: SpinButton::with_range(1.0, 10.0, 1.0),
    };

    ui.message.add_css_class("dim-label");
    ui.message.set_wrap(true);
    ui.message.set_xalign(0.0);
    ui.device_list.add_css_class("boxed-list");
    ui.device_list.set_selection_mode(gtk::SelectionMode::None);
    ui.device_list.set_activate_on_single_click(true);

    ui.mode_row.set_title("Mode de proximité");
    ui.mode_row.set_subtitle("Connexion BT (fiable) ou distance RSSI");
    let model = gtk::StringList::new(&[
        "Connexion Bluetooth (écran éteint)",
        "Distance RSSI (seuil dBm)",
    ]);
    ui.mode_row.set_model(Some(&model));

    ui.rssi_scale.set_draw_value(true);
    ui.rssi_scale.set_value_pos(gtk::PositionType::Right);
    ui.rssi_scale.set_digits(0);

    ui.monitor_btn.set_label("Démarrer la surveillance");
    ui.monitor_btn.add_css_class("suggested-action");
    ui.monitor_btn.set_halign(Align::Start);

    let delay_adj = ui.delay_spin.adjustment();
    let delay_row = adw::SpinRow::new(Some(&delay_adj), 1.0, 0);
    delay_row.set_title("Délai avant verrouillage (s)");

    let attempts_adj = ui.attempts_spin.adjustment();
    let attempts_row = adw::SpinRow::new(Some(&attempts_adj), 1.0, 0);
    attempts_row.set_title("Lectures faibles consécutives");

    let rssi_row = adw::ActionRow::new();
    rssi_row.set_title("Seuil RSSI (dBm)");
    rssi_row.set_subtitle("Verrou si mesure fraîche sous ce seuil");
    ui.rssi_scale.set_hexpand(true);
    rssi_row.add_suffix(&ui.rssi_scale);
    ui.rssi_box.append(&rssi_row);

    let devices_card = adw::PreferencesGroup::new();
    devices_card.set_title("Appareils Bluetooth");
    devices_card.set_description(Some(
        "BlueZ (Linux) — sélectionnez votre téléphone ou montre.",
    ));
    devices_card.add(&ui.device_list);

    let settings_card = adw::PreferencesGroup::new();
    settings_card.set_title("Paramètres");
    settings_card.add(&ui.mode_row);
    settings_card.add(&ui.rssi_box);
    settings_card.add(&delay_row);
    settings_card.add(&attempts_row);

    let status_group = adw::PreferencesGroup::new();
    status_group.set_title("État");

    let mk_row = |title: &str, value: &Label| {
        let row = adw::ActionRow::new();
        row.set_title(title);
        value.set_selectable(true);
        row.add_suffix(value);
        row
    };
    status_group.add(&mk_row("Appareil", &ui.status_device));
    status_group.add(&mk_row("Connexion BT", &ui.status_link));
    status_group.add(&mk_row("RSSI (< 3 s)", &ui.status_rssi));
    status_group.add(&mk_row("Série faible", &ui.status_streak));
    status_group.add(&mk_row("Événement", &ui.status_event));

    let right_panel = GtkBox::new(Orientation::Vertical, 12);
    right_panel.append(&settings_card);
    right_panel.append(&ui.monitor_btn);

    let left = adw::Clamp::new();
    left.set_child(Some(&devices_card));
    let right = adw::Clamp::new();
    right.set_child(Some(&right_panel));

    let grid = GtkBox::new(Orientation::Horizontal, 12);
    grid.set_margin_top(12);
    grid.set_margin_bottom(12);
    grid.set_margin_start(12);
    grid.set_margin_end(12);
    grid.append(&left);
    grid.append(&right);

    let status_clamp = adw::Clamp::new();
    status_clamp.set_child(Some(&status_group));

    let root = GtkBox::new(Orientation::Vertical, 8);
    root.append(&header);
    root.append(&ui.message);
    root.append(&grid);
    root.append(&status_clamp);

    let scroll = gtk::ScrolledWindow::builder()
        .vexpand(true)
        .child(&root)
        .build();
    window.set_content(Some(&scroll));

    let cfg = engine_get_config(&engine);
    load_settings_into_ui(&cfg, &ui);
    refresh_status(&engine, &ui);

    let scanned: Arc<Mutex<Vec<BtDevice>>> = Arc::new(Mutex::new(Vec::new()));

    let ui = Rc::new(ui);
    let ui_scan = Rc::clone(&ui);
    let eng_scan = Arc::clone(&engine);
    let scanned_scan = Arc::clone(&scanned);
    let scan_btn_weak = scan_btn.downgrade();
    scan_btn.connect_clicked(move |_| {
        set_message(&ui_scan, "Recherche des appareils (≈8 s)…");
        if let Some(btn) = scan_btn_weak.upgrade() {
            btn.set_sensitive(false);
        }
        let ui = Rc::clone(&ui_scan);
        let eng = Arc::clone(&eng_scan);
        let btn = scan_btn_weak.clone();
        let scanned_store = Arc::clone(&scanned_scan);
        spawn_bt_task(scan_bt_devices(), move |result| {
            if let Some(scan_btn) = btn.upgrade() {
                scan_btn.set_sensitive(true);
            }
            match result {
                Ok(list) => {
                    *scanned_store.lock().unwrap() = list.clone();
                    let cfg = engine_get_config(&eng);
                    populate_devices(
                        &ui,
                        &list,
                        cfg.device.as_ref().map(|d| d.address.as_str()),
                    );
                    set_message(&ui, &format!("{} appareil(s) trouvé(s).", list.len()));
                }
                Err(e) => set_message(&ui, &e),
            }
        });
    });

    let ui_mode = Rc::clone(&ui);
    let engine_mode = Arc::clone(&engine);
    ui.mode_row.connect_notify_local(Some("selected"), move |row, _| {
        let rssi = row.selected() == 1;
        ui_mode.rssi_box.set_visible(rssi);
        persist_settings_from_ui(&engine_mode, &ui_mode);
    });

    for spin in [&ui.delay_spin, &ui.attempts_spin] {
        let ui_s = Rc::clone(&ui);
        let eng_s = Arc::clone(&engine);
        spin.connect_value_changed(move |_| {
            persist_settings_from_ui(&eng_s, &ui_s);
        });
    }
    let ui_r = Rc::clone(&ui);
    let eng_r = Arc::clone(&engine);
    ui.rssi_scale.connect_value_changed(move |_| {
        persist_settings_from_ui(&eng_r, &ui_r);
    });

    let ui_pick = Rc::clone(&ui);
    let eng_pick = Arc::clone(&engine);
    let scanned_pick = Arc::clone(&scanned);
    ui.device_list.connect_row_activated(move |_, row| {
        let address = row.widget_name().to_string();
        let dev = scanned_pick
            .lock()
            .unwrap()
            .iter()
            .find(|d| d.address == address)
            .cloned();
        let Some(dev) = dev else {
            set_message(&ui_pick, "Relancez une recherche puis sélectionnez l’appareil.");
            return;
        };
        set_message(&ui_pick, &format!("Association avec {}…", dev.name));
        let ui = Rc::clone(&ui_pick);
        let eng = Arc::clone(&eng_pick);
        let eng_async = Arc::clone(&eng);
        spawn_bt_task(
            async move { engine_select_device(&eng_async, dev).await },
            move |result| {
                match result {
                    Ok(_cfg) => {
                        refresh_status(&eng, &ui);
                        set_message(&ui, "Appareil associé.");
                    }
                    Err(e) => set_message(&ui, &e),
                }
            },
        );
    });

    let ui_mon = Rc::clone(&ui);
    let eng_mon = Arc::clone(&engine);
    ui.monitor_btn.connect_clicked(move |_| {
        let active = engine_get_status(&eng_mon).active;
        let ui = Rc::clone(&ui_mon);
        let eng = Arc::clone(&eng_mon);
        if active {
            let _ = engine_stop_monitoring(&eng);
            refresh_status(&eng, &ui);
            set_message(&ui, "Surveillance arrêtée.");
        } else {
            persist_settings_from_ui(&eng, &ui);
            let eng_async = Arc::clone(&eng);
            spawn_bt_task(
                async move { engine_start_monitoring(&eng_async).await },
                move |result| {
                    refresh_status(&eng, &ui);
                    match result {
                        Ok(()) => set_message(&ui, "Surveillance active."),
                        Err(e) => set_message(&ui, &e),
                    }
                },
            );
        }
    });

    let ui_poll = Rc::clone(&ui);
    let eng_poll = Arc::clone(&engine);
    glib::timeout_add_seconds_local(2, move || {
        refresh_status(&eng_poll, &ui_poll);
        glib::ControlFlow::Continue
    });

    window.present();
}
