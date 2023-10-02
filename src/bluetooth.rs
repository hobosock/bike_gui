/*=======================================================================
 * IMPORTS
 * ====================================================================*/
use simplersble::Adapter;
use std::pin::Pin;

/*=======================================================================
 * FUNCTIONS
 * ====================================================================*/
pub fn bt_scan() -> Option<Vec<Pin<Box<Adapter>>>> {
    let mut adapter_result = simplersble::Adapter::get_adapters();
    match adapter_result {
        Ok(adapters) => return Some(adapters),
        Err(_) => return None,
    }
}
