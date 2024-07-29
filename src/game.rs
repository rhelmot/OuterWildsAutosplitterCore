use asr::{
    watcher::{Pair, Watcher},
    Process,
    game_engine::unity::mono::{Image, Module, Version},
    deep_pointer::DeepPointer,
};
use std::sync::Mutex;

use crate::settings::{Settings, StickyState};

pub struct GameProcess {
    pub process: Process,
    pub state: State,
}

static WAITING: Mutex<Option<u32>> = Mutex::new(None);

impl GameProcess {
    pub fn connect(process_name: &str) -> Option<Self> {
        let mut ctr = WAITING.lock().unwrap();
        let (process, module, image) = match ctr.as_ref() {
            Some(d) => {
                if *d > 0 {
                    *ctr = Some(d - 1);
                    return None;
                }
                let process = Process::attach(process_name)?;
                let module = Module::attach(&process, Version::V2)?;
                let image = module.get_default_image(&process)?;
                *ctr = None;
                (process, module, image)
            }
            None => {
                let process = Process::attach(process_name)?;
                let module = Module::attach(&process, Version::V2)?;
                module.get_default_image(&process)?;
                *ctr = Some(300);
                return None;
            }
        };

        let state = State::setup(&process, &module, &image)?;
        Some(Self {
            process,
            state,
        })
    }
}

pub struct Variable<T, const CAP: usize> {
    var: Watcher<T>,
    pointer: DeepPointer<CAP>,
}

impl<T: bytemuck::Pod + std::fmt::Debug + Default, const CAP: usize> Variable<T, CAP> {
    pub fn update(&mut self, process: &Process) -> &Pair<T> {
        self.var.update_infallible(self.pointer.deref(process).unwrap_or_default())
    }

    fn new(static_cls_name: &'static str, static_field: &'static str, dynamic_class_fields: &[(&'static str, &'static str)], process: &Process, module: &Module, image: &Image) -> Option<Self> {
        let Some(static_cls) = image.get_class(process, module, static_cls_name) else {
            asr::print_message(&format!("Failed to get static class for {}", static_cls_name));
            return None;
        };
        let Some(static_cls_addr) = static_cls.get_static_table(process, module) else {
            asr::print_message(&format!("Failed to get static class table for {}", static_cls_name));
            return None;
        };
        let Some(static_field_offset) = static_cls.get_field_offset(process, module, static_field) else {
            asr::print_message(&format!("Failed to get static field offset for {}.{}", static_cls_name, static_field));
            return None;
        };
        let mut offsets = vec![static_field_offset as u64];
        for (dyn_class_name, dyn_class_field) in dynamic_class_fields {
            if dyn_class_name.is_empty() {
                offsets.push(dyn_class_field.parse().unwrap())
            } else {
                let Some(dyn_cls) = image.get_class(process, module, dyn_class_name) else {
                    asr::print_message(&format!("Failed to get dynamic class for {}", dyn_class_name));
                    return None;
                };
                let Some(dyn_field_offset) = dyn_cls.get_field_offset(process, module, dyn_class_field) else {
                    asr::print_message(&format!("Failed to get dynamic field offset for {}.{}", dyn_class_name, dyn_class_field));
                    return None;
                };
                offsets.push(dyn_field_offset as u64);
            }
        }
        let pointer = DeepPointer::new_32bit(static_cls_addr, &offsets);
        Some(Self {
            var: Watcher::new(),
            pointer,
        })
    }
}

