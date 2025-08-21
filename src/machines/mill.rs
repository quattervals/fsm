use super::shared::fsm;

pub fn try_macro() {
    fsm! {
        Off: {
          start_spinning(revs: u32) -> Spinning,
          do_stuff(bla: u32) -> Meier {
            let x = 34;
            data.revs = revs;
          },
        },
          Spinning: {
            stop_spinning(revs: u32, other: i32) -> Off,
            do_stuff(bla: u32) -> Meier,
        },
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn macron() {
        try_macro();
    }

    #[test]
    fn tryfsm() {
        fsm! {
            Off: {
                start_spinning(revs: u32) -> Spinning,
                do_stuff(bla: u32) -> Meier,
            },
             Spinning: {
                stop_spinning(revs: u32) -> Off,
                do_stuff(bla: u32) -> Meier,
            },
        }
    }
}
