use super::shared::{FSM, MachineController, StateHandler, fsm};

use std::marker::PhantomData;

#[derive(Debug)]
pub struct Off;
#[derive(Debug)]
pub struct Spinning;
#[derive(Debug)]
pub struct Moving;
#[derive(Debug)]
pub struct Notaus;

pub fn try_macro() {}

#[derive(Default, Debug)]
pub struct MillData {
    revs: u32,
    linear_move: i32,
}

#[derive(Debug)]
pub enum MillCommand {
    StartSpinning(u32),
    StopSpinning,
    Move(i32),
    StopMoving,
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
    StartSpinning(u32) => start_spinning(self, revs: u32) -> Spinning {
      self.data.revs = revs;
    },
  },
  Spinning: {
    StopSpinning => stop_spinning(self) -> Off{
      self.data.revs = 0;
    },
    Move(i32) => start_moving(self, linear_move: i32) -> Moving {
      self.data.linear_move = linear_move;
    },
  },
  Moving: {
    StopMoving => stop_moving(self) -> Spinning {
      self.data.linear_move = 0;
    },
  },
}

#[cfg(test)]
mod tests {

    mod controller_tests {
        use super::*;

        fn setup_mill_controller() -> FsmController {
            let lathe_data = Box::new(MillData::default());
            FsmController::create(lathe_data)
        }
        fn teardown_mill_controller(controller: FsmController) {
            controller.shutdown().unwrap();
        }

        #[test]
        fn off_to_spinning_transition() {
            let mill_controller = setup_mill_controller();

            mill_controller
                .send_command(MillCommand::StartSpinning(800))
                .unwrap();

            std::thread::sleep(std::time::Duration::from_millis(10));
            let responses = mill_controller.check_responses();
            assert_eq!(responses.len(), 1);
            assert_eq!(responses[0], MillResponse::Status { state: "Spinning" });

            teardown_mill_controller(mill_controller);
        }

        #[test]
        fn command_sequence() {
            let mill_controller = setup_mill_controller();

            mill_controller
                .send_command(MillCommand::StartSpinning(800))
                .unwrap();
            mill_controller
                .send_command(MillCommand::Move(-50))
                .unwrap();
            mill_controller
                .send_command(MillCommand::StopMoving)
                .unwrap();
            mill_controller
                .send_command(MillCommand::StopSpinning)
                .unwrap();

            std::thread::sleep(std::time::Duration::from_millis(10));
            let responses = mill_controller.check_responses();
            assert_eq!(responses.len(), 4);
            assert_eq!(responses[0], MillResponse::Status { state: "Spinning" });
            assert_eq!(responses[1], MillResponse::Status { state: "Moving" });
            assert_eq!(responses[2], MillResponse::Status { state: "Spinning" });
            assert_eq!(responses[3], MillResponse::Status { state: "Off" });

            teardown_mill_controller(mill_controller);
        }

        #[test]
        fn invalid_transition() {
            let mill_controller = setup_mill_controller();

            mill_controller
                .send_command(MillCommand::StartSpinning(800))
                .unwrap();
            mill_controller
                .send_command(MillCommand::Move(-50))
                .unwrap();
            mill_controller
                .send_command(MillCommand::StopSpinning)
                .unwrap();

            std::thread::sleep(std::time::Duration::from_millis(10));
            let responses = mill_controller.check_responses();
            assert_eq!(responses.len(), 3);
            assert_eq!(responses[0], MillResponse::Status { state: "Spinning" });
            assert_eq!(responses[1], MillResponse::Status { state: "Moving" });
            assert_eq!(
                responses[2],
                MillResponse::InvalidTransition {
                    current_state: "Moving",
                    attempted_command: String::from("StopSpinning"),
                }
            );

            teardown_mill_controller(mill_controller);
        }
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

            let gen_fsm = gen_fsm.start_moving(66);

            assert_eq!(12, gen_fsm.data.revs);
            assert_eq!(66, gen_fsm.data.linear_move);
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
            let gen_fsm = gen_fsm.start_moving(66);

            let gen_fsm = gen_fsm.stop_moving();

            assert_eq!(12, gen_fsm.data.revs);
            assert_eq!(0, gen_fsm.data.linear_move);
        }

        #[test]
        fn print() {
            let gen_fsm = setup();
            let gen_fsm = gen_fsm.start_spinning(12);
            let gen_fsm = gen_fsm.start_moving(66);

            gen_fsm.print()
        }
    }
}