pub struct State {
    pub pauses: Variable<[u8; 7], 2>,
    pub campfire_sleep: Variable<u8, 2>,
    pub exiting_dream: Variable<u8, 2>,
    pub scene: Variable<i32, 1>,
    pub scene_current: Variable<i32, 1>,
    pub fade_type: Variable<i32, 1>,
    pub allow_async: Variable<u8, 1>,
    pub death_type: Variable<i32, 2>,
    pub is_reality_shatter_effect_complete: Variable<u8, 3>,
    pub is_dying: Variable<u8, 2>,
    pub is_wearing_suit: Variable<u8, 2>,
    pub in_warp_field: Variable<u8, 2>,
    pub held_item: Variable<i32, 4>,
    pub prompt_item: Variable<i32, 3>,
    pub in_bramble_dimension: Variable<u8, 2>,
    pub in_vessel_dimension: Variable<u8, 2>,
    pub in_quantum_moon: Variable<u8, 2>,
    pub eye_initialized: Variable<u8, 2>,
    pub eye_state: Variable<u8, 2>,
    pub load: bool,
    pub menu: bool,
}

impl State {
    fn setup(process: &Process, module: &Module, image: &Image) -> Option<Self> {
        Some(Self {
            pauses: Variable::new("OWTime", "s_pauseFlags", &[("", "32")], process, module, image)?,
            campfire_sleep: Variable::new("Locator", "_audioMixer", &[("OWAudioMixer", "_sleepingAtCampfire")], process, module, image)?,
            exiting_dream: Variable::new("Locator", "_dreamWorldController", &[("DreamWorldController", "_exitingDream")], process, module, image)?,
            scene: Variable::new("LoadManager", "s_loadingScene", &[], process, module, image)?,
            scene_current: Variable::new("LoadManager", "s_currentScene", &[], process, module, image)?,
            fade_type: Variable::new("LoadManager", "s_fadeType", &[], process, module, image)?,
            allow_async: Variable::new("LoadManager", "s_allowAsyncTransition", &[], process, module, image)?,
            death_type: Variable::new("Locator", "_deathManager", &[("DeathManager", "_deathType")], process, module, image)?,
            is_reality_shatter_effect_complete: Variable::new("Locator", "_timelineObliterationController", &[("TimelineObliterationController", "_cameraEffect"), ("PlayerCameraEffectController", "_isRealityShatterEffectComplete")], process, module, image)?,
            is_dying: Variable::new("Locator", "_deathManager", &[("DeathManager", "_isDying")], process, module, image)?,
            is_wearing_suit: Variable::new("Locator", "_playerController", &[("PlayerCharacterController", "_isWearingSuit")], process, module, image)?,
            in_warp_field: Variable::new("Locator", "_playerController", &[("PlayerCharacterController", "_inWarpField")], process, module, image)?,
            held_item: Variable::new("Locator", "_toolModeSwapper", &[("ToolModeSwapper", "_itemCarryTool"), ("ItemTool", "_heldItem"), ("OWItem", "_type")], process, module, image)?,
            prompt_item: Variable::new("Locator", "_toolModeSwapper", &[("ToolModeSwapper", "_itemCarryTool"), ("ItemTool", "_promptState")], process, module, image)?,
            in_bramble_dimension: Variable::new("Locator", "_playerSectorDetector", &[("PlayerSectorDetector", "_inBrambleDimension")], process, module, image)?,
            in_vessel_dimension: Variable::new("Locator", "_playerSectorDetector", &[("PlayerSectorDetector", "_inVesselDimension")], process, module, image)?,
            in_quantum_moon: Variable::new("Locator", "_quantumMoon", &[("QuantumMoon", "_isPlayerInside")], process, module, image)?,
            eye_initialized: Variable::new("Locator", "_eyeStateManager", &[("EyeStateManager", "_initialized")], process, module, image)?,
            eye_state: Variable::new("Locator", "_eyeStateManager", &[("EyeStateManager", "_state")], process, module, image)?,
            load: false,
            menu: false,
        })
    }
}

