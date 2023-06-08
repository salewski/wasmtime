#![allow(unused_variables)]

use crate::preview2::wasi::{
    clocks::monotonic_clock::{self, Instant},
    clocks::timezone::{self, Timezone, TimezoneDisplay},
    clocks::wall_clock::{self, Datetime},
    poll::poll::Pollable,
};
use crate::preview2::{HostPollable, TablePollableExt, WasiView};
use cap_std::time::SystemTime;

impl TryFrom<SystemTime> for Datetime {
    type Error = anyhow::Error;

    fn try_from(time: SystemTime) -> Result<Self, Self::Error> {
        let duration =
            time.duration_since(SystemTime::from_std(std::time::SystemTime::UNIX_EPOCH))?;

        Ok(Datetime {
            seconds: duration.as_secs(),
            nanoseconds: duration.subsec_nanos(),
        })
    }
}

impl<T: WasiView> wall_clock::Host for T {
    fn now(&mut self) -> anyhow::Result<Datetime> {
        let now = self.ctx().clocks.wall.now();
        Ok(Datetime {
            seconds: now.as_secs(),
            nanoseconds: now.subsec_nanos(),
        })
    }

    fn resolution(&mut self) -> anyhow::Result<Datetime> {
        let res = self.ctx().clocks.wall.resolution();
        Ok(Datetime {
            seconds: res.as_secs(),
            nanoseconds: res.subsec_nanos(),
        })
    }
}

impl<T: WasiView> monotonic_clock::Host for T {
    fn now(&mut self) -> anyhow::Result<Instant> {
        Ok(self.ctx().clocks.monotonic.now())
    }

    fn resolution(&mut self) -> anyhow::Result<Instant> {
        Ok(self.ctx().clocks.monotonic.resolution())
    }

    fn subscribe(&mut self, when: Instant, absolute: bool) -> anyhow::Result<Pollable> {
        Ok(self
            .table_mut()
            .push_host_pollable(HostPollable::new(tokio::time::sleep(
                std::time::Duration::from_millis(1000),
            )))?)
    }
}

impl<T: WasiView> timezone::Host for T {
    fn display(&mut self, timezone: Timezone, when: Datetime) -> anyhow::Result<TimezoneDisplay> {
        todo!()
    }

    fn utc_offset(&mut self, timezone: Timezone, when: Datetime) -> anyhow::Result<i32> {
        todo!()
    }

    fn drop_timezone(&mut self, timezone: Timezone) -> anyhow::Result<()> {
        todo!()
    }
}
