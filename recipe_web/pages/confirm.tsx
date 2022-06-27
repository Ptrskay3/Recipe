import { Box, Center, CircularProgress, Flex, Heading, Link, Text } from '@chakra-ui/react';
import { CheckCircleIcon, CloseIcon } from '@chakra-ui/icons';
import NextLink from 'next/link';
import { Layout } from '../components/layout';
import { useRouter } from 'next/router';
import useSWR from 'swr';
import { fetcherWithoutJson } from '../utils/fetcher';

export default function Confirm() {
  const router = useRouter();
  const { token } = router.query;

  const { data, error } = useSWR(
    !!token ? `http://localhost:3000/confirm?token=${token}` : null,
    fetcherWithoutJson,
    {
      shouldRetryOnError: false,
    }
  );
  if (!error && !data && !!token) {
    return (
      <Layout>
        <Center mt="14">
          <CircularProgress isIndeterminate color="orange.400" />
        </Center>
      </Layout>
    );
  }

  // TODO: The logic is somewhat wrong for this.
  // if (data && !!token && (error || !data.ok))
  //   return (
  //     <Layout>
  //       <Box textAlign="center" py={10} px={6}>
  //         <Box display="inline-block">
  //           <Flex
  //             flexDirection="column"
  //             justifyContent="center"
  //             alignItems="center"
  //             bg={'red.500'}
  //             rounded={'50px'}
  //             w={'55px'}
  //             h={'55px'}
  //             textAlign="center"
  //           >
  //             <CloseIcon boxSize={'20px'} color={'white'} />
  //           </Flex>
  //         </Box>
  //         <Heading as="h2" size="xl" mt={6} mb={2}>
  //           Something went wrong.
  //         </Heading>
  //         <Text color={'gray.500'}>The provided token is invalid, or expired.</Text>
  //       </Box>
  //     </Layout>
  //   );
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
