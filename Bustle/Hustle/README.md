# Summary

>> [Reference](https://gitlab.freedesktop.org/bustle/bustle/-/blob/22f454058f203ab18e735348900151f27708cb59/c-sources/pcap-monitor.c#L875)

## `spawn_monitor`

<details>
<summary>Code</summary>

```c
// Spawns a subprocess with communication through a pseudo-terminal (PTY)
spawn_monitor (BustlePcapMonitor *self,
               const char *const *argv,
               GError **error)
{
  // Create a new subprocess launcher with stdout redirected to a pipe
  g_autoptr(GSubprocessLauncher) launcher =
    g_subprocess_launcher_new (G_SUBPROCESS_FLAGS_STDOUT_PIPE);

  // Open a master pseudo-terminal (PTY) and store its file descriptor in self->pt_master
  self->pt_master = posix_openpt (O_RDWR | O_NOCTTY | O_CLOEXEC);
  if (self->pt_master < 0)
    return throw_errno (error, "posix_openpt failed");

  // Unlock the master PTY
  if (unlockpt (self->pt_master) < 0)
    return throw_errno (error, "unlockpt failed");

  // Get the name of the slave PTY and store it in sname
  char sname[PATH_MAX] = { 0 };
  if (ptsname_r (self->pt_master, sname, G_N_ELEMENTS (sname)) != 0)
    return throw_errno (error, "ptsname_r failed");

  // Open the slave PTY using its name and store its file descriptor in pt_slave
  int pt_slave = open (sname, O_RDWR);
  if (pt_slave < 0)
    return throw_errno (error, "open(sname) failed");

  // Configure the subprocess launcher to use the slave PTY as stdin
  g_subprocess_launcher_take_stdin_fd (launcher, pt_slave);

  // If not running inside a Flatpak environment, set up controlling terminal for the child process
  if (!RUNNING_IN_FLATPAK)
    g_subprocess_launcher_set_child_setup (launcher, set_controlling_tty_to_stdin, NULL, NULL);
  /* otherwise, the session-helper process already does this for us */

  // Spawn the child process with the specified command-line arguments
  GSubprocess *child = g_subprocess_launcher_spawnv (launcher, argv, error);

  // Close the file descriptor of the slave PTY
  g_close (pt_slave, NULL);

  // Return the handle to the spawned subprocess
  return child;
}
```

</details>

> `spawn_monitor` sets up a subprocess launcher to spawn a child process with its standard input connected to a `pseudo-terminal` (PTY). It does this by opening a master PTY, unlocking it, and then opening the corresponding slave PTY. It configures the subprocess launcher to use the slave PTY as the child's standard input and then spawns the child process with the given command-line arguments. If not running in a Flatpak environment, it sets up the child process to use the PTY as its controlling terminal. Finally, it returns the handle to the spawned subprocess.


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

</details><br>

> this function sets up a subprocess launcher to execute the `tee` command, redirecting its standard input to the output of another process. It effectively duplicates the output of the monitored process into a file specified by `self->filename`.


## `initable_init`

<details>
<summary>Code</summary>

```c
static gboolean
initable_init (
    GInitable *initable,
    GCancellable *cancellable,
    GError **error)
{
  BustlePcapMonitor *self = BUSTLE_PCAP_MONITOR (initable); // Casts the 'initable' parameter to a BustlePcapMonitor instance
  g_autofree const char **argv = NULL; // Declares a NULL-initialized array of const char pointers to store arguments
  GInputStream *stdout_pipe = NULL; // Declares a pointer to store the stdout pipe of the spawned process

  // Builds command-line arguments based on 'self', which is a BustlePcapMonitor instance
  argv = build_argv (self, error); 
  if (NULL == argv)
    return FALSE; // Returns FALSE if building command-line arguments fails

  if (self->filename == NULL)
    {
      g_set_error (error, G_IO_ERROR, G_IO_ERROR_INVALID_ARGUMENT,
          "You must specify a filename"); // Sets an error if 'filename' is not specified
      return FALSE; // Returns FALSE if 'filename' is not specified
    }

  self->cancellable_cancelled_id =
    g_cancellable_connect (self->cancellable,
                           G_CALLBACK (cancellable_cancelled_cb),
                           self, NULL); // Connects a callback to handle cancellation

  self->dbus_monitor = spawn_monitor (self, (const char * const *) argv, error); // Spawns a process to monitor D-Bus
  if (self->dbus_monitor == NULL)
    return FALSE; // Returns FALSE if spawning D-Bus monitor process fails

  self->tee_proc = spawn_tee (self, error); // Spawns a process to tee the output
  if (self->tee_proc == NULL)
    return FALSE; // Returns FALSE if spawning tee process fails

  stdout_pipe = g_subprocess_get_stdout_pipe (self->tee_proc); // Retrieves the stdout pipe of the tee process
  g_return_val_if_fail (stdout_pipe != NULL, FALSE); // Checks if stdout pipe is not NULL
  g_return_val_if_fail (G_IS_POLLABLE_INPUT_STREAM (stdout_pipe), FALSE); // Checks if stdout pipe is a pollable input stream
  g_return_val_if_fail (G_IS_UNIX_INPUT_STREAM (stdout_pipe), FALSE); // Checks if stdout pipe is a UNIX input stream

  self->tee_source = g_pollable_input_stream_create_source (
      G_POLLABLE_INPUT_STREAM (stdout_pipe), self->cancellable); // Creates a source for the stdout pipe
  g_source_set_callback (self->tee_source,
      (GSourceFunc) dbus_monitor_readable, self, NULL); // Sets a callback function for the source
  g_source_attach (self->tee_source, NULL); // Attaches the source to the main context

  g_subprocess_wait_check_async (
      self->dbus_monitor,
      self->cancellable,
      wait_check_cb, g_object_ref (self)); // Starts asynchronous waiting for the D-Bus monitor process to finish

  self->state = STATE_STARTING; // Sets the state to 'STARTING'
  return TRUE; // Returns TRUE to indicate successful initialization
}

```

</details><br>

> It builds command-line arguments based on some internal parameters, checks if a filename is specified, setting an error and returning FALSE if not, connects a callback to handle cancellation, spawns processes to monitor D-Bus and tee its output, setting errors and returning FALSE if any of these operations fail, retrieves the stdout pipe of the tee process and sets up a source to monitor it, attaching it to the main context, starts asynchronous waiting for the D-Bus monitor process to finish and sets the state to 'STARTING' and returns TRUE to indicate successful initialization.


## `bustle_pcap_monitor_stop`

<details>
<summary>Code</summary>

```c
void
bustle_pcap_monitor_stop (
    BustlePcapMonitor *self)  // Function definition for stopping a BustlePcapMonitor instance.
{
  // Check if the monitor is already stopped, stopping, or in a new state.
  if (self->state == STATE_STOPPED ||  
      self->state == STATE_STOPPING ||
      self->state == STATE_NEW)
    {
      // If already in one of these states, log a debug message and return.
      g_debug ("%s: already in state %s", G_STRFUNC, STATES[self->state]);
      return;
    }

  // Update the monitor state to stopping.
  self->state = STATE_STOPPING;
  
  // Cancel any ongoing operations associated with the monitor's cancellable.
  g_cancellable_cancel (self->cancellable);
}
```

</details><br>

> State Check:

- It checks if the current state of the BustlePcapMonitor (self->state) is already in a stopped state, stopping state, or a new state. If it's in any of these states, it logs a debug message indicating that the monitor is already in that state and returns without performing any further action.

> State Update:

- If the monitor is not already stopping or stopped, it updates the monitor's state to stopping. This indicates that the monitor is in the process of being stopped.

> Cancellable Operation:

- It cancels any ongoing operations associated with the monitor's cancellable object (`self->cancellable`). This action interrupts any ongoing processes or tasks related to monitoring, effectively stopping them.

In summary, this function is responsible for stopping a BustlePcapMonitor instance by updating its state to stopping and canceling any ongoing operations associated with it.


## ``

<details>
<summary>Code</summary>

```c
static void
initable_iface_init (
    gpointer g_class,
    gpointer unused)
{
  // Cast the passed gpointer to GInitableIface pointer
  GInitableIface *iface = g_class;

  // Assign the init function pointer of the interface to the initable_init function
  iface->init = initable_init;
}

BustlePcapMonitor *
bustle_pcap_monitor_new (
    GBusType bus_type,
    const gchar *address,
    const gchar *filename,
    GError **error)
{
  // Create a new instance of the BUSTLE_TYPE_PCAP_MONITOR type with initialization options
  // using g_initable_new function
  return g_initable_new (
      BUSTLE_TYPE_PCAP_MONITOR, NULL, error,
      "bus-type", bus_type,
      "address", address,
      "filename", filename,
      NULL);
}
```

</details><br>

> initable_iface_init Function:

- This function is a callback used to initialize the GInitable interface.
- It takes two parameters: `g_class`, which is a pointer to the interface structure, and unused, which is not used in this function.
- Inside the function, it casts the `g_class` pointer to `GInitableIface` pointer.
- Then it assigns the init function pointer of the interface to the `initable_init` function.

> bustle_pcap_monitor_new Function:

- This function creates a new instance of the `BustlePcapMonitor` type with initialization options.
- It takes parameters for the bus type, address, filename, and a pointer to a `GError` pointer.
- Inside the function, it calls `g_initable_new` to create a new instance of `BUSTLE_TYPE_PCAP_MONITOR`.
- It passes `NULL` for the parent object, the `error` parameter, and then provides initialization options as key-value pairs for the object's properties ("bus-type", "address", "filename").
- Finally, it returns the newly created instance of `BustlePcapMonitor`.








## ``

<details>
<summary>Code</summary>

```c

```

</details><br>

>