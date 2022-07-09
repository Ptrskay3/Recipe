import { Center } from '@chakra-ui/react';
import dynamic from 'next/dynamic';
import { Layout } from '../../components/layout';
import { useAuth } from '../../utils/useAuth';

const NewRecipe = () => {
  useAuth();
  return (
    <Layout>
      <Center>{'hi there'}</Center>
    </Layout>
  );
};

export default dynamic(() => Promise.resolve(NewRecipe), { ssr: false });
