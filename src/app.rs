/*=======================================================================
 * IMPORTS
 * ====================================================================*/
// local files
use crate::bluetooth::cps::*;
use crate::bluetooth::queue::{bt_q_main, BtAction, CharReadType, QueueChannels, QueueItem};
use crate::bluetooth::{bt_adapter_scan, bt_scan};
use crate::zwo_reader::zwo_command::{create_timeseries, WorkoutTimeSeries};
use crate::zwo_reader::{zwo_read, Workout};

// external crates
use async_std::stream::StreamExt;
use async_std::task;
use btleplug::api::Characteristic;
use btleplug::{
    api::{Central, Peripheral as Peripheral_api},
    platform::{Adapter, Peripheral},
};
use eframe::egui::{self, Ui};
use eframe::epaint::Vec2;
use egui_file::FileDialog;
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use tokio::runtime::Runtime;

/*=======================================================================
 * CONSTANTS
 * ====================================================================*/

/*=======================================================================
 * ENUMS
 * ====================================================================*/
#[derive(PartialEq, Eq)]
enum Tabs {
    Main,
    Workouts,
    Bluetooth,
    Help,
}

/*=======================================================================
 * STRUCTS
 * ====================================================================*/
#[derive(Debug)]
pub struct WorkoutMessage {
    time: usize,
    target_cadence: i32,
    target_power: f32,
}

pub struct BikeApp {
    // app state stuff
    active_tab: Tabs,
    // bluetooth stuff
    bt_adapters: Option<Vec<Adapter>>,
    selected_adapter: Option<Adapter>,
    selected_adapter_number: Option<usize>,
    adapter_moved: bool,
    adapter_text: String,
    peripheral_list: Option<Vec<Peripheral>>,
    selected_peripheral: Option<Peripheral>,
    selected_peripheral_number: Option<usize>,
    peripheral_moved: bool,
    peripheral_text: String,
    peripheral_connected: bool,
    peripheral_channel: (
        std::sync::mpsc::Sender<Option<Vec<Peripheral>>>,
        std::sync::mpsc::Receiver<Option<Vec<Peripheral>>>,
    ),
    bt_data_channel: (
        std::sync::mpsc::Sender<Vec<u8>>,
        std::sync::mpsc::Receiver<Vec<u8>>,
    ),
    power_measurement_subscribed: bool,
    bt_queue_sender: Option<std::sync::mpsc::Sender<QueueItem>>,
    queue_kill_sender: Option<std::sync::mpsc::Sender<bool>>,
    connected_channel: (
        std::sync::mpsc::Sender<bool>,
        std::sync::mpsc::Receiver<bool>,
    ),
    subscribed_channel: (
        std::sync::mpsc::Sender<bool>,
        std::sync::mpsc::Receiver<bool>,
    ),
    cps_channel: (
        std::sync::mpsc::Sender<Vec<u8>>,
        std::sync::mpsc::Receiver<Vec<u8>>,
    ),
    features_channel: (
        std::sync::mpsc::Sender<CpsFeature>,
        std::sync::mpsc::Receiver<CpsFeature>,
    ),
    results_channel: (
        std::sync::mpsc::Sender<String>,
        std::sync::mpsc::Receiver<String>,
    ),
    scanned: bool,
    char_channel: (
        std::sync::mpsc::Sender<BTreeSet<Characteristic>>,
        std::sync::mpsc::Receiver<BTreeSet<Characteristic>>,
    ),
    discovered_characteristics: Option<BTreeSet<Characteristic>>,
    cps_power_feature: Option<Characteristic>,
    cps_power_measurement: Option<Characteristic>,
    cps_control_point: Option<Characteristic>,
    cps_feature_read: bool,
    // workout file stuff
    user_ftp: u32,
    user_ftp_string: String,
    workout_file: Option<PathBuf>,
    workout_file_dialog: Option<FileDialog>,
    workout: Option<Workout>,
    workout_time_series: Option<WorkoutTimeSeries>,
    workout_running: bool,
    display_time: usize,
    display_cadence: i32,
    display_power: f32,
    // Main/testing stuff
    resistance_text: String,
    resistance_value: u8,
    // workout thread:
    workout_channel: (
        std::sync::mpsc::Sender<WorkoutMessage>,
        std::sync::mpsc::Receiver<WorkoutMessage>,
    ),
    // stop workout channel
    stop_workout_flag: bool,
    stop_workout_sender: Option<std::sync::mpsc::Sender<bool>>,
}

