// responses.ts - part of pentagame online under GPLv3.0 @ cobalt
// Collection of interfaces for requests intended as request to main server

export enum REQUEST_CODES {
  FETCH_LATEST_MOVE = 0,
  GET_GAME_METADATA = 1,
  MAKE_MOVE = 2,
  PLACE_STOPPER = 3,
  LEAVE_GAME = 4,
  START_GAME = 5,
  STOP_GAME = 6,
}

export abstract class Request {
  // attributes
  action: number;
  data: any;

  as_string(): string {
    return JSON.stringify({ action: this.action, data: this.data });
  }
}

export class WhoamI extends Request {
  // override data
  data: {};
}

export class MetadataQuery extends Request {
  action = REQUEST_CODES.GET_GAME_METADATA;
  data = {};
}

export class StartGameAction extends Request {
  action = REQUEST_CODES.START_GAME;
  data = {};
}