impl State {
    pub fn update(&mut self, process: &Process, sticky: &mut StickyState) -> Option<Variables> {
        let mut v = Variables {
            pauses: self.pauses.update(process),
            campfire_sleep: self.campfire_sleep.update(process),
            exiting_dream: self.exiting_dream.update(process),
            scene: self.scene.update(process),
            scene_current: self.scene_current.update(process),
            fade_type: self.fade_type.update(process),
            allow_async: self.allow_async.update(process),
            death_type: self.death_type.update(process),
            is_reality_shatter_effect_complete: self.is_reality_shatter_effect_complete.update(process),
            is_dying: self.is_dying.update(process),
            is_wearing_suit: self.is_wearing_suit.update(process),
            in_warp_field: self.in_warp_field.update(process),
            held_item: self.held_item.update(process),
            prompt_item: self.prompt_item.update(process),
            in_bramble_dimension: self.in_bramble_dimension.update(process),
            in_vessel_dimension: self.in_vessel_dimension.update(process),
            in_quantum_moon: self.in_quantum_moon.update(process),
            eye_initialized: self.eye_initialized.update(process),
            eye_state: self.eye_state.update(process),
            load: self.load,
            menu: self.menu,
        };
        if v.pauses.check(|t| t[4] == 0) {
            v.load = false;
        }
        if !v.menu && v.load_compare(0, 1, -1, 1, true) {
            v.menu = true;
        } else if v.menu && (v.pauses.check(|t| t[3] == 0) || v.load_compare(3, 0, 3, 1, true)) {
            v.menu = false;
            sticky.loop_counter += 1;
        } else if v.load_compare(2, 0, 2, 0, true) || v.load_compare(0, 3, 2, 2, false) {
            v.load = !v.menu;
        } else if v.load_compare(0, 2, 1, 1, false) || v.load_compare(0, 3, 1, 1, true) {
            v.load = true;
        }

        self.load = v.load;
        self.menu = v.menu;
        Some(v)
    }
}

pub struct Variables<'a> {
    pub pauses: &'a Pair<[u8; 7]>,
    pub campfire_sleep: &'a Pair<u8>,
    pub exiting_dream: &'a Pair<u8>,
    pub scene: &'a Pair<i32>,
    pub scene_current: &'a Pair<i32>,
    pub fade_type: &'a Pair<i32>,
    pub allow_async: &'a Pair<u8>,
    pub death_type: &'a Pair<i32>,
    pub is_reality_shatter_effect_complete: &'a Pair<u8>,
    pub is_dying: &'a Pair<u8>,
    pub is_wearing_suit: &'a Pair<u8>,
    pub in_warp_field: &'a Pair<u8>,
    pub held_item: &'a Pair<i32>,
    pub prompt_item: &'a Pair<i32>,
    pub in_bramble_dimension: &'a Pair<u8>,
    pub in_vessel_dimension: &'a Pair<u8>,
    pub in_quantum_moon: &'a Pair<u8>,
    pub eye_initialized: &'a Pair<u8>,
    pub eye_state: &'a Pair<u8>,
    pub load: bool,
    pub menu: bool,
}

impl<'a> Variables<'a> {
    pub fn loading(&self) -> bool {
        self.load || self.menu || (self.campfire_sleep.current != 0 && self.exiting_dream.current == 0)
    }

    pub fn starting(&self) -> bool {
        self.pauses.check(|t| t[3] == 0) || self.load_compare(0, 3, 1, 1, true)
    }

    fn load_compare(&self, loading_scene_old: i32, loading_scene_current: i32, current_scene: i32, fade_type: i32, async_transition: bool) -> bool {
        if loading_scene_old == self.scene.old && loading_scene_current == self.scene.current {
            return (current_scene == self.scene_current.current || current_scene == -1) && fade_type == self.fade_type.current && async_transition == (self.allow_async.current != 0);
        }
        false
    }

