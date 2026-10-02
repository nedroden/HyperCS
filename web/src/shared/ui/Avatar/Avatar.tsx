type AvatarProps = {
    name: string;
    size?: 'sm' | 'md' | 'lg';
    shape?: 'square' | 'round';
};

/** Placeholder avatar showing the user's initial. */
export function Avatar({ name, size = 'md', shape = 'square' }: AvatarProps) {
    return (
        <div
            className={`avatar avatar-${size} avatar-${shape}`}
            title={name}
            aria-hidden="true"
        >
            {name.charAt(0).toUpperCase()}
        </div>
    );
}
