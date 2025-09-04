//! Mill FSM implementation using the `rust-fsm` crate
//!
//! This module demonstrates how to use the `rust-fsm` crate to implement
//! a finite state machine for a mill. The rust-fsm crate provides a DSL
//! for defining state machines with readable specifications.

use rust_fsm::*;
use std::sync::mpsc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Commands that can be sent to the mill FSM
#[derive(Debug, Clone)]
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

// Define the state machine using the rust-fsm DSL
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

/// Business data for the mill FSM
#[derive(Default, Debug)]
pub struct MillData {
    revs: u32,
    linear_move: i32,
}

/// Mill FSM wrapper that includes data and state machine
pub struct MillFSM {
    machine: mill_fsm::StateMachine,
    data: MillData,
}

impl MillFSM {
    pub fn new() -> Self {
        Self {
            machine: mill_fsm::StateMachine::new(),
            data: MillData::default(),
        }
    }

    /// Handle commands and update state and data
    pub fn handle_command(&mut self, cmd: MillCommand) -> MillResponse {
        let current_state = match self.machine.state() {
            mill_fsm::State::Off => "Off",
            mill_fsm::State::Spinning => "Spinning",
            mill_fsm::State::Moving => "Moving",
        };

        let result = match (&cmd, self.machine.state()) {
            (MillCommand::StartSpinning(revs), mill_fsm::State::Off) => {
                self.data.revs = *revs;
                match self.machine.consume(&mill_fsm::Input::StartSpinning) {
                    Ok(Some(mill_fsm::Output::SpinningStarted)) => Some("Spinning"),
                    _ => None,
                }
            }
            (MillCommand::StopSpinning, mill_fsm::State::Spinning) => {
                self.data.revs = 0;
                match self.machine.consume(&mill_fsm::Input::StopSpinning) {
                    Ok(Some(mill_fsm::Output::SpinningStopped)) => Some("Off"),
                    _ => None,
                }
            }
            (MillCommand::Move(linear_move), mill_fsm::State::Spinning) => {
                self.data.linear_move = *linear_move;
                match self.machine.consume(&mill_fsm::Input::Move) {
                    Ok(Some(mill_fsm::Output::MovingStarted)) => Some("Moving"),
                    _ => None,
                }
            }
            (MillCommand::StopMoving, mill_fsm::State::Moving) => {
                self.data.linear_move = 0;
                match self.machine.consume(&mill_fsm::Input::StopMoving) {
                    Ok(Some(mill_fsm::Output::MovingStopped)) => Some("Spinning"),
                    _ => None,
                }
            }
            _ => None,
        };

        match result {
            Some(new_state) => MillResponse::Status { state: new_state },
            None => MillResponse::InvalidTransition {
                current_state,
                attempted_command: format!("{:?}", cmd),
            },
        }
    }

    pub fn get_data(&self) -> &MillData {
        &self.data
    }

    pub fn get_state_name(&self) -> &'static str {
        match self.machine.state() {
            mill_fsm::State::Off => "Off",
            mill_fsm::State::Spinning => "Spinning",
            mill_fsm::State::Moving => "Moving",
        }
    }
}

/// Controller for managing the mill FSM in a separate thread
pub struct MillController {
    cmd_tx: mpsc::Sender<MillCommand>,
    response_rx: mpsc::Receiver<MillResponse>,
    #[allow(dead_code)]
    thread_handle: JoinHandle<()>,
    shutdown_tx: mpsc::Sender<()>,
}

