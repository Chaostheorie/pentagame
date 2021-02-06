import { Circle, Svg, Element } from '@svgdotjs/svg.js';
import { PMath } from './math';
import { helper } from './utils';
import { TSMap } from 'typescript-map';

export interface Position {
    x: number;
    y: number;
}

// COLORS may be changed in the future but will stay here for now
export const COLORS = {
    fields: ['blue', 'white', 'green', 'yellow', 'red'],
    background: '#28292b',
    foreground: '#d3d3d3',
};

export class Figure {
    // attributes
    public id: number[];
    public color: number; // refer to COLORS constant
    node: Circle;

    // constructor
    constructor(id: string) {
        // unnest id
        let fragments: number[] = id
            .split('-')
            .map((value: string) => parseInt(value, 10));

        // save internally
        if (fragments[0] > fragments[2]) {
            this.id = fragments;
        } else {
            this.id = [fragments[2], fragments[1], fragments[0]]; // basically reverse
        }
    }

    // methods
    on_click(): Function {
        return () => {
            // NOTE: This will be called from the circle
            console.log(`[BOARD]: Clicked figure`);
            console.log(this); // this -> Figure
        };
    }

    // abstract methods
    draw(container: Svg, target: Field, math: PMath): void {}
}

abstract class Field {
    // attributes
    public id: number[];
    public radius: number;
    public position: Position;
    node: Circle;

    // constructor
    constructor(id: number[]) {
        // save internally
        if (id[0] > id[2]) {
            this.id = id;
        } else {
            if (id[1] > 3) {
                this.id = [id[2], id[1], id[0]]; // reverse to keep conformity
            }
        }
    }

    // methods
    on_click(): Function {
        return () => {
            // NOTE: This will be called from the circle
            console.log(`[BOARD]: Clicked node ${this.id.toString()}`);
            console.log(this); // this -> Field
        };
    }

    // abstract methods
    abstract draw(container: Svg, math: PMath): void;
}

export class Corner extends Field {
    draw(container: Svg, math: PMath) {
        this.radius = math.corner_radius;

        this.position = helper(
            math.center,
            math.outer_radius,
            (this.id[0] - 5) * -72,
            math.shift
        );
        this.node = container
            .circle(this.radius)
            .center(this.position.x, this.position.y)
            .click(this.on_click());
    }
}

export class Junction extends Field {
    draw(container: Svg, math: PMath) {
        this.radius = math.junction_radius;
        this.position = helper(
            math.center,
            math.inner_radius,
            this.id[0] * -72 + 180,
            math.shift
        );
        this.node = container
            .circle(this.radius)
            .center(this.position.x, this.position.y)
            .click(this.on_click());
    }
}

export class Stop extends Field {
    draw(container: Svg, math: PMath) {
        this.radius = math.stop_radius;
        if (this.id[0] < 5) {
            // J -> J
            console.log(this.id);
            this.position = helper(
                helper(
                    math.center,
                    math.inner_radius,
                    this.id[0] * -72 + 180,
                    math.shift
                ),
                math.stop_radius * this.id[1] + math.junction_radius / 4,
                this.id[0] * -72 + 180 + math.constants.theta * 7
            );

            this.node = container
                .circle(this.radius)
                .center(this.position.x, this.position.y)
                .click(this.on_click());
        }
    }
}

export class Stopper extends Figure {}

export class Player extends Figure {}

export class Board {
    fields: TSMap<number[], Field>; // K + S ∊ fields
    /*
  For an interested reader:
    J: junctions 
    C: corners
    S: stops (everything between junctions and corners)
    K: junctions + corners
  */

    // math instance
    math: PMath;

    // background node
    background: Circle;

    // SVG container + SVG viewBox scale
    center: Position;
    container: Svg;
    scale: number;
    shift: boolean; // default: false

    // constructors
    constructor(
        scale: number,
        container: Svg,
        draw?: boolean,
        background?: boolean
    ) {
        this.shift = false;
        this.fields = new TSMap();
        this.math = new PMath(scale, this.shift);
        this.container = container;
        this.center = {
            x: scale / 2,
            y: scale / 2,
        };

        // check if background should be drawn
        if (background !== false) {
            this.draw_background();
        } else {
            this.background = null;
        }

        // check idf board should be drawn
        if (draw !== false && draw !== undefined && draw !== null) {
            this.draw();
        }
    }

    // methods
    draw_background() {
        this.background = this.container
            .circle(this.math.scale)
            .center(this.center.x, this.center.y);
        console.log('[GAME]: Background has been drawn');
    }

    clear() {
        this.container
            .children()
            .forEach((element: Element, _index: number, _array: Element[]) => {
                element.remove();
            });
        this.fields.clear();
        console.log('[GAME]: Board has been cleared');
    }

    // default - clear: true
    draw(clear?: boolean) {
        if (clear !== false) {
            this.clear();
        }

        // assemble field
        // NOTE: This implementations goes counter clockwise
        for (let k = 0; k < 6; k++) {
            // create corners and junctions
            let corner_id = k + 5,
                corner = new Corner([corner_id, 0, 0]),
                junction = new Junction([k, 0, 0]);

            // evaluate adjacent (clockwise) junctions and corner
            let next_corner = k == 4 ? 5 : k + 6,
                next_junction = k == 4 ? 0 : k + 1,
                second_junction = k == 4 ? 1 : k == 3 ? 0 : k + 2;

            // draw corners and save them as fields
            corner.draw(this.container, this.math);
            junction.draw(this.container, this.math);
            this.fields.set([k, 0, 0], corner);
            this.fields.set([corner_id, 0, 0], junction);

            // draw inner + outer arms
            for (let a = 1; a < 4; a++) {
                let inner_arm = new Stop([next_junction, a]);
            }

            // draw legs
        }
    }
}
