use std::marker::PhantomData;
use std::sync::mpsc;
use std::thread::{self, JoinHandle};

macro_rules! my_macro {
    ($x:expr) => {
        println!("Shared macro says: {}", $x);
    };
}

pub(in crate::machines) use my_macro;

macro_rules! fsm {
(
    $(
        $from_state:ident: {
            $(
                $method:ident($($param:ident: $type:ty),*) -> $to_state:ident
                $($body:block)?
            ),*,
        } ,
    )*
) => {
 $(
    println!("From State: {}", stringify!($from_state));
    $(
        println!("  Method: {} -> {}", stringify!($method), stringify!($to_state));
        $(
            print!("    Param: {}: {}", stringify!($param), stringify!($type));

        )*
        print!("\n");

        $(
           println!("    body {}", stringify!($body));
        )?
    )*
)*
};
}

pub(in crate::machines) use fsm;
/// Trait for state-specific command handling
pub trait StateHandler<Command, Actor, Response> {
    fn handle_command(self, cmd: Command) -> (Actor, Response);
}

/// Thread runner for a single machine
struct MachineThread<Command, Response, Actor>
where
    Actor: std::default::Default + StateHandler<Command, Actor, Response>,
{
    cmd_rx: mpsc::Receiver<Command>,
    response_tx: mpsc::Sender<Response>,
    actor: Actor,
}

impl<Command, Response, Actor: std::default::Default + StateHandler<Command, Actor, Response>>
    MachineThread<Command, Response, Actor>
{
    fn new(cmd_rx: mpsc::Receiver<Command>, response_tx: mpsc::Sender<Response>) -> Self {
        Self {
            cmd_rx,
            response_tx,
            actor: Actor::default(),
        }
    }

    fn run(mut self) {
        while let Ok(cmd) = self.cmd_rx.recv() {
            let (new_actor, response) = self.actor.handle_command(cmd);
            self.actor = new_actor;
            let _ = self.response_tx.send(response);
        }
    }
}

pub struct MachineController<Command, Response, Actor>
where
    Command: Send + 'static,
    Response: Send + 'static,
    Actor: std::default::Default + StateHandler<Command, Actor, Response> + Send + 'static,
{
    cmd_tx: mpsc::Sender<Command>,
    response_rx: mpsc::Receiver<Response>,
    thread_handle: JoinHandle<()>,
    _phantom: PhantomData<Actor>,
}

impl<Command, Response, Actor> Default for MachineController<Command, Response, Actor>
where
    Command: Send + 'static,
    Response: Send + 'static,
    Actor: std::default::Default + StateHandler<Command, Actor, Response> + Send + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Command, Response, Actor> MachineController<Command, Response, Actor>
where
    Command: Send + 'static,
    Response: Send + 'static,
    Actor: std::default::Default + StateHandler<Command, Actor, Response> + Send + 'static,
{
    pub fn new() -> Self {
        let (cmd_tx, cmd_rx): (mpsc::Sender<Command>, mpsc::Receiver<Command>) =
            std::sync::mpsc::channel();
        let (response_tx, response_rx): (mpsc::Sender<Response>, mpsc::Receiver<Response>) =
            std::sync::mpsc::channel();

        let machine_thread: MachineThread<Command, Response, Actor> =
            MachineThread::new(cmd_rx, response_tx);
        let thread_handle = thread::spawn(move || {
            machine_thread.run();
        });

        Self {
            cmd_tx,
            response_rx,
            thread_handle,
            _phantom: PhantomData,
        }
    }

    pub fn send_command(&self, cmd: Command) -> Result<(), &'static str> {
        self.cmd_tx.send(cmd).map_err(|_| "Failed to send command")
    }

    pub fn check_responses(&self) -> Vec<Response> {
        let mut responses = Vec::new();
        while let Ok(response) = self.response_rx.try_recv() {
            responses.push(response);
        }
        responses
    }

    pub fn shutdown(self) -> Result<(), Box<dyn std::error::Error>> {
        drop(self.cmd_tx);

        self.thread_handle
            .join()
            .map_err(|_| "Thread join failed")?;
        Ok(())
    }
}