impl MillController {
    pub fn new() -> Self {
        let (cmd_tx, cmd_rx) = mpsc::channel();
        let (response_tx, response_rx) = mpsc::channel();
        let (shutdown_tx, shutdown_rx) = mpsc::channel();

        let thread_handle = thread::spawn(move || {
            let mut mill = MillFSM::new();
            let timeout = Duration::from_millis(100);

            loop {
                match shutdown_rx.try_recv() {
                    Ok(()) => {
                        println!("Mill FSM shutdown requested - terminating");
                        break;
                    }
                    Err(mpsc::TryRecvError::Disconnected) => {
                        println!("Mill FSM controller disconnected - terminating");
                        break;
                    }
                    Err(mpsc::TryRecvError::Empty) => {
                        // No shutdown signal, continue processing
                    }
                }

                match cmd_rx.recv_timeout(timeout) {
                    Ok(cmd) => {
                        let response = mill.handle_command(cmd);

                        if response_tx.send(response).is_err() {
                            println!("Mill FSM response receiver disconnected - terminating");
                            break;
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        continue;
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        println!("Mill FSM command sender disconnected - terminating");
                        break;
                    }
                }
            }

            println!("Mill FSM thread terminated");
        });

        Self {
            cmd_tx,
            response_rx,
            thread_handle,
            shutdown_tx,
        }
    }

    pub fn send_command(&self, cmd: MillCommand) -> Result<(), mpsc::SendError<MillCommand>> {
        self.cmd_tx.send(cmd)
    }

    pub fn check_responses(&self) -> Vec<MillResponse> {
        let mut responses = Vec::new();
        while let Ok(response) = self.response_rx.try_recv() {
            responses.push(response);
        }
        responses
    }

    pub fn create(data: Box<MillData>) -> Self {
        // For compatibility with the existing API, we ignore the data parameter
        // since our controller creates its own MillFSM with default data
        let _ = data;
        Self::new()
    }
}

impl Drop for MillController {
    fn drop(&mut self) {
        let _ = self.shutdown_tx.send(());
    }
}

// Type aliases for compatibility with existing code
pub type FsmController = MillController;

#[cfg(test)]
mod tests {

    mod controller_tests {
        use super::*;

        fn setup_mill_controller() -> FsmController {
            FsmController::new()
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
        }
    }

    use super::*;
    mod state_transitions {
        use super::*;

        fn setup() -> MillFSM {
            MillFSM::new()
        }

        #[test]
        fn off_to_spinning() {
            let mut mill_fsm = setup();

            let response = mill_fsm.handle_command(MillCommand::StartSpinning(12));

            assert_eq!(12, mill_fsm.data.revs);
            assert_eq!(response, MillResponse::Status { state: "Spinning" });
        }

        #[test]
        fn spinning_to_moving() {
            let mut mill_fsm = setup();
            mill_fsm.handle_command(MillCommand::StartSpinning(12));

            let response = mill_fsm.handle_command(MillCommand::Move(66));

            assert_eq!(12, mill_fsm.data.revs);
            assert_eq!(66, mill_fsm.data.linear_move);
            assert_eq!(response, MillResponse::Status { state: "Moving" });
        }

        #[test]
        fn spinning_to_off() {
            let mut mill_fsm = setup();
            mill_fsm.handle_command(MillCommand::StartSpinning(12));

            let response = mill_fsm.handle_command(MillCommand::StopSpinning);

            assert_eq!(0, mill_fsm.data.revs);
            assert_eq!(response, MillResponse::Status { state: "Off" });
        }

        #[test]
        fn moving_to_spinning() {
            let mut mill_fsm = setup();
            mill_fsm.handle_command(MillCommand::StartSpinning(12));
            mill_fsm.handle_command(MillCommand::Move(66));

            let response = mill_fsm.handle_command(MillCommand::StopMoving);

            assert_eq!(12, mill_fsm.data.revs);
            assert_eq!(0, mill_fsm.data.linear_move);
            assert_eq!(response, MillResponse::Status { state: "Spinning" });
        }

        #[test]
        fn print() {
            let mill_fsm = setup();
            println!(
                "Mill FSM state: {}, data: {:?}",
                mill_fsm.get_state_name(),
                mill_fsm.get_data()
            );
        }
    }
}
