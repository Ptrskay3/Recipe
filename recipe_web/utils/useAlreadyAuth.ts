import { useRouter } from 'next/router';
import { useEffect } from 'react';

// TODO: this is dumb
export const useAlreadyAuth = () => {
  const router = useRouter();

  useEffect(() => {
    fetch('http://localhost:3000/me', { credentials: 'include' })
      .then((r) => r.json())
      .then((data) => {
        if (data?.name) {
          router.replace('/');
        }
      });
  }, [router]);
};
