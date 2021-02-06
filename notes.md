# Notes

## User management

Users can "login" anonymously by selecting a user name. This user name will be associated with a uuid. This user data is stored inside the signed cookie and the uuid will be used to reference this user. An anonymous user will lose his session after the amount of time specified in the config.

Users can register themselves to reserve their username. They will also have the ability to see their past game records and e.g. be in a public ranking.

User password are saved as argon2 hashes.

## Application structure

The application has two shared states that are built in `server.rs` and access through shared, thread-save references (`std::sync::Arc`). The same applies for the shared postgresql connection pool.

The main worker threads (serving the data) have each an individual gameserver actor attached (due the gameserver actor being limited to synchronous execution). The gameserver actors know the addresses for the websocket actors for every game through their shared state as part of a concurrent hashmap (`dashmap::DashMap`), which binds each game id (i32) with a Hashset (`dashmap::DashSet`) consisting of the websocket actors addresses.

Websocket sessions are handled by individual actors that handle message sending and receiving for the websocket as well as heartbeat management.

## Graph model

> extensive move: move is recorded with all distinct moves over corners, normal move: move as target and source with distinct inbetween

The graph model is used to validate a move and export _complete_ (with the extended move structure for the file format) games.

The UI sends a move in the extensive format. To validate a game move all possible paths are searched for with a\*. The extensive move will be validated against those and if correct a distinctive field will be evaluated. At this point the server can save the move to the database, alter the saved graph state in the shared gameserver state and send notifications about the move to all players (you may prefer sending the notification before saving to allow for faster responses to the players).

Exporting is done similarly to validating but instead of just checking the moves will be extracted by finding all possible paths and choosing the correct one based on the inbetween field. After all moves were extracted the file will be built and saved to the database as BLOB or as file in tmp. The exported file may be downloaded by the user later on.

NOTE: Exporting games will probably be done with background jobs ([actix-redis-jobs](https://github.com/geofmureithi/actix-jobs) looks fitting) to not block server threads.

## Database

-   Games are only recorded to the database if the host is a registered user (might change this later on)
-   All game moves are recorded as a location, a distinctive `inbetween` location, a figure id and a reference to the player id

On shutdown by admin: All tmp games, with a registered user as host, are saved to the database and all connections are dropped gracefully
