# Summary

>> [Reference](https://gitlab.freedesktop.org/bustle/bustle/-/blob/22f454058f203ab18e735348900151f27708cb59/c-sources/pcap-monitor.c#L875)

## ``

<details>
<summary>Code</summary>

```c

```

</details><br>

>

## ``

<details>
<summary>Code</summary>

```c

```

</details><br>

>

## `dump_names_thread_func`

<details>
<summary>Code</summary>

```c
static void
dump_names_thread_func (
    GTask *task,
    gpointer source_object,
    gpointer task_data,
    GCancellable *cancellable)
{
  // Cast source_object to BustlePcapMonitor pointer
  BustlePcapMonitor *self = BUSTLE_PCAP_MONITOR (source_object);
  
  // Declare GDBusConnection and GDBusProxy pointers with automatic cleanup
  g_autoptr(GDBusConnection) connection = NULL;
  g_autoptr(GDBusProxy) bus = NULL;
  g_autoptr(GError) error = NULL;

  // Get DBus connection
  connection = get_connection (self, cancellable, &error);
  if (connection != NULL)
    {
      // Get unique name of the connection
      const gchar *unique_name = g_dbus_connection_get_unique_name (connection);

      if (unique_name != NULL)
        {
          // Mangle the unique name to form a well-known name
          g_autofree gchar *mangled = g_strdup (unique_name);
          g_autofree gchar *well_known_name =
            g_strconcat (BUSTLE_MONITOR_NAME_PREFIX,
                         /* ":3.14" -> "_3_14", a legal bus name component */
                         g_strcanon (mangled, "0123456789", '_'),
                         NULL);

          // Log attempting to own well-known name
          g_debug ("%s: attempting to own %s", G_STRFUNC, well_known_name);
          
          // Own the well-known name on the connection
          g_bus_own_name_on_connection (connection,
                                        well_known_name,
                                        G_BUS_NAME_OWNER_FLAGS_NONE,
                                        NULL /* acquired */,
                                        NULL /* lost */,
                                        NULL /* user_data */,
                                        NULL /* free_func */);
        }

      // Create a new DBus proxy
      bus = g_dbus_proxy_new_sync (connection,
                                   G_DBUS_PROXY_FLAGS_DO_NOT_LOAD_PROPERTIES |
                                   G_DBUS_PROXY_FLAGS_DO_NOT_CONNECT_SIGNALS |
                                   G_DBUS_PROXY_FLAGS_DO_NOT_AUTO_START,
                                   NULL,
                                   "org.freedesktop.DBus",
                                   "/org/freedesktop/DBus",
                                   "org.freedesktop.DBus",
                                   cancellable,
                                   &error);
    }

  // If bus is valid and listing all names is successful, return TRUE
  if (bus != NULL && list_all_names (bus, &error))
    g_task_return_boolean (task, TRUE);
  else
    // If there's an error, return it
    g_task_return_error (task, g_steal_pointer (&error));

  // Assert that there's no error
  g_assert (error == NULL);
  
  // Close DBus connection if it's open and log any error if encountered
  if (connection != NULL
      && !g_dbus_connection_close_sync (connection, cancellable, &error))
    g_warning ("%s: %s", G_STRFUNC, error->message);
}
```

</details><br>

> Overall, this function seems to be responsible for setting up a DBus connection, registering the application under a well-known name, and listing all available DBus names, possibly for monitoring purposes. 

## `dump_names_async`

<details>
<summary>Code</summary>

```c
static void
dump_names_async (
    BustlePcapMonitor *self) // Function definition for a function named dump_names_async, taking a pointer to BustlePcapMonitor struct as an argument
{
  g_autoptr(GTask) task = g_task_new (self, self->cancellable, dump_names_cb, NULL); // Creates a new GTask object using g_task_new, passing self (pointer to the BustlePcapMonitor instance), self->cancellable (cancellable), dump_names_cb (callback function), and NULL (userdata)

  g_task_run_in_thread (task, dump_names_thread_func); // Runs the GTask in a separate thread using g_task_run_in_thread, passing the task object and the function pointer dump_names_thread_func
}
```

</details><br>

> Overall, this function is a part of an asynchronous operation mechanism where a task (`dump_names_cb`) is executed in a separate thread to avoid blocking the main execution flow. It allows for concurrent processing of tasks without freezing the application's user interface or other operations.

## `send_sigint`

<details>
<summary>Code</summary>

```c
static void
send_sigint (BustlePcapMonitor *self)
{
  /* Send the signal directly; this has no effect on the privileged subprocess
   * used on the system bus.
   */
  if (self->dbus_monitor != NULL)
    g_subprocess_send_signal (self->dbus_monitor, SIGINT);

  /* Send it via the PTY that we set as the subprocess's controlling terminal;
   * this works even for a privileged child.
   */
  if (self->pt_master >= 0)
    {
      char ctrl_c = '\x03'; // Define the character for Ctrl+C (ASCII code 3)

      // Write the Ctrl+C character to the master PTY to send SIGINT to the subprocess
      if (write (self->pt_master, &ctrl_c, 1) != 1)
        {
          // If the write operation fails, handle the error
          g_autoptr(GError) local_error = NULL;
          throw_errno (&local_error, "write to pt_master failed");
          g_warning ("%s: %s", G_STRFUNC, local_error->message); // Log a warning with the error message
        }
    }
}
```

</details><br>

> Overall, this function sends a SIGINT signal both directly to a subprocess (if applicable) and via the PTY, ensuring that even privileged children receive the signal correctly.

## `start_pcap`

<details>
<summary>Code</summary>

```c
static gboolean
start_pcap (
    BustlePcapMonitor *self,
    GError **error)
{
  // Declaring variables to hold stdout pipe, stdout file descriptor, and a FILE pointer for dbus monitor
  GInputStream *stdout_pipe = NULL;
  gint stdout_fd = -1;
  FILE *dbus_monitor_filep = NULL;

  // Retrieving the stdout pipe from the tee process
  stdout_pipe = g_subprocess_get_stdout_pipe (self->tee_proc);
  g_return_val_if_fail (stdout_pipe != NULL, FALSE);

  // Getting the file descriptor of the stdout pipe
  stdout_fd = g_unix_input_stream_get_fd (G_UNIX_INPUT_STREAM (stdout_pipe));
  g_return_val_if_fail (stdout_fd >= 0, FALSE);

  // Opening a FILE pointer using the stdout file descriptor
  dbus_monitor_filep = fdopen (stdout_fd, "r");
  if (dbus_monitor_filep == NULL)
    {
      throw_errno (error, "fdopen failed");
      return FALSE;
    }

  // Ensuring the stream doesn't close its file descriptor when finalized
  g_unix_input_stream_set_close_fd (G_UNIX_INPUT_STREAM (stdout_pipe), FALSE);

  /* This reads the 4-byte pcap header from the pipe, in a single blocking
   * fread(). It's safe to do this on the main thread, since we know the pipe
   * is readable. On short read, pcap_fopen_offline() fails immediately.
   */
  // Opening a pcap reader using the dbus monitor FILE pointer
  self->reader = bustle_pcap_reader_fopen (g_steal_pointer (&dbus_monitor_filep), error);
  if (self->reader == NULL)
    {
      g_prefix_error (error, "Couldn't read messages from dbus-monitor: ");

      /* Try to terminate dbus-monitor immediately. The reader closes the FILE * on error. */
      send_sigint (self);

      return FALSE;
    }

  // Initiating asynchronous dump of names
  dump_names_async (self);
  // Setting state to running
  self->state = STATE_RUNNING;
  // Returning TRUE to indicate successful start
  return TRUE;
}
```

</details><br>

> This function is responsible for starting a pcap monitor. 

## `read_one`

<details>
<summary>Code</summary>

```c
static gboolean
read_one (
    BustlePcapMonitor *self,
    GError **error)
{
  glong sec, usec;
  const guchar *blob;
  guint length;
  g_autoptr(GDBusMessage) message = NULL;

  // Attempt to read a single message from the pcap reader
  if (!bustle_pcap_reader_read_one (self->reader, &sec, &usec, &blob, &length, &message, error))
  {
    // If reading fails, return FALSE to indicate failure
    return FALSE;
  }
  else if (message == NULL)
  {
    // If the message is NULL, it indicates end-of-file (EOF),
    // which shouldn't happen since the function waited for the file descriptor to be readable
    g_set_error (error, G_IO_ERROR, G_IO_ERROR_CONNECTION_CLOSED,
        "EOF when reading from dbus-monitor");
    return FALSE;
  }
  else
  {
    // If a valid message is read, emit a signal to notify listeners about the logged message
    g_signal_emit (self, signals[SIG_MESSAGE_LOGGED], 0,
        sec, usec, blob, length, message);

    // Return TRUE to indicate successful reading
    return TRUE;
  }
}
```

</details><br>

> Overall, this function is designed to read a single message from a pcap reader associated with a BustlePcapMonitor object, handle errors, and emit a signal to notify listeners about the logged message.

## `dbus_monitor_readable`

<details>
<summary>Code</summary>

```c
static gboolean
dbus_monitor_readable (
    GObject *pollable_input_stream,
    gpointer user_data)
{
  // Cast user_data back to BustlePcapMonitor pointer
  BustlePcapMonitor *self = BUSTLE_PCAP_MONITOR (user_data);
  // Function pointer for reading from pcap
  gboolean (*read_func) (BustlePcapMonitor *, GError **);

  // Ensure that pcap_error is not set
  g_return_val_if_fail (self->pcap_error == NULL, FALSE);

  // Set error if operation was cancelled
  if (g_cancellable_set_error_if_cancelled (self->cancellable, &self->pcap_error))
    {
      // Handle cancellation
      await_both_errors (self);
      return FALSE;
    }

  // Choose read function based on current state
  switch (self->state)
    {
    case STATE_STARTING:
      // Set read function to start_pcap
      read_func = start_pcap;
      break;

    case STATE_RUNNING:
    case STATE_STOPPING: /* may have a few last messages to read */
      // Set read function to read_one
      read_func = read_one;
      break;

    default:
      // Log an error for unexpected state
      g_critical ("%s in unexpected state %d (%s)",
                  G_STRFUNC, self->state, STATES[self->state]);
      return FALSE;
    }

  // Call the chosen read function
  if (!read_func (self, &self->pcap_error))
    {
      // Handle errors
      await_both_errors (self);
      return FALSE;
    }

  // Return TRUE to indicate successful reading
  return TRUE;
}
```

</details><br>

- tis function is a callback for when a D-Bus monitor becomes readable. It's typically invoked when there's data to be read from the monitor.
- it first retrieves the BustlePcapMonitor instance from the user_data parameter, assuming it was passed correctly.
- it checks if pcap_error is not set, ensuring there are no previous errors.
- if the operation was cancelled, it sets the pcap_error and handles the cancellation by calling await_both_errors() and then returns FALSE. 
- depending on the current state of the BustlePcapMonitor, it selects a read function (start_pcap if the state is STATE_STARTING, read_one if the state is STATE_RUNNING or STATE_STOPPING).
- it calls the selected read function with the monitor instance and the address of pcap_error as arguments. If the read function returns FALSE, indicating an error, it handles the error by calling await_both_errors() and returns FALSE.

Finally, it returns TRUE if the reading was successful, indicating that there might be more data to read from the monitor.

## `cancellable_cancelled_cb`

<details>
<summary>Code</summary>

```c
static void
cancellable_cancelled_cb (GCancellable *cancellable,
                          gpointer      user_data)
{
  // Casts user_data back to a BustlePcapMonitor pointer
  BustlePcapMonitor *self = BUSTLE_PCAP_MONITOR (user_data);

  /* Closes the stream; should cause dbus-monitor to quit in due course when it
   * tries to write to the other end of the pipe.
   */
  // Closes the stream associated with the pcap reader
  bustle_pcap_reader_close (self->reader);

  /* And try to terminate it immediately. */
  // Sends a SIGINT signal to try to terminate dbus-monitor
  send_sigint (self);
}
```

</details><br>

> `cancellable_cancelled_cb`, which serves as a callback function. It takes two parameters: cancellable, which is a pointer to a GCancellable object, and user_data, which is a pointer to arbitrary user-supplied data. The function retrieves a pointer to a `BustlePcapMonitor` object from the user_data. It then closes the stream associated with the pcap reader of the `BustlePcapMonitor` object using `bustle_pcap_reader_close()`, which should cause `dbus-monitor` to quit eventually when it tries to write to the other end of the pipe. Finally, it tries to immediately terminate `dbus-monitor` by sending a SIGINT signal using the `send_sigint()` function.

## `build_argv`

<details>
<summary>Code</summary>

```c
static const char **
build_argv (BustlePcapMonitor *self,
            GError **error)
{
  // Create a new GPtrArray to store the command-line arguments for dbus-monitor
  g_autoptr(GPtrArray) dbus_monitor_argv = g_ptr_array_sized_new (8);

  // If running inside Flatpak, add "flatpak-spawn" and "--host" to the arguments
  if (RUNNING_IN_FLATPAK)
    {
      g_ptr_array_add (dbus_monitor_argv, "flatpak-spawn");
      g_ptr_array_add (dbus_monitor_argv, "--host");
    }

  // If the bus type is G_BUS_TYPE_SYSTEM, add "pkexec" to the arguments
  if (self->bus_type == G_BUS_TYPE_SYSTEM)
    g_ptr_array_add (dbus_monitor_argv, "pkexec");

  // Add "dbus-monitor" and "--pcap" to the arguments unconditionally
  g_ptr_array_add (dbus_monitor_argv, "dbus-monitor");
  g_ptr_array_add (dbus_monitor_argv, "--pcap");

  // Depending on the bus type, add corresponding options to the arguments
  switch (self->bus_type)
    {
      case G_BUS_TYPE_SESSION:
        // For session bus, ensure address is not provided and add "--session"
        g_return_val_if_fail (self->address == NULL, FALSE);
        g_ptr_array_add (dbus_monitor_argv, "--session");
        break;

      case G_BUS_TYPE_SYSTEM:
        // For system bus, ensure address is not provided and add "--system"
        g_return_val_if_fail (self->address == NULL, FALSE);
        g_ptr_array_add (dbus_monitor_argv, "--system");
        break;

      case G_BUS_TYPE_NONE:
        // For no specific bus type, ensure address is provided and add "--address" with the address
        g_return_val_if_fail (self->address != NULL, FALSE);
        g_ptr_array_add (dbus_monitor_argv, "--address");
        g_ptr_array_add (dbus_monitor_argv, self->address);
        break;

      default:
        // If an unsupported bus type is encountered, set an error and return NULL
        g_set_error (error, G_IO_ERROR, G_IO_ERROR_NOT_SUPPORTED,
            "Can only log the session bus, system bus, or a given address");
        return NULL;
    }

  // Add a NULL terminator to the argument array to mark its end
  g_ptr_array_add (dbus_monitor_argv, NULL);

  // Free the GPtrArray and return its data as a const char ** array
  return (const char **) g_ptr_array_free (g_steal_pointer (&dbus_monitor_argv), FALSE);
}

```

</details><br>

> This function essentially prepares the command-line arguments necessary to execute `dbus-monitor` with specific options based on the bus type and environment considerations like Flatpak.

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

</details><br>

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


## `Misc`

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