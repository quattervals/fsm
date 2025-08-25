use std::{marker::PhantomData};

use super::shared::{MachineController, MachineController2, StateHandler, StateHandler2};

#[derive(Debug)]
pub enum LatheCommand {
    StartSpinning(u32),
    StopSpinning,
    Feed(u32),
    StopFeed,
    Notaus,
    Acknowledge,
}

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
    pub fn acknowledge(mut self) -> Lathe<Off> {
        self.business_data = Default::default();
        Lathe {
            state: PhantomData,
            business_data: self.business_data,
        }
    }
}

#[derive(Debug)]
pub enum LatheActor {
    Off(Lathe<Off>),
    Spinning(Lathe<Spinning>),
    Feeding(Lathe<Feeding>),
    Notaus(Lathe<Notaus>),
}

impl Default for LatheActor {
    fn default() -> Self {
        Self::new()
    }
}
impl LatheActor {
    pub fn new() -> Self {
        let data = Box::new(LatheData::default());
        LatheActor::Off(Lathe::<Off>::new(data))
    }
}

// State-specific handlers

impl StateHandler<LatheCommand, LatheActor, LatheResponse> for Lathe<Off> {
    fn handle_command(self, cmd: LatheCommand) -> (LatheActor, LatheResponse) {
        match cmd {
            LatheCommand::StartSpinning(revs) => {
                let new_lathe = self.start_spinning(revs);
                (
                    LatheActor::Spinning(new_lathe),
                    LatheResponse::Status { state: "Spinning" },
                )
            }
            LatheCommand::Notaus => {
                let new_lathe = self.notaus();
                (
                    LatheActor::Notaus(new_lathe),
                    LatheResponse::Status { state: "Notaus" },
                )
            }
            _ => (
                LatheActor::Off(self),
                LatheResponse::InvalidTransition {
                    current_state: "Off",
                    attempted_command: format!("{:?}", cmd),
                },
            ),
        }
    }
}

impl StateHandler<LatheCommand, LatheActor, LatheResponse> for Lathe<Spinning> {
    fn handle_command(self, cmd: LatheCommand) -> (LatheActor, LatheResponse) {
        match cmd {
            LatheCommand::Feed(feed_rate) => {
                let new_lathe = self.feed(feed_rate);
                (
                    LatheActor::Feeding(new_lathe),
                    LatheResponse::Status { state: "Feeding" },
                )
            }
            LatheCommand::StopSpinning => {
                let new_lathe = self.off();
                (
                    LatheActor::Off(new_lathe),
                    LatheResponse::Status { state: "Off" },
                )
            }
            LatheCommand::Notaus => {
                let new_lathe = self.notaus();
                (
                    LatheActor::Notaus(new_lathe),
                    LatheResponse::Status { state: "Notaus" },
                )
            }
            _ => (
                LatheActor::Spinning(self),
                LatheResponse::InvalidTransition {
                    current_state: "Spinning",
                    attempted_command: format!("{:?}", cmd),
                },
            ),
        }
    }
}

impl StateHandler<LatheCommand, LatheActor, LatheResponse> for Lathe<Feeding> {
    fn handle_command(self, cmd: LatheCommand) -> (LatheActor, LatheResponse) {
        match cmd {
            LatheCommand::StopFeed => {
                let new_lathe = self.stop_feed();
                (
                    LatheActor::Spinning(new_lathe),
                    LatheResponse::Status { state: "Spinning" },
                )
            }
            LatheCommand::Notaus => {
                let new_lathe = self.notaus();
                (
                    LatheActor::Notaus(new_lathe),
                    LatheResponse::Status { state: "Notaus" },
                )
            }
            _ => (
                LatheActor::Feeding(self),
                LatheResponse::InvalidTransition {
                    current_state: "Feeding",
                    attempted_command: format!("{:?}", cmd),
                },
            ),
        }
    }
}

impl StateHandler<LatheCommand, LatheActor, LatheResponse> for Lathe<Notaus> {
    fn handle_command(self, cmd: LatheCommand) -> (LatheActor, LatheResponse) {
        match cmd {
            LatheCommand::Acknowledge => {
                let new_lathe = self.acknowledge();
                (
                    LatheActor::Off(new_lathe),
                    LatheResponse::Status { state: "Off" },
                )
            }
            _ => (
                LatheActor::Notaus(self),
                LatheResponse::InvalidTransition {
                    current_state: "Notaus",
                    attempted_command: format!("{:?}", cmd),
                },
            ),
        }
    }
}

/// LatheActor dispatcher
impl LatheActor {
    pub fn handle_command(self, cmd: LatheCommand) -> (LatheActor, LatheResponse) {
        match self {
            LatheActor::Off(lathe) => lathe.handle_command(cmd),
            LatheActor::Spinning(lathe) => lathe.handle_command(cmd),
            LatheActor::Feeding(lathe) => lathe.handle_command(cmd),
            LatheActor::Notaus(lathe) => lathe.handle_command(cmd),
        }
    }
}

impl StateHandler<LatheCommand, LatheActor, LatheResponse> for LatheActor {
    fn handle_command(self, cmd: LatheCommand) -> (LatheActor, LatheResponse) {
        self.handle_command(cmd)
    }
}

