/// Encapsule pomodoro logic
/// no thread, no timer
use std::cell::Cell;
use std::cell::RefCell;

pub enum PSTATUS {
    INIT,

    StartWork,
    EndWork,

    StartBreak,
    EndBreak,

    LStartBreak,
    LEndBreak,
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
        // if init || LEndBreak || EndBreak:
        //  _cur_phase = StartWork
        //  incre work times
        //  
        // else if StartWork:
        //  _cur_phase = EndWork
        //  
        // else if EndWork:
        //  if not reach thread hold
        //      _cur_phase = StartBreak
        //  else
        //      _cur_phase = LStartBreak
        //      clear work times
        //  
        // else if StartBreak:
        //  _cur_phase = EndBreak
        //  
        // else if LStartBreak:
        //  _cur_phase = LEndBreak
        let mut phase = self._cur_phase.borrow_mut();
        match *phase {
            PSTATUS::INIT | PSTATUS::LEndBreak | PSTATUS::EndBreak => {
                *phase = PSTATUS::StartWork;
                PSTATUS::StartWork
            },

            PSTATUS::StartWork => {
                *phase = PSTATUS::EndWork;
                PSTATUS::EndWork
            },

            PSTATUS::EndWork => {
                if self.args.thread_hold > self._work_times.get() {
                    self._work_times.set(self._work_times.get() + 1);
                    *phase = PSTATUS::StartBreak;
                    PSTATUS::StartBreak

                } else {
                    *phase = PSTATUS::LStartBreak;
                    self._work_times.set(0);
                    PSTATUS::LStartBreak

                }
            },

            PSTATUS::StartBreak => {
                *phase = PSTATUS::EndBreak;
                PSTATUS::EndBreak
            },

            PSTATUS::LStartBreak => {
                *phase = PSTATUS::LEndBreak;
                PSTATUS::LEndBreak
            },
        }
    }

    pub fn status(&self) -> PSTATUS {
        match *self._cur_phase.borrow() {
            PSTATUS::INIT => PSTATUS::INIT,
            PSTATUS::StartWork => PSTATUS::StartWork,
            PSTATUS::EndWork => PSTATUS::EndWork,
            PSTATUS::StartBreak => PSTATUS::StartBreak,
            PSTATUS::EndBreak => PSTATUS::EndBreak,
            PSTATUS::LStartBreak => PSTATUS::LStartBreak,
            PSTATUS::LEndBreak => PSTATUS::LEndBreak,
        }
    }

    pub fn get_ms(&self, st: PSTATUS) -> u32 {
        match st {
            PSTATUS::StartWork => self.args.work_ms,
            PSTATUS::StartBreak => self.args.break_ms,
            PSTATUS::LStartBreak => self.args.lbreak_ms,
            _ => 100,
        }
    }

}

#[cfg(test)]
mod tests {
    use super::Pomodoro;
    use super::PSTATUS;

    #[test]
    fn main() {
        let pomo = Pomodoro::new(100, 1000, 1000, 1);

        match pomo.next_step() {
            PSTATUS::StartWork => assert!(true),
            _ => assert!(false),
        }

        match pomo.next_step() {
            PSTATUS::EndWork => assert!(true),
            _ => assert!(false),
        }

        match pomo.next_step() {
            PSTATUS::StartBreak => assert!(true),
            _ => assert!(false),
        }

        match pomo.next_step() {
            PSTATUS::EndBreak => assert!(true),
            _ => assert!(false),
        }

        match pomo.next_step() {
            PSTATUS::StartWork=> assert!(true),
            _ => assert!(false),
        }

        match pomo.next_step() {
            PSTATUS::EndWork => assert!(true),
            _ => assert!(false),
        }

        match pomo.next_step() {
            PSTATUS::LStartBreak => assert!(true),
            _ => assert!(false),
        }

        match pomo.next_step() {
            PSTATUS::LEndBreak => assert!(true),
            _ => assert!(false),
        }

        match pomo.next_step() {
            PSTATUS::StartWork => assert!(true),
            _ => assert!(false),
        }

        assert_eq!(100, pomo.get_ms(PSTATUS::StartWork));
    }
}
