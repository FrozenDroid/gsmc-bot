use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub struct LiveData {
    pub best_s1: f32,
    pub best_s2: f32,
    pub best_s3: f32,
    pub best_laptime: f32,
    #[serde(rename = "mScoringInfo")]
    pub m_scoring_info: ScoringInfo,
    #[serde(rename = "mVehicles")]
    pub m_vehicles: Vec<Vehicle>,
    pub show_ai_fuel_data: i32,
    pub show_ai_tire_wear: i32,
    pub show_class: i32,
    pub show_vehicle: i32,
    pub show_avg_lap_speed: i32,
    pub show_mph: i32,
    pub server_list: Vec<Server>,
    pub server_names_list: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScoringInfo {
    pub m_server_name: String,
    pub m_track_name: String,
    pub m_session: i32,
    #[serde(rename = "mCurrentET")]
    pub m_current_et: f32,
    #[serde(rename = "mEndET")]
    pub m_end_et: f32,
    pub m_max_laps: i32,
    pub m_lap_dist: f32,
    pub m_num_vehicles: i32,
    pub m_game_phase: i32,
    pub m_yellow_flag_state: i32,
    pub m_sector_flag: Vec<i32>,
    pub m_raining: f32,
    pub m_ambient_temp: f32,
    pub m_track_temp: f32,
    pub m_min_path_wetness: f32,
    pub m_max_path_wetness: f32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Server {
    pub pid: i32,
    pub name: String,
    pub num: i32,
    pub label_name: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Vehicle {
    #[serde(rename = "mID")]
    pub m_id: i32,
    pub m_driver_name: String,
    pub m_vehicle_name: String,
    pub m_total_laps: i32,
    pub m_sector: i32,
    pub m_finish_status: i32,
    pub m_lap_dist: f32,
    pub m_best_sector1: f32,
    pub m_best_sector2: f32,
    pub m_best_lap_time: f32,
    pub m_last_sector1: f32,
    pub m_last_sector2: f32,
    pub m_last_lap_time: f32,
    pub m_cur_sector1: f32,
    pub m_cur_sector2: f32,
    pub m_num_pitstops: String,
    pub m_is_player: i32,
    pub m_control: i32,
    pub m_in_pits: i32,
    pub m_place: i32,
    pub m_vehicle_class: String,
    pub m_time_behind_next: f32,
    pub m_time_behind_leader: f32,
    pub m_laps_behind_leader: i32,
    pub m_in_garage_stall: i32,
    pub m_front_tire_compound_name: String,
    pub m_rear_tire_compound_name: String,
    pub m_rear_tire_compound_index: i32,
    pub m_front_tire_compound_index: i32,
    pub m_fuel: f32,
    pub m_wear: Vec<f32>,
    pub speed: f32,
}
