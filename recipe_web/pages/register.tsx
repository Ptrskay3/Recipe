import {
  Flex,
  Box,
  FormControl,
  FormLabel,
  Input,
  InputGroup,
  HStack,
  InputRightElement,
  Stack,
  Button,
  Heading,
  Text,
  useColorModeValue,
  Link,
  FormErrorMessage,
} from '@chakra-ui/react';
import { useState } from 'react';
import { ViewIcon, ViewOffIcon } from '@chakra-ui/icons';
import NextLink from 'next/link';
import { useFormik } from 'formik';
import { useRouter } from 'next/router';
import { intoFormBody } from '../utils/form';
import { useAlreadyAuth } from '../utils/useAlreadyAuth';
import { AuthFormWrapper } from '../components/auth_form_wrapper';

export default function SignupCard() {
  const [showPassword, setShowPassword] = useState(false);
  useAlreadyAuth();
  const router = useRouter();
  const [loading, setLoading] = useState(false);
  const [errors, setErrors] = useState<{ name?: string; password?: string; email?: string }>({});

  const formik = useFormik({
    initialValues: {
      name: '',
      password: '',
      email: '',
    },
    validate: () => {}, // TODO
    onSubmit: async (values) => {
      setLoading(true);
      const formBody = intoFormBody(values);
      const response = await fetch('http://localhost:3000/register', {
        method: 'POST',
        body: formBody,
        credentials: 'include',
        headers: {
          'Content-Type': 'application/x-www-form-urlencoded',
        },
      });
      setLoading(false);
      if (response.ok) {
        router.replace('/postreg');
      } else {
        let {
          errors: { name, password, email },
        } = await response.json();
        const errorsReceived = {
          name: Array.isArray(name) ? name.join(', ') : name,
          password,
          email: Array.isArray(email) ? email.join(', ') : email,
        };
        setErrors(errorsReceived);
      }
    },
  });

  return (
    <AuthFormWrapper>
      <Stack align={'center'}>
        <Heading fontSize={'4xl'} textAlign={'center'}>
          Sign up
        </Heading>
        <Text fontSize={'lg'} color={'gray.600'}>
          to enjoy all of our cool features
        </Text>
      </Stack>
      <form onSubmit={formik.handleSubmit}>
        <Box rounded={'lg'} bg={useColorModeValue('white', 'gray.700')} boxShadow={'lg'} p={8}>
          <Stack spacing={4}>
            <HStack>
              <Box>
                <FormControl id="firstName" isRequired>
                  <FormLabel>First Name</FormLabel>
                  <Input type="text" />
                </FormControl>
              </Box>
              <Box>
                <FormControl id="lastName">
                  <FormLabel>Last Name</FormLabel>
                  <Input type="text" />
                </FormControl>
              </Box>
            </HStack>
            <FormControl id="name" isRequired isInvalid={!!errors.name}>
              <FormLabel>Username</FormLabel>
              <Input
                type="text"
                id="name"
                name="name"
                required
                onChange={formik.handleChange}
                onBlur={formik.handleBlur}
                value={formik.values.name}
              />
              <FormErrorMessage>{errors.name}</FormErrorMessage>
            </FormControl>
            <FormControl id="email" isRequired isInvalid={!!errors.email}>
              <FormLabel>Email address</FormLabel>
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
            <FormControl id="password" isRequired isInvalid={!!errors.password}>
              <FormLabel>Password</FormLabel>
              <InputGroup>
                <Input
                  type={showPassword ? 'text' : 'password'}
                  id="password"
                  name="password"
                  required
                  onChange={formik.handleChange}
                  onBlur={formik.handleBlur}
                  value={formik.values.password}
                />
                <InputRightElement h={'full'}>
                  <Button
                    variant={'ghost'}
                    onClick={() => setShowPassword((showPassword) => !showPassword)}
                  >
                    {showPassword ? <ViewIcon /> : <ViewOffIcon />}
                  </Button>
                </InputRightElement>
              </InputGroup>
              <FormErrorMessage>{errors.password}</FormErrorMessage>
            </FormControl>
            <Stack spacing={10} pt={2}>
              <Button
                isLoading={loading}
                type="submit"
                loadingText="Submitting"
                size="lg"
                bg={'orange.400'}
                color={'white'}
                _hover={{
                  bg: 'orange.500',
                }}
              >
                Sign up
              </Button>
            </Stack>
            <Stack pt={6}>
              <Text align={'center'}>
                Already a user?{' '}
                <NextLink href="/login" passHref>
                  <Link color={'orange.400'}>Sign in</Link>
                </NextLink>{' '}
                instead.
              </Text>
            </Stack>
          </Stack>
        </Box>
      </form>
    </AuthFormWrapper>
  );
}
