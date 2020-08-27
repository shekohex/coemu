use tracing::error;

pub fn current_tick_ms() -> u32 {
    match uptime_lib::get() {
        Ok(uptime) => uptime.as_millis() as u32,
        Err(err) => {
            error!("uptime: {}", err);
            0
        },
    }
}
