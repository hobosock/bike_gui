/*=======================================================================
 * IMPORTS
 * ====================================================================*/
// local files
use crate::bluetooth::ble_default_services::SPECIAL_CHARACTERISTICS_NAMES;
use crate::bluetooth::{bt_adapter_scan, bt_scan};
use crate::zwo_reader::zwo_parse::file_to_text;
use crate::zwo_reader::{self, zwo_read, Workout};

// external crates
use async_std::task;
use btleplug::{
    api::{Central, Peripheral as Peripheral_api},
    platform::{Adapter, Peripheral},
};
use eframe::egui::{self, Ui};
use egui_file::FileDialog;
use std::{path::PathBuf, str};
//use uuid::Uuid;

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

/*=======================
 * ================================================
 * STRUCTS
 * ====================================================================*/
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
        }
    }
}

impl eframe::App for BikeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // update state
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
                self.peripheral_text = update_peripheral_text(
                    &self.peripheral_list.as_ref().unwrap()
                        [self.selected_peripheral_number.clone().unwrap()],
                );
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
                    if self.peripheral_connected && self.selected_peripheral.is_some() {
                        let peripheral = self.selected_peripheral.clone().unwrap();
                        let _ = task::block_on(peripheral.discover_services());
                        let characteristics = peripheral.characteristics();
                        for chars in characteristics.iter() {
                            let name_result = SPECIAL_CHARACTERISTICS_NAMES.get(&chars.uuid);
                            let mut name: String;
                            if name_result.is_some() {
                                name = name_result.unwrap().to_string();
                            } else {
                                name = chars.uuid.to_string();
                            }
                            let read_result = task::block_on(peripheral.read(chars));
                            let mut value: String;
                            match read_result {
                                Ok(buf) => {
                                    let s = match str::from_utf8(&buf) {
                                        Ok(v) => v.to_string(),
                                        Err(e) => e.to_string(),
                                    };
                                    value = s;
                                }
                                Err(e) => value = e.to_string(),
                            }
                            ui.horizontal(|ui| {
                                ui.text_edit_singleline(&mut name);
                                ui.text_edit_singleline(&mut value);
                            });
                        }
                    }
                }
                Tabs::Workouts => {
                    draw_workout_tab(ctx, ui, self);
                }
                Tabs::Bluetooth => {
                    ui.horizontal(|ui| {
                        if ui.button("Adapter").clicked() {
                            self.bt_adapters = task::block_on(bt_adapter_scan());
                        }
                        if self.adapter_moved {
                            let adapter_info_str: String;
                            match task::block_on(
                                self.selected_adapter.clone().unwrap().adapter_info(),
                            ) {
                                Ok(info) => adapter_info_str = info,
                                Err(e) => adapter_info_str = e.to_string(),
                            }
                            egui::ComboBox::from_label("Choose an adapter.")
                                .selected_text(&self.adapter_text)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.selected_adapter_number,
                                        Some(0 as usize),
                                        adapter_info_str,
                                    );
                                });
                        } else {
                            egui::ComboBox::from_label("Choose an adapter.")
                                .selected_text(&self.adapter_text)
                                .show_ui(ui, |ui| match &self.bt_adapters {
                                    Some(adapters) => {
                                        for (i, adapter) in adapters.iter().enumerate() {
                                            let adapter_info_str: String;
                                            match task::block_on(adapter.adapter_info()) {
                                                Ok(info) => adapter_info_str = info,
                                                Err(e) => adapter_info_str = e.to_string(),
                                            }
                                            ui.selectable_value(
                                                &mut self.selected_adapter_number,
                                                Some(i),
                                                adapter_info_str,
                                            );
                                        }
                                    }
                                    None => {
                                        ui.selectable_value(
                                            &mut self.selected_adapter_number,
                                            None,
                                            "None",
                                        );
                                    }
                                });
                        }
                    });
                    ui.horizontal(|ui| {
                        if ui.button("Scan").clicked() {
                            if self.adapter_moved {
                                println!("Scanning for devices...");
                                self.peripheral_list = task::block_on(bt_scan(
                                    &mut self.selected_adapter.as_mut().unwrap(),
                                ));
                            }
                        }
                        if self.peripheral_moved {
                            let mut name_str =
                                self.selected_peripheral.clone().unwrap().id().to_string();
                            let properties_result = task::block_on(
                                self.selected_peripheral.clone().unwrap().properties(),
                            );
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
                            egui::ComboBox::from_label("Choose a device.")
                                .selected_text(&self.peripheral_text)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.selected_peripheral_number,
                                        Some(0 as usize),
                                        name_str,
                                    );
                                });
                        } else {
                            egui::ComboBox::from_label("Choose a device.")
                                .selected_text(&self.peripheral_text)
                                .show_ui(ui, |ui| match &self.peripheral_list {
                                    Some(peripherals) => {
                                        for (i, peripheral) in peripherals.iter().enumerate() {
                                            let mut name_str = peripheral.id().to_string();
                                            let properties_result =
                                                task::block_on(peripheral.properties());
                                            match properties_result {
                                                Ok(prop_option) => {
                                                    if prop_option.is_some() {
                                                        if prop_option
                                                            .clone()
                                                            .unwrap()
                                                            .local_name
                                                            .is_some()
                                                        {
                                                            name_str = prop_option
                                                                .clone()
                                                                .unwrap()
                                                                .local_name
                                                                .unwrap();
                                                        }
                                                    }
                                                }
                                                Err(_) => {}
                                            }
                                            ui.selectable_value(
                                                &mut self.selected_peripheral_number,
                                                Some(i),
                                                name_str,
                                            );
                                        }
                                    }
                                    None => {
                                        ui.selectable_value(
                                            &mut self.selected_peripheral_number,
                                            None,
                                            "None",
                                        );
                                    }
                                });
                        }
                        if ui.button("Connect").clicked() {
                            if self.peripheral_moved {
                                println!("Connecting to device...");
                                match task::block_on(
                                    self.selected_peripheral.clone().unwrap().connect(),
                                ) {
                                    Ok(()) => {
                                        println!("Device connected.");
                                        self.peripheral_connected = true;
                                    }
                                    Err(e) => {
                                        println!("Failed to connect.  {:?}", e);
                                    }
                                }
                            } else {
                                println!("Please scan for devices and select one to connect");
                            }
                        }
                    });
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
fn update_peripheral_text(peripheral: &Peripheral) -> String {
    let peripheral_str = peripheral.id().to_string();
    return peripheral_str;
}

/// draws the workout tabe
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
            }
        }
    });
}
