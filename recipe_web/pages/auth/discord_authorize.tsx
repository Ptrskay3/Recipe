import { useRouter } from 'next/router';
import { useEffect } from 'react';
import { Layout } from '../../components/layout';

export default function DiscordAuthorize() {
  const router = useRouter();
  const { code, state } = router.query;
  useEffect(() => {
    if (!state || !code) return;
    fetch(`http://localhost:3000/auth/discord_authorize?code=${code}&state=${state}`)
      .then((r) => r.ok)
      .then((ok) => {
        if (ok) {
          router.push('/');
        }
      });
  }, [router, code, state]);
  return <Layout>{'waiting for discord auth...'}</Layout>;
}
