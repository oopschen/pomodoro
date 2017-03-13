#[macro_use]
extern crate clap;
extern crate mio;
mod pomodoro;

use std::char;
use std::io::Write;
use std::thread;
use std::process::Command;
use std::process::Stdio;
use std::sync::mpsc::channel;
use std::option::Option;
use std::net::SocketAddr;
use std::vec::Vec;
use std::collections::VecDeque;
use std::cell::Cell;
use std::cell::RefCell;
use clap::App;

use mio::Events;
use mio::Event;
use mio::Poll;
use mio::Token;
use mio::Ready;
use mio::PollOpt;
use mio::Evented;
use mio::tcp::TcpListener;
use mio::tcp::TcpStream;

use pomodoro::PSTATUS;
use pomodoro::Pomodoro;

macro_rules! errprint(
    ($($arg:tt)*) => { 
        {
            let r = writeln!(&mut ::std::io::stderr(), $($arg)*);
            r.expect("failed printing to stderr");
        } 
    }
    );

macro_rules! dereg(
    ( $holder:expr, $i:ident, $poll:ident) => {
        match $holder[$i-1] {
            Some(ref h) => {
                match *h.hdl.borrow() {
                    Some(hl) => {
                        match $poll.deregister(hl) {
                            Ok(_) => {},
                            Err(e) => errprint!("deregister handler {} fail, {}", $i, e),
                        }
                    },
                    None => {},
                }

            },
            None => {},
        }

    }
    );

macro_rules! cleanup_holder(
    ( $holder:expr ) => {
        match $holder {
            Some(ref h) => {
                *h.hdl.borrow_mut() = None;
                *h.stream.borrow_mut() = None;
            },
            None => {},
        }
    }
    );

// work_ms, break_ms, lbreak_ms, lbreak_thread_hold
type PTime = (u32, u32, u32, u8);

struct PomodoroHolder<'a> {
    pomo: Pomodoro,
    addr: Cell<SocketAddr>,
    msg: RefCell<Option<String>>,
    hdl: RefCell<Option<&'a Evented>>,
    stream: RefCell<Option<TcpStream>>,
    _buf: RefCell<Option<Vec<u8>>>,
}

enum PollAction {
    EXIT(usize), // dereg, close
    CONT, // do nothing continue
    DEREG(usize), // deregister
    ADD_READ_WRITE(usize),
}

const CMD_NEXT:u8 = 0;
const CMD_RESET:u8 = 1;
const CMD_QUIT:u8 = 2;
const CMD_TIMEOUT:u8 = 255;
const SERVER_TOKEN: usize = 0;

const PMPT_STAGE: &'static str = "{sg}";
const PMPT_STATUS: &'static str = "{st}";

fn main() {
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

    let host = cmd_matches.value_of("host").unwrap();

    let port = match cmd_matches.value_of("port").unwrap().parse::<u16>() {
        Ok(v) => {
            if v < 1024 {
                errprint!("port must be in range 1024 - 65536");
                return;
            }
            v
        },
        Err(e) => {
            errprint!("port can't be parse as integer, {}.", e);
            return;
        }
    };

    let maxp = match cmd_matches.value_of("maxuser").unwrap().parse::<u16>() {
        Ok(v) => {
            if v < 1 {
                errprint!("There must exists some pomodoro.");
            }
            v
        },

        Err(e) => {
            errprint!("invalid format for maxuser, {}.", e);
            return;
        }
    };

    run_pomodoro((wms, bms, lbms, lb_thold), notify_progs, host, port, maxp);
}

fn parse_listen_address(host: &str, port: u16) -> SocketAddr {
    format!("{}:{}", host, port).parse().unwrap()
}

