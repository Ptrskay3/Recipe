import { CloseIcon } from '@chakra-ui/icons';
import { Box, Center, CircularProgress, Flex, Heading, VStack } from '@chakra-ui/react';
import { useRouter } from 'next/router';
import { useEffect, useState } from 'react';
import { useSWRConfig } from 'swr';
import { Layout } from '../../components/layout';

export default function GoogleAuthorize() {
  const router = useRouter();
  const { mutate } = useSWRConfig();
  const { code, state, error: googleError } = router.query;
  const [error, setError] = useState<boolean>(false);
  useEffect(() => {
    if (googleError) {
      setError(true);
      return;
    }
    if (!state || !code) return;
    fetch(`http://localhost:3000/auth/google_authorize?code=${code}&state=${state}`, {
      credentials: 'include',
    })
      .then((r) => r.ok)
      .then((ok) => {
        if (ok) {
          mutate(`http://localhost:3000/me`);
          router.push('/');
        } else {
          setError(true);
        }
      });
  }, [router, code, state, mutate, googleError]);

  if (error) {
    return (
      <Layout>
        <Center mt="14">
          <Box display="inline-block">
            <Flex
              flexDirection="column"
              justifyContent="center"
              alignItems="center"
              bg={'red.500'}
              rounded={'50px'}
              w={'55px'}
              h={'55px'}
              textAlign="center"
            >
              <CloseIcon boxSize={'20px'} color={'white'} />
            </Flex>
          </Box>
          <Heading as="h2" size="xl" mt={6} mb={2}>
            Something went wrong.
          </Heading>
        </Center>
      </Layout>
    );
  }

  return (
    <Layout>
      <Center mt="14">
        <VStack>
          <Heading>{'Waiting for Google to authenticate..'}</Heading>
          <CircularProgress isIndeterminate color="orange.400" />
        </VStack>
      </Center>
    </Layout>
  );
}
