use fsm::machines::lathe::{LatheCommand, LatheController};
use fsm::machines::mill::{FsmController, MillCommand};

use std::thread;
use std::time::Duration;

fn main() {
    run_lathe();

    run_mill();
}

fn run_lathe() {
    println!("=== Threaded Lathe Demo ===\n");

    let controller = LatheController::create(Box::default());

    println!("Sending StartSpinning(1000) command...");
    controller
        .send_command(LatheCommand::StartSpinning(1000))
        .unwrap();

    thread::sleep(Duration::from_millis(10));

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

fn run_mill() {
    println!("=== Threaded Mill Demo ===\n");

    let controller = FsmController::create(Box::default());

    println!("Sending StartSpinning(1000) command...");
    controller
        .send_command(MillCommand::StartSpinning(1000))
        .unwrap();

    thread::sleep(Duration::from_millis(10));

    for response in controller.check_responses() {
        println!("Response: {:?}", response);
    }

    println!("\nSending Feed(500) command...");
    controller.send_command(MillCommand::Move(500)).unwrap();

    thread::sleep(Duration::from_millis(10));

    for response in controller.check_responses() {
        println!("Response: {:?}", response);
    }

    println!("\nSending StopFeed command...");
    controller.send_command(MillCommand::StopMoving).unwrap();

    thread::sleep(Duration::from_millis(10));

    for response in controller.check_responses() {
        println!("Response: {:?}", response);
    }

    println!("\nSending StopSpinning");
    controller.send_command(MillCommand::StopSpinning).unwrap();

    thread::sleep(Duration::from_millis(10));

    for response in controller.check_responses() {
        println!("Response: {:?}", response);
    }

    println!("\nSending truly invalid command (Feed while Off)...");
    controller.send_command(MillCommand::Move(300)).unwrap();

    thread::sleep(Duration::from_millis(10));

    for response in controller.check_responses() {
        println!("Response: {:?}", response);
    }

    println!("\n=== Demo Complete ===");
}
