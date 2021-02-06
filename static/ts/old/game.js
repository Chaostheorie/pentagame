import {
  /* webpackMode: "eager" */
  PentaMath,
} from "./pentagame.js";
import {
  /* webpackMode: "eager" */
  create_alert,
  download,
  getJSONP,
} from "./utils.js.js";
import DOMPurify from "dompurify";

const SCALE = 1000; // adjust according top used viewbox dimensions of parent svg

class Game {
  constructor(url) {
    this.url = url;
    this.drawer_args = {
      shift: false,
    };
    this.drawer_added = false;
    this.figures = [];
    this.players = []; // [[id, username]...]
    this.state = 0; // 0 = nothing selected, 1: sth selected
  }

  // Host only
  start_game() {
    if (this.players.length == 1) {
      create_alert(1, "You need at least two players to start a game");
      return false;
    } else {
      this.socket.send(JSON.stringify({ action: 5, data: {} })); // query game data (meta + players)
      return true;
    }
  }

  create_modal() {
    this.loading = {
      modal: undefined,
      content: undefined,
      progress: undefined,
    };

    // bind bs modal
    this.loading.modal_el = document.getElementById("loading-modal");
    this.loading.modal = new bootstrap.Modal(this.loading.modal_el, {
      show: true,
      backdrop: "static",
      keyboard: false,
      focus: true,
    });

    // bind modal-title and modal-content
    this.loading.content = document.getElementById("modal-content");

    // set base progress
    this.loading.progress = document.getElementById("connection-progress");
    this.loading.progress.style.width = "0";
    this.loading.progress.setAttribute("aria-valuenow", "0");

    // show modal
    this.loading.modal.show();
  }

  draw_board() {
    if (!this.drawer_added) {
      this.drawer = SVG().addTo("#penta");
      this.drawer_added = true;
    }
    this.drawer.addClass("allow-overflow responsive-img");
    this.drawer.attr({
      preserveAspectRatio: "xMidYMid meet",
      id: "penta",
    });

    this.drawer.data({ size: SCALE });
    this.math = new PentaMath(this.drawer);
    this.math.draw(SCALE, this.drawer_args);
  }

  make_move(id) {
    console.log(`Attempt Moving figure: ${this.selected[1]} -> ${id}`);
    /* this.socket.send(
      JSON.stringify({ action: 2, figure: this.selected[0], move: [] })
    ); */

    console.log(this.selected[1]);
    this.state = 0;
    // this.select_figure.node.remove();
    let int_id = id.map((i) => parseInt(i, 10));
    console.debug(
      `Selected Figure: ${this.math.drawFigure(
        int_id,
        this.selected[0],
        this.selected[1]
      )}`
    );
  }

  interpret_target(target) {
    let id = target.getAttribute("pentagame-id").split("-");
  }

  select_figure(figure, id) {
    this.state = 1;
    this.selected = [figure.id[0], id, figure.id[1]];
    figure.remove();
    this.selected_figure = this.math.drawFigure(
      [-1, 0, 0],
      figure.id[0],
      figure.id[1]
    );
    console.debug(`Selected Figure: ${this.selected_figure}`);
  }

  on_click(event) {
    console.log("Clicked", event.target);

    // ensure it's actually the current player's turn
    if (
      this.reference.meta.state -
        Math.floor(this.reference.meta.state / 5) * 5 ==
        this.reference.user.index &&
      this.reference.meta.state != 0
    ) {
      if (this.reference.state == 1) {
        let internal_id = event.target.getAttribute("pentagame-id").split("-");

        this.reference.make_move(internal_id);
      } else if (this.reference.meta.state < 6) {
        // fetch Figure instance from reference
        let internal_id = event.target.getAttribute("pentagame-id").split("-");
        let figure = this.reference.math.board.figures[
          `${internal_id[0]},${internal_id[1]}`
        ];
        if (figure == undefined) {
          create_alert(2, "There's no selectable figure on the clicked field");
        } else {
        }
        let parent = this.reference.math.board.fields[figure.parent_id];

        if (internal_id[1] != this.reference.user.index) {
          let owned_figure = null;
          for (let c = parent.children.length - 1; c > -1; c--) {
            if (parent.children[c].id == internal_id) {
              owned_figure = parent.children[c];
              break;
            }
          }

          if (owned_figure == null) {
            create_alert(
              2,
              "There's no selectable figure on the clicked field"
            );
          } else {
            this.reference.select_figure(owned_figure, figure.parent_id);
          }
        } else {
          this.reference.select_figure(figure, figure.parent_id);
        }
      } else if (this.reference.meta.state < 11) {
        element.innerHTML = "Please place a black stopper";
      } else if (this.reference.meta.state < 16) {
        element.innerHTML = "Please place a gray stopper";
      }
    } else {
      console.debug(event.target.getAttribute("pentagame-id"));
    }
  }

