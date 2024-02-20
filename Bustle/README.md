# **D-Bus** 


## **Client Proxy**
>>> **How to make low-level D-Bus method calls.**

### Shell interaction using `[busctl]`

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

### Low-level call from a `zbus::Connection`

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


### Trait-derived proxy call

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

### Signals

>> `Signals` are like methods, except they don’t expect a reply. They are typically emitted by services to notify interested peers of any changes to the state of the service. zbus provides a `Stream`-based API for receiving signals.

```rust
use async_std::stream::StreamExt;
use zbus::Connection;
use zbus_macros::proxy;
use zvariant::OwnedObjectPath;


#[proxy(
    default_service = "org.freedesktop.systemd1",
    default_path = "/org/freedesktop/systemd1",
    interface = "org.freedesktop.systemd1.Manager"
)]
trait Systemd1Manager {
    // Defines signature for D-Bus signal named `JobNew`
    #[zbus(signal)]
    fn job_new(&self, id: u32, job: OwnedObjectPath, unit: String) -> zbus::Result<()>;
}

async fn watch_systemd_jobs() -> zbus::Result<()> {
    let connection = Connection::system().await?;
    // `Systemd1ManagerProxy` is generated from `Systemd1Manager` trait
    let systemd_proxy = Systemd1ManagerProxy::new(&connection).await?;
    // Method `receive_job_new` is generated from `job_new` signal
    let mut new_jobs_stream = systemd_proxy.receive_job_new().await?;

    while let Some(msg) = new_jobs_stream.next().await {
        // struct `JobNewArgs` is generated from `job_new` signal function arguments
        let args: JobNewArgs = msg.args().expect("Error parsing message");

        println!(
            "JobNew received: unit={} id={} path={}",
            args.unit, args.id, args.job
        );
    }

    panic!("Stream ended unexpectedly");
} 
```

### Properties

>> Interfaces can have associated properties, which can be read or set with the `org.freedesktop.DBus.Properties` interface. Here again, the `#[proxy]` attribute comes to the rescue. One can annotate a trait method to be a getter:

```rust
use zbus::{proxy, Result};

#[proxy]
trait MyInterface {
    #[zbus(property)]
    fn state(&self) -> Result<String>;
}
```

> The `state()` method will translate to a "`State`" property `Get` call.<br>To set the property, prefix the name of the property with `set_`.

* reading two properties from systemd’s main service..
```rust
use zbus::{Connection, proxy, Result};

#[proxy(
    interface = "org.freedesktop.systemd1.Manager",
    default_service = "org.freedesktop.systemd1",
    default_path = "/org/freedesktop/systemd1"
)]
trait SystemdManager {
    #[zbus(property)]
    fn architecture(&self) -> Result<String>;
    #[zbus(property)]
    fn environment(&self) -> Result<Vec<String>>;
}

#[async_std::main]
async fn main() -> Result<()> {
    let connection = Connection::system().await?;

    let proxy = SystemdManagerProxy::new(&connection).await?;
    println!("Host architecture: {}", proxy.architecture().await?);
    println!("Environment:");
    for env in proxy.environment().await? {
        println!("  {}", env);
    }

    Ok(())
}
```

<details>
<summary>Output</summary>

```
Host architecture: x86-64
Environment variables:
  HOME=/home/zeenix
  LANG=en_US.UTF-8
  LC_ADDRESS=de_DE.UTF-8
  LC_IDENTIFICATION=de_DE.UTF-8
  LC_MEASUREMENT=de_DE.UTF-8
  LC_MONETARY=de_DE.UTF-8
  LC_NAME=de_DE.UTF-8
  LC_NUMERIC=de_DE.UTF-8
  LC_PAPER=de_DE.UTF-8
  LC_TELEPHONE=de_DE.UTF-8
  LC_TIME=de_DE.UTF-8
  ...
```

</details>

### Generating the trait from an XML interface

>> The `zbus_xmlgen` crate provides a **developer-friendly tool**, that can generate Rust traits from a given D-Bus introspection XML.

