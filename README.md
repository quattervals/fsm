# Distributed FSM Examples

**Features**
- Type Save FSM (type state pattern)
- FSM run in their own thread
- Communication via bidirectional message queues
- State transition and message handling boiler plate managed by `fsm!` macro

One such FSM is a simplistic lathe:
```mermaid
stateDiagram-v2
    state "Off" as off
    state "Spindle Spinning" as spinning
    state "Feed" as feeding
    state "Emergency Stop" as notaus

    [*] --> off
    off --> spinning
    spinning --> feeding
    spinning --> off
    feeding --> spinning
    off --> notaus
    spinning --> notaus
    feeding --> notaus
    notaus --> off : Acknowledge
```

Another FSM is a Mill with different state transitions but based on the same mechanics
```mermaid
stateDiagram-v2
    state "Off" as off
    state "Spinning" as spinning
    state "Moving" as moving

    [*] --> off
    off --> spinning : StartSpinning(revs)
    spinning --> off : StopSpinning
    spinning --> moving : Move(linear_move)
    moving --> spinning : StopMoving
```
