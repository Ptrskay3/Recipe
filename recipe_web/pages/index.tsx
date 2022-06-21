import { Divider } from '@chakra-ui/react';
import type { NextPage } from 'next';
import { Footer } from '../components/footer';
import WithSubnavigation from '../components/navbar';
import ArticleList from '../components/placeholder';

const Home: NextPage = () => {
  return (
    <div>
      <WithSubnavigation />
      {/* TODO: Placeholder */}
      {/* <ArticleList /> */}

      <Footer />
    </div>
  );
};

export default Home;
