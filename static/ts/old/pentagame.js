import {
  /* webpackMode: "eager" */
  Field,
  Figure,
  COLORS,
} from "./core.js.js";
import {
  /* webpackMode: "eager" */
  helper,
  evaluate_figure_id,
} from "./utils.js.js";

export class PentaMath {
  /*
    This class provides an appropriate representation of the sizes and values for the construction of a pentagame board.
    The basic logic was supplied by @penta-jan <https://github.com/penta-jan>.
    This implementation was written by @cobalt <https://cobalt.rocks>
    Enhanced variant based on <https://github.com/Penta-Game/boardgame>
    Inspired by <https://github.com/NikkyAI/pentagame>
    To learn more about pentagame visit https://pentagame.cobalt.rocks
    */

  // TODO: Rework and separate into components

  constructor(drawer) {
    // holds the numerical constants
    this._constants = {
      l: 6, // legs
      k: 3, // arms
      p: Math.sqrt((25 - 11 * Math.sqrt(5)) / (5 - Math.sqrt(5))), // inner
      golden: (Math.sqrt(5) + 1) / 2, // golden section value
      theta: 18, // theta value
    };
    // holds the relative numerical relative values centered on s
    this._sizes = {
      s: 1, // stop on star
      c: Math.sqrt(5), // corner stop
      j: (9 - 2 * Math.sqrt(5)) / Math.sqrt(5), // junction stop
      r: (2 / 5) * Math.sqrt(1570 + 698 * Math.sqrt(5)), // pentagram (diameter)
    };
    this._sizes.R = this._sizes.r + this._sizes.c; // entire board
    this._sizes.outer_circle = (this._sizes.r / this._sizes.R) * 0.2; // background stroke width
    this._sizes.inner_r =
      ((this._constants.k + this._sizes.j) * (1.0 + this._sizes.c)) /
      Math.sqrt(2.0 * (5.0 + this._sizes.c));
    this._constants.sizes = this._sizes;
    this.constants = this._constants;
    this.drawer = drawer;
    this.sizes = {};
    this.helper = helper;
  }

  // draw figure assumes a complete board was drawn at least once
  drawFigure(location, figure_id, user_id) {
    // check if 'off board'
    /*
      - 1-5 (pid): Waiting for move of {pid}
      - 6-10 (pid-5): Waiting for {pid} to set stopper
      */
    let figure;

    if (location[0] == -1) {
      let size = this.board.fields["1-0-0"].node.r.baseVal.value / 0.8;
      let { color, type } = evaluate_figure_id(figure_id, user_id);

      // create node and draw figure
      let node = this.drawer.circle(size);
      node.attr({
        fill: color,
        stroke:
          figure_id > 25 || location[1] == 0
            ? COLORS.foreground
            : COLORS.background,
        "stroke-width": 0.5 * this.sizes.lineWidth,
        type,
        size,
      });
      figure = new Figure(
        {
          size,
          x: 0,
          y: 0,
          node,
          color,
          type,
          id: [figure_id, user_id],
        },
        "-1"
      );

      if (location[1] == 0) {
        // selected figure when e.g. placing stopper

        // update attributes and move to center
        figure.x = this.center.x;
        figure.y = this.center.y;

        figure.node.center(this.center.x, this.center.y);
        figure.node.size(figure.size / 0.8);
      } else {
        // 'out' figure
        let position = helper(
          this.center.x,
          this.center.y,
          size,
          (this.board.out.length + 1) * -72,
          this.shift
        );

        // update attributes and move to position
        figure.x = position.x;
        figure.y = position.y;

        figure.node.center(position.x, position.y);
      }

      this.board.out.push(figure);
    } else {
      // draw element
      let parent = this.board.fields[
        `${location[0]}-${location[1]}-${location[2]}`
      ];
      figure = parent.drawFigure(this.drawer, this.shift, figure_id, user_id);

      this.board.figures[figure.id] = figure;
    }

    return figure;
  }

  clear() {
    // remove all drawn nodes
    for (let i = this.drawer.node.childNodes.length - 1; i > -1; i--) {
      this.drawer.node.childNodes[i].remove();
    }

    // setup new & empty board
    this.board = {
      fields: { "-1": [] },
      figures: {},
      out: [], // number of gray stoppers with location at [-1, -1, -1]
    };
  }

