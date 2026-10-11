import { useQuery } from '@tanstack/react-query';
import { queryKeys } from '@/shared/api';
import { getHomepage, getTerms } from '../api/cms';

export function useHomepage() {
    return useQuery({
        queryKey: queryKeys.cms.homepage(),
        queryFn: getHomepage,
    });
}

export function useTerms() {
    return useQuery({ queryKey: queryKeys.cms.terms(), queryFn: getTerms });
}
