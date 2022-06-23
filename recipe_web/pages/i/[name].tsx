import { useEffect, useState } from 'react';
import { useRouter } from 'next/router';
import { Layout } from '../../components/layout';
import { Center, Text } from '@chakra-ui/react';
import useSWR from 'swr';
import { fetcher } from '../../utils/fetcher';
import dynamic from 'next/dynamic';

function IngredientDetailed() {
  const router = useRouter();
  const { name } = router.query;

  const { data, error } = useSWR(`http://localhost:3000/i/${name}`, fetcher);

  if (error)
    return (
      <Layout>
        <Center>{'failed to load'}</Center>
      </Layout>
    );

  if (!data)
    return (
      <Layout>
        {' '}
        <Center>{'loading...'} </Center>
      </Layout>
    );

  return (
    data && (
      <Layout>
        <Center mt="14">
          <Text color="orange.400">{JSON.stringify(data)}</Text>
        </Center>
      </Layout>
    )
  );
}
export default dynamic(() => Promise.resolve(IngredientDetailed), { ssr: false });
