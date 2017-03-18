#[macro_use]
extern crate clap;
extern crate mio;
mod pomodoro;

use std::char;
use std::io::Write;
use std::io::Read;
use std::thread;
use std::process::Command;
use std::process::Stdio;
use std::option::Option;
use std::net::SocketAddr;
use std::vec::Vec;
use std::collections::VecDeque;
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
use pomodoro::timerfd::TimerFD;

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
                match *h.stream.borrow() {
                    Some(ref hl) => {
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
                h.pomo.reset();
                h.stream.borrow_mut().take();
                h.msg.borrow_mut().take();
                h._buf.borrow_mut().take();
                if let Some(ref timerfd) = *h._timerfd.borrow_mut() {
                    timerfd.stop();
                }
                h._after_write_action.borrow_mut().take();
            },
            None => {},
        }
    }
    );

// work_ms, break_ms, lbreak_ms, lbreak_thread_hold
type PTime = (u32, u32, u32, u8);

struct PomodoroHolder {
    pomo: Pomodoro,
    msg: RefCell<Option<String>>, // message write back
    stream: RefCell<Option<TcpStream>>,
    _buf: RefCell<Option<Vec<u8>>>, // read buf 
    _timerfd: RefCell<Option<TimerFD>>,
    _after_write_action: RefCell<Option<PollAction>>,
}

enum PollAction {
    EXIT(usize), // dereg, close
    CONT, // do nothing continue
    AddRW(usize), // add read and write
}

