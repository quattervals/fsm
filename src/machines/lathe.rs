//! Lathe FSM implementation using hand-coded approach
//!
//! This module demonstrates a fully manual implementation of the finite state machine pattern
//! All boilerplate code is written explicitly to show the underlying mechanics

use std::marker::PhantomData;

use super::shared::{MachineController, StateHandler};

/// Commands that are sent to the lathe FSM
#[derive(Debug)]
pub enum LatheCommand {
    StartSpinning(u32),
    StopSpinning,
    Feed(u32),
    StopFeed,
    Notaus,
    Acknowledge,
}

/// Responses returned by the lathe FSM
#[derive(Debug, Clone, PartialEq)]
pub enum LatheResponse {
    Status {
        state: &'static str,
    },
    InvalidTransition {
        current_state: &'static str,
        attempted_command: String,
    },
}

/// Lathe states - zero-sized types for compile-time state tracking
#[derive(Debug)]
pub struct Off;
#[derive(Debug)]
pub struct Spinning;
#[derive(Debug)]
pub struct Feeding;
#[derive(Debug)]
pub struct Notaus;

/// Business data for the lathe FSM
#[derive(Default, Debug)]
pub struct LatheData {
    revs: u32,
    feed: u32,
}

/// Main FSM struct using type-state pattern
///
/// This is manually implemented.
/// The generic `State` parameter ensures compile-time verification of valid state transitions.
/// The actual data needed for the operation is passed around as a reference to a boxed value.
/// Therefore, no extra stack or heap allocations are needed.
#[derive(Debug)]
pub struct Lathe<State> {
    state: PhantomData<State>,
    lathe_data: Box<LatheData>,
}

/// Generic implementations available for all states
impl<State> Lathe<State> {
    /// Creates a new lathe FSM in the Off state
    pub fn new(data: Box<LatheData>) -> Lathe<Off> {
        Lathe {
            state: PhantomData,
            lathe_data: data,
        }
    }

    /// Emergency stop transition available from any state
    pub fn notaus(self) -> Lathe<Notaus> {
        Lathe {
            state: PhantomData,
            lathe_data: self.lathe_data,
        }
    }

    /// Debug helper to print current state and data
    pub fn print(&self) {
        println!("State {:?}, Data {:#?}", self.state, self.lathe_data)
    }
}

/// State-specific transitions for Off state
impl Lathe<Off> {
    pub fn start_spinning(mut self, revs: u32) -> Lathe<Spinning> {
        self.lathe_data.revs = revs;
        Lathe {
            state: PhantomData,
            lathe_data: self.lathe_data,
        }
    }
}

/// State-specific transitions for Spinning state
impl Lathe<Spinning> {
    pub fn feed(mut self, feed: u32) -> Lathe<Feeding> {
        self.lathe_data.feed = feed;

        Lathe {
            state: PhantomData,
            lathe_data: self.lathe_data,
        }
    }
    pub fn off(mut self) -> Lathe<Off> {
        self.lathe_data = Default::default();
        Lathe {
            state: PhantomData,
            lathe_data: self.lathe_data,
        }
    }
}

/// State-specific transitions for Feeding state
impl Lathe<Feeding> {
    pub fn stop_feed(mut self) -> Lathe<Spinning> {
        self.lathe_data.feed = 0;

        Lathe {
            state: PhantomData,
            lathe_data: self.lathe_data,
        }
    }
}

/// State-specific transitions for Notaus (emergency stop) state
impl Lathe<Notaus> {
    pub fn acknowledge(mut self) -> Lathe<Off> {
        self.lathe_data = Default::default();
        Lathe {
            state: PhantomData,
            lathe_data: self.lathe_data,
        }
    }
}

/// Runtime wrapper enum for handling dynamic state switching
#[derive(Debug)]
pub enum LatheWrapper {
    Off(Lathe<Off>),
    Spinning(Lathe<Spinning>),
    Feeding(Lathe<Feeding>),
    Notaus(Lathe<Notaus>),
}

/// Wrapper implementation for runtime state management
impl LatheWrapper {
    pub fn new(lathe_data: Box<LatheData>) -> Self {
        LatheWrapper::Off(Lathe::<Off>::new(lathe_data))
    }

