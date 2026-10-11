import { Link } from 'react-router-dom';

type UserLinkProps = {
    username: string;
    displayName: string;
};

/** The display name of a member, linking to their public profile. */
export function UserLink({ username, displayName }: UserLinkProps) {
    return (
        <Link to={`/forum/users/${encodeURIComponent(username)}`}>
            {displayName}
        </Link>
    );
}
