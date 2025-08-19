use fsm::machines::lathe::{Feeding, Lathe, LatheData, Notaus, Off, Spinning};

fn main() {
    let lathe_data: LatheData = Default::default();
    let lathe = Lathe::<Off>::new(Box::new(lathe_data));
    lathe.print();

    let lathe = lathe.start_spinning(34);
    lathe.print();
    let lathe = lathe.off();
    lathe.print();
    // let lathe = lathe.feed(344); // prevented by compiler
    let lathe = lathe.start_spinning(34).feed(77);
    lathe.print();
    let lathe = lathe.stop_feed();
    lathe.print();
    let lathe = lathe.notaus();
    lathe.print();
}

