/*=======================================================================
 * IMPORTS
 * ====================================================================*/
use btleplug::api::{Central, Manager as Manager_api, Peripheral as Peripheral_api, ScanFilter};
use btleplug::platform::{Adapter, Manager, Peripheral};
use std::time::Duration;
use tokio::time::{self, sleep, timeout};

pub mod ble_default_services;

/*=======================================================================
 * CONSTANTS
 * ====================================================================*/
const TIMEOUT: Duration = Duration::from_secs(10);

/*=======================================================================
 * FUNCTIONS
 * ====================================================================*/
pub async fn disconnect_with_timeout(peripheral: &btleplug::platform::Peripheral) {
    match timeout(TIMEOUT, peripheral.is_connected()).await {
        Ok(Ok(false)) => {
            return;
        }
        e => {
            println!("Lost peripheral connection: {:?}", e);
        }
    }

    loop {
        if let Err(e) = timeout(TIMEOUT, peripheral.disconnect()).await {
            println!("Error while disconnecting, trying again... {:?}", e);
        } else {
            break;
        }

        sleep(Duration::from_secs(5)).await;
    }
}

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
