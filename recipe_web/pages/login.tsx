import {
  Flex,
  Box,
  FormControl,
  FormLabel,
  Input,
  Checkbox,
  Stack,
  Link,
  Button,
  Heading,
  Text,
  useColorModeValue,
} from '@chakra-ui/react';
import { useRouter } from 'next/router';
import { intoFormBody } from '../utils/form';
import { useAlreadyAuth } from '../utils/useAlreadyAuth';

export default function Login() {
  useAlreadyAuth();
  const router = useRouter();

  const onSubmit = async (e: any) => {
    e.preventDefault();
    const state = {
      name: e.target.elements.name.value,
      password: e.target.elements.password.value,
    };
    const formBody = intoFormBody(state);
    const { ok } = await fetch('http://localhost:3000/auth', {
      method: 'POST',
      body: formBody,
      credentials: 'include',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
      },
    });

    if (ok) {
      router.replace('/');
    }
  };

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
            {/* TODO: Formik*/}
            <form onSubmit={onSubmit}>
              <FormControl id="name">
                <FormLabel htmlFor="name">Username</FormLabel>
                <Input type="text" id="name" name="name" required />
              </FormControl>
              <FormControl id="password" mt={4}>
                <FormLabel htmlFor="password">Password</FormLabel>
                <Input type="password" id="password" name="password" required />
              </FormControl>
              <Stack spacing={10}>
                <Link mt={4} color={'orange.400'}>
                  Forgot password?
                </Link>
                <Button
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
