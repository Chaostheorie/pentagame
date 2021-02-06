# WS

Websocket endpoint and Gameserver Actor.

The websocket route creates a websocket actor for every connected user. This Websocket Actor handles heartbeat and basic responses as well as responding to e.g. game metadata queries.

The GameServer actor handles any processing and mild caching for queries and can e.g. validates moves.
