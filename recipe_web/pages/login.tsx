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
  Divider,
  HStack,
} from '@chakra-ui/react';
import { useRouter } from 'next/router';
import { intoFormBody } from '../utils/form';
import { useAlreadyAuth } from '../utils/useAlreadyAuth';
import { useFormik } from 'formik';
import { useState } from 'react';
import dynamic from 'next/dynamic';
import NextLink from 'next/link';
import { AuthFormWrapper } from '../components/auth_form_wrapper';
import { OAuthButtonGroup } from '../components/OAuthGroup';

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
        let { errors } = await response.json();
        setErrors(errors);
      }
    },
  });

  return (
    <AuthFormWrapper>
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
              <HStack>
                <Divider />
                <Text fontSize="sm" whiteSpace="nowrap" color="muted">
                  or continue with
                </Text>
                <Divider />
              </HStack>
              <OAuthButtonGroup />
            </Stack>
          </form>
        </Stack>
      </Box>
    </AuthFormWrapper>
  );
}

export default dynamic(() => Promise.resolve(Login), { ssr: false });
