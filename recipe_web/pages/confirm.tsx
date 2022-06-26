import { Box, Center, Heading, Link, Text } from '@chakra-ui/react';
import { CheckCircleIcon } from '@chakra-ui/icons';
import NextLink from 'next/link';
import { Layout } from '../components/layout';
import dynamic from 'next/dynamic';
import { useRouter } from 'next/router';
import useSWR from 'swr';
import { fetcher } from '../utils/fetcher';

function Confirm() {
  const router = useRouter();
  const { token } = router.query;

  const { data, error } = useSWR(
    !!token ? `http://localhost:3000/confirm?token=${token}` : null,
    fetcher
  );
  if (!token)
    return (
      <Layout>
        <Center mt={12}>{'nice try'}</Center>
      </Layout>
    );
  if (!data || error)
    return (
      <Layout>
        <Center mt={12}>{'Token is invalid, likely expired'}</Center>
      </Layout>
    );
  return (
    <Layout>
      <Box textAlign="center" py={10} px={6}>
        <CheckCircleIcon boxSize={'50px'} color={'green.500'} />
        <Heading as="h2" size="xl" mt={6} mb={2}>
          Good.
        </Heading>
        <Text fontSize={'lg'} color={'gray.500'}>
          Let&apos;s{' '}
          <NextLink href="/login">
            <Link color={'orange.400'}>cook something.</Link>
          </NextLink>
        </Text>
      </Box>
    </Layout>
  );
}

export default dynamic(() => Promise.resolve(Confirm), { ssr: false });
