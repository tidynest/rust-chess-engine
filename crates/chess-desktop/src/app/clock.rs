//! A pair of chess clocks with a Fischer increment.

use chess::Color;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct Clock {
    remaining: [Duration; 2],
    increment: Duration,
    /// Set once the first move has been played.
    started: bool,
    /// When the running side was last charged; `None` while paused.
    last_tick: Option<Instant>,
    /// Both times after each ply, so a takeback can put them back.
    snapshots: Vec<[Duration; 2]>,
}

impl Clock {
    pub fn new(minutes: u32, increment_seconds: u32) -> Self {
        let remaining = [Duration::from_secs(u64::from(minutes) * 60); 2];
        Self {
            remaining,
            increment: Duration::from_secs(u64::from(increment_seconds)),
            started: false,
            last_tick: None,
            snapshots: vec![remaining],
        }
    }

    pub fn remaining(&self, color: Color) -> Duration {
        self.remaining[color.to_index()]
    }

    pub fn increment(&self) -> Duration {
        self.increment
    }

    /// Ticking once the first move is played and not paused.
    pub fn is_running(&self) -> bool {
        self.started && self.last_tick.is_some()
    }

    /// Charge `side` for the time since the last tick. True when its flag fell.
    pub fn tick(&mut self, side: Color, now: Instant) -> bool {
        if !self.started {
            return false;
        }
        if let Some(last) = self.last_tick {
            let slot = &mut self.remaining[side.to_index()];
            *slot = slot.saturating_sub(now.duration_since(last));
        }
        self.last_tick = Some(now);
        self.remaining(side).is_zero()
    }

    /// Stop charging until the next tick, for instance while browsing the history.
    pub fn pause(&mut self) {
        self.last_tick = None;
    }

    /// `mover` has played the move that makes `ply` moves: settle their time,
    /// add the increment, and start the clock if this was the first move.
    pub fn press(&mut self, mover: Color, now: Instant, ply: usize) {
        self.tick(mover, now);
        self.started = true;
        self.last_tick = Some(now);
        self.remaining[mover.to_index()] += self.increment;
        self.snapshots.truncate(ply);
        self.snapshots.push(self.remaining);
    }

    /// Put both clocks back to where they stood after `ply` moves, for undo,
    /// redo and jumps. Paused until the next tick, so the time spent browsing
    /// is nobody's.
    pub fn restore(&mut self, ply: usize) {
        if let Some(&remaining) = self.snapshots.get(ply) {
            self.remaining = remaining;
            self.started = ply > 0;
            self.last_tick = None;
        }
    }

    /// `m:ss`, with tenths under ten seconds.
    pub fn format(duration: Duration) -> String {
        let secs = duration.as_secs_f64();
        if secs < 10.0 {
            format!("0:{secs:04.1}")
        } else {
            let whole = duration.as_secs();
            format!("{}:{:02}", whole / 60, whole % 60)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_move_is_free_and_increment_is_added() {
        let mut clock = Clock::new(1, 2);
        let t0 = Instant::now();
        assert!(!clock.tick(Color::White, t0 + Duration::from_secs(30)));
        assert_eq!(clock.remaining(Color::White), Duration::from_secs(60));

        clock.press(Color::White, t0, 1);
        assert_eq!(clock.remaining(Color::White), Duration::from_secs(62));

        assert!(!clock.tick(Color::Black, t0 + Duration::from_secs(59)));
        assert!(clock.tick(Color::Black, t0 + Duration::from_secs(61)));
        assert_eq!(clock.remaining(Color::Black), Duration::ZERO);
    }

    #[test]
    fn pause_skips_the_gap() {
        let mut clock = Clock::new(1, 0);
        let t0 = Instant::now();
        clock.press(Color::White, t0, 1);
        clock.pause();
        clock.tick(Color::Black, t0 + Duration::from_secs(50));
        assert_eq!(clock.remaining(Color::Black), Duration::from_secs(60));
    }

    #[test]
    fn takeback_restores_both_clocks() {
        let mut clock = Clock::new(1, 0);
        let t0 = Instant::now();
        clock.press(Color::White, t0, 1);
        clock.press(Color::Black, t0 + Duration::from_secs(20), 2);
        assert_eq!(clock.remaining(Color::Black), Duration::from_secs(40));

        clock.restore(1);
        assert_eq!(clock.remaining(Color::Black), Duration::from_secs(60));
        assert!(!clock.is_running());
        clock.restore(2);
        assert_eq!(clock.remaining(Color::Black), Duration::from_secs(40));

        // Before the first move the clock is not running, so no time is charged.
        clock.restore(0);
        assert!(!clock.tick(Color::White, t0 + Duration::from_secs(100)));
        assert_eq!(clock.remaining(Color::White), Duration::from_secs(60));

        // A different second move replaces the old snapshot.
        clock.restore(1);
        clock.press(Color::Black, t0 + Duration::from_secs(25), 2);
        clock.restore(2);
        assert_eq!(clock.remaining(Color::Black), Duration::from_secs(60));
    }

    #[test]
    fn formatting() {
        assert_eq!(Clock::format(Duration::from_secs(300)), "5:00");
        assert_eq!(Clock::format(Duration::from_secs(65)), "1:05");
        assert_eq!(Clock::format(Duration::from_millis(9_870)), "0:09.9");
    }
}
