import { useRouter } from 'next/router';
import { Layout } from '../../components/layout';
import {
  Center,
  CircularProgress,
  Heading,
  ListItem,
  Stack,
  Text,
  UnorderedList,
} from '@chakra-ui/react';
import useSWR from 'swr';
import { fetcher } from '../../utils/fetcher';
import Ingredient from '../../components/ingredient';
import { ArrowRightIcon } from '@chakra-ui/icons';

export default function IngredientDetailed() {
  const router = useRouter();
  const { name } = router.query;

  const { data, error } = useSWR(!!name ? `http://localhost:3000/i/${name}` : null, fetcher);
  const { data: suggestions, error: _ } = useSWR(
    !!name ? `http://localhost:3000/i/${name}/suggestions` : null,
    fetcher
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
    data &&
    suggestions && (
      <Layout>
        <Center mt="14">
          <Stack>
            <Ingredient {...data} />
            <Heading>Suggestions:</Heading>
            {suggestions && suggestions.length > 0 && (
              <>
                <Stack>
                  {suggestions.map(({ id, suggester, is_delete_vote, ...suggestion }: any) => (
                    <UnorderedList key={id}>
                      <ListItem as={'a'} href={`/i/${name}/suggestion/${id}`}>
                        {'Suggestion by ' + suggester}
                      </ListItem>
                    </UnorderedList>
                  ))}
                </Stack>
              </>
            )}
          </Stack>
        </Center>
      </Layout>
    )
  );
}
