import '@testing-library/jest-dom/vitest';
import { afterEach, expect, vi } from 'vitest';
import { setCsrfToken } from '@/shared/api';
import { i18n } from '@/shared/i18n';
import { takeUnhandled } from './mockApi';

afterEach(async () => {
    const unhandled = takeUnhandled();
    vi.unstubAllGlobals();
    setCsrfToken(undefined);
    await i18n.changeLanguage('en-US'); // remembers the language, so clear storage afterwards
    localStorage.clear();
    // A request without a registered mock route is a bug in the test (see test/mockApi.ts).
    expect(unhandled, 'requests without a mock route').toEqual([]);
});
