import {
  Center,
  CircularProgress,
  Heading,
  ListItem,
  Stack,
  Text,
  UnorderedList,
} from '@chakra-ui/react';
import { useRouter } from 'next/router';
import useSWR from 'swr';
import Ingredient from '../../components/ingredient';
import { Layout } from '../../components/layout';
import { editableMapping } from '../../utils/constants';
import { fetcher } from '../../utils/fetcher';

export default function IngredientDetailed() {
  const router = useRouter();
  const { name } = router.query;

  const { data, error } = useSWR(
    !!name ? `${process.env.NEXT_PUBLIC_BASE_URL}/i/${name}` : null,
    fetcher
  );
  const { data: suggestions, error: _ } = useSWR(
    !!name ? `${process.env.NEXT_PUBLIC_BASE_URL}/i/${name}/suggestions` : null,
    fetcher
  );

  if (error)
    return (
      <Layout>
        <Center mt="4">
          <Text color="orange.400">{'failed to load'}</Text>
        </Center>
      </Layout>
    );

  if (!data)
    return (
      <Layout>
        {' '}
        <Center mt="4">
          <CircularProgress isIndeterminate color="orange.400" />
        </Center>
      </Layout>
    );

  return (
    data && (
      <Layout>
        <Center mt="4">
          <Stack>
            <Ingredient iProps={data} shouldShowEditControls editableMapping={editableMapping} />
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