  open(user) {
    // save current user information
    this.user = user;

    // start drawing
    this.draw_board();
    this.create_modal();

    // INFO: Change for production
    // TODO: Figure out how to effectively do this on build with e.g. sed and regex on compiled assets
    if (this.url !== undefined) {
      this.socket = new WebSocket(this.url);
    } else {
      this.socket = new WebSocket(`ws://${window.location.host}/games/ws/`);
    }

    this.socket.reference = this;
    this.socket.onopen = this.onopen;
    this.socket.onclose = this.onclose;
    this.socket.onmessage = this.onmessage;
  }

  onopen(event) {
    // set new progress
    this.reference.loading.content.innerHTML = "Connected to Websocket";
    this.reference.loading.progress.style.width = "25";
    this.reference.loading.progress.setAttribute("aria-valuenow", "25");

    // startup trigger
    this.reference.startup();
  }

  startup() {
    // start intial setup
    this.socket.send(JSON.stringify({ action: 0, data: { all: "true" } })); // fetch all moves (uses "true" to be parsable as String)
    this.socket.send(JSON.stringify({ action: 1, data: {} })); // query game data (meta + players)

    this.loading.modal.hide();
  }

  add_player(new_player) {
    console.log(`New Player: ${new_player}`);
    for (let p = this.players.length - 1; p > -1; p--) {
      let player = this.players[p];
      if (player[0] == new_player[0]) {
        // return early as player already known (could be triggered by e.g. reconnect)
        create_alert(1, `Player ${DOMPurify.sanitize(player[1])} reconnected`);
        return;
      }
    }

    this.players.push(new_player);
  }

  remove_player(player_id) {
    for (let i = this.players.length - 1; i > -1; i--) {
      if (this.players[i][0] == player_id) {
        this.players.splice(i, 1);
        break;
      }
    }
    this.update_players();
  }

  update_players() {
    let list = document.getElementById("game-players");
    list.innerHTML = "";

    for (let i = this.players.length - 1; i > -1; i--) {
      let user = this.players[i];

      let item = document.createElement("li");
      item.classList.add(
        "list-group-item",
        "d-flex",
        "justify-content-between",
        "align-items-center"
      );

      // point evaluation
      let points = 0;
      for (let f = 0; f < 5; f++) {
        let location = this.board.locations[f + i * 5][0];
        if (location[0] == -1 && location[1] == -1) {
          points++;
        }
      }

      if (user[0] == this.meta.host) {
        item.innerHTML = `${DOMPurify.sanitize(
          user[1]
        )} <span class="badge bg-light text-dark rounded-pill" data-bs-toggle="tooltip" data-bs-placement="top" title="Host" id="player-points-${DOMPurify.sanitize(
          user[0]
        )}">${points}</span>`;
      } else {
        item.innerHTML = `${DOMPurify.sanitize(
          user[1]
        )} <span class="badge bg-primary rounded-pill" id="player-points-${DOMPurify.sanitize(
          user[0]
        )}">${points}</span>`;
      }
      list.appendChild(item);
    }

    // activate new tooltips
    [].slice
      .call(document.querySelectorAll('[data-bs-toggle="tooltip"]'))
      .map(function (tooltipTriggerEl) {
        return new bootstrap.Tooltip(tooltipTriggerEl, {
          placement: "auto",
        });
      });

    this.loading.progress.classList.remove("w-50");
    this.loading.content.innerHTML = "Loaded players";
    this.loading.progress.style.width = "65%";
    this.loading.progress.setAttribute("aria-valuenow", "65");
  }

  redraw_board(args) {
    this.drawer_args = args;

    this.math.clear();
    this.draw_board();
    this.update_board();
  }

  update_message() {
    // fetch elements from DOM
    let element = document.getElementById("game-tmp-text"),
      container = document.getElementById("game-tmp-card"),
      icon = document.getElementById("game-tmp-icon");

    // evaluate and update
    if (this.meta.state == 0 || this.meta.state > 16) {
      container.classList.add("d-none");
    } else {
      if (container.classList.contains("d-none")) {
        container.classList.remove("d-none");
      }

      // ensure icon is set to display a piece
      icon.classList.add("fa-chess-pawn");
      icon.classList.remove("fa-clock");

      console.log(this.user.index);
      console.log(this.meta.state - Math.floor(this.meta.state / 5) * 5);

      if (
        this.meta.state - Math.floor(this.meta.state / 5) * 5 ==
        this.user.index
      ) {
        if (this.meta.state < 6) {
          element.innerHTML = "Please choose a figure to move";
        } else if (this.meta.state < 11) {
          element.innerHTML = "Please place a black stopper";
        } else if (this.meta.state < 16) {
          element.innerHTML = "Please place a gray stopper";
        }
      } else {
        container.classList.add("d-none");
      }
    }
  }

