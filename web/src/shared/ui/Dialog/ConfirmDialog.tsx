import { Dialog } from './Dialog';

type ConfirmDialogProps = {
    title: string;
    text: string;
    confirmLabel: string;
    cancelLabel: string;
    onConfirm: () => void;
    onCancel: () => void;
    /** Disables the buttons while the action runs. */
    pending?: boolean;
    /** Already translated error text, shown inside the dialog. */
    error?: string;
};

/** Confirmation for destructive actions; the confirm button is styled as dangerous. */
export function ConfirmDialog({
    title,
    text,
    confirmLabel,
    cancelLabel,
    onConfirm,
    onCancel,
    pending,
    error,
}: ConfirmDialogProps) {
    return (
        <Dialog title={title} onClose={onCancel}>
            <p>{text}</p>
            {error && (
                <p className="field-error" role="alert">
                    {error}
                </p>
            )}
            <div className="dialog-actions">
                <button
                    type="button"
                    className="btn btn-danger"
                    onClick={onConfirm}
                    disabled={pending}
                >
                    {confirmLabel}
                </button>
                <button
                    type="button"
                    className="btn secondary"
                    data-autofocus
                    onClick={onCancel}
                    disabled={pending}
                >
                    {cancelLabel}
                </button>
            </div>
        </Dialog>
    );
}
