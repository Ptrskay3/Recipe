import { useRouter } from 'next/router';
import { Layout } from '../../../../components/layout';
import { Center, CircularProgress, Heading, IconButton, Stack, Text } from '@chakra-ui/react';
import useSWR from 'swr';
import { fetcher } from '../../../../utils/fetcher';
import Ingredient from '../../../../components/ingredient';
import { ArrowRightIcon } from '@chakra-ui/icons';
import { diffObjects } from '../../../../utils/diff';
import { useState } from 'react';
import { FaCheck } from 'react-icons/fa';

export default function IngredientDetailed() {
  const router = useRouter();
  const { name, id } = router.query;

  const { data, error } = useSWR(!!name ? `http://localhost:3000/i/${name}` : null, fetcher);
  const { data: suggestion, error: suggestionError } = useSWR(
    !!name && !!id ? `http://localhost:3000/i/${name}/suggestion/${id}` : null,
    fetcher
  );

  const applySuggestion = async () => {
    const { ok } = await fetch(`http://localhost:3000/i/${name}/suggestion/${id}/apply`);
    if (ok) {
      router.push(`/i/${name}`);
    }
  };

  if (error || suggestionError)
    return (
      <Layout>
        <Center mt="14">
          <Text color="orange.400">{'This suggestion does not exist.'}</Text>
        </Center>
      </Layout>
    );

  if (!data || !suggestion)
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
    suggestion && (
      <Layout>
        <Center mt="14">
          <Ingredient
            {...data}
            isNew={false}
            withModifiedAttributes={diffObjects(data, suggestion)}
          />
          <>
            <Heading m="4">
              <ArrowRightIcon></ArrowRightIcon>
            </Heading>
            <Stack>
              <Ingredient
                key={id}
                withModifiedAttributes={diffObjects(data, suggestion)}
                isNew={true}
                isDeleteVote={suggestion.is_delete_vote}
                {...suggestion}
              />
            </Stack>
            <IconButton
              onClick={applySuggestion}
              aria-label="apply"
              icon={<FaCheck />}
            ></IconButton>
          </>
        </Center>
      </Layout>
    )
  );
}
