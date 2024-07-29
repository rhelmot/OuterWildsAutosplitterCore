#![allow(unused_assignments)]
use asr::{settings::Gui, timer::{self, TimerState}};
use std::sync::Mutex;

pub mod game;
pub mod settings;

use game::GameProcess;
use settings::{Settings, StickyState};

struct Globals {
    game: Option<GameProcess>,
    settings: Settings,
    sticky_state: StickyState,
}

static GLOBALS: Mutex<Option<Globals>> = Mutex::new(None);


#[no_mangle]
pub extern "C" fn update() {
    let mut mutex = GLOBALS.lock().unwrap();
    let mut game_open: bool = true; // Used to stop and resume game time on game launch/exit
    if mutex.is_none() {
        *mutex = Some(Globals {
            game: None,
            settings: Settings::register(),
            sticky_state: StickyState::default(),
        })
    }
    let globals = mutex.as_mut().unwrap();

    globals.settings.update();

    if timer::state() == TimerState::NotRunning {
        globals.sticky_state = StickyState::default();
    }

    if globals.game.is_none() {
        // (Re)connect to the game and unpause game time
        globals.game = GameProcess::connect("OuterWilds.exe");
    } else {
        let game = globals.game.as_mut().unwrap();

        // Make sure we're still connected to the game, pause game time if not
        if !game.process.is_open() {
            *mutex = None;
            if game_open == true {
                timer::pause_game_time();
                game_open = false;
            }

            return;
        }

        let vars = match game.state.update(&mut game.process, &mut globals.sticky_state) {
            Some(v) => v,
            None => {
                asr::print_message("Error updating state!");
                return;
            }
        };

        // Watch for game opening, and resume timer if it is open again
        if game.process.is_open() && game_open == false {
            timer::resume_game_time();
            game_open = true;
        }

        // business logic
        if vars.loading() {
            timer::pause_game_time();
        } else {
            timer::resume_game_time();
        }
        if vars.starting() {
            timer::start();
        }
        if vars.resetting(&globals.settings) {
            timer::reset();
        }
        if vars.split(&globals.settings, &mut globals.sticky_state) {
            timer::split();
        }
    }
}
