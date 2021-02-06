// responses.ts - part of pentagame online under GPLv3.0 @ cobalt
// Collection of interfaces for responses provided by server

export enum RESPONSE_CODES {
  FETCH_LATEST_MOVE = 0,
  GET_GAME_METADATA = 1,
  // unimplemented MAKE_MOVE: 2,
  // unimplemented PLAYER_JOINED: 3,
  LEAVE_GAME = 4,
  START_GAME = 5,
  // Unimplemented STOP_GAME: 6,
  PLAYER_JOINED = 7,
}

export class Response {
  action: number;
  data: any;

  static from_string(source: string): Response {
    try {
      return <Response>JSON.parse(source);
    } catch (e) {
      console.log(`[WSResponse]: Failed to parse response ${e.message}`);
      throw e;
    }
  }
}

export class NewPlayer extends Response {
  action = RESPONSE_CODES.PLAYER_JOINED;
  data: {
    player: string;
  };
}

export class Metadata extends Response {
  action = RESPONSE_CODES.GET_GAME_METADATA;
  data: {
    name: string;
    description: string;
    icon: string;
    players: string[][];
    state: number;
    host: string;
    pin: string;
  };
}

export class GameStarted extends Response {
  action = RESPONSE_CODES.START_GAME;
  data = {};
}