The tool can be used to generate rust code directly from a D-Bus service running on system:
```rust
zbus-xmlgen session \
  org.freedesktop.Notifications \
  /org/freedesktop/Notifications
```

Alternatively one can also get the XML interface from a different source and use it to generate the interface code.<br>
We can fetch the XML interface of the notification service, using the `--xml-interface` option of the `busctl` command.

```bash
busctl --user --xml-interface introspect \
  org.freedesktop.Notifications \
  /org/freedesktop/Notifications
```

<details>
<summary>Output</summary>

```
<!DOCTYPE node PUBLIC "-//freedesktop//DTD D-BUS Object Introspection 1.0//EN"
                      "http://www.freedesktop.org/standards/dbus/1.0/introspect>
<!-- GDBus 2.78.0 -->
<node>
  <interface name="org.freedesktop.DBus.Properties">
    <method name="Get">
      <arg type="s" name="interface_name" direction="in"/>
      <arg type="s" name="property_name" direction="in"/>
      <arg type="v" name="value" direction="out"/>
    </method>
    <method name="GetAll">
      <arg type="s" name="interface_name" direction="in"/>
      <arg type="a{sv}" name="properties" direction="out"/>
    </method>
    <method name="Set">
      <arg type="s" name="interface_name" direction="in"/>
      <arg type="s" name="property_name" direction="in"/>
      <arg type="v" name="value" direction="in"/>
    </method>
    <signal name="PropertiesChanged">
      <arg type="s" name="interface_name"/>
      <arg type="a{sv}" name="changed_properties"/>
      <arg type="as" name="invalidated_properties"/>
    </signal>
  </interface>
  <interface name="org.freedesktop.DBus.Introspectable">
    <method name="Introspect">
      <arg type="s" name="xml_data" direction="out"/>
    </method>
  </interface>
  <interface name="org.freedesktop.DBus.Peer">
    <method name="Ping"/>
    <method name="GetMachineId">
      <arg type="s" name="machine_uuid" direction="out"/>
    </method>
  </interface>
  <interface name="org.freedesktop.Notifications">
    <method name="Notify">
      <arg type="s" name="arg_0" direction="in">
      </arg>
      <arg type="u" name="arg_1" direction="in">
      </arg>
      <arg type="s" name="arg_2" direction="in">
      </arg>
      <arg type="s" name="arg_3" direction="in">
      </arg>
      <arg type="s" name="arg_4" direction="in">
      </arg>
      <arg type="as" name="arg_5" direction="in">
      </arg>
      <arg type="a{sv}" name="arg_6" direction="in">
      </arg>
      <arg type="i" name="arg_7" direction="in">
      </arg>
      <arg type="u" name="arg_8" direction="out">
      </arg>
    </method>
    <method name="CloseNotification">
      <arg type="u" name="arg_0" direction="in">
      </arg>
    </method>
    <method name="GetCapabilities">
      <arg type="as" name="arg_0" direction="out">
      </arg>
    </method>
    <method name="GetServerInformation">
      <arg type="s" name="arg_0" direction="out">
<!DOCTYPE node PUBLIC "-//freedesktop//DTD D-BUS Object Introspection 1.0//EN"
                      "http://www.freedesktop.org/standards/dbus/1.0/introspect>
<!-- GDBus 2.78.0 -->
<node>
  <interface name="org.freedesktop.DBus.Properties">
    <method name="Get">
      <arg type="s" name="interface_name" direction="in"/>
      <arg type="s" name="property_name" direction="in"/>
      <arg type="v" name="value" direction="out"/>
    </method>
    <method name="GetAll">
      <arg type="s" name="interface_name" direction="in"/>
      <arg type="a{sv}" name="properties" direction="out"/>
    </method>
    <method name="Set">
      <arg type="s" name="interface_name" direction="in"/>
      <arg type="s" name="property_name" direction="in"/>
      <arg type="v" name="value" direction="in"/>
    </method>
    <signal name="PropertiesChanged">
      <arg type="s" name="interface_name"/>
      <arg type="a{sv}" name="changed_properties"/>
      <arg type="as" name="invalidated_properties"/>
```

