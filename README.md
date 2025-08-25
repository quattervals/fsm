# Distributed FSM Examples

**Features**
- Type Save FSM (type state pattern)
- FSM run in their own thread
- communication via message queues
- State Transition and message handling boiler plate managed by `fsm!` macro


One such FSM is a simplistic lathe

```plantuml
@startuml
state "Off" as off
state "Spindle Spinning" as spinning
state "Feed" as feeding
state "Emergency Stop" as notaus

off --> spinning
spinning --> feeding
spinning --> off
feeding --> spinning
off --> notaus
spinning --> notaus
feeding --> notaus
notaus --> off: Acknowledge
@enduml
```