    /// Delegates command handling to the appropriate state-specific handler
    pub fn handle_cmd(self, cmd: LatheCommand) -> (LatheWrapper, LatheResponse) {
        match self {
            LatheWrapper::Off(lathe) => lathe.handle_cmd(cmd),
            LatheWrapper::Spinning(lathe) => lathe.handle_cmd(cmd),
            LatheWrapper::Feeding(lathe) => lathe.handle_cmd(cmd),
            LatheWrapper::Notaus(lathe) => lathe.handle_cmd(cmd),
        }
    }
}

impl From<Box<LatheData>> for LatheWrapper {
    fn from(lathe_data: Box<LatheData>) -> Self {
        LatheWrapper::Off(Lathe::<Off>::new(lathe_data))
    }
}

impl StateHandler<LatheCommand, LatheResponse, LatheWrapper> for LatheWrapper {
    fn handle_cmd(self, cmd: LatheCommand) -> (LatheWrapper, LatheResponse) {
        self.handle_cmd(cmd)
    }
}

/// Type alias for LatheController using the generic MachineController
pub type LatheController = MachineController<LatheCommand, LatheResponse>;
impl LatheController {
    pub fn create(lathe_data: Box<LatheData>) -> Self {
        MachineController::new::<Box<LatheData>, LatheWrapper>(lathe_data)
    }
}

/// Manual implementation of state-specific command handlers
///
/// Each state must implement the StateHandler trait, defining which commands
/// are valid and how they transform the state.
/// Command handler for Off state
impl StateHandler<LatheCommand, LatheResponse, LatheWrapper> for Lathe<Off> {
    fn handle_cmd(self, cmd: LatheCommand) -> (LatheWrapper, LatheResponse) {
        match cmd {
            LatheCommand::StartSpinning(revs) => {
                let new_lathe = self.start_spinning(revs);
                (
                    LatheWrapper::Spinning(new_lathe),
                    LatheResponse::Status { state: "Spinning" },
                )
            }
            LatheCommand::Notaus => {
                let new_lathe = self.notaus();
                (
                    LatheWrapper::Notaus(new_lathe),
                    LatheResponse::Status { state: "Notaus" },
                )
            }
            _ => (
                LatheWrapper::Off(self),
                LatheResponse::InvalidTransition {
                    current_state: "Off",
                    attempted_command: format!("{:?}", cmd),
                },
            ),
        }
    }
}

/// Command handler for Spinning state
impl StateHandler<LatheCommand, LatheResponse, LatheWrapper> for Lathe<Spinning> {
    fn handle_cmd(self, cmd: LatheCommand) -> (LatheWrapper, LatheResponse) {
        match cmd {
            LatheCommand::Feed(feed_rate) => {
                let new_lathe = self.feed(feed_rate);
                (
                    LatheWrapper::Feeding(new_lathe),
                    LatheResponse::Status { state: "Feeding" },
                )
            }
            LatheCommand::StopSpinning => {
                let new_lathe = self.off();
                (
                    LatheWrapper::Off(new_lathe),
                    LatheResponse::Status { state: "Off" },
                )
            }
            LatheCommand::Notaus => {
                let new_lathe = self.notaus();
                (
                    LatheWrapper::Notaus(new_lathe),
                    LatheResponse::Status { state: "Notaus" },
                )
            }
            _ => (
                LatheWrapper::Spinning(self),
                LatheResponse::InvalidTransition {
                    current_state: "Spinning",
                    attempted_command: format!("{:?}", cmd),
                },
            ),
        }
    }
}

/// Command handler for Feeding state
impl StateHandler<LatheCommand, LatheResponse, LatheWrapper> for Lathe<Feeding> {
    fn handle_cmd(self, cmd: LatheCommand) -> (LatheWrapper, LatheResponse) {
        match cmd {
            LatheCommand::StopFeed => {
                let new_lathe = self.stop_feed();
                (
                    LatheWrapper::Spinning(new_lathe),
                    LatheResponse::Status { state: "Spinning" },
                )
            }
            LatheCommand::Notaus => {
                let new_lathe = self.notaus();
                (
                    LatheWrapper::Notaus(new_lathe),
                    LatheResponse::Status { state: "Notaus" },
                )
            }
            _ => (
                LatheWrapper::Feeding(self),
                LatheResponse::InvalidTransition {
                    current_state: "Feeding",
                    attempted_command: format!("{:?}", cmd),
                },
            ),
        }
    }
}

