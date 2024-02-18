# D-Bus 

## Shell interaction using `[busctl]`

>>  service from the shell, and notify the desktop with `[busctl]`

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

>> zbus `Connection` has a `call_method()` method, which you can use directly.

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


Although this is already quite flexible, and handles various details for you (such as the message signature), it is also somewhat inconvenient and error-prone: one can easily miss arguments, or give arguments with the wrong type or other kind of errors.


## Trait-derived proxy call

>> A trait declaration `T` with a `proxy` attribute will have a derived `TProxy` and `TProxyBlocking` implemented thanks to procedural macros. The trait methods will have respective `impl` methods wrapping the D-Bus calls

```rust
use std::collections::HashMap;
use std::error::Error;

use zbus::{zvariant::Value, proxy, Connection};

#[proxy(
    default_service = "org.freedesktop.Notifications",
    default_path = "/org/freedesktop/Notifications"
)]
trait Notifications {
    /// Call the org.freedesktop.Notifications.Notify D-Bus method
    fn notify(&self,
              app_name: &str,
              replaces_id: u32,
              app_icon: &str,
              summary: &str,
              body: &str,
              actions: &[&str],
              hints: HashMap<&str, &Value<'_>>,
              expire_timeout: i32) -> zbus::Result<u32>;
}

// Although we use `async-std` here, you can use any async runtime of choice.
#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let connection = Connection::session().await?;

    let proxy = NotificationsProxy::new(&connection).await?;
    let reply = proxy.notify(
        "my-app",
        0,
        "dialog-information",
        "A summary", "Some body",
        &[],
        HashMap::new(),
        5000,
    ).await?;
    dbg!(reply);

    Ok(())
}
```

> When you define a trait with the `#[proxy]` attribute, the `zbus` crate automatically generates a proxy struct that implements that trait. This proxy struct is named by appending `Proxy` to the trait name. So in this case, since your trait is named `Notifications`, the generated proxy struct will be named `NotificationsProxy`.

A `TProxy` and `TProxyBlocking` has a few associated methods, such as `new`(connection), using the **default associated service name** and **object path**, and an **associated builder** if one need to specify something different.

This should help to avoid mistakes (saw earlier). It’s also a bit easier to use. This makes it also possible to have higher-level types, they fit more naturally with the rest of the code. One can further document the D-Bus API or provide additional helpers.