impl Default for BikeApp {
    fn default() -> Self {
        Self {
            active_tab: Tabs::Main,
            bt_adapters: None,
            selected_adapter: None,
            selected_adapter_number: None,
            adapter_moved: false,
            adapter_text: "None selected".to_string(),
            peripheral_list: None,
            selected_peripheral: None,
            selected_peripheral_number: None,
            peripheral_moved: false,
            peripheral_text: "None selected".to_string(),
            peripheral_connected: false,
            peripheral_channel: std::sync::mpsc::channel(),
            bt_data_channel: std::sync::mpsc::channel(),
            power_measurement_subscribed: false,
            bt_queue_sender: None,
            queue_kill_sender: None,
            connected_channel: std::sync::mpsc::channel(),
            subscribed_channel: std::sync::mpsc::channel(),
            cps_channel: std::sync::mpsc::channel(),
            features_channel: std::sync::mpsc::channel(),
            results_channel: std::sync::mpsc::channel(),
            scanned: false,
            char_channel: std::sync::mpsc::channel(),
            discovered_characteristics: None,
            cps_power_feature: None,
            cps_power_measurement: None,
            cps_control_point: None,
            cps_feature_read: false,
            user_ftp: 100,
            user_ftp_string: "100".to_string(),
            workout_file: None,
            workout_file_dialog: None,
            workout: None,
            workout_time_series: None,
            workout_running: false,
            display_time: 0,
            display_cadence: 0,
            display_power: 0.0,
            resistance_text: "0".to_string(),
            resistance_value: 0,
            workout_channel: std::sync::mpsc::channel(),
            stop_workout_flag: true,
            stop_workout_sender: None,
        }
    }
}

impl eframe::App for BikeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint(); // update gui with new message - otherwise waits on mouse
        update_text_edits(self);

        if self.bt_adapters.is_some() && self.selected_adapter_number.is_some() {
            if self.adapter_moved == false {
                // move adapter, make it active adapter
                self.adapter_text = task::block_on(update_adapter_text(
                    &self.bt_adapters.as_ref().unwrap()
                        [self.selected_adapter_number.clone().unwrap()],
                ));
                self.selected_adapter = Some(
                    self.bt_adapters
                        .as_mut()
                        .unwrap()
                        .remove(self.selected_adapter_number.clone().unwrap()),
                );
                self.adapter_moved = true;

                // start bluetooth queue
                if self.bt_queue_sender.is_none() {
                    let (bt_queue_sender, bt_queue_recv) = std::sync::mpsc::channel();
                    self.bt_queue_sender = Some(bt_queue_sender);
                    let (kill_tx, kill_rx) = std::sync::mpsc::channel();
                    self.queue_kill_sender = Some(kill_tx);
                    let channels = QueueChannels {
                        is_connected: self.connected_channel.0.clone(),
                        peripherals: self.peripheral_channel.0.clone(),
                        subscribed: self.subscribed_channel.0.clone(),
                        cps_power_reading: self.cps_channel.0.clone(),
                        cps_features: self.features_channel.0.clone(),
                        results: self.results_channel.0.clone(),
                        characteristics: self.char_channel.0.clone(),
                    };
                    thread::spawn(move || async {
                        bt_q_main(bt_queue_recv, channels, kill_rx).await;
                    });
                }
            }
        }
        if self.peripheral_list.is_some() && self.selected_peripheral_number.is_some() {
            if self.peripheral_moved == false {
                println!(
                    "Peripheral number: {:?}",
                    self.selected_peripheral_number.clone().unwrap()
                );
                println!(
                    "Peripheral text: {:?}",
                    self.peripheral_list.as_ref().unwrap()
                        [self.selected_peripheral_number.clone().unwrap()]
                );
                // TODO: move to new thread
                self.peripheral_text = task::block_on(update_peripheral_text(
                    &self.peripheral_list.as_ref().unwrap()
                        [self.selected_peripheral_number.clone().unwrap()],
                ));
                self.selected_peripheral = Some(
                    self.peripheral_list
                        .as_mut()
                        .unwrap()
                        .remove(self.selected_peripheral_number.clone().unwrap()),
                );
                self.peripheral_moved = true;
            }
        }
        if self.peripheral_moved && self.selected_peripheral.is_some() {
            match task::block_on(self.selected_peripheral.clone().unwrap().is_connected()) {
                Ok(flag) => self.peripheral_connected = flag,
                Err(_) => {}
            }
        }

        // parse text boxes
        match self.resistance_text.parse::<u8>() {
            Ok(value) => self.resistance_value = value,
            Err(_) => self.resistance_value = 0,
        }

        // main window
        egui::TopBottomPanel::top("Tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.active_tab, Tabs::Main, "Main");
                ui.selectable_value(&mut self.active_tab, Tabs::Workouts, "Workouts");
                ui.selectable_value(&mut self.active_tab, Tabs::Bluetooth, "Bluetooth");
                ui.selectable_value(&mut self.active_tab, Tabs::Help, "Help");
            });
            ui.separator();
            match self.active_tab {
                Tabs::Main => {
                    draw_main_tab(ui, self);
                }
                Tabs::Workouts => {
                    draw_workout_tab(ctx, ui, self);
                }
                Tabs::Bluetooth => {
                    draw_bluetooth_tab(ui, self);
                }
                Tabs::Help => {}
            }
        });
    }
}

