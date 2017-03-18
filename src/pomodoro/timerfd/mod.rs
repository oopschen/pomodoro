extern crate mio;
extern crate timer;
extern crate chrono;

use std::cell::RefCell;
use std::io::Result;
use std::sync::Arc;

use self::mio::event::Evented;
use self::mio::Registration;
use self::mio::SetReadiness;
use self::mio::Ready;
use self::mio::Poll;
use self::mio::PollOpt;
use self::mio::Token;
use self::timer::Timer;

pub struct TimerFD {
    guard: RefCell<Option<timer::Guard>>,
    reg: Registration,
    rediness: Arc<SetReadiness>,
    timer: Timer,
}

impl TimerFD {
    pub fn new() -> TimerFD {
        let (registration, set_readiness) = Registration::new2();
        TimerFD {
            guard: RefCell::new(None),
            reg: registration,
            rediness: Arc::new(set_readiness),
            timer: Timer::with_capacity(4),
        }
    }

    pub fn schedule(&self, mill_secs: i64) {
        let share_ptr = self.rediness.clone();
        *self.guard.borrow_mut() = Some(
            self.timer.schedule_with_delay(chrono::Duration::milliseconds(mill_secs), move || {
                share_ptr.set_readiness(Ready::readable()).unwrap();
            })
        );
    }

    pub fn clear(&self) {
        self.rediness.set_readiness(Ready::empty()).unwrap();
    }

    pub fn stop(&self) {
        *self.guard.borrow_mut() = None;
        self.rediness.set_readiness(Ready::empty()).unwrap();
    }
}

impl Evented for TimerFD {

    fn register(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt) -> Result<()> {
        self.reg.register(poll, token, interest, opts)
    }

    fn reregister(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt) -> Result<()> {
        self.reg.reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> Result<()> {
        self.reg.deregister(poll)
    }

}