</details>

## **Writing a server interface**

>>> Going to implement a server with a method “SayHello”, to greet back the calling client.<br>
>>>* First discuss the need to associate a service name with the server. 
>>>* Going to manually handle incoming messages using the low-level API. 
>>>* Finally, present the `ObjectServer` higher-level API and some of its more advanced concepts.

### Taking a service name

>> As we know, each connection on the bus is given a **unique name** (such as “:1.27”). This could be all we need, depending on our use case, and the design of our D-Bus API. However, typically services use a **service name** (aka well-known name) so peers (clients, in this context) can easily discover them.

Requesting the bus for the service name of our choice:

```rust
use zbus::{Connection, Result};

// Although we use `async-std` here, you can use any async runtime of choice.
#[async_std::main]
async fn main() -> Result<()> {
    let connection = Connection::session()
        .await?;
    connection
        .request_name("org.zbus.MyGreeter")
        .await?;

    loop {}
}
```

We can check our service is running and is associated with the service name:
```bash
busctl --user list | grep SERVICE_NAME
```

### Handling low-level messages

>> At the low-level, one can handle method calls by checking the incoming messages manually.

`SayHello method`, takes a string as argument, and reply with a “hello” greeting by replacing the loop above with this code:
```rust
use futures_util::stream::TryStreamExt;

// Although we use `async-std` here, you can use any async runtime of choice.
#[async_std::main]
async fn main() -> zbus::Result<()> {
    let connection = zbus::Connection::session().await?;
    let mut stream = zbus::MessageStream::from(&connection);
       connection
           .request_name("org.zbus.MyGreeter")
           .await?;

    while let Some(msg) = stream.try_next().await? {
        let msg_header = msg.header();
        dbg!(&msg);

        match msg_header.message_type() {
            zbus::message::Type::MethodCall => {
                // real code would check msg_header path(), interface() and member()
                // handle invalid calls, introspection, errors etc
                let body = msg.body();
                let arg: &str = body.deserialize()?;
                connection.reply(&msg, &(format!("Hello {}!", arg))).await?;

                break;
            }
            _ => continue,
        }
    }
    Ok(())
}
```

Checker:
```bash
busctl --user call org.zbus.MyGreeter /org/zbus/MyGreeter org.zbus.MyGreeter1 SayHello s "zbus"
```

<details>
<summary>Output</summary>

```
"Hello zbus!"
```

</details>

### Using the ObjectServer

>> One can write an impl block with a set of methods and let the interface procedural macro write the D-Bus message handling details. It will dispatch the incoming method calls to their respective handlers, as well as replying to introspection requests.

> `MyGreeter` interface:

```rust
use zbus::{Connection, interface, Result};

struct Greeter;

#[interface(name = "org.zbus.MyGreeter1")]
impl Greeter {
    async fn say_hello(&self, name: &str) -> String {
        format!("Hello {}!", name)
    }
}

// Although we use `async-std` here, you can use any async runtime of choice.
#[async_std::main]
async fn main() -> Result<()> {
    let connection = Connection::session().await?;
    // setup the server
    connection
        .object_server()
        .at("/org/zbus/MyGreeter", Greeter)
        .await?;
    // before requesting the name
    connection
        .request_name("org.zbus.MyGreeter")
        .await?;

    loop {
        // do something else, wait forever or timeout here:
        // handling D-Bus messages is done in the background
        std::future::pending::<()>().await;
    }
}
```

> Service activation `pitfalls`

A possible issue here is that one must request the **service name** after one setup the **handlers**, otherwise incoming messages may be lost. Activated services may receive calls (or messages) right after taking their name. This is why it’s typically better to make use of `connection::Builder` for **setting up interfaces** and **requesting names**

