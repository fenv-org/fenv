use chrono::{DateTime, Utc};

pub trait Clock {
    fn utc_now(&self) -> DateTime<Utc>;
}

#[derive(Clone)]
pub struct SystemClock;

impl SystemClock {
    pub fn new() -> Self {
        Self
    }
}

impl Clock for SystemClock {
    fn utc_now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}
