import { useRouter } from 'next/router';
import { useEffect } from 'react';

export const useAuth = () => {
  const router = useRouter();

  useEffect(() => {
    (async () => {
      const { ok } = await fetch('http://localhost:3000/u/me', { credentials: 'include' });
      if (!ok) {
        router.replace(`/`);
      }
    })();
  }, [router]);
};
