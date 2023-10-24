/*=======================================================================
 * IMPORTS
 * ====================================================================*/
// local files
use crate::bluetooth::*;

// external crates
use btleplug::{
    api::{Central, Peripheral as Peripheral_api},
    platform::{Adapter, Manager, Peripheral, PeripheralId},
};
use eframe::egui;

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
        }
    }
}

impl eframe::App for BikeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // update state
        if self.bt_adapters.is_some() && self.selected_adapter_number.is_some() {
            if self.adapter_moved == false {
                async {
                    self.adapter_text = update_adapter_text(
                        &self.bt_adapters.as_ref().unwrap()
                            [self.selected_adapter_number.clone().unwrap()],
                    )
                    .await;
                };
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
            async {
                match self
                    .selected_peripheral
                    .clone()
                    .unwrap()
                    .is_connected()
                    .await
                {
                    Ok(flag) => self.peripheral_connected = flag,
                    Err(_) => {}
                }
            };
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
                        async {
                            let peripheral = self.selected_peripheral.clone().unwrap();
                            peripheral.discover_services().await;
                            let characteristics = peripheral.characteristics();
                            for chars in characteristics.iter() {
                                ui.add_sized(
                                    ui.available_size(),
                                    egui::TextEdit::singleline(&mut chars.uuid.to_string()),
                                );
                            }
                        };
                    }
                }
                Tabs::Workouts => {}
                Tabs::Bluetooth => {
                    ui.horizontal(|ui| {
                        if ui.button("Adapter").clicked() {
                            async {
                                self.bt_adapters = bt_adapter_scan().await;
                            };
                        }
                        if self.adapter_moved {
                            async {
                                let adapter_info_str: String;
                                match self.selected_adapter.clone().unwrap().adapter_info().await {
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
                            };
                        } else {
                            egui::ComboBox::from_label("Choose an adapter.")
                                .selected_text(&self.adapter_text)
                                .show_ui(ui, |ui| match &self.bt_adapters {
                                    Some(adapters) => {
                                        for (i, adapter) in adapters.iter().enumerate() {
                                            async {
                                                let adapter_info_str: String;
                                                match adapter.adapter_info().await {
                                                    Ok(info) => adapter_info_str = info,
                                                    Err(e) => adapter_info_str = e.to_string(),
                                                }
                                                ui.selectable_value(
                                                    &mut self.selected_adapter_number,
                                                    Some(i),
                                                    adapter_info_str,
                                                );
                                            };
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
                                async {
                                    println!("Scanning for devices...");
                                    self.peripheral_list =
                                        bt_scan(&mut self.selected_adapter.as_mut().unwrap()).await;
                                };
                            }
                        }
                        if self.peripheral_moved {
                            egui::ComboBox::from_label("Choose a device.")
                                .selected_text(&self.peripheral_text)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.selected_peripheral_number,
                                        Some(0 as usize),
                                        self.selected_peripheral.clone().unwrap().id().to_string(),
                                    );
                                });
                        } else {
                            egui::ComboBox::from_label("Choose a device.")
                                .selected_text(&self.peripheral_text)
                                .show_ui(ui, |ui| match &self.peripheral_list {
                                    Some(peripherals) => {
                                        for (i, peripheral) in peripherals.iter().enumerate() {
                                            ui.selectable_value(
                                                &mut self.selected_peripheral_number,
                                                Some(i),
                                                peripheral.id().to_string(),
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
