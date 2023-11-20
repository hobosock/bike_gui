/*=======================================================================
 * IMPORTS
 * ====================================================================*/
use btleplug::api::{Central, Manager as Manager_api, ScanFilter};
use btleplug::platform::{Adapter, Manager, Peripheral};
use std::time::Duration;
use tokio::time::{self};

pub mod ble_default_services;
pub mod cps;
pub mod cscs;
pub mod queue;

/*=======================================================================
 * CONSTANTS
 * ====================================================================*/

/*=======================================================================
 * FUNCTIONS
 * ====================================================================*/
pub async fn bt_adapter_scan() -> Option<Vec<Adapter>> {
    println!("Checking for bluetooth manager...");
    let manager_result = Manager::new().await;
    match manager_result {
        Ok(manager) => {
            println!("Checking for Bluetooth adapter...");
            let adapter_result = manager.adapters().await;
            match adapter_result {
                Ok(adapters) => {
                    return Some(adapters);
                }
                Err(e) => {
                    println!("No adapters found: {:?}", e);
                    return None;
                }
            }
        }
        Err(e) => {
            println!("Error finding Bluetooth manager: {:?}", e);
            return None;
        }
    }
}

pub async fn bt_scan(adapter: &Adapter) -> Option<Vec<Peripheral>> {
    println!("Scanning for peripherals...");
    let scan_results = adapter.start_scan(ScanFilter::default()).await;
    time::sleep(Duration::from_secs(10)).await;
    println!("Here are the results:");

    match scan_results {
        Ok(()) => {
            let peripherals = adapter.peripherals().await.unwrap();
            println!("{:?}", peripherals);
            return Some(peripherals);
        }
        Err(e) => {
            println!("Error scanning for peripherals: {:?}", e);
            return None;
        }
    }
}
