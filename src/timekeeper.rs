use std::time::{SystemTime, UNIX_EPOCH};

pub struct TimeKeeper {
    start_time: u128,
    last_tick_time: u128,
    end_time: u128,
}

#[derive(Debug)]
pub struct Tick {
    time: u128,
    time_since_last_tick: u128,
    time_since_start: u128,
}

impl TimeKeeper {
    pub fn new() -> Self {
        Self {
            start_time: 0,
            end_time: 0,
            last_tick_time: 0,
        }
    }

    pub fn start(&mut self) -> Tick {
        let time = self.time();
        self.start_time = time;
        self.last_tick_time = time;
        self.tick()
    }

    pub fn end(&mut self) -> Tick {
        let time = self.time();
        self.end_time = time;
        self.last_tick_time = time;
        self.tick()
    }

    pub fn tick(&mut self) -> Tick {
        let time = self.time();
        let time_since_last_tick = time - self.last_tick_time;
        let time_since_start = time - self.start_time;
        self.last_tick_time = time;

        Tick {
            time,
            time_since_last_tick,
            time_since_start,
        }
    }

    fn time(&self) -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timekeeper_new() {
        // Arrange
        let expected_start_time = 0;
        let expected_end_time = 0;
        let expected_last_tick_time = 0;

        // Act
        let timekeeper = TimeKeeper::new();

        // Assert
        assert_eq!(timekeeper.start_time, expected_start_time);
        assert_eq!(timekeeper.end_time, expected_end_time);
        assert_eq!(timekeeper.last_tick_time, expected_last_tick_time);
    }

    #[test]
    fn test_timekeeper_start() {
        // Act
        let mut timekeeper = TimeKeeper::new();
        let tick = timekeeper.start();

        // Assert
        assert!(tick.time > 0);
        assert_eq!(tick.time_since_last_tick, 0);
        assert_eq!(tick.time_since_start, 0);
    }

    #[test]
    fn test_timekeeper_end() {
        // Act
        let mut timekeeper = TimeKeeper::new();
        let tick = timekeeper.end();

        // Assert
        assert!(tick.time > 0);
        assert_eq!(tick.time_since_last_tick, 0);
        assert!(tick.time_since_start > 0);
    }

    #[test]
    fn test_timekeeper_tick_after_start() {
        // Act
        let mut timekeeper = TimeKeeper::new();
        let _ = timekeeper.start();
        let tick = timekeeper.tick();

        // Assert
        assert!(tick.time > 0);
        assert_eq!(tick.time_since_last_tick, 0);
        assert_eq!(tick.time_since_start, 0);
        assert_eq!(tick.time_since_start, 0);
        assert_eq!(tick.time_since_start, 0);
    }
}
