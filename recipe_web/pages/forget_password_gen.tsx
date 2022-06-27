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

function ForgetPasswordGen() {
  const router = useRouter();
  const [loading, setLoading] = useState(false);
  const [errors, setErrors] = useState<{ name?: string; email?: string }>({});

  const formik = useFormik({
    initialValues: {
      name: '',
      email: '',
    },
    validate: () => {}, // TODO
    onSubmit: async (values) => {
      setLoading(true);
      const formBody = intoFormBody(values);
      const response = await fetch('http://localhost:3000/forget_password_gen', {
        method: 'POST',
        body: formBody,
        credentials: 'include',
        headers: {
          'Content-Type': 'application/x-www-form-urlencoded',
        },
      });

      setLoading(false);
      if (response.ok) {
        router.replace('/post_pw_reset');
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
          <Heading fontSize={'4xl'}>Forget Password</Heading>
          <Text fontSize={'lg'} color={'gray.600'}>
            Enter your details below, and we&apos;ll get back to you.
          </Text>
        </Stack>
        <Box rounded={'lg'} bg={useColorModeValue('white', 'gray.700')} boxShadow={'lg'} p={8}>
          <Stack spacing={4}>
            <form onSubmit={formik.handleSubmit}>
              <FormControl id="name" isInvalid={!!errors.name}>
                <FormLabel htmlFor="name">Name</FormLabel>
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
                <FormErrorMessage>{errors.name}</FormErrorMessage>
              </FormControl>
              <FormControl id="email" mt={4} isInvalid={!!errors.email}>
                <FormLabel htmlFor="email">Email</FormLabel>
                <Input
                  type="email"
                  id="email"
                  name="email"
                  required
                  onChange={formik.handleChange}
                  onBlur={formik.handleBlur}
                  value={formik.values.email}
                />
                <FormErrorMessage>{errors.email}</FormErrorMessage>
              </FormControl>
              <Stack spacing={10}>
                <NextLink href="/login">
                  <Link mt={4} color={'orange.400'}>
                    Login instead?
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
                  Reset password
                </Button>
              </Stack>
            </form>
          </Stack>
        </Box>
      </Stack>
    </Flex>
  );
}

export default dynamic(() => Promise.resolve(ForgetPasswordGen), { ssr: false });
