//#![cfg_attr(not(debug_assertions), window_subsystem = "windows")] // hide consolewindow on Windows

/*=======================================================================
 * IMPORTS
 * ====================================================================*/
// project imports
use app::*;
mod app;
mod bluetooth;

// external crates
use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(500.0, 300.0)),
        ..Default::default()
    };
    eframe::run_native("Bike", options, Box::new(|_cc| Box::<BikeApp>::default()))
}
