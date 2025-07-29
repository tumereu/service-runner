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

TODO: explain

