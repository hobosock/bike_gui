use std::{
    collections::BTreeSet,
    sync::mpsc::{Receiver, Sender},
};

/*=======================================================================
 * IMPORTS
 * ====================================================================*/
use async_std::stream::StreamExt;
use btleplug::{
    api::{Characteristic, Peripheral as Peripheral_api},
    platform::Peripheral,
};
use std::error::Error;

use super::cps::CpsFeature;

/*=======================================================================
 * ENUMS
 * ====================================================================*/
#[derive(Clone, Debug)]
pub enum BtAction {
    IsConnected,
    Read,
    Subscribe,
    Notifications,
    Connect,
    Disconnect,
    Discover,
    Properties,
}

#[derive(Clone, Debug)]
pub enum CharReadType {
    CPSPowerFeature,
    CPSPowerReading,
    CPSControlPoint,
}

/*=======================================================================
 * STRUCTS
 * ====================================================================*/
#[derive(Clone, Debug)]
pub struct QueueItem {
    pub action: BtAction,
    pub peripheral: Peripheral,
    pub characteristic: Option<Characteristic>,
    pub read_type: Option<CharReadType>,
}

/// channels that bluetooth queue thread will use to send information back
/// to main GUI thread
#[derive(Clone, Debug)]
pub struct QueueChannels {
    pub is_connected: Sender<bool>,
    pub peripherals: Sender<Option<Vec<Peripheral>>>, // TODO: remove option???
    pub subscribed: Sender<bool>,
    pub cps_power_reading: Sender<Vec<u8>>,
    pub cps_features: Sender<CpsFeature>,
    pub results: Sender<String>,
    pub characteristics: Sender<BTreeSet<Characteristic>>,
    pub peripheral_name: Sender<String>,
}

/*=======================================================================
 * FUNCTIONS
 * ====================================================================*/
/// function to process queue (vec of QueueItem)
/// takes a mutable reference to a vector, pops elements to process them
/// exits when queue is cleared, will need to be restarted from caller
// TODO: this seems like a bad idea? not using it
pub async fn process_queue(
    queue: &mut Vec<QueueItem>,
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
    action: QueueItem,
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
            // safe enough to unwrap here, never ask for read without a read type
            match action.read_type.unwrap() {
                CharReadType::CPSPowerFeature => {
                    let reading = peripheral.read(&action.characteristic.unwrap()).await?;
                    let buffer = u32::from_le_bytes(reading.try_into().unwrap());
                    let feature_struct = CpsFeature(buffer);
                    println!("CPS Feature: {:?}", feature_struct);
                }
                CharReadType::CPSPowerReading => {
                    // TODO: can't actually read this, delete later
                    let reading = peripheral.read(&action.characteristic.unwrap()).await?;
                    // TODO: can program crash here?
                    let buffer = u32::from_le_bytes(reading.try_into().unwrap());
                    let read_struct = CpsFeature(buffer);
                    channels.cps_features.send(read_struct)?;
                }
                CharReadType::CPSControlPoint => {}
            }
            return Ok(());
        }
        BtAction::Subscribe => {
            let subscribe_result = peripheral
                .subscribe(&action.characteristic.clone().unwrap())
                .await?;
            println!(
                "Subscribed to characteristic: {:?}",
                action.characteristic.clone().unwrap()
            );
            channels.subscribed.send(true)?;
            return Ok(subscribe_result);
        }
        BtAction::Notifications => {
            let notification = peripheral.notifications().await?;
            let mut reading = notification.take(1);
            while let Some(data) = reading.next().await {
                println!("Reading: {:?}", data.value);
                // send reading
                channels.cps_power_reading.send(data.value)?;
            }
            return Ok(());
        }
        BtAction::Connect => {
            println!("Connecting to {:?}...", peripheral);
            peripheral.connect().await?;
            println!("Connected.");
            channels.is_connected.send(true)?;
            return Ok(());
        }
        BtAction::Disconnect => {
            println!("Disconnecting from {:?}...", peripheral);
            peripheral.disconnect().await?;
            println!("Disconnected.");
            channels.is_connected.send(false)?;
            return Ok(());
        }
        BtAction::Discover => {
            println!("Discovering services...");
            let _ = peripheral.discover_services();
            let characteristics = peripheral.characteristics();
            let _ = channels.characteristics.send(characteristics); // TODO: send error handling
            return Ok(());
        }
        BtAction::Properties => {
            println!("Getting peripheral properties...");
            let mut peripheral_str = peripheral.id().to_string();
            match peripheral.properties().await {
                Ok(properties) => match properties {
                    Some(prop) => match prop.local_name {
                        Some(name) => peripheral_str = name,
                        None => {}
                    },
                    None => {}
                },
                Err(_) => {}
            }
            let _ = channels.peripheral_name.send(peripheral_str); // TODO: send error handling
            return Ok(());
        }
    }
}

/// bluetooth queue main loop
pub async fn bt_q_main(rx: Receiver<QueueItem>, channels: QueueChannels, kill: Receiver<bool>) {
    let mut queue: Vec<QueueItem> = Vec::new();
    let mut stop_loop = false;
    loop {
        // receive kill signal from main GUI thread
        if let Ok(b) = kill.try_recv() {
            stop_loop = b;
        }
        if stop_loop {
            break;
        }

        // otherwise keep processing queue
        // first, check for new actions to add
        if let Ok(q) = rx.try_recv() {
            queue.push(q);
            // TODO: move certain items up in the queue, ie. prioritize reconnects
        }

        // then process first item on the queue
        if queue.len() >= 1 {
            let proc_item = queue.remove(0); // TODO: use vecdequeue pop_front instead?
            let proc_result =
                process_queue_item(proc_item.clone(), &proc_item.peripheral, channels.clone())
                    .await;
            // TODO: honestly not sure why I need to "handle a result" here, I think maybe it's about the
            // sends potentially failing?  I don't think I care if they do
            let _ = match proc_result {
                Ok(()) => channels.results.send(format!("{:?} Ok!", proc_item)),
                Err(e) => channels.results.send(e.to_string()),
            };
        }
    }
    // NOTE: hopefully items get added to the queue slower than they are removed?  I have no idea
    // what I'm doing here
}
