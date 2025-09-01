/// Finite State Machine (FSM) implementation
/// This module provides a generic FSM framework
///
/// The FSM is implemented using a type-state pattern where the state is represented by a generic parameter.
/// This allows for compile-time checking of valid state transitions.
use std::marker::PhantomData;
use std::sync::mpsc;
use std::thread::{self, JoinHandle};

/// Represents a Finite State Machine with a specific state and data.
///
/// # Type Parameters
/// * `State` - The current state of the FSM
/// * `FsmData` - The data associated with the FSM
pub struct FSM<State, FsmData> {
    pub state: PhantomData<State>,
    pub data: Box<FsmData>,
}

/// Macro for defining a Finite State Machine.
///
/// This macro generates the necessary implementations for the FSM based on the provided states and transitions.
///
/// # Parameters
/// * `StartState` - The initial state of the FSM
/// * `MachineData` - The type of data associated with the FSM
/// * `MachineCommand` - The type of commands that are be sent to the FSM
/// * `MachineResponse` - The type of responses that are returned by the FSM
/// * `StateHandlerTrait` - The trait that defines the interface for handling commands
/// * `Controller` - The type of controller for the FSM
/// * The rest of the parameters define the states and transitions of the FSM
macro_rules! fsm {
(
    StartState: $start_state:ident,
    MachineData: $data:ident,
    MachineCommand: $command_type:ident,
    MachineResponse: $response:ident,
    StateHandlerTrait: $state_handler:ident,
    Controller: $controller:ident,
    $(
        $from_state:ident: {
            $(
               $command:ident $(($($param:ident: $param_type:ty),+))? => $handler_fn:ident => $to_state:ident
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
                pub fn $handler_fn(mut self $(, $($param: $param_type),+)?) -> FSM<$to_state, $data> {
                    $handler_fn(&mut self $(, $($param),+)?);
                    FSM {
                        state: PhantomData,
                        data: self.data,
                    }
                }
            )*
        }
    )*

    pub enum FsmWrapper {
        $(
            $from_state(FSM<$from_state, $data>),
        )*
    }

    impl FsmWrapper {
        pub fn new(machine_data: Box<$data>) -> Self {
            FsmWrapper::$start_state(FSM::<$start_state, $data>::new(machine_data))
        }

        pub fn handle_cmd(self, cmd: $command_type) -> (FsmWrapper, $response){
            match self {
                $(
                    FsmWrapper::$from_state(machine) => machine.handle_cmd(cmd),
                )*
            }
        }
    }

    impl From<Box<$data>> for FsmWrapper {
        fn from(data: Box<$data>) -> Self {
            FsmWrapper::$start_state(FSM::<$start_state, $data>::new(data))
        }
    }

    impl $state_handler<$command_type, $response, FsmWrapper> for FsmWrapper {
        fn handle_cmd(self, cmd: $command_type) -> (FsmWrapper, $response) {
            self.handle_cmd(cmd)
        }
    }

    pub type FsmController = $controller<$command_type, $response>;
    impl FsmController {
        pub fn create(data: Box<$data>) -> Self {
            $controller::new::<Box<$data>, FsmWrapper>(data)
        }
    }

    $(
        impl $state_handler<$command_type, $response, FsmWrapper> for FSM<$from_state, $data>{
            fn handle_cmd(self, cmd: $command_type) -> (FsmWrapper, $response){
                match cmd {
                    $(
                        $command_type::$command$(($($param),+))? => {
                            let new_fsm = self.$handler_fn($($($param),+)?);
                            (
                                FsmWrapper::$to_state(new_fsm),
                                $response::Status {state: stringify!($to_state)},
                            )
                        }
                    )*
                    _ => (
                        FsmWrapper::$from_state(self),
                        $response::InvalidTransition {
                            current_state: stringify!($from_state),
                            attempted_command: format!("{:?}", cmd),
                        }
                    )
                }
            }
        }
    )*
}


}

pub(in crate::machines) use fsm;

