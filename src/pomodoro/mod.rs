/// Encapsule pomodoro logic
/// no thread, no timer
use std::cell::Cell;
use std::cell::RefCell;
use std::clone::Clone;

pub enum PSTATUS {
    INIT,

    START_WORK,
    END_WORK,

    START_BREAK,
    END_BREAK,

    LSTART_BREAK,
    LEND_BREAK,
}

struct PomodoroArgs {
    work_ms: u32,
    break_ms: u32,
    lbreak_ms: u32,
    thread_hold: u8,
}

pub struct Pomodoro {
    _cur_phase: RefCell<PSTATUS>,
    _work_times: Cell<u8>,
    args: PomodoroArgs,
}

impl Pomodoro {
    pub fn new(work_ms: u32, break_ms: u32, lbreak_ms: u32, thread_hold:u8) -> Self {
        Pomodoro {
            _cur_phase: RefCell::new(PSTATUS::INIT),
            _work_times: Cell::new(0),
            args: PomodoroArgs {
                work_ms: work_ms,
                break_ms: break_ms,
                lbreak_ms: lbreak_ms,
                thread_hold: thread_hold,
            },
        }
    }

    pub fn reset(&self) -> bool {
        *self._cur_phase.borrow_mut() = PSTATUS::INIT;
        self._work_times.set(0);
        true
    }

    // return next step status
    pub fn next_step(&self) -> PSTATUS {
        // if init || LEND_BREAK || END_BREAK:
        //  _cur_phase = START_WORK
        //  incre work times
        //  
        // else if START_WORK:
        //  _cur_phase = END_WORK
        //  
        // else if END_WORK:
        //  if not reach thread hold
        //      _cur_phase = START_BREAK
        //  else
        //      _cur_phase = LSTART_BREAK
        //      clear work times
        //  
        // else if START_BREAK:
        //  _cur_phase = END_BREAK
        //  
        // else if LSTART_BREAK:
        //  _cur_phase = LEND_BREAK
        match *self._cur_phase.borrow() {
            PSTATUS::INIT | PSTATUS::LEND_BREAK | PSTATUS::END_BREAK => {
                *self._cur_phase.borrow_mut() = PSTATUS::START_WORK;
                self._work_times.set(self._work_times.get() + 1);
                PSTATUS::START_WORK
            },

            PSTATUS::START_WORK => {
                *self._cur_phase.borrow_mut() = PSTATUS::END_WORK;
                PSTATUS::END_WORK
            },

            PSTATUS::END_WORK => {
                if self.args.thread_hold > self._work_times.get() {
                    *self._cur_phase.borrow_mut() = PSTATUS::START_BREAK;
                    PSTATUS::START_BREAK

                } else {
                    *self._cur_phase.borrow_mut() = PSTATUS::LSTART_BREAK;
                    self._work_times.set(0);
                    PSTATUS::LSTART_BREAK

                }
            },

            PSTATUS::START_BREAK => {
                *self._cur_phase.borrow_mut() = PSTATUS::END_BREAK;
                PSTATUS::END_BREAK
            },

            PSTATUS::LSTART_BREAK => {
                *self._cur_phase.borrow_mut() = PSTATUS::LEND_BREAK;
                PSTATUS::LEND_BREAK
            },
        }
    }

    pub fn status(&self) -> PSTATUS {
        match *self._cur_phase.borrow() {
            PSTATUS::INIT => PSTATUS::INIT,
            PSTATUS::START_WORK => PSTATUS::START_WORK,
            PSTATUS::END_WORK => PSTATUS::END_WORK,
            PSTATUS::START_BREAK => PSTATUS::START_BREAK,
            PSTATUS::END_BREAK => PSTATUS::END_BREAK,
            PSTATUS::LSTART_BREAK => PSTATUS::LSTART_BREAK,
            PSTATUS::LEND_BREAK => PSTATUS::LEND_BREAK,
        }
    }

    pub fn get_ms(&self, st: PSTATUS) -> u32 {
        match st {
            PSTATUS::START_WORK => self.args.work_ms,
            PSTATUS::START_BREAK => self.args.break_ms,
            PSTATUS::LSTART_BREAK => self.args.lbreak_ms,
            _ => 100,
        }
    }

}
