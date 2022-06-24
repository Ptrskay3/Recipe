import { Box, Center, useColorModeValue, Heading, Text, Stack } from '@chakra-ui/react';

interface IngredientProps {
  name: string;
  calories_per_100g: number;
  category: string[];
}

export default function Ingredient({ name, calories_per_100g, category }: IngredientProps) {
  return (
    <Center py={12}>
      <Box
        role={'group'}
        p={6}
        maxW={'330px'}
        w={'full'}
        bg={useColorModeValue('white', 'gray.800')}
        boxShadow={'2xl'}
        rounded={'lg'}
        pos={'relative'}
        zIndex={1}
      >
        <Box
          rounded={'lg'}
          mt={-12}
          pos={'relative'}
          _after={{
            transition: 'all .3s ease',
            content: '""',
            w: 'full',
            h: 'full',
            pos: 'absolute',
            top: 5,
            left: 0,
            filter: 'blur(15px)',
            zIndex: -1,
          }}
          _groupHover={{
            _after: {
              filter: 'blur(20px)',
            },
          }}
        ></Box>
        <Stack pt={10} align={'center'}>
          <Text color={'orange.400'} fontSize={'xl'} textTransform={'uppercase'}>
            {name}
          </Text>
          <Text fontWeight={400} fontSize={'xs'}>
            {'Calories per 100g'}
          </Text>
          <Heading fontSize={'2xl'} fontFamily={'body'} fontWeight={500}>
            {calories_per_100g}
          </Heading>
          <Text fontWeight={400} fontSize={'xs'}>
            {'Category'}
          </Text>
          <Stack direction={'row'} align={'center'}>
            <Text fontSize={'2xl'} fontFamily={'body'} fontWeight={500}>
              {category.map(intoCategory).join(', ')}
            </Text>
          </Stack>
        </Stack>
      </Box>
    </Center>
  );
}

// TODO: this is dumb
const intoCategory = (c: string) => {
  return c.split('_').join(' ');
};
