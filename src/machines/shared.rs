/// Finite State Machine (FSM) implementation
/// This module provides a generic FSM framework
///
/// The FSM is implemented using a type-state pattern where the state is represented by a generic parameter.
/// This allows for compile-time checking of valid state transitions.
use std::marker::PhantomData;
use std::sync::mpsc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Represents a Finite State Machine with a specific state and data.
///
/// # Type Parameters
/// * `State` - The current state of the FSM
/// * `FsmData` - The data associated with the FSM
pub struct FSM<State, FsmData> {
    pub state: PhantomData<State>,
    pub data: Box<FsmData>,
}

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
    #[allow(dead_code)] // allow the join handle for dev purposes. It might be needed later
    thread_handle: JoinHandle<()>,
    shutdown_tx: mpsc::Sender<()>,
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
        let (shutdown_tx, shutdown_rx) = std::sync::mpsc::channel();

        let thread_handle = thread::spawn(move || {
            machine_thread.run(shutdown_rx);
        });

        Self {
            cmd_tx,
            response_rx,
            thread_handle,
            shutdown_tx,
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
}

impl<Command, Response> Drop for MachineController<Command, Response>
where
    Command: Send + 'static,
    Response: Send + 'static,
{
    fn drop(&mut self) {
        let _ = self.shutdown_tx.send(());
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

    /// Runs the FSM thread
    ///
    /// - Terminates on reception of shutdown signal
    /// - Channel disconnection
    /// - Graceful shutdown when no more commands are expected
    fn run(mut self, shutdown_rx: mpsc::Receiver<()>) {
        let timeout = Duration::from_millis(100);

        loop {
            match shutdown_rx.try_recv() {
                Ok(()) => {
                    println!("FSM shutdown requested - terminating");
                    break;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    println!("FSM controller disconnected - terminating");
                    break;
                }
                Err(mpsc::TryRecvError::Empty) => {
                    // No shutdown signal, continue processing
                }
            }

            match self.cmd_rx.recv_timeout(timeout) {
                Ok(cmd) => {
                    let (new_actor, response) = self.fsm_wrapper.handle_cmd(cmd);
                    self.fsm_wrapper = new_actor;

                    if self.response_tx.send(response).is_err() {
                        println!("FSM response receiver disconnected - terminating");
                        break;
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    continue;
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    println!("FSM command sender disconnected - terminating");
                    break;
                }
            }
        }

        println!("FSM thread terminated");
    }
}
