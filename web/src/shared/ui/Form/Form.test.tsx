import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { zodResolver } from '@hookform/resolvers/zod';
import { useForm } from 'react-hook-form';
import { expect, test } from 'vitest';
import { z } from 'zod';
import { i18n } from '@/shared/i18n';
import { applyProblemToForm, type FormProblem } from '@/shared/lib';
import { Field } from './Field';
import { FormErrorSummary } from './FormErrorSummary';

const schema = z.object({
    username: z.string().min(1, 'errors.required'),
    email: z.string().email('errors.invalid_email'),
});
type Values = z.infer<typeof schema>;

function TestForm({ problem }: { problem?: FormProblem }) {
    const {
        register,
        handleSubmit,
        setError,
        formState: { errors },
    } = useForm<Values>({ resolver: zodResolver(schema) });
    return (
        <form
            noValidate
            onSubmit={handleSubmit(() => {
                if (problem)
                    applyProblemToForm(problem, setError, [
                        'username',
                        'email',
                    ]);
            })}
        >
            <FormErrorSummary errors={errors} />
            <Field
                id="username"
                label="Username"
                error={errors.username}
                hint="Hint"
            >
                {(c) => <input {...c} {...register('username')} />}
            </Field>
            <Field id="email" label="Email" error={errors.email}>
                {(c) => <input {...c} {...register('email')} />}
            </Field>
            <button type="submit">Send</button>
        </form>
    );
}

test('shows validation errors with aria wiring and a summary', async () => {
    render(<TestForm />);
    await userEvent.click(screen.getByRole('button', { name: 'Send' }));
    const input = await screen.findByLabelText('Username');
    expect(input).toHaveAttribute('aria-invalid', 'true');
    const describedBy = input.getAttribute('aria-describedby') ?? '';
    expect(describedBy).toContain('username-error');
    expect(describedBy).toContain('username-hint');
    expect(document.getElementById('username-error')).toHaveTextContent(
        'This field is required.',
    );
    const alert = screen.getByRole('alert');
    expect(alert).toHaveTextContent('2 errors need your attention');
    await userEvent.click(
        screen.getByRole('link', { name: 'Enter a valid email address.' }),
    );
    expect(screen.getByLabelText('Email')).toHaveFocus();
});

test('valid fields have no aria-invalid and no summary', () => {
    render(<TestForm />);
    expect(screen.getByLabelText('Username')).not.toHaveAttribute(
        'aria-invalid',
    );
    expect(screen.queryByRole('alert')).toBeNull();
});

test('maps API problem errors onto fields using errors.<code> keys', async () => {
    render(
        <TestForm
            problem={{
                code: 'validation_failed',
                errors: [
                    { field: 'username', code: 'username_taken', message: 'x' },
                    { field: 'nope', code: 'forbidden', message: 'y' },
                ],
            }}
        />,
    );
    await userEvent.type(screen.getByLabelText('Username'), 'bob');
    await userEvent.type(screen.getByLabelText('Email'), 'bob@example.com');
    await userEvent.click(screen.getByRole('button', { name: 'Send' }));
    await waitFor(() =>
        expect(screen.getByLabelText('Username')).toHaveAttribute(
            'aria-invalid',
            'true',
        ),
    );
    expect(document.getElementById('username-error')).toHaveTextContent(
        'This username is already taken.',
    );
    expect(screen.getByRole('alert')).toHaveTextContent(
        'You are not allowed to do that.',
    );
});

test('German error text', async () => {
    await i18n.changeLanguage('de');
    render(<TestForm />);
    await userEvent.click(screen.getByRole('button', { name: 'Send' }));
    expect(await screen.findByRole('alert')).toHaveTextContent(
        '2 Fehler brauchen deine Aufmerksamkeit',
    );
    await i18n.changeLanguage('en-US');
});
