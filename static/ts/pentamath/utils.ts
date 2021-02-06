import { Position, COLORS } from './models';

export function helper(
    center: Position,
    radius: number,
    angle: number,
    shift?: boolean
): Position {
    if (shift) {
        angle = (angle * Math.PI) / 180 + (Math.PI / 180.0) * -18;
    } else {
        angle = (angle * Math.PI) / 180;
    }

    return {
        x: center.x + radius * Math.cos(angle),
        y: center.y + radius * Math.sin(angle),
    };
}

export function evaluate_figure_attributes(
    id: number
): { color: string; type: string } {
    // evaluate color and type based on id
    let color, type;

    if (id < 26) {
        // player
        type = 'player';
        // evaluate color
        color = COLORS.fields[Math.floor(id)];
    } else if (id < 31) {
        // black stopper
        color = 'black';
        type = 'black-stopper';
    } else {
        // gray stopper
        color = 'gray';
        type = 'gray-stopper';
    }

    return { color, type };
}
