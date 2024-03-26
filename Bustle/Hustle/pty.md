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