use std::fmt::Debug;

fn main() {
    let lathe = Lathe::<Off>::new();
    lathe.print();

    let lathe = lathe.start_spinning(34);
    lathe.print();
    let lathe = lathe.off();
    lathe.print();
    // let lathe = lathe.feed(344); // prevented by compiler
    let lathe = lathe.start_spinning(34).feed(77);
    lathe.print();
    let lathe = lathe.stop_feed();
    lathe.print();
    let lathe = lathe.notaus();
    lathe.print();
}

#[derive(Debug)]
pub struct Off;
#[derive(Debug)]
pub struct Spinning;
#[derive(Debug)]
pub struct Feeding;
#[derive(Debug)]
pub struct Notaus;

#[derive(Debug)]
pub struct Lathe<State> {
    state: State,
    revs: u32,
    feed: u32,
}

impl<State> Lathe<State>
where
    State: Debug,
{
    pub fn new() -> Lathe<Off> {
        Lathe {
            state: Off,
            revs: 0,
            feed: 0,
        }
    }

    pub fn notaus(self) -> Lathe<Notaus> {
        Lathe {
            state: Notaus,
            revs: 0,
            feed: 0,
        }
    }

    pub fn print(&self) {
        println!(
            "State {:?}, Rev: {}, Feed: {}",
            self.state, self.revs, self.feed
        )
    }
}

impl Default for Lathe<Off> {
    fn default() -> Self {
        Self {
            state: Off,
            revs: 0,
            feed: 0,
        }
    }
}

impl Lathe<Off> {
    pub fn start_spinning(self, revs: u32) -> Lathe<Spinning> {
        Lathe {
            state: Spinning,
            revs,
            feed: 0,
        }
    }
}

impl Lathe<Spinning> {
    pub fn feed(self, feed: u32) -> Lathe<Feeding> {
        Lathe {
            state: Feeding,
            revs: self.revs,
            feed,
        }
    }
    pub fn off(self) -> Lathe<Off> {
        Lathe::default()
    }
}

impl Lathe<Feeding> {
    pub fn stop_feed(self) -> Lathe<Spinning> {
        Lathe {
            state: Spinning,
            revs: self.revs,
            feed: 0,
        }
    }
}

impl Lathe<Notaus> {
    pub fn quittieren(self) -> Lathe<Off> {
        Lathe {
            state: Off,
            revs: 0,
            feed: 0,
        }
    }
}
