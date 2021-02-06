import {
    /* webpackMode: "eager" */
    create_alert,
    create_modal,
    download,
    getJSONP,
    init_ui,
    build_element,
    on_load,
} from './utils';
import { Board } from './pentamath/models';
import { Request, MetadataQuery, StartGameAction } from './interfaces/requests';
import {
    RESPONSE_CODES,
    Metadata,
    NewPlayer,
    Response,
    GameStarted,
} from './interfaces/responses';
import { sanitize } from 'dompurify';
import { LRUBuffer, Websocket, WebsocketBuilder } from 'websocket-ts';
import { Tooltip } from 'bootstrap';
import { create, Svg, SVG } from '@svgdotjs/svg.js';

// constants
const host = 'localhost:8443';
const route = '/games/ws/';
const url =
    location.protocol === 'https:'
        ? `wss://${host}${route}`
        : `ws://${host}${route}`;

// test

/*
action & data
---
| action | description         | data                | host only |
| ------ | ------------------- | ----------------    | --------- |
| 0      | fetch latest move   | {"all": boolean}    |           |
| 1      | get game meta       | {}                  |     X     |
| 2      | make move           | {"move": [MOVE]}    |     X     |
| 3      | Place Stopper       | {"move": String}    |     X     |
| 4      | leave game          | {}                  |     X     |
| 5      | start game          | {"message": String} |     ✓     |
| 6      | stop game           | {"message": String} |     ✓     |

NOTE: In general all request codes respond with the same response code. 
      This distinction in constants is done to mark unimplemented responses
*/

export class Game {
    ui: {
        description: string;
        name: string;
        icon: string;
        pin: string;
        host: string;
        shift: boolean;
        players: string[][];
    };
    socket: Websocket;
    user: String[];
    state: number;
    board: Board;

    // null == unknown
    constructor() {
        // internal content
        this.state = null;
        this.ui = {
            description: null,
            name: null,
            icon: null,
            shift: false,
            pin: null,
            host: null,
            players: null,
        };
        this.user = null;

        // board
        this.board = null;

        // websocket to server
        console.log(`[WS]: Connecting to ${url} 🐈`);
        try {
            this.socket = new WebsocketBuilder(url)
                .withBuffer(new LRUBuffer(10))
                // Not really needed though it would be neat to catch all outgoing messages when losing connection
                .onClose(this.socket_close())
                .onError(this.socket_error)
                .onMessage(this.socket_message()) // Those methods return functions to circumvent the scope of a bound callback
                .build();

            console.log(`[WS]: Connected successfully 🐱`);
        } catch (e) {
            this.socket = null;
            console.error(e);
            create_alert(0, 'Websocket Error', '[WS]: Creation failed 🙀');
        }
    }

    init() {
        // fetch metadata
        if (this.socket === null) {
            console.log(
                '[WS]: Init failed since no connected socket was defined 🙀'
            );
        } else {
            console.log('[WS]: Sending initial query 🐾');

            this.send_message(new MetadataQuery());
        }

        // create board
        let container = document.getElementById('board'),
            viewbox = Number(container.getAttribute('viewBox').split(' ')[3]);

        if (container === undefined || container === null || viewbox == NaN) {
            create_alert(0, 'UI', 'Failed to initialize game board');
            console.log('[GAME]: Failed to initialize game board');
        } else {
            this.board = new Board(viewbox, SVG().addTo(container), true, true);
            console.log('[GAME]: Initialized game board');
        }
    }

    redraw_board(shift?: boolean) {
        this.ui.shift = shift === undefined ? false : shift;
        this.board.shift = this.ui.shift;
        this.board.clear();
        this.board.draw();
    }

    set_user(user: string[]) {
        this.user = user;
    }

    // Websocket binds
    socket_error(): Function {
        return (_: Websocket, event: CloseEvent) => {
            console.error(event);
            create_alert(
                0,
                sanitize(`[WS Error]: ${event.code} ${event.reason} 🙀`),
                'Websocket Error'
            );
        };
    }

