import useSWR from 'swr';
import { fetcherOk } from '../utils/fetcher';

export function useValidToken(token: string | string[] | undefined) {
  const { data, error: tokenError } = useSWR(
    !!token ? `${process.env.NEXT_PUBLIC_BASE_URL}/is_token_valid?token=${token}` : null,
    fetcherOk
  );

  return {
    isValid: data,
    isLoading: typeof data === 'undefined' && !tokenError,
    tokenError,
  };
}