/// updates stored values based on text in textedit boxes
fn update_text_edits(app_struct: &mut BikeApp) {
    match app_struct.user_ftp_string.parse::<u32>() {
        Ok(ftp) => app_struct.user_ftp = ftp,
        Err(_) => {} // dont do anything if user is typing, weird characters, etc.
    }
}

/// update bluetooth adapter combobox text based on selection
async fn update_adapter_text(adapter: &Adapter) -> String {
    let adapter_str: String;
    match adapter.adapter_info().await {
        Ok(info) => adapter_str = info.to_string(),
        Err(e) => adapter_str = e.to_string(),
    }
    return adapter_str;
}

/// update bluetooth peripheral combobox text based on selection
async fn update_peripheral_text(peripheral: &Peripheral) -> String {
    let mut peripheral_str = peripheral.id().to_string();
    match peripheral.properties().await {
        Ok(properties) => match properties {
            Some(prop) => match prop.local_name {
                Some(name) => peripheral_str = name,
                None => {}
            },
            None => {}
        },
        Err(_) => {}
    }
    return peripheral_str;
}

/// draws the main tab
fn draw_main_tab(ui: &mut Ui, app_struct: &mut BikeApp) {
    // if a peripheral is selected (and connected), scan for services
    if app_struct.peripheral_connected && app_struct.selected_peripheral.is_some() {
        if !app_struct.scanned {
            let peripheral = app_struct.selected_peripheral.clone().unwrap();
            if app_struct.bt_queue_sender.is_some() {
                let tx = app_struct.bt_queue_sender.clone().unwrap();
                let scan_req = QueueItem {
                    action: BtAction::Discover,
                    peripheral: peripheral,
                    characteristic: None,
                    read_type: None,
                };
                let _ = tx.send(scan_req); // TODO: error handling
                app_struct.scanned = true;
            }
        }
        // check for characteristics (once!)
        if app_struct.discovered_characteristics.is_none() {
            if let Ok(char) = app_struct.char_channel.1.try_recv() {
                app_struct.discovered_characteristics = Some(char);
            }
        }
        // once characteristics are found, grab the important ones
        if app_struct.discovered_characteristics.is_some()
            && app_struct.cps_power_measurement.is_none()
        {
            let chars = app_struct.discovered_characteristics.clone().unwrap();
            app_struct.cps_power_feature = Some(
                chars
                    .iter()
                    .find(|c| c.uuid == CPS_POWER_FEATURE)
                    .unwrap()
                    .clone(),
            );
            app_struct.cps_power_measurement = Some(
                chars
                    .iter()
                    .find(|c| c.uuid == CPS_POWER_MEASUREMENT)
                    .unwrap()
                    .clone(),
            );
            app_struct.cps_control_point = Some(
                chars
                    .iter()
                    .find(|c| c.uuid == CPS_CONTROL_POINT)
                    .unwrap()
                    .clone(),
            );
        }
        // TODO: better feature display
        if app_struct.cps_power_feature.is_some() && !app_struct.cps_feature_read {
            // only do this once
            if app_struct.bt_queue_sender.is_some() {
                let tx = app_struct.bt_queue_sender.unwrap().clone();
                let read_req = QueueItem {
                    action: BtAction::Read,
                    peripheral: app_struct.selected_peripheral.clone().unwrap(),
                    characteristic: app_struct.cps_power_feature.clone(),
                    read_type: Some(CharReadType::CPSPowerFeature),
                };
                let _ = tx.send(read_req); // TODO: send error handling
                app_struct.cps_feature_read = true;
            }
        }
        if ui.button("Subscribe to CPS Power Measurement").clicked() {
            let subscribe_result = task::block_on(peripheral.subscribe(feature_char2));
            match subscribe_result {
                Ok(k) => {
                    println!("Subscribed to Power Measurement. {:?}", k);
                    app_struct.power_measurement_subscribed = true;
                    // spawn a thread to receive notifications
                    let notification_sender = app_struct.bt_data_channel.0.clone();
                    let subscribed_peripheral = peripheral.clone();
                    thread::spawn(move || {
                        task::block_on(async move {
                            let notification_result = subscribed_peripheral.notifications().await;
                            match notification_result {
                                Ok(notif) => {
                                    let mut reading = notif.take(1);
                                    while let Some(data) = reading.next().await {
                                        println!("Reading: {:?}", data.value);
                                        // TODO: better error handling here
                                        notification_sender.send(data.value).unwrap();
                                        println!("Reading sent.");
                                    }
                                }
                                Err(_) => {} // noting here yet
                            }
                        });
                    });
                }
                Err(e) => {
                    println!("Failed to subscribe to Power Measurement: {:?}", e);
                }
            }
        }
        if ui.button("Read Feature 3").clicked() {
            // IDK, don't bother with this one yet I guess
            // it's writable and supports indicate (probably for write result?)
            // don't think this one is necessary either
            let read_result3 = task::block_on(peripheral.read(feature_char3));
            match read_result3 {
                Ok(buf) => {
                    println!("Control Point length: {:?}", buf.len());
                    //let combined_buffer = u16::from_le_bytes(buf.clone().try_into().unwrap());
                    //let control_struct = Cps
                }
                Err(e) => {
                    println!("{:?}", e);
                }
            }
        }
        if app_struct.power_measurement_subscribed {
            // receive notifications sent and display here
            match app_struct.bt_data_channel.1.try_recv() {
                Ok(message) => {
                    println!("Message received!");
                    for msg in message.iter() {
                        // TODO: but label values in app struct or something
                        ui.label(msg.to_string());
                    }
                }
                Err(_) => {}
            }
        }
    }
}