    pub fn split(&self, settings: &Settings, sticky: &mut StickyState) -> bool {
        if settings.menu_split && self.load_compare(0, 1, -1, 1, true) {
            return true;
        }
        if settings.big_bang && self.death_type.check(|t| *t == 6) {
            return true;
        }
        if settings.dst && self.is_reality_shatter_effect_complete.check(|t| *t != 0) {
            return true;
        }
        if settings.death_hp && self.is_dying.check(|t| *t != 0) && self.death_type.current == 0 {
            return true;
        }
        if settings.death_impact && self.death_type.check(|t| *t == 1) {
            return true;
        }
        if settings.death_oxygen && self.death_type.check(|t| *t == 2) {
            return true;
        }
        if settings.death_sun && self.death_type.check(|t| *t == 3) {
            return true;
        }
        if settings.death_supernova && self.death_type.check(|t| *t == 4) {
            return true;
        }
        if settings.death_fish && self.death_type.check(|t| *t == 5) {
            return true;
        }
        if settings.death_crushed && self.death_type.check(|t| *t == 7) {
            return true;
        }
        if settings.death_meditation && self.death_type.check(|t| *t == 8) {
            return true;
        }
        if settings.death_time_loop && self.death_type.check(|t| *t == 9) {
            return true;
        }
        if settings.death_lava && self.death_type.check(|t| *t == 10) {
            return true;
        }
        if settings.death_black_hole && self.death_type.check(|t| *t == 11) {
            return true;
        }
        if settings.death_dream && self.death_type.check(|t| *t == 12) {
            return true;
        }
        if settings.death_dream_explosion && self.death_type.check(|t| *t == 13) {
            return true;
        }
        if settings.death_elevator && self.death_type.check(|t| *t == 14) {
            return true;
        }
        if settings.sleep && !sticky.sleep && self.campfire_sleep.check(|t| *t != 0) {
            sticky.sleep = true;
            return true;
        }
        if settings.wear_suit && !sticky.wear_suit && self.is_wearing_suit.check(|t| *t != 0) {
            sticky.wear_suit = true;
            return true;
        }
        if self.in_warp_field.check(|t| *t != 0) && !sticky.warp_core {
            sticky.warp_core_loop = Some(sticky.loop_counter);
            if !sticky.first_warp {
                sticky.first_warp = true;
                return settings.first_warp;
            }
        }
        if !sticky.warp_core && ((self.held_item.current == 2 && self.prompt_item.old == 3 && self.prompt_item.current > 3) || ((self.held_item.current == 2 && self.prompt_item.old < 4 && self.prompt_item.current == 4))) && sticky.warp_core_loop == Some(sticky.loop_counter) {
            sticky.warp_core = true;
            return true;
        }
        if settings.exit_warp && !sticky.exit_warp && self.in_warp_field.check(|t| *t == 0) && sticky.warp_core && sticky.warp_core_loop == Some(sticky.loop_counter) {
            sticky.exit_warp = true;
            return true;
        }
        if settings.dark_bramble && !sticky.dark_bramble && self.in_bramble_dimension.check(|t| *t != 0) {
            sticky.dark_bramble = true;
            return true;
        }
        if settings.dark_bramble_vessel && !sticky.dark_bramble_vessel && self.in_vessel_dimension.check(|t| *t != 0) {
            sticky.dark_bramble_vessel = true;
            return true;
        }
        if settings.quantum_moon_in && !sticky.quantum_moon_in && self.in_quantum_moon.check(|t| *t != 0) {
            sticky.quantum_moon_in = true;
            return true;
        }
        if settings.vessel_warp && !sticky.vessel_warp && self.eye_initialized.check(|t| *t != 0) {
            sticky.vessel_warp = true;
            return true;
        }
        if settings.eye_surface && self.eye_state.check(|t| *t == 10) {
            return true;
        }
        if settings.eye_tunnel && self.eye_state.check(|t| *t == 20) {
            return true;
        }
        if settings.eye_observatory && self.eye_state.check(|t| *t == 40) {
            return true;
        }
        if settings.eye_map && self.eye_state.check(|t| *t == 50) {
            return true;
        }
        if settings.eye_instruments && self.eye_state.check(|t| *t == 80) {
            return true;
        }
        false
    }

    pub fn resetting(&self, settings: &Settings) -> bool {
        self.menu && settings.menu_reset
    }
}
