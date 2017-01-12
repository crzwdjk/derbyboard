/* The derby clock state machine. */

use std::time::{Instant, Duration};

#[derive(Clone, Copy)]
pub enum Clocktype {
    Jam,
    Lineup,
    TeamTimeout,
    OtherTimeout,
    Intermission,
}

impl Clocktype {
    pub fn next_state(self) -> Clocktype {
        match self {
            Clocktype::Jam => Clocktype::Lineup,
            Clocktype::Lineup => Clocktype::Jam,
            Clocktype::TeamTimeout => Clocktype::Lineup,
            _ => self,
        }
    }
    pub fn counts_down(self) -> bool {
        match self {
            Clocktype::Jam | Clocktype::Lineup | Clocktype::TeamTimeout
                | Clocktype::Intermission
                => true,
            _ => false,
        }
    }
    pub fn game_clock_runs(self) -> bool {
        match self {
            Clocktype::Jam|Clocktype::Lineup => true,
            _ => false,
        }
    }
}

pub struct Clock {
    gameclock: Duration,
    period: u8,
    clocktype: Clocktype,
    activeclock: Duration,
    lastupdate: Instant,
}

/* tick is called on every tick. Also,
 * every command has an implicit tick. */
impl Clock {
    pub fn new() -> Clock {
        Clock {
            gameclock: Duration::new(30*60, 0),
            period: 0,
            clocktype: Clocktype::Intermission,
            activeclock: Duration::new(0, 0),
            lastupdate: Instant::now(),
        }
    }
    fn clock_start_amt(&self, ty: Clocktype) -> Duration {
        // TODO: use clock policy
        match ty {
            Clocktype::Jam => Duration::new(120, 0),
            Clocktype::Lineup => Duration::new(30, 0),
            Clocktype::TeamTimeout => Duration::new(60, 0),
            Clocktype::Intermission => Duration::new(10 * 60, 0),
            _ => Duration::new(0, 0),
        }
    }

    fn start_clock(&mut self, ty: Clocktype, decrement: Option<Duration>)
                   -> () {
        let mut amt = self.clock_start_amt(ty);
        if let Some(d) = decrement {
            amt -= d;
        }
        self.clocktype = ty;
        self.activeclock = amt;
    }

    // Jam -> Lineup, Lineup -> Jam, TeamTimeout -> Lineup
    pub fn tick(&mut self) -> () {
        let now = Instant::now();
        let decrement = now - self.lastupdate;

        if self.clocktype.counts_down() {
            if self.activeclock > decrement {
                self.activeclock -= decrement;
            } else {
                let rem = decrement - self.activeclock;
                let nextstate = Clocktype::next_state(self.clocktype);
                self.start_clock(nextstate, Some(rem))
            };
        } else {
            self.activeclock += decrement;
        }

        /* Update game clock */
        if self.clocktype.game_clock_runs() {
            if decrement < self.gameclock {
                self.gameclock = self.gameclock - decrement;
            } else {
                self.gameclock = Duration::new(0, 0);
                if let Clocktype::Lineup = self.clocktype {
                    self.start_clock(Clocktype::Intermission, None);
                }
            }
        }

        self.lastupdate = now;
    }

    // Valid when clock is any but Jam.
    //  * -> Jam
    pub fn start_jam(&mut self) -> () {
        self.tick();
        if let Clocktype::Intermission = self.clocktype {
            self.period += 1;
            self.gameclock = Duration::new(30 * 60, 0);
        }

        if let Clocktype::Jam = self.clocktype {
            // don't do anything, it's already running.
        } else {
            self.start_clock(Clocktype::Jam, None);
        };
    }

    // {Jam, Lineup, OtherTimeout} -> TeamTimeout
    pub fn team_timeout(&mut self) -> () {
        self.tick();

        match self.clocktype {
            Clocktype::Jam | Clocktype::Lineup | Clocktype::OtherTimeout => {
                self.start_clock(Clocktype::TeamTimeout, None);
            },
            Clocktype::Intermission | Clocktype::TeamTimeout => {
                // can't start a team timeout
            }
        }
    }

    // {Jam, Lineup} -> OtherTimeout
    pub fn other_timeout(&mut self) -> () {
        self.tick();

        match self.clocktype {
            Clocktype::Jam | Clocktype::Lineup => {
                self.start_clock(Clocktype::OtherTimeout, None);
            },
            Clocktype::TeamTimeout => { /* convert to a team timeout */ },
            Clocktype::Intermission | Clocktype::OtherTimeout =>  {
                // can't start Other Timeout
            }
        }
    }

    // pub fn end_timeout(&mut self) -> ActiveClock {}
    // Jam -> Lineup
    pub fn stop_jam(&mut self) -> () {
        self.tick();

        match self.clocktype {
            Clocktype::Jam => {
                self.start_clock(Clocktype::Lineup, None);
            },
            _ => (), // Jam not running, can't stop.

        }
    }

    pub fn get_time(&self) -> (u8, Duration) {
        (self.period, self.gameclock)
    }

    pub fn get_active_clock(&self) -> (Clocktype, Duration) {
        (self.clocktype, self.activeclock)
    }
}