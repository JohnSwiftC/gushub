# GusHub - Simple Reverse Shell Session Manager

GusHub is a simple reverse shell session manager with no special requirements. GusHub functions in the same way as a netcat listener with managed sessions.

This works with any reverse shell, however I developed the tool to manage my (gus)[https://github.com/JohnSwiftC/gus] sessions. Have fun!

# Features

- Manage several remote shell sessions.
- Interactively disconnect shells.
- Quick access to IP information of connected shells.

# Usage

This is a CLI tool that takes the host port as an argument. Ex. `gushub.exe 4444` on Windows.

`clients` - Show a connected clients list.
`mainmenu` - Return to the session manager main menu when managing a shell session.
`close \<id\>` - Close a remote shell session when in the main menu.
`\<shell id\>` - Start managing a shell session.

# Build

Like any Rust Cargo project, use `cargo build --release` to build. The executable will be found in `/target/release`.

> Technically, you could also use `cargo run -- \<port\>` to run without building a binary.