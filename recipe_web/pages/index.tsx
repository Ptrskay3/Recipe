import type { NextPage } from 'next';
import styles from '../styles/Home.module.css';
import { intoFormBody } from '../utils/form';
import { useRouter } from 'next/router';
import { FormControl, FormLabel, Button, Input, Flex } from '@chakra-ui/react';

const Home: NextPage = () => {
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
      router.replace('/test_redirect_to_auth');
    }
  };

  return (
    <div className={styles.container}>
      <form onSubmit={onSubmit}>
        <FormControl>
          <FormLabel htmlFor="name"> name:</FormLabel>
          <Input type="text" id="name" name="name" />
          <FormLabel htmlFor="password">password:</FormLabel>
          <Input type="password" id="password" name="password" />
          <Button type="submit" colorScheme="teal" size="sm">
            Submit
          </Button>
        </FormControl>
      </form>
    </div>
  );
};

export default Home;
