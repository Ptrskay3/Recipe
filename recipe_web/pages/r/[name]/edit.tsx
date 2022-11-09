import { CloseIcon } from '@chakra-ui/icons';
import {
  Box,
  Center,
  CircularProgress,
  Flex,
  Heading,
  IconButton,
  Text,
  VStack,
  Wrap,
  WrapItem,
} from '@chakra-ui/react';
import { useRouter } from 'next/router';
import { FaHeart, FaHeartBroken } from 'react-icons/fa';
import useSWR, { useSWRConfig } from 'swr';
import { AddIngredientForm } from '../../../components/add_ingredient_form';
import IncludedIngredient from '../../../components/included_ingredient';
import { Layout } from '../../../components/layout';
import { useMe } from '../../../hooks/me';
import { fetcher } from '../../../utils/fetcher';
import NextLink from 'next/link';
import { useEffect } from 'react';

export default function RecipeDetailedEdit() {
  const { me } = useMe();
  const { mutate } = useSWRConfig();
  const router = useRouter();
  const { name } = router.query;

  const { data, error } = useSWR(
    !!name ? `${process.env.NEXT_PUBLIC_BASE_URL}/r/${name}` : null,
    fetcher
  );
  const toggleFavorite = async () => {
    const { ok } = await fetch(`${process.env.NEXT_PUBLIC_BASE_URL}/r/${name}/favorite`, {
      method: 'POST',
      credentials: 'include',
    });
    if (ok) {
      // Let's just refetch on favorite change, we can always optimize later
      // if this proves to be a slow logic.
      mutate(`${process.env.NEXT_PUBLIC_BASE_URL}/r/${name}`);
    }
  };
  useEffect(() => {
    if (name && data && !data.is_author && data !== true) {
      router.push(`/r/${name}`);
    }
  }, [data, name, router]);

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
    <Layout>
      <Box>
        <Center mt="14">
          <Flex>
            <VStack>
              <Heading>{data.name}</Heading>
              <Text fontSize={'sm'} m={4}>
                {data.description}
              </Text>
              <Text m={4} fontSize="xl">{`⏱️ Preparation time: ${data.prep_time} minues`}</Text>
              <Text m={4} fontSize="xl">{`🧑‍🍳 Cook time: ${data.cook_time} minues`}</Text>
              <Text m={4} fontSize="xl">{`📈 Difficulty: ${data.difficulty}`}</Text>
              <Text m={4} fontSize="xl">{`🏳️ Cuisine: ${data.cuisine}`}</Text>
              <Text m={4} fontSize="xl">{`🍳 Type: ${data.meal_type}`}</Text>
              <Text m={4} fontSize="xl">{`⚖️ Calories: ${data.full_calories}`}</Text>
              {data.steps && data.steps.length > 0 ? (
                <>
                  <Heading>Steps:</Heading>
                  {data.steps.map((step: any, i: number) => (
                    <Text key={i} m={4}>{`${i + 1}. step: ${step}`}</Text>
                  ))}
                </>
              ) : null}

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
              <Wrap>
                {data.ingredients &&
                  data.ingredients.map((i: any) => (
                    <WrapItem key={i.name}>
                      <IncludedIngredient {...i} />
                    </WrapItem>
                  ))}
              </Wrap>
            </VStack>
            <NextLink passHref href={`/r/${name}`}>
              <IconButton
                aria-label="edit recipe"
                size="md"
                icon={<CloseIcon></CloseIcon>}
                as={'a'}
              />
            </NextLink>
          </Flex>
        </Center>
        <Center>
          <AddIngredientForm />
        </Center>
      </Box>
    </Layout>
  );
}
