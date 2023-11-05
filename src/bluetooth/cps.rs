/*=======================================================================
 * IMPORTS
 * ====================================================================*/
// external crates
use uuid::{uuid, Uuid};

/*=======================================================================
 * CONSTANTS
 * ====================================================================*/
pub const CPS_POWER_MEASUREMENT: Uuid = uuid!("00002a63-0000-1000-8000-00805f9b34fb");
pub const CPS_POWER_VECTOR: Uuid = uuid!("00002a64-0000-1000-8000-00805f9b34fb");
pub const CPS_POWER_FEATURE: Uuid = uuid!("00002a65-0000-1000-8000-00805f9b34fb");
pub const CPS_CONTROL_POINT: Uuid = uuid!("00002a66-0000-1000-8000-00805f9b34fb");

/*=======================================================================
 * STRUCTS
 * ====================================================================*/
pub struct CpsFlag {
    pub pedal_power_balance_present: bool,
    pub pedal_power_balance_reference: bool,
    pub accumulated_torque_present: bool,
    pub accumulated_torque_source: bool,
    pub wheel_revolution_data_present: bool,
    pub crank_revolution_data_present: bool,
    pub extreme_force_magnitudes_present: bool,
    pub extreme_torque_magnitudes_present: bool,
    pub extreme_angles_present: bool,
    pub top_dead_spot_angle_present: bool,
    pub bottom_dead_spot_angle_present: bool,
    pub accumulated_energy_present: bool,
    pub offset_compensation_indicator: bool,
}

/*=======================================================================
 * FUNCTIONS
 * ====================================================================*/
