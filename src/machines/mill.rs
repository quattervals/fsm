use super::shared::{FSM, FsmData, fsm};

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

fsm! {
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

    #[test]
    fn off_to_spinning_transition() {}
}
