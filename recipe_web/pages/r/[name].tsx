import { PlusSquareIcon } from '@chakra-ui/icons';
import {
  Box,
  Button,
  Center,
  CircularProgress,
  Flex,
  Heading,
  IconButton,
  Text,
} from '@chakra-ui/react';
import { useRouter } from 'next/router';
import { FaHeart, FaHeartBroken } from 'react-icons/fa';
import useSWR, { mutate, useSWRConfig } from 'swr';
import { AddIngredientForm } from '../../components/add_ingredient_form';
import IncludedIngredient from '../../components/included_ingredient';
import { Layout } from '../../components/layout';
import { useMe } from '../../hooks/me';
import { useAddIngredient } from '../../stores/useAddIngredient';
import { fetcher } from '../../utils/fetcher';

export default function RecipeDetailed() {
  const { me } = useMe();
  const { mutate } = useSWRConfig();
  const router = useRouter();
  const { name } = router.query;
  const addIngredientOpen = useAddIngredient((state) => state.addIngredientOpen);
  const setAddIngredientOpen = useAddIngredient((state) => state.setAddIngredientOpen);

  const { data, error } = useSWR(!!name ? `http://localhost:3000/r/${name}` : null, fetcher);
  const toggleFavorite = async () => {
    const { ok } = await fetch(`http://localhost:3000/r/${name}/favorite`, {
      method: 'POST',
      credentials: 'include',
    });
    if (ok) {
      // Let's just refetch on favorite change, we can always optimize later
      // if this proves to be a slow logic.
      mutate(`http://localhost:3000/r/${name}`);
    }
  };

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
              <Box>
                {me ? (
                  <IconButton
                    aria-label="toggle favorite"
                    size="md"
                    icon={data.favorited ? <FaHeart color="red" /> : <FaHeartBroken />}
                    onClick={toggleFavorite}
                  />
                ) : null}
              </Box>
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
