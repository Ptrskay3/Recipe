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
    data && (
      <Layout>
        <Center mt="14">
          <Stack>
            <Ingredient {...data} />
            {suggestions && suggestions.length > 0 && (
              <>
                <Heading>Suggestions:</Heading>
                <Stack>
                  <UnorderedList>
                    {suggestions.map(({ id, suggester }: { id: number; suggester: string }) => (
                      <Stack mb="2" key={id}>
                        <ListItem as={'a'} href={`/i/${name}/suggestion/${id}`}>
                          {'Suggestion by ' + suggester}
                        </ListItem>
                      </Stack>
                    ))}
                  </UnorderedList>
                </Stack>
              </>
            )}
          </Stack>
        </Center>
      </Layout>
    )
  );
}
