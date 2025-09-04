//! Mill FSM implementation using the `rust-fsm` crate
//!
//! This module demonstrates how to leverage the `rust-fsm` crate along with
//! the existing FSM framework from `shared.rs` to implement a mill state machine.
//! It provides better type safety than manual state matching through rust-fsm's
//! generated state machine and integrates with the generic controller architecture.

use super::shared::{MachineController, StateHandler};
use rust_fsm::*;

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
        state: String,
    },
    InvalidTransition {
        current_state: String,
        attempted_command: String,
    },
}

// Define the state machine using the rust-fsm DSL
// This provides compile-time guarantees about valid transitions
state_machine! {
    #[derive(Debug)]
    /// Mill state machine with states and transitions
    mill_fsm(Off)

    Off(StartSpinning) => Spinning [SpinningStarted],
    Spinning => {
        StopSpinning => Off [SpinningStopped],
        Move => Moving [MovingStarted],
    },
    Moving => {
        StopMoving => Spinning [MovingStopped],
    }
}

/// Mill FSM wrapper that integrates rust-fsm with our data and framework
#[derive(Debug)]
pub struct MillFsm {
    machine: mill_fsm::StateMachine,
    data: Box<MillData>,
}

impl MillFsm {
    pub fn new(data: Box<MillData>) -> Self {
        Self {
            machine: mill_fsm::StateMachine::new(),
            data,
        }
    }

    /// Get current state name using rust-fsm's built-in capabilities
    pub fn get_state_name(&self) -> String {
        // rust-fsm generates Debug implementation that gives us the state name
        format!("{:?}", self.machine.state())
    }
}

impl From<Box<MillData>> for MillFsm {
    fn from(data: Box<MillData>) -> Self {
        Self::new(data)
    }
}

impl StateHandler<MillCommand, MillResponse, MillFsm> for MillFsm {
    fn handle_cmd(mut self, cmd: MillCommand) -> (MillFsm, MillResponse) {
        // rust-fsm provides compile-time guarantees that only valid transitions
        // will succeed, eliminating the need for manual state/command matching
        let result = match &cmd {
            MillCommand::StartSpinning(revs) => {
                self.data.revs = *revs;
                self.machine.consume(&mill_fsm::Input::StartSpinning)
            }
            MillCommand::StopSpinning => {
                self.data.revs = 0;
                self.machine.consume(&mill_fsm::Input::StopSpinning)
            }
            MillCommand::Move(linear_move) => {
                self.data.linear_move = *linear_move;
                self.machine.consume(&mill_fsm::Input::Move)
            }
            MillCommand::StopMoving => {
                self.data.linear_move = 0;
                self.machine.consume(&mill_fsm::Input::StopMoving)
            }
        };

        let response = match result {
            Ok(_) => MillResponse::Status {
                state: self.get_state_name(),
            },
            Err(_) => MillResponse::InvalidTransition {
                current_state: self.get_state_name(),
                attempted_command: format!("{:?}", cmd),
            },
        };

        (self, response)
    }
}

// Type aliases and controller setup using the generic framework
pub type FsmController = MachineController<MillCommand, MillResponse>;

impl FsmController {
    pub fn create(data: Box<MillData>) -> Self {
        MachineController::new::<Box<MillData>, MillFsm>(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod controller_tests {
        use super::*;

        fn setup_mill_controller() -> FsmController {
            let mill_data = Box::new(MillData::default());
            FsmController::create(mill_data)
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
            assert_eq!(
                responses[0],
                MillResponse::Status {
                    state: "Spinning".to_string()
                }
            );
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
            assert_eq!(
                responses[0],
                MillResponse::Status {
                    state: "Spinning".to_string()
                }
            );
            assert_eq!(
                responses[1],
                MillResponse::Status {
                    state: "Moving".to_string()
                }
            );
            assert_eq!(
                responses[2],
                MillResponse::Status {
                    state: "Spinning".to_string()
                }
            );
            assert_eq!(
                responses[3],
                MillResponse::Status {
                    state: "Off".to_string()
                }
            );
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
            assert_eq!(
                responses[0],
                MillResponse::Status {
                    state: "Spinning".to_string()
                }
            );
            assert_eq!(
                responses[1],
                MillResponse::Status {
                    state: "Moving".to_string()
                }
            );
            assert_eq!(
                responses[2],
                MillResponse::InvalidTransition {
                    current_state: "Moving".to_string(),
                    attempted_command: String::from("StopSpinning"),
                }
            );
        }
    }

    mod state_transitions {
        use super::*;

        fn setup() -> MillFsm {
            let data = Box::new(MillData::default());
            MillFsm::new(data)
        }

        #[test]
        fn off_to_spinning() {
            let mill_fsm = setup();

            let (mill_fsm, response) = mill_fsm.handle_cmd(MillCommand::StartSpinning(12));

            assert_eq!(12, mill_fsm.data.revs);
            assert_eq!(
                response,
                MillResponse::Status {
                    state: "Spinning".to_string()
                }
            );
        }

        #[test]
        fn spinning_to_moving() {
            let mill_fsm = setup();
            let (mill_fsm, _) = mill_fsm.handle_cmd(MillCommand::StartSpinning(12));

            let (mill_fsm, response) = mill_fsm.handle_cmd(MillCommand::Move(66));

            assert_eq!(12, mill_fsm.data.revs);
            assert_eq!(66, mill_fsm.data.linear_move);
            assert_eq!(
                response,
                MillResponse::Status {
                    state: "Moving".to_string()
                }
            );
        }

        #[test]
        fn spinning_to_off() {
            let mill_fsm = setup();
            let (mill_fsm, _) = mill_fsm.handle_cmd(MillCommand::StartSpinning(12));

            let (mill_fsm, response) = mill_fsm.handle_cmd(MillCommand::StopSpinning);

            assert_eq!(0, mill_fsm.data.revs);
            assert_eq!(
                response,
                MillResponse::Status {
                    state: "Off".to_string()
                }
            );
        }

        #[test]
        fn moving_to_spinning() {
            let mill_fsm = setup();
            let (mill_fsm, _) = mill_fsm.handle_cmd(MillCommand::StartSpinning(12));
            let (mill_fsm, _) = mill_fsm.handle_cmd(MillCommand::Move(66));

            let (mill_fsm, response) = mill_fsm.handle_cmd(MillCommand::StopMoving);

            assert_eq!(12, mill_fsm.data.revs);
            assert_eq!(0, mill_fsm.data.linear_move);
            assert_eq!(
                response,
                MillResponse::Status {
                    state: "Spinning".to_string()
                }
            );
        }

        #[test]
        fn print() {
            let mill_fsm = setup();
            println!(
                "Mill FSM state: {}, data: {:?}",
                mill_fsm.get_state_name(),
                mill_fsm.data
            );
        }
    }
}
