use chrono::{DateTime, Local};
use mockall::automock;

#[automock]
pub trait Clock: Send + Sync + 'static {
    fn now(&self) -> DateTime<Local>;
}

pub struct AppClock;

impl Clock for AppClock {
    fn now(&self) -> DateTime<Local> {
        Local::now()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn app_clock_returns_current_time() {
        let clock = AppClock;
        let now = clock.now();
        assert!(now <= Local::now());
    }
}
