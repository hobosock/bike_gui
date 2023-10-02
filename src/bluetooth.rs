/*=======================================================================
 * IMPORTS
 * ====================================================================*/
use simplersble::{Adapter, Peripheral};
use std::pin::Pin;

/*=======================================================================
 * FUNCTIONS
 * ====================================================================*/
pub fn bt_adapter_scan() -> Option<Vec<Pin<Box<Adapter>>>> {
    let adapter_result = simplersble::Adapter::get_adapters();
    match adapter_result {
        Ok(adapters) => return Some(adapters),
        Err(_) => return None,
    }
}

pub fn bt_scan(adapter: &mut Pin<Box<Adapter>>) -> Option<Vec<Pin<Box<Peripheral>>>> {
    adapter.set_callback_on_scan_start(Box::new(|| {
        println!("Scan started.");
    }));
    adapter.set_callback_on_scan_stop(Box::new(|| {
        println!("Scan stopped.");
    }));

    adapter.scan_for(5000).unwrap();
    println!("Scan complete.");

    match adapter.scan_get_results() {
        Ok(peripherals) => return Some(peripherals),
        Err(_) => return None,
    }
}