/// draws the workout tab
fn draw_workout_tab(ctx: &egui::Context, ui: &mut Ui, app_struct: &mut BikeApp) {
    ui.horizontal(|ui| {
        ui.label("Workout:");
        // TODO: make this a text box so you can manually enter the file path???
        if let Some(workout_file) = &app_struct.workout_file {
            ui.label(format!("{:?}", workout_file));
        } else {
            ui.label("None");
        }
        if ui.button("Open").clicked() {
            let mut dialog = FileDialog::open_file(app_struct.workout_file.clone())
                .default_size(Vec2::new(500.0, 200.0));
            dialog.open();
            app_struct.workout_file_dialog = Some(dialog);
        }
        if let Some(dialog) = &mut app_struct.workout_file_dialog {
            if dialog.show(ctx).selected() {
                if let Some(file) = dialog.path() {
                    app_struct.workout_file = Some(file);
                    // TODO: maybe actually load the file here?
                }
            }
        }

        // separate load button for now, worry about it later
        if ui.button("Load").clicked() {
            println!("Loading workout...");
            // TODO: error message if file not correct type, etc.
            if app_struct.workout_file.is_some() {
                let filepath = app_struct.workout_file.clone().unwrap();
                match zwo_read(filepath) {
                    Ok(workout) => app_struct.workout = Some(workout),
                    Err(e) => println!("{:?}", e),
                }
            }
            if app_struct.workout.is_some() {
                let workout = app_struct.workout.clone().unwrap();
                match create_timeseries(workout) {
                    Ok(time_series) => app_struct.workout_time_series = Some(time_series),
                    Err(e) => println!("Error: {:?}", e),
                }
            }
            println!("Complete.");
        }

        // demo of time series data
        if ui.button("Start").clicked() {
            app_struct.stop_workout_flag = false; // assuming this is necessary after clicking stop
            if app_struct.workout_time_series.is_some() {
                app_struct.workout_running = true;
                // create receiver to give to new thread
                let (tx, stop_receiver) = std::sync::mpsc::channel();
                app_struct.stop_workout_sender = Some(tx);
                println!("Workout started!");
                let time_series = app_struct.workout_time_series.clone().unwrap();
                let workout_sender = app_struct.workout_channel.0.clone();
                thread::spawn(move || {
                    for (i, _) in time_series.time.iter().enumerate() {
                        // check to see if stop button has been clicked
                        match stop_receiver.try_recv() {
                            Ok(flag) => {
                                if flag {
                                    break;
                                }
                            }
                            Err(_) => {}
                        }
                        let message = WorkoutMessage {
                            time: time_series.time[i].clone(),
                            target_cadence: time_series.cadence[i].clone(),
                            target_power: time_series.power[i].clone(),
                        };
                        // TODO: better error handling here
                        workout_sender.send(message).unwrap();
                        thread::sleep(Duration::from_secs(1));
                    }
                });
            } else {
                println!("Load a workout first.");
            }
        }

        if ui.button("Stop").clicked() {
            app_struct.stop_workout_flag = true;
            // TODO: app_struct.workout running = false;
            if app_struct.stop_workout_sender.is_some() {
                let stop_sender = app_struct.stop_workout_sender.as_ref().unwrap();
                // TODO: better error handling here
                stop_sender
                    .send(app_struct.stop_workout_flag.clone())
                    .unwrap();
            }
        }
    });

    // GUI for running workout
    // user input
    ui.horizontal(|ui| {
        ui.label("FTP:");
        ui.text_edit_singleline(&mut app_struct.user_ftp_string);
    });

    if app_struct.workout_running {
        // receive message from workout thread
        match app_struct.workout_channel.1.try_recv() {
            Ok(message) => {
                app_struct.display_time = message.time;
                app_struct.display_cadence = message.target_cadence;
                app_struct.display_power = message.target_power;
            }
            Err(_) => {}
        }
        ui.horizontal(|ui| {
            ui.label("Time:");
            ui.label(app_struct.display_time.to_string());
        });
        ui.horizontal(|ui| {
            ui.label("Cadence:");
            ui.label(app_struct.display_cadence.to_string());
        });
        ui.horizontal(|ui| {
            ui.label("Power:");
            ui.label((app_struct.display_power * app_struct.user_ftp as f32).to_string());
        });
    }
}

