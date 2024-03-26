# PTY

>> The pty crate provides pty::fork(). That makes a parent process fork with new pseudo-terminal (PTY).<br>
>> This crate depends on followings:
>> - `libc` library
>> - `POSIX` environment

### `pty::fork()`

> This function returns pty::Child. It represents the child process and its PTY.<br>
> For example, the following code spawns `tty(1)` command by `pty::fork()` and outputs the result of the command.

<details>
<summary>Code</summary>

```rust
extern crate pty;
extern crate libc;

use std::ffi::CString;
use std::io::Read;
use std::process::{Command};

use pty::fork::*;

fn main() {
  let fork = Fork::from_ptmx().unwrap();

  if let Some(mut master) = fork.is_parent().ok() {
    // Read output via PTY master
    let mut output = String::new();

    match master.read_to_string(&mut output) {
      Ok(_nread) => println!("child tty is: {}", output.trim()),
      Err(e)     => panic!("read error: {}", e),
    }
  }
  else {
    // Child process just exec `tty`
    Command::new("tty").status().expect("could not execute tty");
  }
}
```

</details>

### `pty_process`

> This crate is a wrapper around [tokio::process::Command] or `std::process::Command` which provides the ability to allocate a pty and spawn new processes attached to that pty, with the pty as their controlling terminal. This allows for manipulation of interactive programs.<br>

> The basic functionality looks like this:

```rust
let mut pty = pty_process::Pty::new()?;
pty.resize(pty_process::Size::new(24, 80))?;
let mut cmd = pty_process::Command::new("nethack");
let child = cmd.spawn(&pty.pts()?)?;
let mut pty = pty_process::blocking::Pty::new()?;
pty.resize(pty_process::Size::new(24, 80))?;
let mut cmd = pty_process::blocking::Command::new("nethack");
let child = cmd.spawn(&pty.pts()?)?;
```

The returned child is a normal instance of [tokio::process::Child] (or `std::process::Child` for the blocking variant), with its `stdin/stdout/stderr` file descriptors pointing at the given pty. The `pty` instance implements [tokio::io::AsyncRead] and [tokio::io::AsyncWrite] (or `std::io::Read` and `std::io::Write` for the blocking variant), and can be used to communicate with the child process. The child process will also be made a session leader of a new session, and the controlling terminal of that session will be set to the given pty.

> **Features** :: By default, only the `blocking` APIs are available. To include the asynchronous APIs, you must enable the `async` feature.

### `pty_exec`

```rust
use std::os::fd::{AsRawFd, FromRawFd};
use pty_exec::Pty;

// spawn Pty...
let pty = Pty::spawn(move |_fd, res| {
    println!("-> {}", res.unwrap());
}, move |fd| {
    println!("-> {fd} died");
})?;

// (optional) create new pty, this maintains the on_read and on_death callbacks
let pty = unsafe { Pty::from_raw_fd(pty.as_raw_fd()) };

// write to original pty with new pty from_raw_fd
pty.write("echo 'Hello, World'\r")?;

pty.kill();
```