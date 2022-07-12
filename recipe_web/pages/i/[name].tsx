import { useRouter } from 'next/router';
import { Layout } from '../../components/layout';
import { Center, CircularProgress, Heading, Stack, Text } from '@chakra-ui/react';
import useSWR from 'swr';
import { fetcher } from '../../utils/fetcher';
import Ingredient from '../../components/ingredient';
import { ArrowRightIcon } from '@chakra-ui/icons';

export default function IngredientDetailed() {
  const router = useRouter();
  const { name } = router.query;

  const { data, error } = useSWR(!!name ? `http://localhost:3000/i/${name}` : null, fetcher);
  // TODO: move out the suggestion part to another route, leave this as a barebone ingredient
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
          <Ingredient
            {...data}
            isNew={false}
            withModifiedAttributes={diffService(data, suggestions[0])}
          />
          {suggestions && suggestions.length > 0 && (
            <>
              <Heading m="4">
                <ArrowRightIcon></ArrowRightIcon>
              </Heading>
              <Stack>
                {suggestions.map(({ id, suggester, is_delete_vote, ...suggestion }: any) => (
                  <Ingredient
                    key={suggester}
                    withModifiedAttributes={diffService(data, suggestion)}
                    isNew={true}
                    isDeleteVote={is_delete_vote}
                    {...suggestion}
                  />
                ))}
              </Stack>
            </>
          )}
        </Center>
      </Layout>
    )
  );
}

// TODO: Maybe this is not even frontend stuff.. idk
const diffService = (original: Record<string, any>, suggested: Record<string, any>): any[] => {
  if (!original || !suggested) {
    return [];
  }
  const originalKeys = Object.keys(original);
  return Object.entries(suggested)
    .map(([key, value]) => {
      if (originalKeys.includes(key)) {
        if (Array.isArray(value) && !arraysEqual(original[key], value)) {
          return key;
        } else if (!Array.isArray(value) && original[key] !== value) {
          return key;
        }
      }
    })
    .filter(Boolean);
};

function arraysEqual(a: any[], b: any[]): boolean {
  a = Array.isArray(a) ? a : [];
  b = Array.isArray(b) ? b : [];
  return a.length === b.length && a.every((el, ix) => el === b[ix]);
}
