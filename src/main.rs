use std::marker::PhantomData;

fn main() {
    let lathe_data: LatheData = Default::default();
    let lathe = Lathe::<Off>::new(Box::new(lathe_data));
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

#[derive(Default, Debug)]
pub struct LatheData {
    revs: u32,
    feed: u32,
}

#[derive(Debug)]
pub struct Lathe<State> {
    state: PhantomData<State>,
    business_data: Box<LatheData>,
}

impl<State> Lathe<State> {
    pub fn new(data: Box<LatheData>) -> Lathe<Off> {
        Lathe {
            state: PhantomData,
            business_data: data,
        }
    }

    pub fn notaus(self) -> Lathe<Notaus> {
        Lathe {
            state: PhantomData,
            business_data: self.business_data,
        }
    }

    pub fn print(&self) {
        println!("State {:?}, Data {:#?}", self.state, self.business_data)
    }
}

impl Lathe<Off> {
    pub fn start_spinning(mut self, revs: u32) -> Lathe<Spinning> {
        self.business_data.revs = revs;
        Lathe {
            state: PhantomData,
            business_data: self.business_data,
        }
    }
}

impl Lathe<Spinning> {
    pub fn feed(mut self, feed: u32) -> Lathe<Feeding> {
        self.business_data.feed = feed;

        Lathe {
            state: PhantomData,
            business_data: self.business_data,
        }
    }
    pub fn off(mut self) -> Lathe<Off> {
        self.business_data = Default::default();
        Lathe {
            state: PhantomData,
            business_data: self.business_data,
        }
    }
}

impl Lathe<Feeding> {
    pub fn stop_feed(mut self) -> Lathe<Spinning> {
        self.business_data.feed = 0;

        Lathe {
            state: PhantomData,
            business_data: self.business_data,
        }
    }
}

impl Lathe<Notaus> {
    pub fn quittieren(mut self) -> Lathe<Off> {
        self.business_data = Default::default();
        Lathe {
            state: PhantomData,
            business_data: self.business_data,
        }
    }
}
