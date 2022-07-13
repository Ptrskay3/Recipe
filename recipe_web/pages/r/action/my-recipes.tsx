import {
  Box,
  Center,
  CircularProgress,
  Flex,
  Heading,
  ListItem,
  Text,
  UnorderedList,
} from '@chakra-ui/react';
import useSWR from 'swr';
import { Layout } from '../../../components/layout';
import { fetcher } from '../../../utils/fetcher';
import { useAuth } from '../../../utils/useAuth';

interface IRecipe {
  name: string;
  description: string;
  ingredient_count: number;
}

export default function MyRecipes() {
  useAuth();
  const { data, error } = useSWR(`http://localhost:3000/r/action/my-recipes`, fetcher);

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
            <UnorderedList>
              {data.map((recipe: IRecipe) => (
                <ListItem key={recipe.name}>
                  <Flex as="a" href={`/r/${recipe.name}`} _hover={{ color: 'orange.400' }}>
                    <Heading>{recipe.name}</Heading>
                    <Text m={4}>{recipe.description}</Text>
                    <Text m={4}>{'ingredients: ' + recipe.ingredient_count}</Text>
                  </Flex>
                </ListItem>
              ))}
            </UnorderedList>
          </Center>
        </Box>
      </Layout>
    )
  );
}
