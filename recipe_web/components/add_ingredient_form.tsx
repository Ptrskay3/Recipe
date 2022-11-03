import {
  Box,
  Button,
  Center,
  FormControl,
  FormErrorMessage,
  FormLabel,
  Input,
  Stack,
  useToast,
} from '@chakra-ui/react';
import { useFormik } from 'formik';
import { useRouter } from 'next/router';
import { useState } from 'react';
import { useSWRConfig } from 'swr';
import { useAddIngredient } from '../stores/useAddIngredient';
import { intoFormBody } from '../utils/form';
import IngredientSearch from './search/IngredientSearch';

export function AddIngredientForm() {
  const { mutate } = useSWRConfig();
  const selected = useAddIngredient((state) => state.selected);

  const toast = useToast();

  const router = useRouter();
  const { name } = router.query;
  const [loading, setLoading] = useState(false);
  const [errors, setErrors] = useState<{
    quantity?: string;
    quantity_unit?: string;
  }>({});

  const formik = useFormik({
    initialValues: {
      quantity: '100',
      quantity_unit: 'g',
    },
    validate: () => {},
    onSubmit: async (values) => {
      setErrors({});
      if (!selected) {
        // TODO: make this focus the search element instead and highlight invalidity
        toast({
          title: 'Select an ingredient',
          status: 'error',
          duration: 9000,
          isClosable: true,
        });
        return;
      }
      setLoading(true);
      const response = await fetch(`${process.env.NEXT_PUBLIC_BASE_URL}/r/${name}/ingredient`, {
        method: 'POST',
        body: intoFormBody({ ...values, name: selected! }),
        credentials: 'include',
        headers: {
          'Content-Type': 'application/x-www-form-urlencoded',
        },
      });

      setLoading(false);
      if (response.status === 400) {
        toast({
          title: 'Not authorized to perform that',
          status: 'error',
          duration: 9000,
          isClosable: true,
        });
        return;
      }
      if (!response.ok) {
        toast({
          title: 'This ingredient does not exist',
          status: 'error',
          duration: 9000,
          isClosable: true,
        });
        return;
      }

      mutate(`${process.env.NEXT_PUBLIC_BASE_URL}/r/${name}`, true);
    },
  });
  return (
    <Box>
      <Center>
        <IngredientSearch />
        <form onSubmit={formik.handleSubmit}>
          <FormControl id="quantity" mt={4} isInvalid={!!errors.quantity}>
            <FormLabel htmlFor="quantity">Quantity</FormLabel>
            <Input
              type="text"
              id="quantity"
              name="quantity"
              required
              onChange={formik.handleChange}
              onBlur={formik.handleBlur}
              value={formik.values.quantity}
            />
            <FormErrorMessage>{errors.quantity}</FormErrorMessage>
          </FormControl>
          <FormControl id="quantity_unit" mt={4} isInvalid={!!errors.quantity_unit}>
            <FormLabel htmlFor="quantity_unit">Unit</FormLabel>
            <Input
              type="text"
              id="quantity_unit"
              name="quantity_unit"
              required
              onChange={formik.handleChange}
              onBlur={formik.handleBlur}
              value={formik.values.quantity_unit}
            />
            <FormErrorMessage>{errors.quantity_unit}</FormErrorMessage>
          </FormControl>
          <Stack spacing={10} mt="4">
            <Button
              isLoading={loading}
              type="submit"
              bg={'orange.400'}
              color={'white'}
              _hover={{
                bg: 'orange.500',
              }}
            >
              Add
            </Button>
          </Stack>
        </form>
      </Center>
    </Box>
  );
}
