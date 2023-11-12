/*=======================================================================
 * IMPORTS
 * ====================================================================*/
// local files
use crate::bluetooth::cps::*;
use crate::bluetooth::{bt_adapter_scan, bt_scan};
use crate::zwo_reader::zwo_command::{create_timeseries, WorkoutTimeSeries};
use crate::zwo_reader::{zwo_read, Workout};

// external crates
use async_std::task;
use btleplug::{
    api::{Central, Peripheral as Peripheral_api},
    platform::{Adapter, Peripheral},
};
use eframe::egui::{self, Ui};
use egui_file::FileDialog;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

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
    // workout file stuff
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
        }
    }
}

impl eframe::App for BikeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint(); // update gui with new message - otherwise waits on mouse

        if self.bt_adapters.is_some() && self.selected_adapter_number.is_some() {
            if self.adapter_moved == false {
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
    if app_struct.peripheral_connected && app_struct.selected_peripheral.is_some() {
        let peripheral = app_struct.selected_peripheral.clone().unwrap();
        let _ = task::block_on(peripheral.discover_services());
        let characteristics = peripheral.characteristics();
        let feature_char = characteristics
            .iter()
            .find(|c| c.uuid == CPS_POWER_FEATURE)
            .unwrap();
        if ui.button("Read Features").clicked() {
            let read_result = task::block_on(peripheral.read(feature_char));
            match read_result {
                Ok(buf) => {
                    println!("Feature buffer length: {:?}", buf.len());
                    let hack_buffer = u32::from_le_bytes(buf.clone().try_into().unwrap());
                    let hack_struct = CpsFeature(hack_buffer);
                    println!("{:?}", hack_struct);
                    let combined_buffer = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
                    let feature_struct = CpsFeature(combined_buffer);
                    println!("{:?}", feature_struct);
                }
                Err(e) => {
                    ui.label(e.to_string());
                }
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
            let mut dialog = FileDialog::open_file(app_struct.workout_file.clone());
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
            // TODO: zwo read function here
            if app_struct.workout_file.is_some() {
                let filepath = app_struct.workout_file.clone().unwrap();
                match zwo_read(filepath) {
                    Ok(workout) => app_struct.workout = Some(workout),
                    Err(e) => println!("{:?}", e),
                }
                /*
                if app_struct.workout.is_some() {
                    println!("{:?}", app_struct.workout.clone().unwrap());
                }
                */
            }
            if app_struct.workout.is_some() {
                let workout = app_struct.workout.clone().unwrap();
                match create_timeseries(workout) {
                    Ok(time_series) => app_struct.workout_time_series = Some(time_series),
                    Err(e) => println!("Error: {:?}", e),
                }
            }
        }

        // demo of time series data
        if ui.button("Start").clicked() {
            println!("Trying to start workout...");
            if app_struct.workout_time_series.is_some() {
                app_struct.workout_running = true;
                println!("Workout time series exists.");
                let time_series = app_struct.workout_time_series.clone().unwrap();
                let workout_sender = app_struct.workout_channel.0.clone();
                println!("Spawning workout thread...");
                thread::spawn(move || {
                    for (i, _) in time_series.time.iter().enumerate() {
                        let message = WorkoutMessage {
                            time: time_series.time[i].clone(),
                            target_cadence: time_series.cadence[i].clone(),
                            target_power: time_series.power[i].clone(),
                        };
                        // TODO: better error handling here
                        println!("Sending message...");
                        workout_sender.send(message).unwrap();
                        thread::sleep(Duration::from_secs(1));
                    }
                });
            } else {
                println!("Load a workout first.");
            }
        }
    });

    // GUI for running workout
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
            ui.label(app_struct.display_power.to_string());
        });
    }
}

/// draws the bluetooth tab
fn draw_bluetooth_tab(ui: &mut Ui, app_struct: &mut BikeApp) {
    ui.horizontal(|ui| {
        if ui.button("Adapter").clicked() {
            println!("Searching for adapter...");
            app_struct.bt_adapters = task::block_on(bt_adapter_scan());
            println!("COmplete.");
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
                app_struct.peripheral_list =
                    task::block_on(bt_scan(&mut app_struct.selected_adapter.as_mut().unwrap()));
            }
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
