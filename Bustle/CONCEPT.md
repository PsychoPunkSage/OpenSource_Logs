# **D-Bus concepts**

## Bus

>> A D-Bus “bus” is a kind of server that handles several connections in a `bus-topology` fashion. As such, it relays messages between connected endpoints, and allows to discover endpoints or sending broadcast messages (signals).<br>
>>Typically, a Linux system has a **system** **bus**, and a **session** **bus**. The latter is per-user. It is also possible to have private buses or no bus at all (i-e direct peer-to-peer communication instead).