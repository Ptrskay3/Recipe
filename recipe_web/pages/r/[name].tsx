import { PlusSquareIcon } from '@chakra-ui/icons';
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
import { instantMeiliSearch } from '@meilisearch/instant-meilisearch';
import { useRouter } from 'next/router';
import { useRef } from 'react';
import { FaHeart, FaHeartBroken } from 'react-icons/fa';
import {
  Configure,
  Highlight,
  Hits,
  InstantSearch,
  SearchBox,
} from 'react-instantsearch-hooks-web';
import useSWR, { useSWRConfig } from 'swr';
import { AddIngredientForm } from '../../components/add_ingredient_form';
import IncludedIngredient from '../../components/included_ingredient';
import { Layout } from '../../components/layout';
import { CustomSearchBox } from '../../components/search/SearchBox';
import { useMe } from '../../hooks/me';
import { useAddIngredient } from '../../stores/useAddIngredient';
import { fetcher } from '../../utils/fetcher';

export default function RecipeDetailed() {
  const searchClient = instantMeiliSearch('http://localhost:7700');
  const { me } = useMe();
  const { mutate } = useSWRConfig();
  const router = useRouter();
  const { name } = router.query;
  const addIngredientOpen = useAddIngredient((state) => state.addIngredientOpen);
  const setAddIngredientOpen = useAddIngredient((state) => state.setAddIngredientOpen);
  const searchBoxInputRef = useRef(null);

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
  const Hit = ({ hit }: any) => (
    <Highlight
      attribute="name"
      hit={hit}
      key={hit.id}
      // @ts-ignore
      onClick={() => (searchBoxInputRef!.current.value = hit.name)}
    />
  );

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
              <VStack>
                <Heading>{data.name}</Heading>
                <Text fontSize={'sm'} m={4}>
                  {data.description}
                </Text>
                <Text m={4}>{`Preparation time: ${data.prep_time} minues`}</Text>
                <Text m={4}>{`Cook time: ${data.cook_time} minues`}</Text>
                <Text m={4}>{`Difficulty: ${data.difficulty}`}</Text>
                <Text m={4}>{`Cuisine: ${data.cuisine}`}</Text>
                <Text m={4}>{`Type: ${data.meal_type}`}</Text>
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
            </Flex>
          </Center>
          <Center>
            {addIngredientOpen ? (
              <>
                <InstantSearch indexName="ingredients" searchClient={searchClient}>
                  <VStack>
                    <Configure hitsPerPage={10} analytics={false} distinct />
                    <CustomSearchBox passRef={searchBoxInputRef} />
                    <Hits results={1} hitComponent={Hit} />
                  </VStack>
                </InstantSearch>
                <AddIngredientForm />
              </>
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
