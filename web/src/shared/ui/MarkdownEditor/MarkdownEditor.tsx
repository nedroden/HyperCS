import { useId, useState, type KeyboardEvent } from 'react';
import { useTranslation } from 'react-i18next';
import { MARKDOWN_MAX_LENGTH } from '@/shared/config';
import { Markdown } from '../Markdown';

type Mode = 'write' | 'preview';

type MarkdownEditorProps = {
    /** Id of the textarea (also what a `Field` label and error summary refer to). */
    id: string;
    value: string;
    onChange: (value: string) => void;
    onBlur?: () => void;
    name?: string;
    placeholder?: string;
    maxLength?: number;
    invalid?: boolean;
    describedBy?: string;
    disabled?: boolean;
};

/**
 * Markdown textarea with Write/Preview tabs (the preview uses the same sanitizing renderer as posts) and a
 * character counter. Controlled; works with react-hook-form's `Controller`.
 */
export function MarkdownEditor({
    id,
    value,
    onChange,
    onBlur,
    name,
    placeholder,
    maxLength = MARKDOWN_MAX_LENGTH,
    invalid,
    describedBy,
    disabled,
}: MarkdownEditorProps) {
    const { t } = useTranslation();
    const [mode, setMode] = useState<Mode>('write');
    const uid = useId();
    const tab = (m: Mode) => `${uid}-tab-${m}`;
    const panel = (m: Mode) => `${uid}-panel-${m}`;
    const counterId = `${uid}-counter`;

    const onTabKeyDown = (e: KeyboardEvent) => {
        if (e.key === 'ArrowLeft' || e.key === 'ArrowRight') {
            e.preventDefault();
            const next: Mode = mode === 'write' ? 'preview' : 'write';
            setMode(next);
            document.getElementById(tab(next))?.focus();
        }
    };

    const tabButton = (m: Mode, label: string) => (
        <button
            type="button"
            role="tab"
            id={tab(m)}
            aria-selected={mode === m}
            aria-controls={panel(m)}
            tabIndex={mode === m ? 0 : -1}
            className={mode === m ? 'active' : undefined}
            onClick={() => setMode(m)}
            onKeyDown={onTabKeyDown}
        >
            {label}
        </button>
    );

    return (
        <div className="md-editor">
            <div
                className="preview-toggle"
                role="tablist"
                aria-label={t('markdown.tabs')}
            >
                {tabButton('write', t('markdown.write'))}
                {tabButton('preview', t('markdown.preview'))}
            </div>
            <div
                role="tabpanel"
                id={panel('write')}
                aria-labelledby={tab('write')}
                hidden={mode !== 'write'}
            >
                <textarea
                    id={id}
                    name={name}
                    className="md-editor-input"
                    value={value}
                    placeholder={placeholder}
                    maxLength={maxLength}
                    disabled={disabled}
                    aria-invalid={invalid ? true : undefined}
                    aria-describedby={[describedBy, counterId]
                        .filter(Boolean)
                        .join(' ')}
                    onChange={(e) => onChange(e.target.value)}
                    onBlur={onBlur}
                />
            </div>
            <div
                role="tabpanel"
                id={panel('preview')}
                aria-labelledby={tab('preview')}
                hidden={mode !== 'preview'}
                className="preview-box"
            >
                {value.trim() ? (
                    <Markdown>{value}</Markdown>
                ) : (
                    <p className="hint">{t('markdown.previewEmpty')}</p>
                )}
            </div>
            <div
                className={
                    value.length >= maxLength
                        ? 'md-editor-counter md-editor-counter-full'
                        : 'md-editor-counter'
                }
                id={counterId}
            >
                {t('markdown.counter', { count: value.length, max: maxLength })}
            </div>
        </div>
    );
}
