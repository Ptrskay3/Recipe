import { useRouter } from 'next/router';
import { Layout } from '../../components/layout';
import { Center, CircularProgress, Text } from '@chakra-ui/react';
import useSWR from 'swr';
import { fetcher } from '../../utils/fetcher';
import dynamic from 'next/dynamic';
import Ingredient from '../../components/ingredient';

function IngredientDetailed() {
  const router = useRouter();
  const { name } = router.query;

  const { data, error } = useSWR(`http://localhost:3000/i/${name}`, fetcher);

  if (error)
    return (
      <Layout>
        <Center mt="14">
          <Text color="orange.400">{'failed to load'}</Text>
        </Center>
      </Layout>
    );

  if (!data)
    return (
      <Layout>
        {' '}
        <Center mt="14">
          <CircularProgress isIndeterminate color="orange.400" />
        </Center>
      </Layout>
    );

  return (
    data && (
      <Layout>
        <Center mt="14">
          <Ingredient {...data} />
        </Center>
      </Layout>
    )
  );
}
export default dynamic(() => Promise.resolve(IngredientDetailed), { ssr: false });
