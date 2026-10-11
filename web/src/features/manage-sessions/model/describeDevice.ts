export type DeviceHint = { browser: string; os: string };

/** Browser, in the order that matters (Edge and Opera also say "Chrome", Chrome also says "Safari"). */
const BROWSERS: [RegExp, string][] = [
    [/Edg(e|A|iOS)?\//, 'Edge'],
    [/OPR\/|Opera/, 'Opera'],
    [/Firefox\/|FxiOS\//, 'Firefox'],
    [/Chrome\/|CriOS\//, 'Chrome'],
    [/Safari\//, 'Safari'],
];

/** OS, iOS before macOS (iPad/iPhone UAs contain "like Mac OS X"), Android before Linux. */
const SYSTEMS: [RegExp, string][] = [
    [/iPhone/, 'iPhone'],
    [/iPad/, 'iPad'],
    [/Android/, 'Android'],
    [/Windows/, 'Windows'],
    [/Mac OS X|Macintosh/, 'macOS'],
    [/CrOS/, 'ChromeOS'],
    [/Linux|X11/, 'Linux'],
];

/** A coarse "Chrome on macOS" hint from a raw `User-Agent`; each part is `null` when not recognized. */
export function describeDevice(userAgent: string | null | undefined): {
    browser: string | null;
    os: string | null;
} {
    const ua = userAgent ?? '';
    return {
        browser: BROWSERS.find(([re]) => re.test(ua))?.[1] ?? null,
        os: SYSTEMS.find(([re]) => re.test(ua))?.[1] ?? null,
    };
}
