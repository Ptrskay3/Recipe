import { CloseIcon } from '@chakra-ui/icons';
import {
  Box,
  Center,
  CircularProgress,
  Flex,
  Heading,
  VStack,
  Text,
  Link,
  Stack,
} from '@chakra-ui/react';
import { useRouter } from 'next/router';
import { useEffect, useState } from 'react';
import { useSWRConfig } from 'swr';
import { Layout } from '../../components/layout';
import NextLink from 'next/link';

export default function DiscordAuthorize() {
  const router = useRouter();
  const { mutate } = useSWRConfig();
  const { code, state, error: discordError } = router.query;
  const [error, setError] = useState<{ isError: boolean; message?: string }>({ isError: false });
  useEffect(() => {
    if (discordError) {
      setError({ isError: true });
      return;
    }
    if (!state || !code) return;
    fetch(
      `${process.env.NEXT_PUBLIC_BASE_URL}/auth/discord_authorize?code=${code}&state=${state}`,
      {
        credentials: 'include',
      }
    ).then((r) => {
      if (r.ok) {
        mutate(`${process.env.NEXT_PUBLIC_BASE_URL}/me`);
        router.push('/');
      } else if (r.status === 422) {
        setError({
          isError: true,
          message: 'User with this email already exists as a regular non-OAuth user.',
        });
      }
    });
  }, [router, code, state, mutate, discordError]);

  if (error.isError) {
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
          <Stack>
            <Heading as="h2" size="xl" mt={6} mb={2}>
              Something went wrong. <br /> Details might be shown below.
            </Heading>
            {error.message && (
              <>
                <Text fontSize="md">{error.message}</Text>
                <NextLink href="/login">
                  <Link mt={4} color={'orange.400'}>
                    Login with that email instead?
                  </Link>
                </NextLink>
              </>
            )}
          </Stack>
        </Center>
      </Layout>
    );
  }

  return (
    <Layout>
      <Center mt="14">
        <VStack>
          <Heading>{'Waiting for Discord to authenticate..'}</Heading>
          <CircularProgress isIndeterminate color="orange.400" />
        </VStack>
      </Center>
    </Layout>
  );
}
