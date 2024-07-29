use asr::settings::{gui::Title, Gui};

#[derive(Gui)]
pub struct Settings {
    /// Options
    _general_options: Title,
    pub menu_split: bool,
    pub menu_reset: bool,

    /// General Splits
    _general_splits: Title,
    pub sleep: bool,
    pub wear_suit: bool,
    pub first_warp: bool,
    pub warp_core: bool,
    pub exit_warp: bool,
    pub dark_bramble: bool,
    pub dark_bramble_vessel: bool,
    pub quantum_moon_in: bool,
    pub vessel_warp: bool,
    pub big_bang: bool,
    pub dst: bool,

    /// Eye Splits
    _eye_splits: Title,
    pub eye_surface: bool,
    pub eye_tunnel: bool,
    pub eye_observatory: bool,
    pub eye_map: bool,
    pub eye_instruments: bool,

    /// Death Splits
    _death_splits: Title,
    pub death_hp: bool,
    pub death_impact: bool,
    pub death_oxygen: bool,
    pub death_sun: bool,
    pub death_supernova: bool,
    pub death_fish: bool,
    pub death_crushed: bool,
    pub death_elevator: bool,
    pub death_lava: bool,
    pub death_dream: bool,
    pub death_dream_explosion: bool,
    pub death_black_hole: bool,
    pub death_meditation: bool,
    pub death_time_loop: bool,
}

#[derive(Default)]
pub struct StickyState {
    pub sleep: bool,
    pub wear_suit: bool,
    pub first_warp: bool,
    pub exit_warp: bool,
    pub warp_core: bool,
    pub dark_bramble: bool,
    pub dark_bramble_vessel: bool,
    pub quantum_moon_in: bool,
    pub vessel_warp: bool,
    pub loop_counter: u32,
    pub warp_core_loop: Option<u32>,
}

