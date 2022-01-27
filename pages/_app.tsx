import '../styles/globals.css'
import type { AppProps } from 'next/app'
import { AnimatePresence } from 'framer-motion';
import Head from 'next/head';

function MyApp({ Component, pageProps }: AppProps) {
  return (
    <AnimatePresence exitBeforeEnter >
      <Head key={'Head'}>
        <title>Gustavo Lee!</title>
      </Head>
      <Component {...pageProps} />
    </AnimatePresence>
  );
}

export default MyApp
