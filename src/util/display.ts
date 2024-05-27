import { GamepadButtonSelection } from '../backend';

export function labelForCamelCase(s: string, separator = ' '): string {
    const splitIndexes: number[] = [];
    s = s[0].toUpperCase() + s.slice(1);

    [...s].forEach((c, i) => {
        if (c === c.toUpperCase()) {
            splitIndexes.push(i);
        }
    });

    splitIndexes.push(s.length);
    let start = splitIndexes.shift();

    const words = [];
    for (const next of splitIndexes) {
        words.push(s.slice(start, next));
        start = next;
    }

    return words.join(separator);
}

export function labelForKebabCase(s: string): string {
    return s
        .split('-')
        .map((v) => v[0].toUpperCase() + v.slice(1).toLowerCase())
        .join('-');
}

export function labelForGamepadButton(btn: GamepadButtonSelection): string {
    switch (btn) {
        case 'Start':
        case 'Select':
        case 'RightThumb':
        case 'LeftThumb':
            return labelForCamelCase(btn);
        case 'North':
            return 'Y (North)';
        case 'East':
            return 'B (East)';
        case 'South':
            return 'A (South)';
        case 'West':
            return 'X (West)';
        case 'DPadUp':
        case 'DPadLeft':
        case 'DPadRight':
        case 'DPadDown':
            return btn.replace('Pad', 'Pad ');
        case 'L1':
            return 'L1 (Left Bumper)';
        case 'L2':
            return 'L2 (Left Trigger)';
        case 'R1':
            return 'R1 (Right Bumper)';
        case 'R2':
            return 'R2 (Right Trigger)';
        default:
            const typecheck: never = btn;
            throw `display gamepad button failed to typecheck: ${typecheck}`;
    }
}
