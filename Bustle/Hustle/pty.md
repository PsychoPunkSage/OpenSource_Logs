# PTY

>> The pty crate provides pty::fork(). That makes a parent process fork with new pseudo-terminal (PTY).<br>
>> This crate depends on followings:
>> - `libc` library
>> - `POSIX` environment

### `pty::fork()`

> This function returns pty::Child. It represents the child process and its PTY.<br>
> For example, the following code spawns `tty(1)` command by `pty::fork()` and outputs the result of the command.

