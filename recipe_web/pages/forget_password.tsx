import {
  Flex,
  Box,
  FormControl,
  FormLabel,
  Input,
  Stack,
  Link,
  Button,
  Heading,
  Text,
  useColorModeValue,
  FormErrorMessage,
} from '@chakra-ui/react';
import { useRouter } from 'next/router';
import { intoFormBody } from '../utils/form';
import { useFormik } from 'formik';
import { useEffect, useState } from 'react';
import dynamic from 'next/dynamic';
import NextLink from 'next/link';
import { CloseIcon } from '@chakra-ui/icons';
import { useToast } from '@chakra-ui/react';
import { useValidToken } from '../hooks/token';

function ForgetPasswordGen() {
  const router = useRouter();
  const {
    query: { token },
    isReady,
  } = router;
  const [loading, setLoading] = useState(false);
  const [errors, setErrors] = useState<{ password?: string; password_ensure?: string }>({});
  const toast = useToast();

  const { isLoading, isValid } = useValidToken(token);
  useEffect(() => {
    if (!isLoading && isValid === false) {
      toast({
        title: 'Token is no longer valid',
        description: 'Redirecting to the homepage.',
        status: 'error',
        duration: 9000,
        isClosable: true,
      });
      router.replace('/');
    }
  }, [token, isLoading, isValid, router, toast]);

  const formik = useFormik({
    initialValues: {
      password: '',
      password_ensure: '',
    },
    validate: () => {}, // TODO
    onSubmit: async (values) => {
      if (values.password !== values.password_ensure) {
        setErrors({
          password: 'Password fields do not match',
          password_ensure: 'Password fields do not match',
        });
        return;
      }
      setLoading(true);
      const formBody = intoFormBody({ password: values.password });
      const response = await fetch(
        `${process.env.NEXT_PUBLIC_BASE_URL}/forget_password?token=${token}`,
        {
          method: 'POST',
          body: formBody,
          credentials: 'include',
          headers: {
            'Content-Type': 'application/x-www-form-urlencoded',
          },
        }
      );

      setLoading(false);
      if (response.ok) {
        toast({
          title: 'Password successfully changed.',
          description: 'You can now sign in!',
          status: 'success',
          duration: 9000,
          isClosable: true,
        });
        router.replace('/login');
      } else if (response.status === 422) {
        // Uuid serialization fail. Probably someone's playing with urls..
        toast({
          title: 'The provided token seems to be wrong',
          description: 'Redirecting to the homepage.',
          status: 'error',
          duration: 9000,
          isClosable: true,
        });
        router.replace('/');
      } else {
        toast({
          title: 'Token is no longer valid',
          description: 'Redirecting to the homepage.',
          status: 'error',
          duration: 9000,
          isClosable: true,
        });
        router.replace('/');
      }
    },
  });

  return (
    <Flex
      minH={'100vh'}
      align={'center'}
      justify={'center'}
      bg={useColorModeValue('gray.50', 'gray.800')}
    >
      <Stack spacing={8} mx={'auto'} maxW={'lg'} py={12} px={6}>
        <Stack align={'center'}>
          <Heading fontSize={'4xl'}>Reset Password</Heading>
          <Text fontSize={'lg'} color={'gray.600'}>
            Enter your new password below.
          </Text>
        </Stack>
        <Box rounded={'lg'} bg={useColorModeValue('white', 'gray.700')} boxShadow={'lg'} p={8}>
          {isReady && !token ? (
            <Box textAlign="center" py={10} px={6}>
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
              <Heading as="h2" size="lg" mt={6} mb={2}>
                Something went wrong.
              </Heading>
              <Text color={'gray.500'}>
                No token provided. <br />
                <NextLink href="/">
                  <Link color={'orange.400'}>Go back to homepage.</Link>
                </NextLink>
              </Text>
            </Box>
          ) : (
            <Stack spacing={4}>
              <form onSubmit={formik.handleSubmit}>
                <FormControl id="name" isInvalid={!!errors.password}>
                  <FormLabel htmlFor="name">Password</FormLabel>
                  <Input
                    type="password"
                    id="password"
                    name="password"
                    required
                    autoFocus
                    onChange={formik.handleChange}
                    onBlur={formik.handleBlur}
                    value={formik.values.password}
                  />
                  <FormErrorMessage>{errors.password}</FormErrorMessage>
                </FormControl>

                <FormControl mt={4} id="password_ensure" isInvalid={!!errors.password_ensure}>
                  <FormLabel htmlFor="password_ensure">Password again</FormLabel>
                  <Input
                    type="password"
                    id="password_ensure"
                    name="password_ensure"
                    required
                    onChange={formik.handleChange}
                    onBlur={formik.handleBlur}
                    value={formik.values.password_ensure}
                  />
                  <FormErrorMessage>{errors.password}</FormErrorMessage>
                </FormControl>

                <Button
                  mt={4}
                  isLoading={loading}
                  type="submit"
                  bg={'orange.400'}
                  color={'white'}
                  _hover={{
                    bg: 'orange.500',
                  }}
                >
                  Set new password
                </Button>
              </form>
            </Stack>
          )}
        </Box>
      </Stack>
    </Flex>
  );
}

export default dynamic(() => Promise.resolve(ForgetPasswordGen), { ssr: false });
