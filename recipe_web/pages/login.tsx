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
import { useAlreadyAuth } from '../utils/useAlreadyAuth';
import { useFormik } from 'formik';
import { useState } from 'react';
import dynamic from 'next/dynamic';
import NextLink from 'next/link';

function Login() {
  useAlreadyAuth();
  const router = useRouter();
  const [loading, setLoading] = useState(false);
  const [errors, setErrors] = useState<{ username?: string; password?: string }>({});

  const formik = useFormik({
    initialValues: {
      name: '',
      password: '',
    },
    validate: () => {}, // TODO
    onSubmit: async (values) => {
      setErrors({});
      setLoading(true);
      const formBody = intoFormBody(values);
      const response = await fetch('http://localhost:3000/auth', {
        method: 'POST',
        body: formBody,
        credentials: 'include',
        headers: {
          'Content-Type': 'application/x-www-form-urlencoded',
        },
      });

      setLoading(false);
      if (response.ok) {
        router.replace('/');
      } else {
        let err = await response.json();
        setErrors(err.errors);
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
          <Heading fontSize={'4xl'}>Sign in</Heading>
          <Text fontSize={'lg'} color={'gray.600'}>
            to enjoy all of our cool features
          </Text>
        </Stack>
        <Box rounded={'lg'} bg={useColorModeValue('white', 'gray.700')} boxShadow={'lg'} p={8}>
          <Stack spacing={4}>
            <form onSubmit={formik.handleSubmit}>
              <FormControl id="name" isInvalid={!!errors.username}>
                <FormLabel htmlFor="name">Username</FormLabel>
                <Input
                  type="text"
                  id="name"
                  name="name"
                  required
                  autoFocus
                  onChange={formik.handleChange}
                  onBlur={formik.handleBlur}
                  value={formik.values.name}
                />
                <FormErrorMessage>{errors.username}</FormErrorMessage>
              </FormControl>
              <FormControl id="password" mt={4} isInvalid={!!errors.password}>
                <FormLabel htmlFor="password">Password</FormLabel>
                <Input
                  type="password"
                  id="password"
                  name="password"
                  required
                  onChange={formik.handleChange}
                  onBlur={formik.handleBlur}
                  value={formik.values.password}
                />
                <FormErrorMessage>{errors.password}</FormErrorMessage>
              </FormControl>
              <Stack spacing={10}>
                <NextLink href="/forget_password_gen">
                  <Link mt={4} color={'orange.400'}>
                    Forgot password?
                  </Link>
                </NextLink>
                <Button
                  isLoading={loading}
                  type="submit"
                  bg={'orange.400'}
                  color={'white'}
                  _hover={{
                    bg: 'orange.500',
                  }}
                >
                  Sign in
                </Button>
              </Stack>
            </form>
          </Stack>
        </Box>
      </Stack>
    </Flex>
  );
}

export default dynamic(() => Promise.resolve(Login), { ssr: false });
