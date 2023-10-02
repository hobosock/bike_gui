/*=======================================================================
 * IMPORTS
 * ====================================================================*/
// local files
use crate::bluetooth::*;

// external crates
use eframe::egui;
use simplersble::{self, Adapter, Peripheral};
use std::pin::Pin;

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
pub struct BikeApp {
    // app state stuff
    active_tab: Tabs,
    // bluetooth stuff
    bt_adapters: Option<Vec<Pin<Box<Adapter>>>>,
    //selected_adapter: Option<Pin<Box<Adapter>>>,
    selected_adapter_number: Option<usize>,
    adapter_text: String,
    peripheral_list: Option<Vec<Pin<Box<Peripheral>>>>,
    //selected_peripheral: Option<Pin<Box<Peripheral>>>,
    selected_peripheral_number: Option<usize>,
    peripheral_text: String,
}

impl Default for BikeApp {
    fn default() -> Self {
        Self {
            active_tab: Tabs::Main,
            bt_adapters: None,
            //selected_adapter: None,
            selected_adapter_number: None,
            adapter_text: "None selected".to_string(),
            peripheral_list: None,
            //selected_peripheral: None,
            selected_peripheral_number: None,
            peripheral_text: "None_selected".to_string(),
        }
    }
}

impl eframe::App for BikeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // update state
        if self.bt_adapters.is_some() && self.selected_adapter_number.is_some() {
            self.adapter_text = update_adapter_text(
                &self.bt_adapters.as_ref().unwrap()[self.selected_adapter_number.clone().unwrap()],
            );
            /* this copies adapter/peripheral over to new field, but breaks on subsequent loops
            self.selected_adapter = Some(
                self.bt_adapters
                    .as_mut()
                    .unwrap()
                    .remove(self.selected_adapter_number.clone().unwrap()),
            );*/
        }
        if self.peripheral_list.is_some() && self.selected_peripheral_number.is_some() {
            self.peripheral_text = update_peripheral_text(
                &self.peripheral_list.as_ref().unwrap()
                    [self.selected_peripheral_number.clone().unwrap()],
            );
            /* this copies peripheral over to new field, but breaks on subsequent loops
            self.selected_peripheral = Some(
                self.peripheral_list
                    .as_mut()
                    .unwrap()
                    .remove(self.selected_peripheral_number.clone().unwrap()),
            );*/
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
                Tabs::Main => {}
                Tabs::Workouts => {}
                Tabs::Bluetooth => {
                    ui.horizontal(|ui| {
                        if ui.button("Adapter").clicked() {
                            self.bt_adapters = bt_adapter_scan();
                        }
                        egui::ComboBox::from_label("Choose an adapter.")
                            .selected_text(&self.adapter_text)
                            .show_ui(ui, |ui| match &self.bt_adapters {
                                Some(adapters) => {
                                    for (i, adapter) in adapters.iter().enumerate() {
                                        ui.selectable_value(
                                            &mut self.selected_adapter_number,
                                            Some(i),
                                            adapter.identifier().unwrap().to_string(),
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
                    });
                    ui.horizontal(|ui| {
                        if ui.button("Scan").clicked() {
                            if self.selected_adapter_number.is_some() {
                                println!("Scanning for devices...");
                                self.peripheral_list = bt_scan(
                                    &mut self.bt_adapters.as_mut().unwrap()
                                        [self.selected_adapter_number.clone().unwrap()],
                                );
                            }
                        };
                        egui::ComboBox::from_label("Choose a device.")
                            .selected_text(&self.peripheral_text)
                            .show_ui(ui, |ui| match &self.peripheral_list {
                                Some(peripherals) => {
                                    for (i, peripheral) in peripherals.iter().enumerate() {
                                        let name_str: String;
                                        if peripheral.identifier().unwrap().is_empty() {
                                            name_str = "Unknown device".to_string();
                                        } else {
                                            name_str = peripheral.identifier().unwrap();
                                            // could put UID here or something?
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
                        if ui.button("Connect").clicked() {
                            if self.selected_peripheral_number.is_some()
                                && self.peripheral_list.is_some()
                            {
                                println!("Connecting to device...");
                                // TODO: error handling here
                                //let peripheral = self.peripheral_list.as_mut().unwrap()
                                //    [self.selected_peripheral_number.clone().unwrap()];
                            } else {
                                println!("Please scan for devices and select one to connect.");
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
fn update_adapter_text(adapter: &Pin<Box<Adapter>>) -> String {
    let adapter_str = adapter.identifier().unwrap().to_string();
    return adapter_str;
}

/// update bluetooth peripheral combobox text based on selection
fn update_peripheral_text(peripheral: &Pin<Box<Peripheral>>) -> String {
    let peripheral_str: String;
    if peripheral.identifier().unwrap().is_empty() {
        peripheral_str = "Unknown device".to_string();
    } else {
        peripheral_str = peripheral.identifier().unwrap().to_string();
    }
    return peripheral_str;
}