  draw(scale, args) {
    // evaluate args
    if (args !== undefined && args.shift !== false) {
      this.shift = { shift: true };
    } else {
      this.shift = { shift: false };
    }

    // setup board
    this.board = {
      fields: { "-1": [] },
      figures: {},
      out: [],
    };

    // save scale
    this.scale = scale;

    // ensure drawer aspect ratio
    this.drawer.viewbox(`0 0 ${this.scale} ${this.scale}`);
    this.drawer.attr({ preserveAspectRatio: "xMidYMid meet" });

    // evaluate basic points and values
    this.center = { x: 0.5 * scale, y: 0.5 * scale };
    scale = scale * 0.8; // prevent overflow
    this.sizes.lineWidth = (0.1 / this.constants.sizes.R) * scale;
    this.sizes.InnerRadius =
      (scale / this.constants.sizes.R) * this.constants.sizes.inner_r;
    this.sizes.OuterRadius =
      scale / this.constants.sizes.c + this.sizes.lineWidth * 3.5;
    this.sizes.JunctionRadius =
      (scale / this.constants.sizes.R) * this.constants.sizes.j;
    this.sizes.CornerRadius =
      (scale / this.constants.sizes.R) * this.constants.sizes.c;
    this.sizes.StopRadius =
      (scale / this.constants.sizes.R) * this.constants.sizes.s;

    // bg circle
    const BGCircle = this.drawer.circle(scale + this.sizes.lineWidth * 5);
    BGCircle.attr({
      cx: this.center.x,
      cy: this.center.y,
      fill: COLORS.background,
      id: "background-circle",
    });

    // draw outer circle
    const OuterBGCircle = this.drawer.circle(this.sizes.OuterRadius * 2);
    OuterBGCircle.attr({
      cx: this.center.x,
      cy: this.center.y,
      fill: "none",
      stroke: COLORS.foreground,
      "stroke-width": this.sizes.lineWidth,
    });
    OuterBGCircle.data({ id: "outer-circle" });

    // drawing corners and junctions
    for (var i = 0; i < 5; i++) {
      let CornerAngle = i * -72;
      let CornerPoints = this.helper(
        this.center.x,
        this.center.y,
        this.sizes.OuterRadius,
        CornerAngle,
        this.shift
      );
      let JunctionAngle = CornerAngle + 180;
      let JunctionPoints = this.helper(
        this.center.x,
        this.center.y,
        this.sizes.InnerRadius,
        JunctionAngle,
        this.shift
      );

      // evaluate target ids
      let outer_target, arm_target, reversed;
      if (i == 0) {
        outer_target = 9;
        arm_target = i + 1;
        reversed = [true, false];
      } else if (i == 4) {
        outer_target = i + 4;
        arm_target = 0;
        reversed = [false, true];
      } else {
        outer_target = i + 4;
        arm_target = i + 1;
        reversed = [false, false];
      }

      for (let x = 3; x !== 0; x--) {
        // evaluate ids
        let outer_id = reversed[0]
          ? `${outer_target}-${x}-${i + 5}`
          : `${i + 5}-${x}-${outer_target}`;
        let arm_id = reversed[1]
          ? `${i}-${x}-${arm_target}`
          : `${arm_target}-${x}-${i}`;

        // evaluate angles
        let StopAngle = CornerAngle + this.constants.theta * x;
        let StopPoints = this.helper(
          this.center.x,
          this.center.y,
          this.sizes.OuterRadius,
          StopAngle,
          this.shift
        );

        // create and move object
        let OuterStop = this.drawer.circle(this.sizes.StopRadius);
        OuterStop.center(StopPoints.x, StopPoints.y);

        // assign attributes to node
        OuterStop.attr({
          fill: COLORS.foreground,
          stroke: COLORS.background,
          size: this.sizes.StopRadius,
          type: "stop",
          "stroke-width": this.sizes.lineWidth * 0.5,
        });

        OuterStop.node.setAttribute("pentagame-id", `id-${outer_id}`);

        this.board.fields[outer_id] = new Field({
          x: StopPoints.x,
          y: StopPoints.y,
          angle: StopAngle,
          node: OuterStop.node,
          type: "stop",
          children: [],
        });

        let ArmAngle = JunctionAngle - this.constants.theta * 7;
        let ArmPoints = this.helper(
          JunctionPoints.x,
          JunctionPoints.y,
          this.sizes.StopRadius * x + this.sizes.JunctionRadius / 4,
          ArmAngle,
          this.shift
        );
        let ArmStop = this.drawer.circle(this.sizes.StopRadius);
        ArmStop.attr({
          fill: COLORS.foreground,
          stroke: COLORS.background,
          "stroke-width": this.sizes.lineWidth * 0.5,
          size: this.sizes.StopRadius,
          type: "stop",
        });
        ArmStop.center(ArmPoints.x, ArmPoints.y);
        ArmStop.node.setAttribute("pentagame-id", arm_id);
        this.board.fields[arm_id] = new Field({
          x: ArmPoints.x,
          y: ArmPoints.y,
          angle: ArmAngle,
          node: ArmStop.node,
          size: this.sizes.StopRadius,
          type: "stop",
          children: [],
        });
      }

      // draw legs (connecting corner -> 2 junctions)
      for (let x = 6; x !== 0; x--) {
        // target id evaluation
        let left, right;
        if (i == 4) {
          left = 2;
          right = 1;
        } else {
          left = i + 3;
          right = i + 2;
        }

        // angles and target id
        const LegAngles = [
          [this.constants.theta + 180 + CornerAngle, left],
          [this.constants.theta * -1 + 180 + CornerAngle, right],
        ];

        for (let y = LegAngles.length - 1; y > -1; y--) {
          // create and move leg
          let leg_id = `${i + 5}-${x}-${LegAngles[y][1]}`;
          let Leg = this.drawer.circle(this.sizes.StopRadius);
          let LegPoints = this.helper(
            CornerPoints.x,
            CornerPoints.y,
            this.sizes.StopRadius * x +
              this.sizes.CornerRadius / 4 +
              this.sizes.lineWidth * 1.5,
            LegAngles[y][0],
            this.shift
          );
          Leg.center(LegPoints.x, LegPoints.y);

          // assign data to leg node
          Leg.attr({
            fill: COLORS.foreground,
            stroke: COLORS.background,
            "stroke-width": this.sizes.lineWidth * 0.5,
          });

          Leg.node.setAttribute("pentagame-id", leg_id);

          // store in internal representation
          this.board.fields[leg_id] = new Field({
            x: LegPoints.x,
            y: LegPoints.y,
            node: Leg.node,
            angle: LegAngles[y][0],
            children: [],
            id: leg_id,
          });
        }
      }

      // draw Corners and Junctions
      let corner_id = `${i + 5}-0-0`,
        junction_id = `${i}-0-0`;
      let Corner = this.drawer.circle(this.sizes.CornerRadius);
      Corner.attr({
        fill: COLORS.foreground,
        stroke: COLORS.fields[i],
        "stroke-width": 0.75 * this.sizes.lineWidth,
        type: "corner",
        size: this.sizes.CornerRadius,
        id: corner_id,
      });
      Corner.center(CornerPoints.x, CornerPoints.y);
      Corner.node.setAttribute("pentagame-id", corner_id);

      var Junction = this.drawer.circle(this.sizes.JunctionRadius);
      Junction.attr({
        fill: COLORS.foreground,
        stroke: COLORS.fields[i],
        "stroke-width": 0.75 * this.sizes.lineWidth,
      });
      Junction.center(JunctionPoints.x, JunctionPoints.y);
      Junction.node.setAttribute("pentagame-id", junction_id);

      this.board.fields[corner_id] = new Field({
        id: corner_id,
        x: CornerPoints.x,
        y: CornerPoints.y,
        next: i + 6,
        node: Corner.node,
        angle: CornerAngle,
        color: COLORS.fields[i],
        type: "corner",
        size: this.sizes.JunctionRadius,
        children: [],
      });
      this.board.fields[junction_id] = new Field({
        id: junction_id,
        x: JunctionPoints.x,
        y: JunctionPoints.y,
        next: i + 2,
        type: "junction",
        node: Junction.node,
        angle: JunctionAngle,
        color: COLORS.fields[i],
        children: [],
      });
    }
  }
}

export default { PentaMath };
