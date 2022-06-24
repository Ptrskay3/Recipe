import useSWR from 'swr';
import { fetcher } from '../utils/fetcher';

export function useMe() {
  const { data, error } = useSWR(`http://localhost:3000/me`, fetcher);

  return {
    me: data,
    isLoading: !error && !data,
    isError: error,
  };
}
