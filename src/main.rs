use fsm::machines::lathe::{LatheCommand, LatheController};
use fsm::machines::mill::try_macro;
use std::thread;
use std::time::Duration;

fn main() {

    try_macro();

    println!("=== Threaded Lathe Demo ===\n");

    // Create a lathe controller (spawns the lathe thread)
    let controller = LatheController::new();

    // Send some commands
    println!("Sending StartSpinning(1000) command...");
    controller
        .send_command(LatheCommand::StartSpinning(1000))
        .unwrap();

    // Give the thread time to process
    thread::sleep(Duration::from_millis(10));

    // Check for responses
    for response in controller.check_responses() {
        println!("Response: {:?}", response);
    }

    println!("\nSending Feed(500) command...");
    controller.send_command(LatheCommand::Feed(500)).unwrap();

    thread::sleep(Duration::from_millis(10));

    for response in controller.check_responses() {
        println!("Response: {:?}", response);
    }

    println!("\nSending StopFeed command...");
    controller.send_command(LatheCommand::StopFeed).unwrap();

    thread::sleep(Duration::from_millis(10));

    for response in controller.check_responses() {
        println!("Response: {:?}", response);
    }

    println!("\nSending StopSpinning");
    controller.send_command(LatheCommand::StopSpinning).unwrap();

    thread::sleep(Duration::from_millis(10));

    for response in controller.check_responses() {
        println!("Response: {:?}", response);
    }

    println!("\nSending truly invalid command (Feed while Off)...");
    controller.send_command(LatheCommand::Feed(300)).unwrap();

    thread::sleep(Duration::from_millis(10));

    for response in controller.check_responses() {
        println!("Response: {:?}", response);
    }

    println!("\nSending Notaus command...");
    controller.send_command(LatheCommand::Notaus).unwrap();

    thread::sleep(Duration::from_millis(10));

    for response in controller.check_responses() {
        println!("Response: {:?}", response);
    }

    println!("\nSending Acknowledge command...");
    controller.send_command(LatheCommand::Acknowledge).unwrap();

    thread::sleep(Duration::from_millis(10));

    for response in controller.check_responses() {
        println!("Response: {:?}", response);
    }

    println!("\n=== Demo Complete ===");
}
