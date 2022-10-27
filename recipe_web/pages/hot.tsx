import {
  Box,
  Center,
  CircularProgress,
  Flex,
  Heading,
  ListItem,
  Text,
  UnorderedList,
  VStack,
} from '@chakra-ui/react';
import useSWR from 'swr';
import { Layout } from '../components/layout';
import { fetcher } from '../utils/fetcher';

export default function HotRecipes() {
  const { data, error } = useSWR(`${process.env.NEXT_PUBLIC_BASE_URL}/r/action/hot`, fetcher);

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
        <Box>
          <Center mt="14">
            <VStack>
              <Heading>{'Most popular recipes (last two weeks)'}</Heading>
              <UnorderedList>
                {data.map((recipe: any) => (
                  <ListItem key={recipe.name}>
                    <Flex as="a" href={`/r/${recipe.name}`} _hover={{ color: 'orange.400' }}>
                      <Heading>{recipe.name}</Heading>
                      <Text m={4}>{'likes: ' + recipe.count}</Text>
                    </Flex>
                  </ListItem>
                ))}
              </UnorderedList>
            </VStack>
          </Center>
        </Box>
      </Layout>
    )
  );
}
