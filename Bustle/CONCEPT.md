# **D-Bus concepts**

## Bus

>> A D-Bus “bus” is a kind of server that handles several connections in a `bus-topology` fashion. As such, it relays messages between connected endpoints, and allows to discover endpoints or sending broadcast messages (signals).<br>
>>Typically, a Linux system has a **system** **bus**, and a **session** **bus**. The latter is per-user. It is also possible to have private buses or no bus at all (i-e direct peer-to-peer communication instead).

## Bus name / service name

>>An endpoint can have various names, which allows to address messages to it on the bus. All endpoints are assigned a unique name by the bus at start. Since this name is not static, most services use something called a well-known bus name and typically it’s this name, that you’ll be concerned with.

>An example would be the FreeDesktop Notifications Service that uses `org.freedesktop.Notifications` as its well-known bus name.