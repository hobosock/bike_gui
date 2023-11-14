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

// Cycling Power Feature [https://www.bluetooth.com/specifications/specs/cycling-power-service-1-1/] (Section 4.4)
// Table 4.5 AND Table 4.6???
bitfield! {
    pub struct CpsFeature(pub u32): Debug {
        pub pedal_power_balance_supported: bool @ 0,
        pub accumulated_torque_supported: bool @ 1,
        pub wheel_revolution_data_supported: bool @ 2,
        pub crank_revolution_data_supported: bool @ 3,
        pub extreme_magnitudes_supported: bool @ 4,
        pub extreme_angles_supported: bool @ 5,
        pub dead_spot_angles_supported: bool @ 6,
        pub accumulated_energy_supported: bool @ 7,
        pub offset_compensation_indicator_supported: bool @ 8,
        pub sensor_measurement_context_0: bool @ 9,
        pub sensor_measurement_context_1: bool @ 10,
        pub instantaneous_measurement_direction_supported: bool @ 11,
        pub offset_compensation_supported: bool @ 12,
        pub cycling_power_measurement_characteristic_content_masking_supported: bool @ 13,
        pub multiple_sensor_locations_supported: bool @ 14,
        pub crank_length_adjustment_supported: bool @ 15,
        pub chain_length_adjustment_supported: bool @ 16,
        pub chain_weight_adjustment_supported: bool @ 17,
        pub span_length_adjustment_supported: bool @ 18,
        pub factory_calibration_date_supported: bool @ 19,
        pub enhanced_offset_compensation_supported: bool @ 20,
    }
}

// Cycling Power Measurement [https://www.bluetooth.com/specifications/specs/cycling-power-service-1-1/] (Section 4.5 3.2)
// Cycling Power Control Point [https://www.bluetooth.com/specifications/specs/cycling-power-service-1-1/] (Section 4.7)
// Mask Cycling Power Measurement Characteristic Content Procedure (4.7.2.13)

bitfield! {
    pub struct CpsFlag(pub u16): Debug {
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