/// Trait for handling commands in the FSM.
///
/// # Type Parameters
/// * `Command` - The type of commands that can be handled
/// * `Response` - The type of responses that can be returned
/// * `FsmWrapper` - The type of FSM wrapper
pub trait StateHandler<Command, Response, FsmWrapper> {
    /// Handles a command and returns the new state and response.
    ///
    /// # Arguments
    /// * `self` - The current FSM instance
    /// * `cmd` - The command to handle
    ///
    /// # Returns
    /// A tuple containing the new FSM wrapper instance and the response
    fn handle_cmd(self, cmd: Command) -> (FsmWrapper, Response);
}

/// Controller for managing an FSM in a separate thread.
///
/// # Type Parameters
/// * `Command` - The type of commands that can be sent to the FSM
/// * `Response` - The type of responses that can be returned by the FSM
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
    /// Creates a new FSM controller with the given data.
    ///
    /// # Arguments
    /// * `machine_data` - The data to associate with the FSM
    ///
    /// # Returns
    /// A new FSM controller instance
    pub fn new<MachineData, FsmWrapper>(machine_data: MachineData) -> Self
    where
        FsmWrapper:
            Send + 'static + StateHandler<Command, Response, FsmWrapper> + From<MachineData>,
    {
        let fsm_wrapper = FsmWrapper::from(machine_data);

        let (cmd_tx, cmd_rx) = std::sync::mpsc::channel();
        let (response_tx, response_rx) = std::sync::mpsc::channel();
        let machine_thread = MachineThread::new(cmd_rx, response_tx, fsm_wrapper);

        let thread_handle = thread::spawn(move || {
            machine_thread.run();
        });

        Self {
            cmd_tx,
            response_rx,
            thread_handle,
        }
    }

    /// Sends a command to the FSM.
    ///
    /// # Arguments
    /// * `cmd` - The command to send
    ///
    /// # Returns
    /// `Ok(())` if the command was sent successfully, `Err` otherwise
    pub fn send_command(&self, cmd: Command) -> Result<(), mpsc::SendError<Command>> {
        self.cmd_tx.send(cmd)
    }

    /// Checks for any responses from the FSM.
    ///
    /// # Returns
    /// A vector of responses
    pub fn check_responses(&self) -> Vec<Response> {
        let mut responses = Vec::new();
        while let Ok(response) = self.response_rx.try_recv() {
            responses.push(response);
        }
        responses
    }

    /// Shuts down the FSM controller.
    ///
    /// # Returns
    /// `Ok(())` if the controller was shut down successfully, `Err` otherwise
    pub fn shutdown(self) -> Result<(), Box<dyn std::error::Error>> {
        drop(self.cmd_tx);

        self.thread_handle
            .join()
            .map_err(|_| "Thread join failed")?;
        Ok(())
    }
}

/// Thread for running the FSM.
///
/// # Type Parameters
/// * `Command` - The type of commands that can be sent to the FSM
/// * `Response` - The type of responses that can be returned by the FSM
/// * `FsmWrapper` - The type of FSM wrapper
struct MachineThread<Command, Response, FsmWrapper> {
    cmd_rx: mpsc::Receiver<Command>,
    response_tx: mpsc::Sender<Response>,
    fsm_wrapper: FsmWrapper,
}

impl<Command, Response, FsmWrapper> MachineThread<Command, Response, FsmWrapper>
where
    FsmWrapper: StateHandler<Command, Response, FsmWrapper>,
{
    /// Creates a new FSM thread.
    ///
    /// # Arguments
    /// * `cmd_rx` - The receiver for commands
    /// * `response_tx` - The sender for responses
    /// * `fsm_wrapper` - The FSM wrapper
    ///
    /// # Returns
    /// A new FSM thread instance
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

    /// Runs the FSM thread.
    fn run(mut self) {
        while let Ok(cmd) = self.cmd_rx.recv() {
            let (new_actor, response) = self.fsm_wrapper.handle_cmd(cmd);
            self.fsm_wrapper = new_actor;
            let _ = self.response_tx.send(response);
        }
    }
}
