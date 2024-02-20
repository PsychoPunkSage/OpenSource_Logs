# **D-Bus concepts**

## Bus

>> A D-Bus “bus” is a kind of server that handles several connections in a `bus-topology` fashion. As such, it relays messages between connected endpoints, and allows to discover endpoints or sending broadcast messages (signals).<br>
>>Typically, a Linux system has a **system** **bus**, and a **session** **bus**. The latter is per-user. It is also possible to have private buses or no bus at all (i-e direct peer-to-peer communication instead).

## Bus name / service name

>>An endpoint can have various names, which allows to address messages to it on the bus. All endpoints are assigned a unique name by the bus at start. Since this name is not static, most services use something called a well-known bus name and typically it’s this name, that you’ll be concerned with.

>An example would be the FreeDesktop Notifications Service that uses `org.freedesktop.Notifications` as its well-known bus name.

## Objects and Object paths

>> An object is akin to the concept of an object or an instance in many programming languages. All services expose at least one object on the bus and all clients interact with the service through these objects. These objects can be ephemeral or they could live as long as the service itself.

>> Every object is identified by a string, which is referred to as its path. An example of an object path is `/org/freedesktop/Notifications`, which identities the only object exposed by the FreeDesktop Notifications Service.

## Interface

>> An interface defines the API exposed by object on the bus. They are akin to the concept of interfaces in many programming languages and traits in Rust. Each object can (and typically do) provide multiple interfaces at the same time. A D-Bus interface can have **methods**, **properties** and **signals**.

>> While each interface of a service is identified by a **unique name**, its API is described by an XML description. It is mostly a machine-level detail. Most services can be queried for this description through a D-Bus standard **introspection interface**.

>> zbus provides convenient macro that implements the introspection interface for services, and helper to generate client-side Rust API, given an XML description.