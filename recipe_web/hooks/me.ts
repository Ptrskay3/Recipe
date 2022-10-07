import useSWR from 'swr';
import { fetcher } from '../utils/fetcher';

export function useMe() {
  const { data, error } = useSWR(`${process.env.NEXT_PUBLIC_BASE_URL}/me`, fetcher);

  return {
    me: data,
    isLoading: !error && !data,
    isError: error,
  };
}