// run pomodoro 
//  init array for datastore
//  init avail index 
//  listen on host and port
//      accept
//      if connect:
//          get next index
//          if null:
//              write msg
//              close
//      else if read:
//          read to eof
//          compare cmd do jobs
fn run_pomodoro(time_args: PTime, notify_progs: &str, host: &str, port: u16, maxp: u16) {
    let sock_addr = parse_listen_address(host, port);
    let mut holder_arr: Vec<Option<PomodoroHolder>> = Vec::new();
    let mut avail_index: VecDeque<usize> = VecDeque::with_capacity(maxp as usize);
    for i in 0..maxp {
        holder_arr.push(None);
        avail_index.push_back(i as usize);
    }

    let listener = match TcpListener::bind(&sock_addr) {
        Ok(v) => v,
        Err(e) => {
            errprint!("can not listen to {}:{}, {}", host, port, e);
            return;
        }

    };

    let mut evts = Events::with_capacity(maxp as usize);

    let poll = match Poll::new() {
        Ok(p) => p,
        Err(e) => {
            errprint!("create poll error, {}", e);
            return;
        }
    };

    match poll.register(&listener, Token(SERVER_TOKEN), Ready::readable()|Ready::writable(), PollOpt::level()) {
        Ok(_) => {},
        Err(e) => {
            errprint!("register listener fail, {}", e);
            return;
        }
    };

    loop {
        let num = match poll.poll(&mut evts, None) {
            Ok(n) => n,
            Err(e) => {
                errprint!("poll error, {}", e);
                continue;
            }
        };

        for i in 0..num {
            match evts.get(i) {
                None => continue,
                Some(evt) => {
                    match dispatch_event(time_args, &evt, &mut holder_arr, &mut avail_index, &listener) {
                        PollAction::EXIT(inx) => {
                            dereg!(&holder_arr, inx, poll);
                            cleanup_holder!(holder_arr[inx]);
                        },

                        PollAction::CONT => {},

                        PollAction::DEREG(inx) => {
                            dereg!(&holder_arr, inx, poll);
                        },

                        PollAction::ADD_READ_WRITE(inx) => {
                            if let Some(ref pomo) = holder_arr[inx] {
                                if let Some(hdl) = *pomo.hdl.borrow() {
                                    poll.register(hdl, Token(inx),
                                        Ready::readable()|Ready::writable(), PollOpt::level()).unwrap();
                                }
                            }
                        }
                    }
                }
            }
        }

    }
    /*
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
        let pomodo = pomodoro::;
        let mut hdl: Option<timer::Guard> = Option::None;

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
            PSTATUS::StartWork | PSTATUS::StartBreak | PSTATUS::LStartBreak => {
            let (sg, sta) = match st {
            PSTATUS::StartWork => ("Work", "Started"),
            PSTATUS::StartBreak => ("Break", "Started"),
            PSTATUS::LStartBreak => ("Long-Break", "Started"),
            _ => ("Unknown", "Undefined")

            };

            hdl = Option::Some(tr.schedule_with_delay(
            chrono::Duration::milliseconds(pomodo.get_ms(st) as i64),
            CMD_TIMEOUT));

            run_notify_command(notify_progs_clone.replace(PMPT_STAGE, sg).replace(PMPT_STATUS, sta));
            },

            PSTATUS::EndWork => {
            run_notify_command(notify_progs_clone.replace(PMPT_STAGE, "Work").replace(PMPT_STATUS, "Stopped"));
            },

            PSTATUS::EndBreak => {
            run_notify_command(notify_progs_clone.replace(PMPT_STAGE, "Break").replace(PMPT_STATUS, "Stopped"));
            },

            PSTATUS::LEndBreak => {
            run_notify_command(notify_progs_clone.replace(PMPT_STAGE, "Long-Break").replace(PMPT_STATUS, "Stopped"));
            },

    _ => {
        run_notify_command(notify_progs_clone.replace(PMPT_STAGE, "Unknow").replace(PMPT_STATUS, "Undefined"));
    },
    }
},

    CMD_QUIT => break,

    CMD_TIMEOUT => {
        match pomodo.status() {
            PSTATUS::StartWork | PSTATUS::StartBreak | PSTATUS::LStartBreak => {},
            _ => continue,
        };

        let st = pomodo.next_step();
        // call notify progs
        if "" != notify_progs_clone {
            match st {
                PSTATUS::EndWork => {
                    run_notify_command(notify_progs_clone.replace(PMPT_STAGE, "Work").replace(PMPT_STATUS, "End"));
                },

                PSTATUS::EndBreak => {
                    run_notify_command(notify_progs_clone.replace(PMPT_STAGE, "Break").replace(PMPT_STATUS, "End"));
                },

                PSTATUS::LEndBreak => {
                    run_notify_command(notify_progs_clone.replace(PMPT_STAGE, "Long-Break").replace(PMPT_STATUS, "End"));
                },

                _ => {}
            }
        }
    },

    CMD_RESET => {
        pomodo.reset();
        run_notify_command(notify_progs_clone.replace(PMPT_STAGE, "Reset").replace(PMPT_STATUS, "Done"));
    },

    _ => continue,
    }
}
});
// end thread

// listen on char input
loop {
    let cr = getch();

    if ERR == cr {
        clear_screen();
        print_help(time_args);
        continue;
    }

    let char_str = match char::from_u32(cr as u32) {
        Some(ch) => ch,
        None => {
            clear_screen();
            print_help(time_args);
            continue;
        }

    };

    // print to terminal
    printw(format!("{}", char_str).as_ref());

    match char_str {
        'q' | 'Q' => {
            tx_input.send(CMD_QUIT).unwrap();
            pomodoro_thread.join().unwrap();
            break;
        },

        'n' | 'N' => tx_input.send(CMD_NEXT).unwrap(),
        'r' | 'R' => tx_input.send(CMD_RESET).unwrap(),

        _ => {
            clear_screen();
            print_help(time_args);
        }
    }
}

}


fn run_notify_command(progs: String) {
    let mut splits_by_space = progs.split_whitespace();
    let mut cmd_builder: Command;

    if let Some(prog) = splits_by_space.next() {
        cmd_builder = Command::new(prog);
        cmd_builder.stderr(Stdio::null()).stdout(Stdio::null());
    } else {
        return ();
    }

    while let Some(arg) = splits_by_space.next() {
        cmd_builder.arg(arg);
    }

    cmd_builder.spawn().unwrap();
    */
}

