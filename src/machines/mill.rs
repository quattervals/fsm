//! Mill FSM implementation using the `fsm!` macro
//!
//! This module demonstrates how to leverage the `fsm!` macro from `shared.rs` to generate
//! boilerplate code for finite state machine implementation. The macro automatically creates
//! state transition methods, wrapper enums, and command handling logic based on the declarative
//! state machine definition.
//!
//! Compare this with `lathe.rs` which implements the same FSM pattern manually to understand
//! the code generation benefits of the macro approach.

use super::shared::{FSM, MachineController, StateHandler, fsm};

use std::marker::PhantomData;

/// Mill states - these are zero-sized types used for compile-time state tracking
#[derive(Debug)]
pub struct Off;
#[derive(Debug)]
pub struct Spinning;
#[derive(Debug)]
pub struct Moving;
#[derive(Debug)]
pub struct Notaus;

/// Business data for the mill FSM
#[derive(Default, Debug)]
pub struct MillData {
    revs: u32,
    linear_move: i32,
}

/// Commands that can be sent to the mill FSM
#[derive(Debug)]
pub enum MillCommand {
    StartSpinning(u32),
    StopSpinning,
    Move(i32),
    StopMoving,
}

/// Responses returned by the mill FSM
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

// FSM definition using the `fsm!` macro
//
// This macro call generates all the boilerplate code that would otherwise need to be
// written manually.
// It creates:
// - State transition methods for each FSM struct
// - A wrapper enum to handle runtime state switching
// - Command handling implementations for each state
// - Controller type alias and factory method
//
// The declarative syntax makes the state machine structure clear and reduces
// the chance of implementation errors compared to manual coding.
fsm! {
  StartState: Off,
  MachineData: MillData,
  MachineCommand: MillCommand,
  MachineResponse: MillResponse,
  StateHandlerTrait: StateHandler,
  Controller: MachineController,
  Off: {
    StartSpinning(revs: u32) => start_spinning(self) -> Spinning {
      self.data.revs = revs;
    },
  },
  Spinning: {
    StopSpinning => stop_spinning(self) -> Off {
      self.data.revs = 0;
    },
    Move(linear_move: i32) => start_moving(self) -> Moving {
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
