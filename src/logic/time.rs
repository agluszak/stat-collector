use mockall::automock;
use time::OffsetDateTime;

#[automock]
pub trait Clock: Send + Sync + 'static {
    fn now(&self) -> OffsetDateTime;
}

struct AppClock;

impl Clock for AppClock {
    fn now(&self) -> OffsetDateTime {
        OffsetDateTime::now_utc()
    }
}
