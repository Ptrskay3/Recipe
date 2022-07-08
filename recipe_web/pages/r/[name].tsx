import { useRouter } from 'next/router';
import { Layout } from '../../components/layout';
import { Box, Center, CircularProgress, Flex, Heading, Stack, Text } from '@chakra-ui/react';
import useSWR from 'swr';
import { fetcher } from '../../utils/fetcher';
import IncludedIngredient from '../../components/included_ingredient';

export default function RecipeDetailed() {
  const router = useRouter();
  const { name } = router.query;

  const { data, error } = useSWR(!!name ? `http://localhost:3000/r/${name}` : null, fetcher);

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
          <Flex>
            <Heading>{data.name}</Heading>
            <Text m={4}>{data.description}</Text>
            {data.ingredients.map((i: any) => (
              <IncludedIngredient key={i.name} {...i} />
            ))}
          </Flex>
        </Center>
      </Layout>
    )
  );
}
