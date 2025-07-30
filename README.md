# Service Runner

This repository contains a tool written in Rust that is intended for running services locally during development.
A service here refers to some application that requires some preparation (such as compiling it) and/or can be started
and stopped.
Service-runner offers a terminal based user-interface for managing the services, allowing quick restarts, compilations 
and other operations that are often convenient during software development.

## Building

Before building the first time, ensure that the openssl dev library is installed.
On Ubuntu 24.04, you can do this using

```bash
apt-get install libssl-dev pkg-config
```

After that, compile the project using cargo.

```bash
cargo build
```

## Setup

TODO: explain the setup here. Example folder of services and settings?

## Usage

Run the app from your favorite terminal, passing the location of the configuration directory as the sole argument.

```bash
target/debug/client ./config-local
```

### Profile selection

The first screen presented lists all profile found from the configuration directory.
Use arrow keys to select a profile and press Enter to activate it.

### Main screen

This screen features a dual-pane interface: the **Services Pane** and the **Output Pane**. 
Below is a primer on the controls for this.

#### General Controls
- **`Ctrl+Q`** — Quit the tool
- **`Tab`** — Switch focus between the Services Pane and the Output Pane

---

### Output Pane Controls
- **Arrow Keys** — Navigate output
- **`Ctrl + Arrow Keys`** — Faster navigation
- **`g`** — Jump to the beginning of the output
- **`Shift+G`** — Jump to the end of the output
- **`w`** — Toggle line wrapping on/off

---

### Services Pane Controls
- **Arrow Keys** — Navigate service selection
- **`r`** — Toggle a service on/off
- **`e`** — Restart a service
- **`c`** — Recompile a service
- **`o`** — Toggle service output visibility
- **`a`** — Toggle autocompilation for a service

> **Note on Autocompilation**:  
> Autocompilation may interfere with tasks like compiling unit tests. It's recommended to disable it in such cases to avoid conflicts from simultaneous recompilation.

#### Bulk Actions
Hold **Shift** while pressing a command key to apply it to **all services**.  
For example:
- **`Shift+A`** — Toggle autocompilation for all services

