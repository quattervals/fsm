use super::shared::{FSM, fsm, StateHandler, MachineController};

use std::marker::PhantomData;

#[derive(Debug)]
pub struct Off;
#[derive(Debug)]
pub struct Spinning;
#[derive(Debug)]
pub struct Feeding;
#[derive(Debug)]
pub struct Notaus;

pub fn try_macro() {}

#[derive(Default, Debug)]
pub struct MillData {
    revs: u32,
    feed: u32,
}



#[derive(Debug)]
pub enum MillCommand {
    StartSpinning(u32),
    StopSpinning,
    Move(i32),
    StopMoving,
    Notaus,
    Acknowledge,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MillResponse {
    Status {
        state: &'static str,
    },
    InvalidTransition {
        current_state: &'static str,
        attempted_command: String,
    },
}

fsm! {
  StartState: Off,
  MachineData: MillData,
  MachineCommand: MillCommand,
  MachineResponse: MillResponse,
  StateHandlerTrait: StateHandler,
  Controller: MachineController,
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

    mod controller_tests {
        use super::*;

        // fn setup_lathe_controller() -> LatheController {
        //     let lathe_data = Box::new(LatheData::default());
        //     LatheController::create(lathe_data)
        // }
    }

    use super::*;
    mod state_transitions {
        use super::*;

        fn setup() -> FSM<Off, MillData> {
            let data = Box::new(MillData::default());
            FSM::<Off, MillData>::new(data)
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
}
