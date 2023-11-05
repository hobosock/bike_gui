/*=======================================================================
 * IMPORTS
 * ====================================================================*/
// external crates
use proc_bitfield::{self, bitfield};
use uuid::{uuid, Uuid};

/*======================================================@=================
 * CONSTANTS
 * ====================================================================*/
pub const CPS_POWER_MEASUREMENT: Uuid = uuid!("00002a63-0000-1000-8000-00805f9b34fb");
pub const CPS_POWER_VECTOR: Uuid = uuid!("00002a64-0000-1000-8000-00805f9b34fb");
pub const CPS_POWER_FEATURE: Uuid = uuid!("00002a65-0000-1000-8000-00805f9b34fb");
pub const CPS_CONTROL_POINT: Uuid = uuid!("00002a66-0000-1000-8000-00805f9b34fb");

/*=======================================================================
 * STRUCTS
 * ====================================================================*/
bitfield! {
    pub struct CpsFlag(u16): Debug {
        pub pedal_power_balance_present: bool @ 0,
        pub pedal_power_balance_reference: bool @ 1,
        pub accumulated_torque_present: bool @ 2,
        pub accumulated_torque_source: bool @ 3,
        pub wheel_revolution_data_present: bool @ 4,
        pub crank_revolution_data_present: bool @ 5,
        pub extreme_force_magnitudes_present: bool @ 6,
        pub extreme_torque_magnitudes_present: bool @ 7,
        pub extreme_angles_present: bool @ 8,
        pub top_dead_spot_angle_present: bool @ 9,
        pub bottom_dead_spot_angle_present: bool @ 10,
        pub accumulated_energy_present: bool @ 11,
        pub offset_compensation_indicator: bool @ 12,
    }
}

/*=======================================================================
 * FUNCTIONS
 * ====================================================================*/
