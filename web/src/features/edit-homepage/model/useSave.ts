import { putHomepage, putTerms } from '@/entities/cms-page';
import { queryKeys, useApiMutation } from '@/shared/api';
import type { HomepageContent, TermsContent } from '@/entities/cms-page';

export function useSaveHomepage() {
    return useApiMutation({
        mutationFn: (content: HomepageContent) => putHomepage(content),
        invalidates: [queryKeys.cms.homepage()],
    });
}

export function useSaveTerms() {
    return useApiMutation({
        mutationFn: (content: TermsContent) => putTerms(content),
        invalidates: [queryKeys.cms.terms()],
    });
}