    socket_message(): (socket: Websocket, event: MessageEvent) => void {
        return (_: Websocket, event: MessageEvent) => {
            let data = Response.from_string(event.data);
            switch (data.action) {
                case RESPONSE_CODES.GET_GAME_METADATA:
                    this.process_metadata(data);
                    break;
                case RESPONSE_CODES.PLAYER_JOINED:
                    this.process_new_player(data);
                    break;
                default:
                    create_alert(
                        0,
                        sanitize(`[WS]: Unexpected Response <br> ${data}`),
                        'Websocket Error'
                    );
                    break;
            }
        };
    }

    socket_close(): (socket: Websocket, event: CloseEvent) => void {
        return (_: Websocket, event: CloseEvent) => {
            create_modal(
                'Server closed connection. Game was possibly ended.',
                'Websocket Error',
                (_) => {
                    window.location.href = '/';
                },
                1,
                'Redirect to landing page'
            );
        };
    }

    send_message_callback(socket: Websocket, data: Request, counter: number) {
        // timeout counter
        if (counter > 20) {
            create_modal(
                'Unable to establish connection to server.',
                'Websocket Error',
                (_) => {
                    window.location.href = '/';
                },
                1,
                'Redirect to landing page'
            );
        } else {
            counter++;
        }

        // check socket and delay if not ready
        if (socket.underlyingWebsocket.readyState === 1) {
            socket.send(JSON.stringify(data));
        } else {
            console.log('[WS]: Socket not ready. Delaying message 💤');
            setTimeout(this.send_message_callback, 100, socket, data, counter);
        }
    }

    send_message(data: Request) {
        // check socket and delay if not ready
        if (this.socket.underlyingWebsocket.readyState === 1) {
            this.socket.send(JSON.stringify(data));
        } else {
            console.log('[WS]: Socket not ready. Delaying message 💤');
            setTimeout(this.send_message_callback, 100, this.socket, data, 0);
        }
    }

    // data processing
    process_game_start(req: GameStarted) {
        console.log('[WS]: Game Started');

        // update state
        this.state = 1;
    }

    process_metadata(req: Metadata) {
        console.log('[WS]: Processed new metadata');

        // name, description, state, players, host, icon, pin
        this.state = req.data.state;
        this.ui.description = req.data.description;
        this.ui.icon = req.data.icon;
        this.ui.players = req.data.players;
        this.ui.pin = req.data.pin;
        this.ui.host = req.data.host;
        if (sanitize(req.data.name) == '') {
            this.ui.name = 'Unsafe Name';
        } else {
            this.ui.name = req.data.name;
        }

        // trigger callbacks to update UI
        this.update_name();
        this.update_description();
        this.update_pin();
    }

    process_new_player(rep: NewPlayer) {
        // [uid, username]
        let player = rep.data.player.split('|');

        // check if already initialized
        if (this.ui.players === null) {
            this.ui.players = [player];
        } else {
            // check if duplicate
            for (let i = this.ui.players.length - 1; i > -1; i--) {
                if (this.ui.players[i] == player) {
                    return;
                }
            }

            this.ui.players.push(player);
        }

        // update ui
        this.update_players();
    }

    // ui updates
    update_name() {
        let name = document.getElementById('game-name');
        name.innerHTML = `<i class="bi ${sanitize(
            this.ui.icon
        )}"></i> ${sanitize(this.ui.name)}`;
    }

    update_message() {
        // update message
        let message = document.getElementById('game-state');

        // evaluate and update
        if (this.state == 0) {
            message.innerHTML = 'Game is not running';
        } else if (this.state < 6) {
            message.innerHTML = sanitize(
                `Waiting for move by ${this.ui.players[this.state - 1][1]}`
            );
        } else if (this.state < 11) {
            message.innerHTML = sanitize(
                `Waiting for black stopper by of ${
                    this.ui.players[this.state - 1][1]
                }`
            );
        } else if (this.state < 16) {
            message.innerHTML = sanitize(
                `Waiting for gray stopper by of ${
                    this.ui.players[this.state - 1][1]
                }`
            );
        } else {
            let winners = '',
                winner_amount = this.state - 10;
            for (let i = 0; i < winner_amount; i++) {
                if (i + 2 == winner_amount) {
                    winners.concat(`${this.ui.players[i]} and `);
                } else {
                    winners.concat(`${this.ui.players[i]}, `);
                }
            }
            message.innerHTML = sanitize(
                `Game won by ${winners} Congratulations!`
            );
        }
    }

