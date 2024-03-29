import type { NextPage } from 'next';
import { Footer } from '../components/footer';
import WithSubnavigation from '../components/navbar';
import PlaceHolder from '../components/placeholder';

import dynamic from 'next/dynamic';

const Home: NextPage = () => {
  return (
    <>
      <WithSubnavigation />
      <PlaceHolder />
      <Footer />
    </>
  );
};

export default dynamic(() => Promise.resolve(Home), { ssr: false });
