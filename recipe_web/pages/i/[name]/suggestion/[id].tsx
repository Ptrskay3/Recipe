import { ArrowRightIcon, CloseIcon } from '@chakra-ui/icons';
import {
  Center,
  CircularProgress,
  Heading,
  HStack,
  IconButton,
  Text,
  useToast,
} from '@chakra-ui/react';
import { useRouter } from 'next/router';
import { FaCheck } from 'react-icons/fa';
import useSWR, { useSWRConfig } from 'swr';
import Ingredient from '../../../../components/ingredient';
import { Layout } from '../../../../components/layout';
import { useIngredientEditMode } from '../../../../stores/useIngredientEditMode';
import { editableMapping } from '../../../../utils/constants';
import { diffObjects } from '../../../../utils/diff';
import { fetcher } from '../../../../utils/fetcher';

export default function IngredientDetailed() {
  const setEditModeOpen = useIngredientEditMode((state) => state.setEditModeOpen);
  setEditModeOpen(false);
  const router = useRouter();
  const { mutate } = useSWRConfig();
  const { name, id } = router.query;
  const toast = useToast();

  const { data, error } = useSWR(
    !!name ? `${process.env.NEXT_PUBLIC_BASE_URL}/i/${name}` : null,
    fetcher
  );
  const { data: suggestion, error: suggestionError } = useSWR(
    !!name && !!id ? `${process.env.NEXT_PUBLIC_BASE_URL}/i/${name}/suggestion/${id}` : null,
    fetcher
  );

  const suggestionAction = async (action: 'apply' | 'decline') => {
    const { ok, status } = await fetch(
      `${process.env.NEXT_PUBLIC_BASE_URL}/i/${name}/suggestion/${id}/${action}`,
      { credentials: 'include' }
    );
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
        description: status === 409 ? 'Cannot update due to name conflicts.' : undefined,
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
            iProps={data}
            isNew={false}
            withModifiedAttributes={diffObjects(data, suggestion)}
            editableMapping={editableMapping}
          />
          <Heading m="4">
            <ArrowRightIcon></ArrowRightIcon>
          </Heading>
          <Ingredient
            key={id as string}
            withModifiedAttributes={diffObjects(data, suggestion)}
            isNew={true}
            isDeleteVote={suggestion.is_delete_vote}
            iProps={suggestion}
            editableMapping={editableMapping}
          />
          <HStack>
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
          </HStack>
        </Center>
      </Layout>
    )
  );
}
