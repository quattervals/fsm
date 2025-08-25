use super::shared::{FSM, fsm};

use std::marker::PhantomData;

#[derive(Debug)]
pub struct Off;
#[derive(Debug)]
pub struct Spinning;
#[derive(Debug)]
pub struct Feeding;
#[derive(Debug)]
pub struct Notaus;

pub fn try_macro() {
    //     fsm! {
    //         Off: {
    //           start_spinning(self, revs: u32) -> Spinning{
    //             self.data.revs = revs;
    //           },
    //           do_stuff(self, bla: u32) -> Notaus,
    //         },
    //           Spinning: {
    //             stop_spinning(self, revs: u32) -> Off{
    //                 self.data.revs = revs;
    //             },
    //             do_otherstuff(self, bla: u32) -> Notaus,
    //         },
    //     }
}

#[derive(Default, Debug)]
pub struct GenericFsmData {
    revs: u32,
    feed: u32,
}

fsm! {
  Off, GenericFsmData,
  Off: {
    start_spinning(self, revs: u32) -> Spinning {
      self.data.revs = revs;
    },
  },
  Spinning: {
    stop_spinning(self) -> Off{
      self.data.revs = 0;
    },
    start_feeding(self, feed: u32) -> Feeding {
      self.data.feed = feed;
    },
  },
  Feeding: {
    stop_feeding(self) -> Spinning {
      self.data.feed = 0;
    },
  },
}

#[cfg(test)]
mod tests {

    use super::*;

    fn setup() -> FSM<Off, GenericFsmData> {
        let data = Box::new(GenericFsmData::default());
        FSM::<Off, GenericFsmData>::new(data)
    }

    #[test]
    fn off_to_spinning() {
        let gen_fsm = setup();

        let gen_fsm = gen_fsm.start_spinning(12);

        assert_eq!(12, gen_fsm.data.revs);
    }

    #[test]
    fn spinning_to_feeding() {
        let gen_fsm = setup();
        let gen_fsm = gen_fsm.start_spinning(12);

        let gen_fsm = gen_fsm.start_feeding(66);

        assert_eq!(12, gen_fsm.data.revs);
        assert_eq!(66, gen_fsm.data.feed);
    }

    #[test]
    fn spinning_to_off() {
        let gen_fsm = setup();
        let gen_fsm = gen_fsm.start_spinning(12);

        let gen_fsm = gen_fsm.stop_spinning();

        assert_eq!(0, gen_fsm.data.revs);
    }

    #[test]
    fn feeding_to_spinning() {
        let gen_fsm = setup();
        let gen_fsm = gen_fsm.start_spinning(12);
        let gen_fsm = gen_fsm.start_feeding(66);

        let gen_fsm = gen_fsm.stop_feeding();

        assert_eq!(12, gen_fsm.data.revs);
        assert_eq!(0, gen_fsm.data.feed);
    }

    #[test]
    fn print() {
        let gen_fsm = setup();
        let gen_fsm = gen_fsm.start_spinning(12);
        let gen_fsm = gen_fsm.start_feeding(66);

        gen_fsm.print()
    }
}