#[derive(Debug)]
enum UserCommand {
    EXIT,
    RESET,
    NEXT,
    HELP,
    STATUS,
    NONE, // can not read any command
}

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

    let maxp = match cmd_matches.value_of("maxuser").unwrap().parse::<usize>() {
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
fn run_pomodoro(time_args: PTime, notify_progs: &str, host: &str, port: u16, maxp: usize) {
    let sock_addr = parse_listen_address(host, port);
    let mut holder_arr: Vec<Option<PomodoroHolder>> = Vec::new();
    let max_pomo_num = maxp + 1;
    // trick: 0 means server, others means pomodoro
    let mut avail_index: VecDeque<usize> = VecDeque::with_capacity(max_pomo_num); 
    for i in 1..max_pomo_num {
        holder_arr.push(None);
        avail_index.push_back(i);
    }

    let listener = match TcpListener::bind(&sock_addr) {
        Ok(v) => v,
        Err(e) => {
            errprint!("can not listen to {}:{}, {}", host, port, e);
            return;
        }

    };

    let mut evts = Events::with_capacity(maxp);

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
                    match dispatch_event(time_args, &evt, &mut holder_arr, &mut avail_index, &listener, max_pomo_num) {
                        PollAction::EXIT(inx) => {
                            dereg!(&holder_arr, inx, poll);
                            cleanup_holder!(holder_arr[inx]);
                            avail_index.push_back(inx);
                            println!("Close pomodoro for {}", inx);
                        },

                        PollAction::CONT => {},

                        PollAction::AddRW(inx) => {
                            if let Some(ref pomo) = holder_arr[inx] {
                                if let Some(ref stream) = *pomo.stream.borrow() {
                                    poll.register(stream, Token(inx),
                                    Ready::readable()|Ready::writable(), PollOpt::level()).unwrap();

                                } else {
                                    errprint!("no handler is found for {}", inx);

                                }

                                // reg timerd
                                if let Some(ref timerfd) = *pomo._timerfd.borrow() {
                                    poll.register(timerfd, Token(inx+maxp),
                                    Ready::readable(), PollOpt::level()).unwrap();

                                } else {
                                    errprint!("no timerfd is found for {}", inx);

                                }

                            } else {
                                errprint!("no holder is found for {}", inx);

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
                              avails: &mut VecDeque<usize>, listener: &TcpListener, maxp: usize) -> PollAction {
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
                match token {
                    0 => {
                        // server token
                        if !evt.readiness().is_readable() {
                            return PollAction::CONT;
                        }

                        match listener.accept() {
                            Ok((stream, addr)) => {
                                let mut mut_st = stream;
                                let inx = match avails.pop_front() {
                                    Some(i) => i,
                                    None => {
                                        mut_st.write("Max user reached, please increament it.\n".to_string().as_bytes()).unwrap();
                                        return PollAction::CONT;
                                    }
                                };

                                println!("Recv {} for new Pomodoro \"{}\"", addr, inx);

                                // write hello message
                                let hello_msg = format!("Hello, You! Welcome to Pomodoro.\nPress 'h/H' for help\n");
                                mut_st.write(hello_msg.to_string().as_bytes()).unwrap();
                                // new holder
                                holders[inx] = Some(PomodoroHolder {
                                    pomo: Pomodoro::new(time_args.0, time_args.1, time_args.2, time_args.3),
                                    msg: RefCell::new(None),
                                    stream: RefCell::new(Some(mut_st)),
                                    _buf: RefCell::new(None),
                                    _timerfd: RefCell::new(Some(TimerFD::new())),
                                    _after_write_action: RefCell::new(None),
                                });

                                PollAction::AddRW(inx)
                            },

                            Err(e) => {
                                errprint!("accept fail, {}", e);
                                PollAction::CONT
                            },

                        }
                    },

                    x if x > 0 && x < maxp => {
                        if let Some(ref hdl) = holders[x] {
                            deal_stream_token(hdl, x, evt)
                        } else {
                            PollAction::EXIT(x)
                        }
                    },

                    y if y > maxp => {
                        if let Some(ref hdl) = holders[y-maxp] {
                            deal_timer_token(hdl, y)
                        } else {
                            PollAction::EXIT(y)
                        }
                    },

                    _ => PollAction::EXIT(token),
                }
            }

fn deal_stream_token(holder: &PomodoroHolder, inx: usize, evt: &Event) -> PollAction {
    // if read :
    //  parse command 
    //  deal with command
    // if write and have data:
    //  write data
    if evt.readiness().is_readable() {
        if let Some(ref mut stream) = *holder.stream.borrow_mut() {
            let mut read_buf: [u8;2048] = [0; 2048];
            let read_num = match stream.read(&mut read_buf) {
                Ok(n) => n,
                Err(e) => {
                    errprint!("read stream {} error: {}", inx, e);
                    return PollAction::EXIT(inx);
                },
            };

            let (rt, msg) = match parse_command(holder, &read_buf, read_num) {
                UserCommand::EXIT => (PollAction::EXIT(inx), "Goodbye.".to_string()),

                UserCommand::RESET => {
                    holder.pomo.reset();
                    (PollAction::CONT, "Reset Done".to_string())
                },

                UserCommand::NEXT => {
                    if let Some(ref timerfd) = *holder._timerfd.borrow() {
                        timerfd.stop();
                        match holder.pomo.next_step() {
                            PSTATUS::StartWork => {
                                timerfd.schedule(holder.pomo.get_ms(PSTATUS::StartWork) as i64);
                                (PollAction::CONT, "Work time started".to_string())
                            },

                            PSTATUS::StartBreak => {
                                timerfd.schedule(holder.pomo.get_ms(PSTATUS::StartBreak) as i64);
                                (PollAction::CONT, "Break time started".to_string())
                            },

                            PSTATUS::LStartBreak => {
                                timerfd.schedule(holder.pomo.get_ms(PSTATUS::LStartBreak) as i64);
                                (PollAction::CONT, "Long Break time started".to_string())
                            },

                            _ => {
                                (PollAction::EXIT(inx), "Status incorrect, please restart the program".to_string())
                            }
                        }


                    } else {
                        (PollAction::EXIT(inx), "There is no timer, restart program.".to_string())

                    }

                },

                UserCommand::STATUS => {
                    let st = match holder.pomo.status() {
                        PSTATUS::INIT => "Init",
                        PSTATUS::StartWork => "Working",
                        PSTATUS::StartBreak => "Breaking",
                        PSTATUS::LStartBreak => "LongBreaking",
                        PSTATUS::EndWork => "EndWork",
                        PSTATUS::EndBreak => "EndBreak",
                        PSTATUS::LEndBreak => "LongEndBreak",
                    };
                    (PollAction::CONT, format!("Current Status is {}", st))
                },

                _ => {
                    (PollAction::CONT, format!("\
    Pomodoro runs a long break time of {} msecs after {} times work time of {} msecs which followd by a break time of {} msecs.
    Help list:
    n/N\t\t\tGo to the next step
    r/R\t\t\tReset the pomodoro
    q/Q\t\t\tQuit the programs\n",
    holder.pomo.get_ms(PSTATUS::LStartBreak), 
    holder.pomo.get_thread_hold(),
    holder.pomo.get_ms(PSTATUS::StartWork), 
    holder.pomo.get_ms(PSTATUS::StartBreak))
                    )
                },
            };

            *holder.msg.borrow_mut() = Some(msg);
            *holder._after_write_action.borrow_mut() = Some(rt);
            PollAction::CONT

        } else {
            *holder.msg.borrow_mut() = Some("There is no connection at all, restart it.".to_string());
            *holder._after_write_action.borrow_mut() = Some(PollAction::EXIT(inx));
            PollAction::CONT

        }

    } else if evt.readiness().is_writable() {
        if let Some(ref v) = holder.msg.borrow_mut().take() {
            if let Some(ref mut stream) = *holder.stream.borrow_mut() {
                match stream.write_all(v.as_bytes()) {
                    Ok(_) => {},
                    Err(e) => {
                        errprint!("Write msg {}", e);
                    }
                }
            }

        }

        match holder._after_write_action.borrow_mut().take() {
            None => PollAction::CONT,
            Some(v) => v,
        }

    } else {
        PollAction::EXIT(inx)
    }

}

fn deal_timer_token(holder: &PomodoroHolder, inx: usize) -> PollAction {
    // get current status
    // assemble status text, write to buf
    // call notify program
    let (rt, msg) = match holder.pomo.next_step() {
        PSTATUS::EndWork => (PollAction::CONT, "Work time done."),

        PSTATUS::EndBreak => (PollAction::CONT, "Break time done."),

        PSTATUS::LEndBreak => (PollAction::CONT, "Long Break time done."),

        _ => (PollAction::EXIT(inx), "Sth is wrong, please restart program."),
    };

    *holder.msg.borrow_mut() = Some(msg.to_string());
    *holder._after_write_action.borrow_mut() = Some(rt);
    PollAction::CONT
}

fn parse_command(holder: &PomodoroHolder, input: &[u8], len: usize) -> UserCommand {
    let mut buf: Vec<u8> = Vec::new();
    let mut read_buf = holder._buf.borrow_mut();

    // insert data
    if let Some(ref mut v) = read_buf.take() {
        buf.append(v);
    }

    for i in 0..len {
        buf.push(input[i]);
    }

    let mut del_pos_excl: usize = buf.len();
    let mut cmd_eol: usize = 0;
    let mut found_eol = false;
    if let Some(eol_pos) = buf.iter().position(|&x| x as char == '\r' || x as char == '\n') {
        if '\r' == buf[eol_pos] as char && (eol_pos + 1) < buf.len() && '\n' == buf[eol_pos+1] as char {
            del_pos_excl = eol_pos + 2;
            cmd_eol = eol_pos;
            found_eol = true;
        } else if '\n' == buf[eol_pos] as char {
            del_pos_excl = eol_pos + 1;
            cmd_eol = eol_pos;
            found_eol = true;
        }
    }

    let str_buf: Vec<u8>; 
    if found_eol {
        let mut tmp_str = buf.drain(0..del_pos_excl).collect::<Vec<u8>>(); 
        str_buf = tmp_str.drain(0..cmd_eol).collect();
        *read_buf = match buf.is_empty() {
            true => None,
            false => Some(buf),
        };

    } else {
        str_buf = buf;
        *read_buf = Some(str_buf.clone());
        return UserCommand::NONE;

    }

    // compare cmd
    let cmd_str = match String::from_utf8(str_buf) {
        Ok(s) => s,
        Err(e) => {
            errprint!("parse command: {}", e);
            return UserCommand::NONE;
        }
    };

    match cmd_str.to_lowercase().as_ref() {
        "n" | "next" => UserCommand::NEXT, 
        "r" | "reset" => UserCommand::RESET, 
        "s" | "status" => UserCommand::STATUS, 
        "h" | "help" => UserCommand::HELP, 
        "q" | "quit" => UserCommand::EXIT, 
        _ => UserCommand::NONE,
    }
}
