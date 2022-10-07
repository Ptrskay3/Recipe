import {
  Box,
  Button,
  Center,
  FormControl,
  FormErrorMessage,
  FormLabel,
  Input,
  Stack,
} from '@chakra-ui/react';
import { useFormik } from 'formik';
import { useRouter } from 'next/router';
import { useState } from 'react';
import { useSWRConfig } from 'swr';
import { useAddIngredient } from '../stores/useAddIngredient';
import { intoFormBody } from '../utils/form';

export function AddIngredientForm() {
  const { mutate } = useSWRConfig();
  const setAddIngredientOpen = useAddIngredient((state) => state.setAddIngredientOpen);

  const router = useRouter();
  const { name } = router.query;
  const [loading, setLoading] = useState(false);
  const [errors, setErrors] = useState<{
    name?: string;
    quantity?: string;
    quantity_unit?: string;
  }>({});

  const formik = useFormik({
    initialValues: {
      name: '',
      quantity: '100',
      quantity_unit: 'g',
    },
    validate: () => {},
    onSubmit: async (values) => {
      setErrors({});
      setLoading(true);
      const response = await fetch(`${process.env.NEXT_PUBLIC_BASE_URL}/r/${name}/ingredient`, {
        method: 'POST',
        body: intoFormBody(values),
        credentials: 'include',
        headers: {
          'Content-Type': 'application/x-www-form-urlencoded',
        },
      });

      setLoading(false);
      if (!response.ok) {
        setErrors({ name: 'This ingredient does not exist' });
        return;
      }
      setAddIngredientOpen(false);
      mutate(`${process.env.NEXT_PUBLIC_BASE_URL}/r/${name}`, true);
    },
  });
  return (
    <Box>
      <Center>
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