/*
   fn print_prompt() {
   printw(format!("Command me[n/N/r/R/q/Q]:").as_ref());
   refresh();
   }

   fn print_help(time_args: PTime) {
   printw(format!("\
   Pomodoro runs a long break time of {} msecs after {} times work time of {} msecs which followd by a break time of {} msecs.
   Help list:
   n/N\t\t\tGo to the next step
   r/R\t\t\tReset the pomodoro
   q/Q\t\t\tQuit the programs\n",
   time_args.2, time_args.3, time_args.0, time_args.1).as_ref());
   refresh();
   print_prompt();
   }

   fn clear_screen() {
   clear();
   refresh();
   }
   */

fn dispatch_event(time_args: PTime, evt: &Event, 
                  holders: &mut Vec<Option<PomodoroHolder>>, 
                  avails: &mut VecDeque<usize>, listener: &TcpListener) -> PollAction {
    // if token is server
    //  is reable:
    //      write help message
    //      get next index
    //      if none:
    //          write error message
    //      new pomodoro
    //      set up holder
    // else if is wriable && message is not null
    //  write message
    // else if is reaable
    //  read to eof
    let Token(token) = evt.token();
    let mut read_buf: [u8; 2048] = [0; 2048];
    match token {
        0 => {
            // server token
            if !evt.readiness().is_readable() {
                return PollAction::CONT;
            }

            match listener.accept() {
                Ok((ref mut stream, addr)) => {
                    let inx = match avails.pop_front() {
                        Some(i) => i,
                        None => {
                            stream.write("Max user reached, please increament it.\n".to_string().as_bytes()).unwrap();
                            return PollAction::CONT;
                        }
                    };

                    // write hello message
                    let hello_msg = format!("Hello, You! Welcome to Pomodoro.\nPress 'h/H' for help\n");
                    stream.write(hello_msg.to_string().as_bytes()).unwrap();
                    // new holder
                    holders[inx] = Some(PomodoroHolder {
                       pomo: Pomodoro::new(time_args.0, time_args.1, time_args.2, time_args.3),
                       addr: Cell::new(addr),
                       msg: RefCell::new(None),
                       hdl: RefCell::new(None),
                       stream: RefCell::new(None),
                       _buf: RefCell::new(None),
                    });

                    PollAction::ADD_READ_WRITE(inx)
                },

                Err(e) => {
                    errprint!("accept fail, {}", e);
                    PollAction::CONT
                },

            }
        },

        x if x > 1 => {
            PollAction::CONT
        },

        _ => PollAction::DEREG(token),
    }
}