    update_description() {
        // set description
        let description = document.getElementById('game-description');
        description.innerHTML = sanitize(this.ui.description);
    }

    update_pin() {
        // set pin
        let pin = document.getElementById('game-pin');
        if (this.ui.pin !== undefined && this.ui.pin.length !== 0) {
            pin.innerHTML = sanitize(this.ui.pin);
        } else {
            let parent = document.getElementById('game-pin-parent');
            if (parent !== null && parent !== undefined) {
                parent.remove();
            }
            if (pin !== null && pin !== undefined) {
                pin.remove();
            }
        }
    }

    update_players() {
        // fetch player container
        let container = document.getElementById('game-players');
        container.innerHTML = '';

        for (let i = this.ui.players.length - 1; i > -1; i--) {
            let user = this.ui.players[i];

            let item = build_element('li', [
                'list-group-item',
                'd-flex',
                'justify-content-between',
                'align-items-center',
            ]);

            // point evaluation
            let points;
            if (this.state != 0) {
                points = 0; /*
        for (let f = 0; f < 5; f++) {
          let location = this.board.fields[f + i * 5][0];
          if (location[0] == -1 && location[1] == -1) {
            points++;
          }
        }*/
            } else {
                points = '?';
            }

            if (user[0] == this.ui.host) {
                item.innerHTML = `${sanitize(
                    user[1]
                )} <span class="badge bg-light text-dark rounded-pill" data-bs-toggle="tooltip" data-bs-placement="top" title="Host" id="player-points-${sanitize(
                    user[0]
                )}">${points}</span>`;
            } else {
                item.innerHTML = `${sanitize(
                    user[1]
                )} <span class="badge bg-primary rounded-pill" id="player-points-${sanitize(
                    user[0]
                )}">${points}</span>`;
            }

            // append new player item to container
            container.appendChild(item);
        }

        // activate new tooltips
        this.update_tooltips();
    }

    update_tooltips() {
        let tooltip_els = document.querySelectorAll(
            '[data-bs-toggle="tooltip"]'
        );
        for (let i = tooltip_els.length - 1; i > -1; i--) {
            new Tooltip(tooltip_els[i]);
        }
    }

    // actions bound to buttons
    start_game() {
        this.send_message(new StartGameAction());
    }
}

// will be reworked later

let shift = false;

on_load(() => {
    /*
   Check for screen size and show popup if incompatible
  */
    if (screen.height < 600 || screen.width < 1000) {
        // Create the element using the create_alertScreen Size method.
        create_alert(2, 'UI', 'Your screen is too small for this game.');
    }

    /*  
    Init UI
    */

    init_ui('GAME');

    /*
   This require authentication as the request is handled with SessionCookies
   Though this script should only be used by a template from a protected route
   */
    let instance = new Game();
    let user = getJSONP('/api/users/who-am-i', 'GET');
    instance.set_user(user);
    instance.init();

    // fab buttons binds
    document.getElementById('btn-rotate').addEventListener('click', (event) => {
        event.preventDefault();
        shift = shift ? false : true; // toggle shift value
        instance.redraw_board(shift);
    });

    document
        .getElementById('btn-download')
        .addEventListener('click', (event) => {
            event.preventDefault();
            download('board.svg', instance.board.container.svg());
        });

    // host-only fab binds
    let start_btn = document.getElementById('start-btn');
    if (start_btn !== undefined && start_btn !== null) {
        start_btn.addEventListener('click', (event) => {
            event.preventDefault();
            instance.start_game();
        });
    }
});
