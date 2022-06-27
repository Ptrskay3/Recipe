import { Box, Heading, Link, Text } from '@chakra-ui/react';
import { CheckCircleIcon } from '@chakra-ui/icons';
import NextLink from 'next/link';
import { Layout } from '../components/layout';
import dynamic from 'next/dynamic';

function PostPasswordReset() {
  return (
    <Layout>
      <Box textAlign="center" py={10} px={6}>
        <CheckCircleIcon boxSize={'50px'} color={'green.500'} />
        <Heading as="h2" size="xl" mt={6} mb={2}>
          Good.
        </Heading>
        <Text color={'gray.500'}>
          If the provided credentials are correct, we sent you an email.
        </Text>
      </Box>
    </Layout>
  );
}

export default dynamic(() => Promise.resolve(PostPasswordReset), { ssr: false });
