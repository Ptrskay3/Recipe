import { PlusSquareIcon } from '@chakra-ui/icons';
import { Box, Center, CircularProgress, Flex, Heading, IconButton, Text } from '@chakra-ui/react';
import { useRouter } from 'next/router';
import useSWR from 'swr';
import { AddIngredientForm } from '../../components/add_ingredient_form';
import IncludedIngredient from '../../components/included_ingredient';
import { Layout } from '../../components/layout';
import { useAddIngredient } from '../../stores/useAddIngredient';
import { fetcher } from '../../utils/fetcher';

export default function RecipeDetailed() {
  const router = useRouter();
  const { name } = router.query;
  const addIngredientOpen = useAddIngredient((state) => state.addIngredientOpen);
  const setAddIngredientOpen = useAddIngredient((state) => state.setAddIngredientOpen);

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
        <Box>
          <Center mt="14">
            <Flex>
              <Heading>{data.name}</Heading>
              <Text m={4}>{data.description}</Text>
              {data.ingredients &&
                data.ingredients.map((i: any) => <IncludedIngredient key={i.name} {...i} />)}
            </Flex>
          </Center>
          <Center>
            {addIngredientOpen ? (
              <AddIngredientForm />
            ) : (
              <IconButton
                aria-label="delete ingredient"
                size="md"
                icon={<PlusSquareIcon />}
                onClick={() => setAddIngredientOpen(true)}
              />
            )}
          </Center>
        </Box>
      </Layout>
    )
  );
}
