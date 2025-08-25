use std::marker::PhantomData;
use std::sync::mpsc;
use std::thread::{self, JoinHandle};

pub struct FSM<State, FsmData> {
    pub state: PhantomData<State>,
    pub data: Box<FsmData>,
}

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
               $command:ident $(($($cmd_type:ty),*))? => $method:ident($self:ident $(, $($param:ident: $type:ty),*)?) -> $to_state:ident
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


  pub enum FsmWrapper {
    $(
        $from_state(FSM<$from_state, $data>),
    )*
  }


  impl FsmWrapper{
    pub fn new(machine_data: Box<$data>) -> Self {
        FsmWrapper::$start_state(FSM::<$start_state, $data>::new(machine_data))
    }

    pub fn handle_cmd(self, cmd: $command_type) -> (FsmWrapper, $response){
        match self{
            $(
                FsmWrapper::$from_state(machine) => machine.handle_cmd(cmd),
            )*
        }
    }
  }

  impl From<Box<$data>> for FsmWrapper {
    fn from(lathe_data: Box<$data>) -> Self {
        FsmWrapper::Off(FSM::<$start_state, $data>::new(lathe_data))
    }
  }

  impl $state_handler<$command_type, $response, FsmWrapper> for FsmWrapper {
    fn handle_cmd(self, cmd: $command_type) -> (FsmWrapper, $response) {
        self.handle_cmd(cmd)
    }
  }


  pub type FsmController = $controller<$command_type, $response>;
  impl FsmController {
    pub fn create(lathe_data: Box<$data>) -> Self {
        $controller::new::<Box<$data>, FsmWrapper>(lathe_data)
    }
  }


  $(
    impl $state_handler<$command_type, $response, FsmWrapper> for FSM<$from_state, $data>{
        fn handle_cmd(self, cmd: $command_type) -> (FsmWrapper, $response){
            match cmd {
                $(
                    $command_type::$command$(($($param),+))? => {
                        let new_fsm = self.$method($($($param),+)?);
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

};

}

pub(in crate::machines) use fsm;

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