/// draws the bluetooth tab
fn draw_bluetooth_tab(ui: &mut Ui, app_struct: &mut BikeApp) {
    ui.horizontal(|ui| {
        if ui.button("Adapter").clicked() {
            println!("Searching for adapter...");
            app_struct.bt_adapters = task::block_on(bt_adapter_scan());
            println!("Complete.");
        }
        if app_struct.adapter_moved {
            let adapter_info_str: String;
            match task::block_on(app_struct.selected_adapter.clone().unwrap().adapter_info()) {
                Ok(info) => adapter_info_str = info,
                Err(e) => adapter_info_str = e.to_string(),
            }
            egui::ComboBox::from_label("Choose an adapter.")
                .selected_text(&app_struct.adapter_text)
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut app_struct.selected_adapter_number,
                        Some(0 as usize),
                        adapter_info_str,
                    );
                });
        } else {
            egui::ComboBox::from_label("Choose an adapter.")
                .selected_text(&app_struct.adapter_text)
                .show_ui(ui, |ui| match &app_struct.bt_adapters {
                    Some(adapters) => {
                        for (i, adapter) in adapters.iter().enumerate() {
                            let adapter_info_str: String;
                            match task::block_on(adapter.adapter_info()) {
                                Ok(info) => adapter_info_str = info,
                                Err(e) => adapter_info_str = e.to_string(),
                            }
                            ui.selectable_value(
                                &mut app_struct.selected_adapter_number,
                                Some(i),
                                adapter_info_str,
                            );
                        }
                    }
                    None => {
                        ui.selectable_value(&mut app_struct.selected_adapter_number, None, "None");
                    }
                });
        }
    });
    ui.horizontal(|ui| {
        if ui.button("Scan").clicked() {
            if app_struct.adapter_moved {
                println!("Scanning for devices...");
                let peripheral_sender = app_struct.peripheral_channel.0.clone();
                let selected_adapter = app_struct.selected_adapter.clone().unwrap();
                // lmao, this doesn't seem ideal
                thread::spawn(move || {
                    let rt = Runtime::new().unwrap();
                    let peripheral_list = rt.block_on(async move {
                        let peripheral_list = task::block_on(bt_scan(&selected_adapter));
                        return peripheral_list;
                    });
                    // TODO: error handling instead of unwrap
                    peripheral_sender.send(peripheral_list).unwrap();
                });
            }
        }
        // don't want receiver in the button click function, timelines get weird
        // even here it's kind of a problem, it only receives when on the bluetooth tab
        match app_struct.peripheral_channel.1.try_recv() {
            Ok(message) => {
                app_struct.peripheral_list = message;
                println!("Complete.");
            }
            Err(_) => {}
        }
        if app_struct.peripheral_moved {
            let mut name_str = app_struct
                .selected_peripheral
                .clone()
                .unwrap()
                .id()
                .to_string();
            let properties_result =
                task::block_on(app_struct.selected_peripheral.clone().unwrap().properties());
            match properties_result {
                Ok(prop_option) => {
                    if prop_option.is_some() {
                        if prop_option.clone().unwrap().local_name.is_some() {
                            name_str = prop_option.clone().unwrap().local_name.unwrap();
                        }
                    }
                }
                Err(_) => {}
            }
            egui::ComboBox::from_label("Choose a device.")
                .selected_text(&app_struct.peripheral_text)
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut app_struct.selected_peripheral_number,
                        Some(0 as usize),
                        name_str,
                    );
                });
        } else {
            egui::ComboBox::from_label("Choose a device.")
                .selected_text(&app_struct.peripheral_text)
                .show_ui(ui, |ui| match &app_struct.peripheral_list {
                    Some(peripherals) => {
                        for (i, peripheral) in peripherals.iter().enumerate() {
                            let mut name_str = peripheral.id().to_string();
                            let properties_result = task::block_on(peripheral.properties());
                            match properties_result {
                                Ok(prop_option) => {
                                    if prop_option.is_some() {
                                        if prop_option.clone().unwrap().local_name.is_some() {
                                            name_str =
                                                prop_option.clone().unwrap().local_name.unwrap();
                                        }
                                    }
                                }
                                Err(_) => {}
                            }
                            ui.selectable_value(
                                &mut app_struct.selected_peripheral_number,
                                Some(i),
                                name_str,
                            );
                        }
                    }
                    None => {
                        ui.selectable_value(
                            &mut app_struct.selected_peripheral_number,
                            None,
                            "None",
                        );
                    }
                });
        }
        if ui.button("Connect").clicked() {
            if app_struct.peripheral_moved {
                println!("Connecting to device...");
                match task::block_on(app_struct.selected_peripheral.clone().unwrap().connect()) {
                    Ok(()) => {
                        println!("Device connected.");
                        app_struct.peripheral_connected = true;
                    }
                    Err(e) => {
                        println!("Failed to connect.  {:?}", e);
                    }
                }
            } else {
                println!("Please scan for devices and select one to connect");
            }
        }
        if ui.button("Disconnect").clicked() {
            if app_struct.selected_peripheral.is_some() {
                println!("Disconnecting from device...");
                let peripheral = app_struct.selected_peripheral.clone().unwrap();
                let disconnect_result = task::block_on(peripheral.disconnect());
                match disconnect_result {
                    Ok(()) => println!("Successfully disconnected."),
                    Err(e) => println!("Failed to disconnect: {:?}", e),
                }
            }
        }
    });
}
