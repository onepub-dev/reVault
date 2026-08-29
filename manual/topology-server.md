# Topology server

The topology server is used to describe the set of key sharing servers and their fall over configuration.

The topology server is just a set of end points within a regular Key Sharing Server that has been enabled to act as a topology server as well as a key server.

We are able to (optionally) run a master-slave pair of topology servers.  The topology servers are responsible for telling each key server who its secondary is.&#x20;

The key servers are arranged in a ring, with each link in the ring acting as a primary as well as a secondary to the prior server in the ring.  A ring of one is possible (in which case no replication occurs).

The topology server can insert/remove key servers from the ring as they come and go.

The topology server expects a heart beat from each key server every five minute, if the heartbeat does not arrive, the topology server will rearrange the ring to remove the missing server.  If the server recovers the ring will once again be reorganised.

Each key server has a security token used to auth its connection to the topology server. When a key server registered with token server, the token server provides the key server with the security token of its secondary.  The token server must have a list of all potential key servers and their security keys. When a key server registers with the topology server the topology server provides the key with a randomly generated session key which the key server can use to validate communications with the topology server.&#x20;

If the primary topology server goes down, key servers will failover to the secondary topology server and go through the registration process again.  This means that there is no requirement for synchronisation between the topology servers.&#x20;



