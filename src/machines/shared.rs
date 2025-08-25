use std::marker::PhantomData;
use std::sync::mpsc;
use std::thread::{self, JoinHandle};

macro_rules! my_macro {
    ($x:expr) => {
        println!("Shared macro says: {}", $x);
    };
}

pub(in crate::machines) use my_macro;

pub struct FSM<State, FsmData> {
    pub state: PhantomData<State>,
    pub data: Box<FsmData>,
}

macro_rules! fsm {
(
    $start_state:ident, $data:ident,
    $(
        $from_state:ident: {
            $(
                $method:ident($self:ident $(, $($param:ident: $type:ty),*)?) -> $to_state:ident
                $({ $($body:tt)* })?
            ),*,
        } ,
    )*
) => {
    impl <$start_state, $data> FSM<$start_state, $data>{
        pub fn new(data: Box<$data>) -> FSM<$start_state, $data> {
            FSM{
                state: PhantomData,
                data
            }
        }
    }

    /// Common functions for every state
    impl<State, $data> FSM<State, $data>
    where $data : std::fmt::Debug
    {
        pub fn print(&self) {
            println!("State {:?}, Data {:#?}", self.state, self.data)
        }
    }

 $(

    impl FSM<$from_state, $data> {
        $(
            pub fn $method(mut $self $(, $($param: $type),+)?) -> FSM<$to_state, $data> {
                $($($body)*)?
                FSM {
                   state: PhantomData,
                   data: $self.data,
                }

            }
        )*
    }
  )*
};
}

pub(in crate::machines) use fsm;

/// Trait for state-specific command handling
pub trait StateHandler<Command, Response, FsmWrapper> {
    fn handle_cmd(self, cmd: Command) -> (FsmWrapper, Response);
}

pub struct MachineController<Command, Response>
where
    Command: Send + 'static,
    Response: Send + 'static,
{
    cmd_tx: mpsc::Sender<Command>,
    response_rx: mpsc::Receiver<Response>,
    thread_handle: JoinHandle<()>,
}

impl<Command, Response> MachineController<Command, Response>
where
    Command: Send + 'static,
    Response: Send + 'static,
{
    pub fn new<MachineData, FsmWrapper>(machine_data: MachineData) -> Self
    where
        FsmWrapper:
            Send + 'static + StateHandler<Command, Response, FsmWrapper> + From<MachineData>,
    {
        let fsm_wrapper = FsmWrapper::from(machine_data);

        let (cmd_tx, cmd_rx): (mpsc::Sender<Command>, mpsc::Receiver<Command>) =
            std::sync::mpsc::channel();
        let (response_tx, response_rx): (mpsc::Sender<Response>, mpsc::Receiver<Response>) =
            std::sync::mpsc::channel();

        let machine_thread: MachineThread<Command, Response, FsmWrapper> =
            MachineThread::new(cmd_rx, response_tx, fsm_wrapper);
        let thread_handle = thread::spawn(move || {
            machine_thread.run();
        });

        Self {
            cmd_tx,
            response_rx,
            thread_handle,
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

/// Thread runner for a single machine
struct MachineThread<Command, Response, FsmWrapper> {
    cmd_rx: mpsc::Receiver<Command>,
    response_tx: mpsc::Sender<Response>,
    fsm_wrapper: FsmWrapper,
}

impl<Command, Response, FsmWrapper> MachineThread<Command, Response, FsmWrapper>
where
    FsmWrapper: StateHandler<Command, Response, FsmWrapper>,
{
    fn new(
        cmd_rx: mpsc::Receiver<Command>,
        response_tx: mpsc::Sender<Response>,
        fsm_wrapper: FsmWrapper,
    ) -> Self {
        Self {
            cmd_rx,
            response_tx,
            fsm_wrapper,
        }
    }

    fn run(mut self) {
        while let Ok(cmd) = self.cmd_rx.recv() {
            let (new_actor, response) = self.fsm_wrapper.handle_cmd(cmd);
            self.fsm_wrapper = new_actor;
            let _ = self.response_tx.send(response);
        }
    }
}
