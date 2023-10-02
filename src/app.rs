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
    selected_adapter: Option<Pin<Box<Adapter>>>,
    selected_adapter_number: Option<usize>,
    adapter_text: String,
    peripheral_list: Option<Vec<Pin<Box<Peripheral>>>>,
    selected_peripheral_number: Option<usize>,
}

impl Default for BikeApp {
    fn default() -> Self {
        Self {
            active_tab: Tabs::Main,
            bt_adapters: None,
            selected_adapter: None,
            selected_adapter_number: None,
            adapter_text: "None selected".to_string(),
            peripheral_list: None,
            selected_peripheral_number: None,
        }
    }
}

impl eframe::App for BikeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
                                            adapter.identifier().unwrap(),
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
                        // TODO: this shit sucks, IDK what to do here
                        /*
                        // update adapter variable based on selected number
                        if self.selected_adapter_number.is_some() {
                            self.selected_adapter = Some(
                                self.bt_adapters.as_ref().unwrap()
                                    [self.selected_adapter_number.clone().unwrap()]
                                .clone(),
                            );
                        }
                        */
                        if ui.button("Scan").clicked() {
                            if self.selected_adapter.is_some() {
                                self.peripheral_list =
                                    bt_scan(&mut self.selected_adapter.as_mut().unwrap());
                            }
                        }
                    });
                }
                Tabs::Help => {}
            }
        });
    }
}
