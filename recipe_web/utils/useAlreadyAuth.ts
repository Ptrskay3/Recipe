import { useRouter } from 'next/router';
import { useEffect } from 'react';

// TODO: this is dumb
export const useAlreadyAuth = () => {
  const router = useRouter();

  useEffect(() => {
    (async () => {
      const { ok } = await fetch('http://localhost:3000/me', { credentials: 'include' });
      if (ok) {
        router.replace(`/`);
      }
    })();
  }, [router]);
};
