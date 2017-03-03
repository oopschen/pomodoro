#[macro_use]
extern crate clap;
extern crate ncurses;
extern crate timer;
extern crate chrono;
mod pomodoro;

use std::char;
use std::io::Write;
use std::thread;
use std::sync::mpsc::channel;
use std::option::Option;
use ncurses::*;
use pomodoro::PSTATUS;

macro_rules! errprint(
    ($($arg:tt)*) => { {
        let r = writeln!(&mut ::std::io::stderr(), $($arg)*);
        r.expect("failed printing to stderr");
    } }
);

// work_ms, break_ms, lbreak_ms, lbreak_thread_hold
type PTime = (u32, u32, u32, u8);

const CMD_NEXT:u8 = 0;
const CMD_RESET:u8 = 1;
const CMD_QUIT:u8 = 2;
const CMD_TIMEOUT:u8 = 255;

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

    run_pomodoro((wms, bms, lbms, lb_thold), notify_progs);
}

fn run_pomodoro(time_args: PTime, notify_progs: &str) {
    // child thread is responsible for pomodoro logic
    // main thread is responsible for listen input from keyboard and output
    // 
    // new channel
    // clone  tx for chlid timer
    // spawn a thread run pomodoro 
    // listen on input
     
    let (tx_input, rx) = channel::<u8>();
    let tx_timer = tx_input.clone();
    let notify_progs_clone = String::from(notify_progs);

    let pomodoro_thread = thread::spawn(move || {
        // new timer 
        // new pomodoro
        // listen on channel for cmd:
        //  if next:
        //      if has timer:
        //          clean it
        //      call next
        //      if start_work || start_break || start_lbreak :
        //          set up timer
        //  else if reset
        //      if has timer:
        //          clean it
        //      call reset
        //  else if timeout:
        //      clear timer handler
        //      call next
        //      notify progs if any
        //  else if quit :
        //      if has timer:
        //          clean it
        //      break
        let tr = timer::MessageTimer::new(tx_timer);
        let pomodo = pomodoro::Pomodoro::new(time_args.0, time_args.1, time_args.2, time_args.3);
        let mut hdl: Option<timer::Guard> = Option::None;
        let cmd:u8;

        loop {
            let cmd = rx.recv().unwrap();
            // clean up timer
            if let Some(v) = hdl {
                drop(v);
                hdl = Option::None;
            }

            // response to command
            match cmd {
                CMD_NEXT => {
                    let st = pomodo.next_step(); 
                    match st {
                        PSTATUS::START_WORK | PSTATUS::START_BREAK | PSTATUS::LSTART_BREAK => {
                            hdl = Option::Some(tr.schedule_with_delay(
                                    chrono::Duration::milliseconds(pomodo.get_ms(st) as i64),
                                    CMD_TIMEOUT));
                        },

                        _ => {},
                    }
                },

                CMD_QUIT => break,

                CMD_TIMEOUT => {
                    let st = pomodo.next_step();
                    // TODO call notify progs
                    if "" != notify_progs_clone {

                    }
                },

                CMD_RESET => {
                    pomodo.reset();
                },

                _ => continue,
            }
        }
    });
    // end thread

    // listen on char input
    // init term
    let window = initscr();
    raw();
    noecho();

    loop {
        let cr = getch();

        if ERR == cr {
            // TODO
            continue;
        }

        let char_str = match char::from_u32(cr as u32) {
            Some(ch) => ch,
            None => {
                // TODO
                continue;
            }

        };

        match char_str {
            'q' | 'Q' => {
                tx_input.send(CMD_QUIT).unwrap();
                pomodoro_thread.join();
                break;
            },

            'n' | 'N' => tx_input.send(CMD_NEXT).unwrap(),
            'r' | 'R' => tx_input.send(CMD_RESET).unwrap(),

            _ => {
                // TODO show help
            }
        }
    }

    endwin();
}
