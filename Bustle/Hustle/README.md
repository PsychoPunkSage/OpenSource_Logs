# Summary

## `spawn_monitor`

<details>
<summary>Code</summary>

```c
spawn_monitor (BustlePcapMonitor *self,
               const char *const *argv,
               GError **error)
{
  g_autoptr(GSubprocessLauncher) launcher =
    g_subprocess_launcher_new (G_SUBPROCESS_FLAGS_STDOUT_PIPE);

  self->pt_master = posix_openpt (O_RDWR | O_NOCTTY | O_CLOEXEC);
  if (self->pt_master < 0)
    return throw_errno (error, "posix_openpt failed");

  if (unlockpt (self->pt_master) < 0)
    return throw_errno (error, "unlockpt failed");

  char sname[PATH_MAX] = { 0 };
  if (ptsname_r (self->pt_master, sname, G_N_ELEMENTS (sname)) != 0)
    return throw_errno (error, "ptsname_r failed");

  int pt_slave = open (sname, O_RDWR);
  if (pt_slave < 0)
    return throw_errno (error, "open(sname) failed");

  g_subprocess_launcher_take_stdin_fd (launcher, pt_slave);
  if (!RUNNING_IN_FLATPAK)
    g_subprocess_launcher_set_child_setup (launcher, set_controlling_tty_to_stdin, NULL, NULL);
  /* otherwise, the session-helper process already does this for us */

  GSubprocess *child = g_subprocess_launcher_spawnv (launcher, argv, error);

  g_close (pt_slave, NULL);
  return child;
}

```

</details>




## `spawn_tee`

<details>
<summary>Code</summary>

```c
static GSubprocess *
spawn_tee (BustlePcapMonitor  *self,
           GError            **error)
{
  // This line initializes a new subprocess launcher
  g_autoptr(GSubprocessLauncher) launcher =
    g_subprocess_launcher_new (G_SUBPROCESS_FLAGS_STDOUT_PIPE); // `G_SUBPROCESS_FLAGS_STDOUT_PIPE`, indicate that the subprocess's standard output will be redirected to a pipe.
  GInputStream *stdout_pipe = NULL;
  gint stdout_fd = -1;
  //  Retrieves the standard output pipe from self->dbus_monitor, which is presumably a GSubprocess object associated with some process.
  stdout_pipe = g_subprocess_get_stdout_pipe (self->dbus_monitor);
  g_return_val_if_fail (stdout_pipe != NULL, FALSE);

  stdout_fd = g_unix_input_stream_get_fd (G_UNIX_INPUT_STREAM (stdout_pipe));
  g_return_val_if_fail (stdout_fd >= 0, FALSE);

  // Configures the subprocess launcher to take the standard input from the file descriptor obtained from the standard output pipe.
  g_subprocess_launcher_take_stdin_fd (launcher, stdout_fd);

  // Spawns a child process... The child process is expected to execute the command "tee"
  return g_subprocess_launcher_spawn (launcher, error,
                                      "tee", self->filename, NULL);
}
```

</details>

> this function sets up a subprocess launcher to execute the `tee` command, redirecting its standard input to the output of another process. It effectively duplicates the output of the monitored process into a file specified by `self->filename`.