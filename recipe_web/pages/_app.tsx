import '../styles/globals.css';
import type { AppProps } from 'next/app';
import { ChakraProvider } from '@chakra-ui/react';

function MyApp({ Component, pageProps }: AppProps) {
  return (
    <ChakraProvider>
      <div>
        <Component {...pageProps} />
      </div>
    </ChakraProvider>
  );
}

export default MyApp;
