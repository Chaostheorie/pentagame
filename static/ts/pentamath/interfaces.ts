import { Position } from './models';

enum MOVE_MODES {
    SIMPLE_MODE = 1,
    EXTENSIVE_MODE = 2,
}

interface Move {
    player: string; // player identification
    figure: string; // figure id
    mode?: MOVE_MODES; // move type
    move: string[]; // move related data
}

interface FigurePosition {
    position: Position;
    figure: string;
}

interface Game {
    players: string[];
    base: FigurePosition[];
    moves: Move[];
}

interface RuleSet {
    figure: Number; // figures required to win
    compression: Boolean;
}

class GameFile {
    // attributes
    game: Game;
    rule_set: RuleSet;

    // constructors
    constructor(game: Game, rule_set?: RuleSet) {
        this.game = game;
        if (rule_set !== undefined && rule_set !== null) {
            this.rule_set = rule_set;
        } else {
            this.rule_set = {
                figure: game.players.length - 2,
                compression: true,
            };
        }
    }
}