  update_state() {
    // fetch state element
    let element = document.getElementById("game-state");

    // evaluate and update
    if (this.meta.state == 0) {
      element.innerHTML = "Game is not running";
    } else if (this.meta.state < 6) {
      element.innerHTML = DOMPurify.sanitize(
        `Waiting for move by ${this.meta.players[this.meta.state - 1][1]}`
      );
    } else if (this.meta.state < 11) {
      element.innerHTML = DOMPurify.sanitize(
        `Waiting for black stopper by of ${
          this.meta.players[this.meta.state - 1][1]
        }`
      );
    } else if (this.meta.state < 16) {
      element.innerHTML = DOMPurify.sanitize(
        `Waiting for gray stopper by of ${
          this.meta.players[this.meta.state - 1][1]
        }`
      );
    } else {
      let winners = "",
        winner_amount = this.meta.state - 10;
      for (let i = 0; i < winner_amount; i++) {
        if (i + 2 == winner_amount) {
          winners.concat(`${this.meta.players[i]} and `);
        } else {
          winners.concat(`${this.meta.players[i]}, `);
        }
      }
      element.innerHTML = DOMPurify.sanitize(
        `Game won by ${winners} Congratulations!`
      );
    }
  }

  update_metadata(meta) {
    // save meta (includes players)
    this.meta = meta;

    // update players
    this.players = this.meta.players;

    // update internal index
    for (let index = this.players.length - 1; index > -1; index--) {
      if (this.players[index][0] == this.user.id) {
        // id is enough to compare as Uuids are unique
        this.user.index = index;
        break;
      }
    }
    this.update_players();

    /*
    Evaluate State description
    see server/db/model for mapping
    */
    this.meta.state = Number(meta.state);
    this.update_state();
    this.update_message();

    // set description
    let description = document.getElementById("game-description");
    description.innerHTML = DOMPurify.sanitize(this.meta.description);

    // set pin
    let pin = document.getElementById("game-pin");
    if (this.meta.pin !== undefined && this.meta.pin.length !== 0) {
      pin.innerHTML = DOMPurify.sanitize(this.meta.pin.join("-"));
    } else {
      let parent = document.getElementById("game-pin-parent");
      if (parent !== null && parent !== undefined) {
        parent.remove();
      }
      if (pin !== null && pin !== undefined) {
        pin.remove();
      }
    }

    // set name
    let name = document.getElementById("game-name");
    name.innerHTML = `<i class="fas ${DOMPurify.sanitize(
      this.meta.icon
    )}"></i> ${DOMPurify.sanitize(this.meta.name)}`;

    this.loading.content.innerHTML = "Loaded Metadata";
    this.loading.progress.style.width = "75%";
    this.loading.progress.setAttribute("aria-valuenow", "75");
  }

  update_board() {
    for (let i = this.board.locations.length - 1; i > -1; i--) {
      this.math.drawFigure(
        this.board.locations[i][0],
        this.board.locations[i][1],
        i - Math.floor(i / 5) * 5
      );
    }

    // ensure all elements are bound
    let elements = document.querySelectorAll("[pentagame-listener]");
    for (let i = elements.length - 1; i > -1; i--) {
      elements[i].reference = this;
      elements[i].addEventListener("click", this.on_click);
    }
  }

  onmessage(event) {
    let data = JSON.parse(event.data);
    switch (data.action) {
      case 1:
        this.reference.update_metadata(data.data);
        break;
      case 2: // move made (-> GraphState)
        this.reference.board = data.data; //  [([i16;3], u8)]
        this.reference.update_board();
        break;
      case 3: // player joined
        console.log(data.data.player.split("|"));
        this.reference.add_player(data.data.player.split("|"));
        this.reference.update_players();
        break;
      case 4: // game started
        this.reference.meta.state = 1;
        this.reference.update_metadata(this.reference.meta);
        break;
      case 5: // player left
        this.reference.remove_player(data.data.player); // Uuid
        break;
      default:
        console.debug(data);
        create_alert(
          0,
          "Protocol Error: Server responded with an unknown action code"
        );
        break;
    }
  }

  onclose(event) {
    this.reference.loading.content.innerHTML =
      "Websocket Closed by server. Are you connected to the internet?";
    this.reference.loading.progress.style.backgroundColor = "red";
    this.reference.loading.progress.style.width = "100";
    this.reference.loading.progress.setAttribute("aria-valuenow", "100");
  }
}

const instance = new Game("ws://localhost:8080/games/ws/");
let shift = false;

document.addEventListener("DOMContentLoaded", async function () {
  /*
   Check for screen size and show popup if incompatible
  */
  if (screen.height < 600 || screen.width < 1000) {
    // Create the element using the create_alertScreen Size method.
    create_alert(2, "Your screen is too small for this game.");
  }

  /*
   This doesn't do authentication as the request is handled with SessionCookies
   */

  let user = await getJSONP({ url: "/api/users/who-am-i", method: "GET" });
  instance.open(user);
  globalThis.instance = instance;

  // fab buttons binds
  document.getElementById("btn-rotate").addEventListener("click", (event) => {
    event.preventDefault();
    shift = shift ? false : true; // toggle shift value
    instance.redraw_board({ shift });
  });

  document.getElementById("btn-download").addEventListener("click", (event) => {
    event.preventDefault();
    download("board.svg", instance.math.drawer.svg());
  });

  // host-only fab binds
  let start_btn = document.getElementById("start-btn");
  if (start_btn !== undefined && start_btn !== null) {
    start_btn.addEventListener("click", (event) => {
      event.preventDefault();
      if (instance.start_game()) {
        start_btn.remove();
      }
    });
  }
});
