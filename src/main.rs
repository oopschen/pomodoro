#[macro_use]
extern crate clap;
extern crate ncurses;
mod pomodoro;

use std::io::Write;
use pomodoro;

macro_rules! errprint(
    ($($arg:tt)*) => { {
        let r = writeln!(&mut ::std::io::stderr(), $($arg)*);
        r.expect("failed printing to stderr");
    } }
);

fn main() {
    use clap::App;

    let cmd_yaml = load_yaml!("cmd.yml");
    let cmd_matches = App::from_yaml(cmd_yaml).get_matches();
    let notify_progs = match cmd_matches.value_of("notifycmd") {
        None => "",
        Some(v) => v,
    };

    let bms = match cmd_matches.value_of("break").unwrap().parse::<u32>() {
        Ok(v) => v*1000,
        Err(e) => {
            errprint!("break time must be numberic: {}", e);
            return
        }
    };

    let wms = match cmd_matches.value_of("work").unwrap().parse::<u32>() {
        Ok(v) => v*1000,
        Err(e) => {
            errprint!("work time must be numberic: {}", e);
            return
        }
    };

    let lbms = match cmd_matches.value_of("longbreak").unwrap().parse::<u32>() {
        Ok(v) => v*1000,
        Err(e) => {
            errprint!("long break time must be numberic: {}", e);
            return
        }
    };

    let lb_thold = match cmd_matches.value_of("longbreakthreadhold").unwrap().parse::<u8>() {
        Ok(v) => {
            if v < 1 {
                errprint!("long break thread hold must be in range [1, 255]");
                return
            }
            v
        },
        Err(e) => {
            errprint!("long break thread hold must be numberic and in range [1, 255]: {}", e);
            return
        }
    };

    println!("Pomodoro runs a long break time of {} msecs after {} times work time of {} msecs which followd by a break time of {} msecs.\n",
             lbms, lb_thold, wms, bms);

    run_pomodoro(lbms, lb_thold, wms, bms, notify_progs);
}

fn run_pomodoro(lbreak_ms: u32, lbreak_thread_hold: u8, work_ms: u32, break_ms: u32, notify_progs: &str) {
}
