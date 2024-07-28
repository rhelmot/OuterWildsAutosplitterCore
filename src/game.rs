use asr::{
    watcher::{Pair, Watcher},
    Process,
    game_engine::unity::mono::{Image, Module, Version},
    deep_pointer::DeepPointer,
};
use std::sync::Mutex;

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
        let static_cls = image.get_class(process, module, static_cls_name)?;
        let static_cls_addr = static_cls.get_static_table(process, module)?;
        let static_field_offset = static_cls.get_field_offset(process, module, static_field)?;
        let mut offsets = vec![static_field_offset as u64];
        for (dyn_class_name, dyn_class_field) in dynamic_class_fields {
            if dyn_class_name.is_empty() {
                offsets.push(dyn_class_field.parse().unwrap())
            } else {
                let dyn_cls = image.get_class(process, module, dyn_class_name)?;
                let dyn_field_offset = dyn_cls.get_field_offset(process, module, dyn_class_field)?;
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
            load: false,
            menu: false,
        })
    }
}

impl State {
    pub fn update(&mut self, process: &Process) -> Option<Variables> {
        let mut v = Variables {
            pauses: self.pauses.update(process),
            campfire_sleep: self.campfire_sleep.update(process),
            exiting_dream: self.exiting_dream.update(process),
            scene: self.scene.update(process),
            scene_current: self.scene_current.update(process),
            fade_type: self.fade_type.update(process),
            allow_async: self.allow_async.update(process),
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
}