```rust
// use zbus::{connection, interface, Result};

// struct Greeter;

// #[interface(name = "org.zbus.MyGreeter1")]
// impl Greeter {
//     async fn say_hello(&self, name: &str) -> String {
//         format!("Hello {}!", name)
//     }
// }

// #[async_std::main]
// async fn main() -> Result<()> {
    let _connection = connection::Builder::session()?
        .name("org.zbus.MyGreeter")?
        .serve_at("/org/zbus/MyGreeter", Greeter)?
        .build()
        .await?;
//     loop {
//         // do something else, wait forever or timeout here:
//         // handling D-Bus messages is done in the background
//         std::future::pending::<()>().await;
//     }
// }
```
* It should work with the same `busctl` command used previously.

<details>
<summary>Introspect Service</summary>

```
$ busctl --user introspect org.zbus.MyGreeter /org/zbus/MyGreeter
NAME                                TYPE      SIGNATURE RESULT/VALUE FLAGS
bla-bla-bla-bla-bla-bla-bla         bla       -         -             -
................................
```

</details>

> Complete Example

* Shows a way to synchronize with the interface handlers from outside, thanks to the `event_listener` crate.
  
```rust
use zbus::{object_server::SignalContext, connection::Builder, interface, fdo, Result};

use event_listener::{Event, Listener};

struct Greeter {
    name: String,
    done: Event,
}

#[interface(name = "org.zbus.MyGreeter1")]
impl Greeter {
    async fn say_hello(&self, name: &str) -> String {
        format!("Hello {}!", name)
    }

    // Rude!
    async fn go_away(
        &self,
        #[zbus(signal_context)]
        ctxt: SignalContext<'_>,
    ) -> fdo::Result<()> {
        Self::greeted_everyone(&ctxt).await?;
        self.done.notify(1);

        Ok(())
    }

    /// A "GreeterName" property.
    #[zbus(property)]
    async fn greeter_name(&self) -> &str {
        &self.name
    }

    /// A setter for the "GreeterName" property.
    ///
    /// Additionally, a `greeter_name_changed` method has been generated for you if you need to
    /// notify listeners that "GreeterName" was updated. It will be automatically called when
    /// using this setter.
    #[zbus(property)]
    async fn set_greeter_name(&mut self, name: String) {
        self.name = name;
    }

    /// A signal; the implementation is provided by the macro.
    #[zbus(signal)]
    async fn greeted_everyone(ctxt: &SignalContext<'_>) -> Result<()>;
}

// Although we use `async-std` here, you can use any async runtime of choice.
#[async_std::main]
async fn main() -> Result<()> {
    let greeter = Greeter {
        name: "GreeterName".to_string(),
        done: event_listener::Event::new(),
    };
    let done_listener = greeter.done.listen();
    let _connection = Builder::session()?
        .name("org.zbus.MyGreeter")?
        .serve_at("/org/zbus/MyGreeter", greeter)?
        .build()
        .await?;

    done_listener.wait();

    Ok(())
}
```

<details>
<summary>Introspect Service</summary>

```
$ busctl --user introspect org.zbus.MyGreeter /org/zbus/MyGreeter
NAME                                TYPE      SIGNATURE RESULT/VALUE FLAGS
[...]
org.zbus.MyGreeter1                 interface -         -             -
....................
```

</details>


## **Blocking API**

>>> While zbus API being primarily asynchronous (since 2.0), it could easily feel daunting for simple use cases. For “ease” zbus provides blocking wrapper types, under the **blocking module**.

<details>
<summary>Note</summary>

```
Use of the blocking API in an async context will likely result in panics and hangs. This is not a limitation of zbus but rather a well-known general problem in the Rust async/await. The blocking crate, async-std and tokio crates provide a easy way around this problem.
```

</details>

### Establishing a connection

>> The only difference to that of asynchronous Connection API is that you use blocking::Connection type instead. This type’s API is almost identical to that of Connection, except all its methods are blocking.

### Client

>> Similar to `blocking::Connection`, use `blocking::Proxy` type. Its constructors require `blocking::Connection` instead of `Connection`. Moreover, `proxy` macro generates a `blocking::Proxy wrapper` for you as well.

