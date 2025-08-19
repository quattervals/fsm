use std::marker::PhantomData;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::{self, JoinHandle};

#[derive(Debug)]
pub enum LatheCommand {
    StartSpinning(u32),
    StopSpinning,
    Feed(u32),
    StopFeed,
    Notaus,
    Acknowledge,
}

#[derive(Debug, Clone)]
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

/// Trait for state-specific command handling
pub trait StateHandler {
    fn handle_command(self, cmd: LatheCommand) -> (LatheActor, LatheResponse);
}

// State-specific handlers

impl StateHandler for Lathe<Off> {
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

impl StateHandler for Lathe<Spinning> {
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

impl StateHandler for Lathe<Feeding> {
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

impl StateHandler for Lathe<Notaus> {
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

/// Thread runner for a single lathe
pub struct LatheThread {
    cmd_rx: Receiver<LatheCommand>,
    response_tx: Sender<LatheResponse>,
    actor: LatheActor,
}

impl LatheThread {
    pub fn new(cmd_rx: Receiver<LatheCommand>, response_tx: Sender<LatheResponse>) -> Self {
        Self {
            cmd_rx,
            response_tx,
            actor: LatheActor::new(),
        }
    }

    pub fn run(mut self) {
        while let Ok(cmd) = self.cmd_rx.recv() {
            let (new_actor, response) = self.actor.handle_command(cmd);
            self.actor = new_actor;
            let _ = self.response_tx.send(response);
        }
    }
}

/// Coordinator for the lathe thread
pub struct LatheController {
    cmd_tx: Sender<LatheCommand>,
    response_rx: Receiver<LatheResponse>,
    thread_handle: JoinHandle<()>,
}

impl Default for LatheController {
    fn default() -> Self {
        Self::new()
    }
}

impl LatheController {
    pub fn new() -> Self {
        let (cmd_tx, cmd_rx) = std::sync::mpsc::channel();
        let (response_tx, response_rx) = std::sync::mpsc::channel();

        let lathe_thread = LatheThread::new(cmd_rx, response_tx);
        let thread_handle = thread::spawn(move || {
            lathe_thread.run();
        });

        Self {
            cmd_tx,
            response_rx,
            thread_handle,
        }
    }

    pub fn send_command(&self, cmd: LatheCommand) -> Result<(), &'static str> {
        self.cmd_tx.send(cmd).map_err(|_| "Failed to send command")
    }

    pub fn check_responses(&self) -> Vec<LatheResponse> {
        let mut responses = Vec::new();
        while let Ok(response) = self.response_rx.try_recv() {
            responses.push(response);
        }
        responses
    }

    pub fn shutdown(self) -> Result<(), Box<dyn std::error::Error>> {
        // Drop the command sender to signal the thread to exit
        drop(self.cmd_tx);
        // Wait for the thread to finish
        self.thread_handle
            .join()
            .map_err(|_| "Thread join failed")?;
        Ok(())
    }
}
