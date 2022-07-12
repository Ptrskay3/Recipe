import { useRouter } from 'next/router';
import { Layout } from '../../../../components/layout';
import {
  Center,
  CircularProgress,
  Heading,
  IconButton,
  Stack,
  Text,
  useToast,
} from '@chakra-ui/react';
import useSWR, { useSWRConfig } from 'swr';
import { fetcher } from '../../../../utils/fetcher';
import Ingredient from '../../../../components/ingredient';
import { ArrowRightIcon, CloseIcon } from '@chakra-ui/icons';
import { diffObjects } from '../../../../utils/diff';
import { FaCheck } from 'react-icons/fa';

export default function IngredientDetailed() {
  const router = useRouter();
  const { mutate } = useSWRConfig();
  const { name, id } = router.query;
  const toast = useToast();

  const { data, error } = useSWR(!!name ? `http://localhost:3000/i/${name}` : null, fetcher);
  const { data: suggestion, error: suggestionError } = useSWR(
    !!name && !!id ? `http://localhost:3000/i/${name}/suggestion/${id}` : null,
    fetcher
  );

  const suggestionAction = async (action: 'apply' | 'decline') => {
    const { ok, status } = await fetch(`http://localhost:3000/i/${name}/suggestion/${id}/${action}`);
    if (ok) {
      toast({
        title: `Action "${action}" was successful.`,
        status: 'success',
        duration: 9000,
        isClosable: true,
      });
      mutate(`i/${name}`);
      router.push(`/i/${name}`);
    } else {
      toast({
        title: `Something went wrong`,
        description:
          status === 409
            ? 'Cannot update name as it is already an existing ingredient.'
            : undefined,
        status: 'error',
        duration: 9000,
        isClosable: true,
      });
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
              onClick={() => suggestionAction('apply')}
              aria-label="apply"
              icon={<FaCheck />}
            ></IconButton>
            <IconButton
              onClick={() => suggestionAction('decline')}
              aria-label="decline"
              icon={<CloseIcon />}
            ></IconButton>
          </>
        </Center>
      </Layout>
    )
  );
}