Blocking connection and proxy usage:
```rust
#![allow(unused)]
fn main() {
use zbus::{blocking::Connection, zvariant::ObjectPath, proxy, Result};

#[proxy(
    default_service = "org.freedesktop.GeoClue2",
    interface = "org.freedesktop.GeoClue2.Manager",
    default_path = "/org/freedesktop/GeoClue2/Manager"
)]
trait Manager {
    #[zbus(object = "Client")]
    /// The method normally returns an `ObjectPath`.
    /// With the object attribute, we can make it return a `ClientProxy` directly.
    fn get_client(&self);
}

#[proxy(
    default_service = "org.freedesktop.GeoClue2",
    interface = "org.freedesktop.GeoClue2.Client"
)]
trait Client {
    fn start(&self) -> Result<()>;
    fn stop(&self) -> Result<()>;

    #[zbus(property)]
    fn set_desktop_id(&mut self, id: &str) -> Result<()>;

    #[zbus(signal)]
    fn location_updated(&self, old: ObjectPath<'_>, new: ObjectPath<'_>) -> Result<()>;
}

#[proxy(
    default_service = "org.freedesktop.GeoClue2",
    interface = "org.freedesktop.GeoClue2.Location"
)]
trait Location {
    #[zbus(property)]
    fn latitude(&self) -> Result<f64>;
    #[zbus(property)]
    fn longitude(&self) -> Result<f64>;
}
let conn = Connection::system().unwrap();
let manager = ManagerProxyBlocking::new(&conn).unwrap();
let mut client = manager.get_client().unwrap();
// Gotta do this, sorry!
client.set_desktop_id("org.freedesktop.zbus").unwrap();

let mut location_updated = client.receive_location_updated().unwrap();

client.start().unwrap();

// Wait for the signal.
let signal = location_updated.next().unwrap();
let args = signal.args().unwrap();

let location = LocationProxyBlocking::builder(&conn)
    .path(args.new())
    .unwrap()
    .build()
    .unwrap();
println!(
    "Latitude: {}\nLongitude: {}",
    location.latitude().unwrap(),
    location.longitude().unwrap(),
);
}
```

### Server

>> Similarly here, use `blocking::ObjectServer` that is associated with every `blocking::Connection` instance. While there is no blocking version of Interface, interface allows to write non-async methods.

<details>
<summary>Note</summary>

```
Even though you can write non-async methods, these methods are still called from an async context. Therefore, you can not use blocking API in the method implementation directly. See note at the beginning of this chapter for details on why and a possible workaround.
```

</details>

```rust
use std::error::Error;
use zbus::{blocking::connection, interface, fdo, SignalContext};

use event_listener::{Event, Listener};

struct Greeter {
    name: String,
    done: Event,
}

#[interface(name = "org.zbus.MyGreeter1")]
impl Greeter {
    fn say_hello(&self, name: &str) -> String {
        format!("Hello {}!", name)
    }

    // Rude!
    async fn go_away(
        &self,
        #[zbus(signal_context)]
        ctxt: SignalContext<'_>,
    ) -> fdo::Result<()> {
        Self::greeted_everyone(&ctxt).await?;
        self.done.notify(1);

        Ok(())
    }

    /// A "GreeterName" property.
    #[zbus(property)]
    fn greeter_name(&self) -> &str {
        &self.name
    }

    /// A setter for the "GreeterName" property.
    ///
    /// Additionally, a `greeter_name_changed` method has been generated for you if you need to
    /// notify listeners that "GreeterName" was updated. It will be automatically called when
    /// using this setter.
    #[zbus(property)]
    fn set_greeter_name(&mut self, name: String) {
        self.name = name;
    }

    /// A signal; the implementation is provided by the macro.
    #[zbus(signal)]
    async fn greeted_everyone(ctxt: &SignalContext<'_>) -> zbus::Result<()>;
}

fn main() -> Result<(), Box<dyn Error>> {
    let greeter = Greeter {
        name: "GreeterName".to_string(),
        done: event_listener::Event::new(),
    };
    let done_listener = greeter.done.listen();
    let _handle = connection::Builder::session()?
        .name("org.zbus.MyGreeter")?
        .serve_at("/org/zbus/MyGreeter", greeter)?
        .build()?;

    done_listener.wait();

    Ok(())
}
```