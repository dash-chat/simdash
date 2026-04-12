# simdash-simulation

Core simulation engine. Contains the `Simulation` struct that wraps the state machine, manages scenarios and behaviors, and supports stepping forward/back and jumping to arbitrary ticks.

Also provides `SimulationSnapshot` for serializing simulation state to JSON, and `setup_from_input` for detecting whether an input file is a config or a snapshot and creating the appropriate simulation.
