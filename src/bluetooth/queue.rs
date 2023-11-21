use std::sync::mpsc::Sender;

/*=======================================================================
 * IMPORTS
 * ====================================================================*/
use async_std::stream::StreamExt;
use btleplug::{
    api::{Characteristic, Peripheral as Peripheral_api},
    platform::Peripheral,
    Error as BtleError,
};
use std::error::Error;

use super::cps::CpsFeature;

/*=======================================================================
 * ENUMS
 * ====================================================================*/
pub enum BtAction {
    IsConnected,
    Read,
    Subscribe,
    Notifications,
    Connect,
    Disconnect,
}

/*=======================================================================
 * STRUCTS
 * ====================================================================*/
pub struct QueueItem<'a> {
    pub action: BtAction,
    pub characteristic: &'a Characteristic,
}

#[derive(Clone)]
pub struct QueueChannels {
    pub is_connected: Sender<bool>,
    pub peripherals: Sender<Option<Vec<Peripheral>>>, // TODO: remove option???
    pub cps_power_reading: Sender<Vec<u8>>,
    pub cps_features: Sender<CpsFeature>,
}

/*=======================================================================
 * FUNCTIONS
 * ====================================================================*/
/// function to process queue (vec of QueueItem)
/// takes a mutable reference to a vector, pops elements to process them
/// exits when queue is cleared, will need to be restarted from caller
pub async fn process_queue(
    queue: &mut Vec<QueueItem<'_>>,
    peripheral: &Peripheral,
    channels: QueueChannels,
) {
    while queue.len() > 0 {
        let pop_result = queue.pop();
        match pop_result {
            Some(action) => {
                let process_result = process_queue_item(action, peripheral, channels.clone());
            }
            None => {
                println!("Nothing popped.  Queue probably empty.");
            }
        }
    }
}

// TODO: update app_struct or something??? use channels
/// processes individual bluetooth actions from queue
async fn process_queue_item(
    action: QueueItem<'_>,
    peripheral: &Peripheral,
    channels: QueueChannels,
) -> Result<(), Box<dyn Error>> {
    match action.action {
        BtAction::IsConnected => {
            let is_con = peripheral.is_connected().await?;
            channels.is_connected.send(is_con)?;
            return Ok(());
        }
        BtAction::Read => {
            let reading = peripheral.read(action.characteristic).await?;
            // TODO: can program crash here?
            let buffer = u32::from_le_bytes(reading.try_into().unwrap());
            let read_struct = CpsFeature(buffer);
            channels.cps_features.send(read_struct)?;
            return Ok(());
        }
        BtAction::Subscribe => {
            let subscribe_result = peripheral.subscribe(action.characteristic).await?;
            println!("Subscribed to characteristic: {:?}", action.characteristic);
            return Ok(subscribe_result);
        }
        BtAction::Notifications => {
            let notification = peripheral.notifications().await?;
            let mut reading = notification.take(1);
            while let Some(data) = reading.next().await {
                println!("Reading: {:?}", data.value);
                // send reading
            }
            return Ok(());
        }
        BtAction::Connect => {
            println!("Connecting to {:?}...", peripheral);
            peripheral.connect().await?;
            println!("Connected.");
            return Ok(());
        }
        BtAction::Disconnect => {
            println!("Disconnecting from {:?}...", peripheral);
            peripheral.disconnect().await?;
            println!("Disconnected.");
            return Ok(());
        }
    }
}