/// Type alias for LatheController using the generic MachineController
pub type LatheController = MachineController<LatheCommand, LatheResponse, LatheActor>;

#[derive(Debug)]
pub enum LatheWrapper {
    Off(Lathe<Off>),
    Spinning(Lathe<Spinning>),
    Feeding(Lathe<Feeding>),
    Notaus(Lathe<Notaus>),
}

impl LatheWrapper {
    pub fn new(lathe_data: Box<LatheData>) -> Self {
        LatheWrapper::Off(Lathe::<Off>::new(lathe_data))
    }
}

/// LatheActor dispatcher
impl LatheWrapper {
    pub fn handle_cmd(self, cmd: LatheCommand) -> (LatheWrapper, LatheResponse) {
        match self {
            LatheWrapper::Off(lathe) => lathe.handle_cmd(cmd),
            LatheWrapper::Spinning(lathe) => lathe.handle_cmd(cmd),
            LatheWrapper::Feeding(lathe) => lathe.handle_cmd(cmd),
            LatheWrapper::Notaus(lathe) => lathe.handle_cmd(cmd),
        }
    }
}

impl StateHandler2<LatheCommand, LatheResponse, LatheWrapper> for LatheWrapper {
    fn handle_cmd(self, cmd: LatheCommand) -> (LatheWrapper, LatheResponse) {
        self.handle_cmd(cmd)
    }
}

impl StateHandler2<LatheCommand, LatheResponse, LatheWrapper> for Lathe<Off> {
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

impl StateHandler2<LatheCommand, LatheResponse, LatheWrapper> for Lathe<Spinning> {
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

impl StateHandler2<LatheCommand, LatheResponse, LatheWrapper> for Lathe<Feeding> {
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

impl StateHandler2<LatheCommand, LatheResponse, LatheWrapper> for Lathe<Notaus> {
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

    fn setup() -> MachineController2<LatheCommand, LatheResponse, LatheWrapper> {
        let lathe_data = Box::new(LatheData::default());

        let lathe_wrapper = LatheWrapper::new(lathe_data);
        MachineController2::<LatheCommand, LatheResponse, LatheWrapper>::new(lathe_wrapper)
    }

    #[test]
    fn command_sequence() {
        let controller = setup();

        controller
            .send_command(LatheCommand::StartSpinning(800))
            .unwrap();
        controller.send_command(LatheCommand::Feed(150)).unwrap();

        std::thread::sleep(std::time::Duration::from_millis(10));
        let responses = controller.check_responses();
        assert_eq!(responses.len(), 2);
        assert_eq!(responses[0], LatheResponse::Status { state: "Spinning" });
        assert_eq!(responses[1], LatheResponse::Status { state: "Feeding" });

        controller.shutdown().unwrap();
    }

    mod state_transitions {
        use super::*;

        #[test]
        fn off_to_spinning_transition() {
            let data = Box::new(LatheData::default());
            let lathe = Lathe::<Off>::new(data);

            let spinning_lathe = lathe.start_spinning(1500);

            assert_eq!(spinning_lathe.business_data.revs, 1500);
        }

        #[test]
        fn spinning_to_feeding_transition() {
            let data = Box::new(LatheData::default());
            let lathe = Lathe::<Off>::new(data).start_spinning(1000);

            let feeding_lathe = lathe.feed(250);

            assert_eq!(feeding_lathe.business_data.feed, 250);
            assert_eq!(feeding_lathe.business_data.revs, 1000);
        }

        #[test]
        fn feeding_to_spinning_transition() {
            let data = Box::new(LatheData::default());
            let lathe = Lathe::<Off>::new(data).start_spinning(1200).feed(300);

            let spinning_lathe = lathe.stop_feed();

            assert_eq!(spinning_lathe.business_data.feed, 0);
            assert_eq!(spinning_lathe.business_data.revs, 1200);
        }

        #[test]
        fn emergency_stop_from_feeding() {
            let data = Box::new(LatheData::default());
            let lathe = Lathe::<Off>::new(data).start_spinning(1000).feed(200);

            let notaus_lathe = lathe.notaus();
            let off_lathe = notaus_lathe.acknowledge();

            assert_eq!(off_lathe.business_data.revs, 0);
            assert_eq!(off_lathe.business_data.feed, 0);
        }
    }

    mod controller_tests {
        use super::*;

        #[test]
        fn command_sequence() {
            let controller = LatheController::new();

            controller
                .send_command(LatheCommand::StartSpinning(800))
                .unwrap();
            controller.send_command(LatheCommand::Feed(150)).unwrap();

            std::thread::sleep(std::time::Duration::from_millis(10));
            let responses = controller.check_responses();
            assert_eq!(responses.len(), 2);
            assert_eq!(responses[0], LatheResponse::Status { state: "Spinning" });
            assert_eq!(responses[1], LatheResponse::Status { state: "Feeding" });

            controller.shutdown().unwrap();
        }

        #[test]
        fn emergency_stop() {
            let controller = LatheController::new();

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

            controller.shutdown().unwrap();
        }

        #[test]
        fn invalid_transition() {
            let controller = LatheController::new();

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
            controller.shutdown().unwrap();
        }
    }
}
