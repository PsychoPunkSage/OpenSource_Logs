# D-Bus 

## Shell interaction using `[busctl]`

>  service from the shell, and notify the desktop with `[busctl]`

```bash
busctl --user call \
  org.freedesktop.Notifications \
  /org/freedesktop/Notifications \
  org.freedesktop.Notifications \
  Notify \
  susssasa\{sv\}i \
  "my-app" 0 "dialog-information" "A summary" "Some body" 0 0 5000
```

Running this command should pop-up a notification dialog on your desktop. If it does not, your desktop does not support the notification service, and this example will be less interactive.

This command shows us several aspects of the D-Bus communication:

* **--user**     : Connect to and use the user/session bus.
* **call**       : Send a method call message. (D-Bus also supports signals, error messages, and method replies)
* **destination**: The name of the service (org.freedesktop.Notifications).
* **object path**: Object/interface path (/org/freedesktop/Notifications).
* **interface**  : The interface name (methods are organized in interfaces, here org.freedesktop.Notifications, same name as the service).
* **method**     : The name of the method to call, Notify.
* **signature**  : That susssasa{sv}i means the method takes 8 arguments of various types. ‘s’, for example, is for a string. ‘as’ is for array of strings.
* The method arguments.

## Low-level call from a `zbus::Connection`

> zbus `Connection` has a `call_method()` method, which you can use directly.

```rust
use std::collections::HashMap;
use std::error::Error;

use zbus::{zvariant::Value, Connection};

// Although we use `async-std` here, you can use any async runtime of choice.
#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let connection = Connection::session().await?;

    let m = connection.call_method(
        Some("org.freedesktop.Notifications"),
        "/org/freedesktop/Notifications",
        Some("org.freedesktop.Notifications"),
        "Notify",
        &("my-app", 0u32, "dialog-information", "A summary", "Some body",
          vec![""; 0], HashMap::<&str, &Value>::new(), 5000),
    ).await?;
    let reply: u32 = m.body().deserialize().unwrap();
    dbg!(reply);
    Ok(())
}
```


Although this is already quite flexible, and handles various details for you (such as the message signature), it is also somewhat inconvenient and error-prone: one can easily miss arguments, or give arguments with the wrong type or other kind of errors
