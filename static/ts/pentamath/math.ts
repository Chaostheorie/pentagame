import { Position } from './models';

const root_five = Math.sqrt(5); // due to the high usage of this value it's precomputed

export class PMath {
    // constant values
    relative_sizes = {
        s: 1, // stop on star
        c: root_five, // corner stop
        j: (9 - 2 * root_five) / root_five, // junction stop
        r: (2 / 5) * Math.sqrt(1570 + 698 * root_five), // pentagram (diameter)
    };

    constants = {
        l: 6, // legs
        k: 3, // arms
        p: Math.sqrt((25 - 11 * root_five) / (5 - root_five)), // inner
        golden: (root_five + 1) / 2, // golden section value
        theta: 18, // theta value
    };

    // sizes
    line_width: number;
    R: number;
    inner_radius: number;
    outer_radius: number;
    junction_radius: number;
    corner_radius: number;
    stop_radius: number;

    // shift
    shift: boolean;

    // some more sizes
    center: Position;
    scale: number;
    adjusted_scale: number;

    // constructor
    constructor(scale: number, shift: boolean) {
        // save current shift state
        this.shift = shift;

        // save scale for drawing Fields and Figures later on
        this.scale = scale;
        this.adjusted_scale = scale * 0.8;
        this.center = { x: 0.5 * scale, y: 0.5 * scale };

        // calculate all the absolute sizes
        // please refer to the pentagame book
        this.R = this.relative_sizes.r + this.relative_sizes.c;
        this.inner_radius =
            ((this.constants.k + this.relative_sizes.j) *
                (1.0 + this.relative_sizes.c)) /
            Math.sqrt(10 + 2 * this.relative_sizes.c);
        this.line_width = (0.1 / this.R) * this.adjusted_scale;
        this.inner_radius =
            ((this.adjusted_scale / this.R) *
                ((this.constants.k + this.relative_sizes.j) *
                    (1.0 + this.relative_sizes.c))) /
            Math.sqrt(2.0 * (5.0 + this.relative_sizes.c)); // This needs to be optimized at some point
        this.outer_radius =
            this.adjusted_scale / this.relative_sizes.c + this.line_width * 3.5;
        this.junction_radius =
            (this.adjusted_scale / this.R) * this.relative_sizes.j;
        this.corner_radius =
            (this.adjusted_scale / this.R) * this.relative_sizes.c;
        this.stop_radius =
            (this.adjusted_scale / this.R) * this.relative_sizes.s;
    }
}
