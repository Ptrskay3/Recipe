import { ChakraProvider, useToast } from '@chakra-ui/react';
import type { AppProps } from 'next/app';
import Head from 'next/head';
import { useEffect, useState } from 'react';
import theme from '../common/theme';
import '../styles/globals.css';
import { prettyNotification } from '../utils/pretty';

function MyApp({ Component, pageProps }: AppProps) {
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  const toast = useToast();
  useEffect(() => {
    const sse = new EventSource(`${process.env.NEXT_PUBLIC_BASE_URL}/sse`, {
      withCredentials: true,
    });
    sse.onerror = () => {
      sse.close();
    };

    sse.onmessage = (e) => {
      const { t, name } = JSON.parse(e.data);
      toast({
        title: prettyNotification(t),
        description: `someone just added a recipe called "${name}"`,
        status: 'info',
        duration: 9000,
        isClosable: true,
        position: 'bottom-right',
      });
    };
    return () => {
      sse.close();
    };
  }, [toast]);

  const body = (
    <>
      <Head>
        <title>Recipes</title>
        <meta name="description" content="A recipe app made with Rust and Next.js"></meta>
      </Head>
      <ChakraProvider theme={theme}>
        <Component {...pageProps} />
      </ChakraProvider>
    </>
  );

  // prevents ssr flash for mismatched dark mode
  if (!mounted) {
    return <div style={{ visibility: 'hidden' }}>{body}</div>;
  }
  return body;
}

export default MyApp;
