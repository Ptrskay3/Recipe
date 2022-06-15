import type { NextPage } from 'next';
import styles from '../styles/Home.module.css';
import { intoFormBody } from '../utils/form';
import { useRouter } from 'next/router';

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
        <label htmlFor="name"> name:</label>
        <input type="text" id="name" name="name" />
        <label htmlFor="password">password:</label>
        <input type="password" id="password" name="password" />
        <button type="submit">Submit</button>
      </form>
    </div>
  );
};

export default Home;
