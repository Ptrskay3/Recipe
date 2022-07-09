import { Flex, useColorModeValue, Stack } from '@chakra-ui/react';

export const AuthFormWrapper = ({ children }: any) => {
  return (
    <Flex
      minH={'100vh'}
      align={'center'}
      justify={'center'}
      bg={useColorModeValue('gray.50', 'gray.800')}
    >
      <Stack spacing={8} mx={'auto'} maxW={'lg'} py={12} px={6}>
        {children}
      </Stack>
    </Flex>
  );
};
