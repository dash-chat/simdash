# simdash-runner

CLI simulation runner. Reads a config file or a previous simulation snapshot, runs the simulation for a given number of ticks (or until Ctrl-C), and writes the results to `gen/snapshot.json`.

Supports resuming: if the input file is a simulation snapshot, it replays to the saved tick and continues from there.
