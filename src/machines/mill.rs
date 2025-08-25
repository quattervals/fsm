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
      start_spinning(self, revs: u32) -> Spinning{
        self.data.revs = revs;
      },
      do_stuff(self, bla: u32) -> Notaus,
    },
      Spinning: {
        stop_spinning(self) -> Off{
            self.data.revs = 0;
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
    fn off_to_spinning_transition() {
        let gen_fsm = setup();

        let gen_fsm = gen_fsm.start_spinning(12);

        assert_eq!(12, gen_fsm.data.revs);
    }
}
