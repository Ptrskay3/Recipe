import { useRouter } from 'next/router';
import { useEffect } from 'react';

export const useAuth = () => {
  const router = useRouter();

  useEffect(() => {
    fetch(`${process.env.NEXT_PUBLIC_BASE_URL}/me`, { credentials: 'include' })
      .then((r) => r.json())
      .then((data) => {
        if (!data?.name) {
          router.replace('/');
        }
      });
  }, [router]);
};
