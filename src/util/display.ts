import { gamepadButtonSelectionOptions } from '../backend';

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

export function labelForGamepadButton(btn: number): string {
    return gamepadButtonSelectionOptions.get(btn) ?? 'unknown';
}
