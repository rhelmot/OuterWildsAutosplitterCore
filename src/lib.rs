#![allow(unused_assignments)]
use asr::timer;
use std::sync::Mutex;

pub mod game;
use game::{GameProcess, Variables};

static GAME_PROCESS: Mutex<Option<GameProcess>> = Mutex::new(None);

#[no_mangle]
pub extern "C" fn update() {
    let mut mutex = GAME_PROCESS.lock().unwrap();
    let mut game_open: bool = true; // Used to stop and resume game time on game launch/exit

    if mutex.is_none() {
        // (Re)connect to the game and unpause game time
        *mutex = GameProcess::connect("OuterWilds.exe");
    } else {
        let game = mutex.as_mut().unwrap();

        // Make sure we're still connected to the game, pause game time if not
        if !game.process.is_open() {
            *mutex = None;
            if game_open == true {
                timer::pause_game_time();
                game_open = false;
            }

            return;
        }

        let vars = match game.state.update(&mut game.process) {
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

        handle_load(&vars);
    }
}

fn handle_load(vars: &Variables) {
    timer::set_variable("pauses", &format!("menu: {:?}, load: {:?}", vars.menu, vars.load));
    if vars.loading() {
        timer::pause_game_time();
    } else {
        timer::resume_game_time();
    }

    if vars.starting() {
        timer::start();
    }
}
