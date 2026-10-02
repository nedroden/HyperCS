import de from './flags/de.svg';
import us from './flags/us.svg';

const FLAGS = { de, us } as const;

export type FlagCode = keyof typeof FLAGS;

/** Decorative country flag. Put the country/language name next to it as text. */
export function Flag({ code }: { code: FlagCode }) {
    return (
        <img className="flag" src={FLAGS[code]} alt="" width={20} height={15} />
    );
}