/// Command handler for Notaus (emergency stop) state
impl StateHandler<LatheCommand, LatheResponse, LatheWrapper> for Lathe<Notaus> {
    fn handle_cmd(self, cmd: LatheCommand) -> (LatheWrapper, LatheResponse) {
        match cmd {
            LatheCommand::Acknowledge => {
                let new_lathe = self.acknowledge();
                (
                    LatheWrapper::Off(new_lathe),
                    LatheResponse::Status { state: "Off" },
                )
            }
            _ => (
                LatheWrapper::Notaus(self),
                LatheResponse::InvalidTransition {
                    current_state: "Notaus",
                    attempted_command: format!("{:?}", cmd),
                },
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod state_transitions {
        use super::*;

        #[test]
        fn off_to_spinning_transition() {
            let data = Box::new(LatheData::default());
            let lathe = Lathe::<Off>::new(data);

            let spinning_lathe = lathe.start_spinning(1500);

            assert_eq!(spinning_lathe.lathe_data.revs, 1500);
        }

        #[test]
        fn spinning_to_feeding_transition() {
            let data = Box::new(LatheData::default());
            let lathe = Lathe::<Off>::new(data).start_spinning(1000);

            let feeding_lathe = lathe.feed(250);

            assert_eq!(feeding_lathe.lathe_data.feed, 250);
            assert_eq!(feeding_lathe.lathe_data.revs, 1000);
        }

        #[test]
        fn feeding_to_spinning_transition() {
            let data = Box::new(LatheData::default());
            let lathe = Lathe::<Off>::new(data).start_spinning(1200).feed(300);

            let spinning_lathe = lathe.stop_feed();

            assert_eq!(spinning_lathe.lathe_data.feed, 0);
            assert_eq!(spinning_lathe.lathe_data.revs, 1200);
        }

        #[test]
        fn emergency_stop_from_feeding() {
            let data = Box::new(LatheData::default());
            let lathe = Lathe::<Off>::new(data).start_spinning(1000).feed(200);

            let notaus_lathe = lathe.notaus();
            let off_lathe = notaus_lathe.acknowledge();

            assert_eq!(off_lathe.lathe_data.revs, 0);
            assert_eq!(off_lathe.lathe_data.feed, 0);
        }
    }

    mod controller_tests {
        use super::*;

        fn setup_lathe_controller() -> LatheController {
            let lathe_data = Box::new(LatheData::default());
            LatheController::create(lathe_data)
        }

        #[test]
        fn command_sequence() {
            let controller = setup_lathe_controller();

            controller
                .send_command(LatheCommand::StartSpinning(800))
                .unwrap();
            controller.send_command(LatheCommand::Feed(150)).unwrap();

            std::thread::sleep(std::time::Duration::from_millis(10));
            let responses = controller.check_responses();
            assert_eq!(responses.len(), 2);
            assert_eq!(responses[0], LatheResponse::Status { state: "Spinning" });
            assert_eq!(responses[1], LatheResponse::Status { state: "Feeding" });
        }

        #[test]
        fn emergency_stop() {
            let controller = setup_lathe_controller();

            controller
                .send_command(LatheCommand::StartSpinning(1000))
                .unwrap();
            controller.send_command(LatheCommand::Feed(200)).unwrap();
            controller.send_command(LatheCommand::Notaus).unwrap();

            std::thread::sleep(std::time::Duration::from_millis(10));
            let responses = controller.check_responses();
            assert_eq!(responses.len(), 3);
            assert_eq!(responses[0], LatheResponse::Status { state: "Spinning" });
            assert_eq!(responses[1], LatheResponse::Status { state: "Feeding" });
            assert_eq!(responses[2], LatheResponse::Status { state: "Notaus" });
        }

        #[test]
        fn invalid_transition() {
            let controller = setup_lathe_controller();

            controller.send_command(LatheCommand::Feed(200)).unwrap();

            std::thread::sleep(std::time::Duration::from_millis(10));
            let responses = controller.check_responses();
            assert_eq!(responses.len(), 1);
            assert_eq!(
                responses[0],
                LatheResponse::InvalidTransition {
                    current_state: "Off",
                    attempted_command: String::from("Feed(200)")
                }
            );
        }
    }
}
