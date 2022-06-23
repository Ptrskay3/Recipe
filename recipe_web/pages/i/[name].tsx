import { useEffect, useState } from 'react';
import { useRouter } from 'next/router';
import { Layout } from '../../components/layout';
import { Center, Text } from '@chakra-ui/react';

export default function IngredientDetailed() {
  const [data, setData] = useState(null);
  const router = useRouter();
  const { name } = router.query;
  useEffect(() => {
    if (!name) {
      return;
    }
    const fetchData = () => {
      fetch(`http://localhost:3000/i/${name}`, { credentials: 'include' })
        .then((r) => r.json())
        .then((data) => {
          setData(data);
        })
        .catch(() => router.replace('/'));
    };

    fetchData();
  }, [name, router]);
  return (
    data && (
      <Layout>
        <Center mt="14">
          <Text color="orange.400">{JSON.stringify(data)}</Text>
        </Center>
      </Layout>
    )
  );
}
