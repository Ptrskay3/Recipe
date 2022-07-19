import { Box, Button, Center } from '@chakra-ui/react';
import { Form, Formik } from 'formik';
import dynamic from 'next/dynamic';
import { useRouter } from 'next/router';
import { InputField } from '../../../components/form/field';
import { Layout } from '../../../components/layout';
import { DurationSlider } from '../../../components/slider';
import { DifficultyLevel, MealType } from '../../../utils/types';
import { useAuth } from '../../../utils/useAuth';

const NewRecipe = () => {
  const { push } = useRouter();
  useAuth();
  return (
    <Layout>
      <DurationSlider></DurationSlider>
      <Center mt="4">
        <Formik<{
          name: string;
          description: string;
          prep_time: number;
          cook_time: number;
          difficulty: DifficultyLevel;
          steps: string[];
          cuisine: string;
          meal_type: MealType;
          ingredients: { name: string; quantity: string; quantity_unit: string }[];
        }>
          initialValues={{
            name: '',
            description: '',
            prep_time: 0,
            cook_time: 0,
            difficulty: 'easy',
            steps: [],
            cuisine: 'hungarian',
            meal_type: 'breakfast',
            ingredients: [],
          }}
          validateOnChange={false}
          validateOnBlur={false}
          validate={({ name, description }) => {
            const errors: Record<string, string> = {};

            if (name.length < 3) {
              errors.name = 'name should be longer than 2 characters';
            }
            if (description.length < 2) {
              errors.description = 'description should be longer than 2 characters';
            }
            return errors;
          }}
          onSubmit={async (values, { setFieldError }) => {
            const { ok } = await fetch(`http://localhost:3000/r`, {
              method: 'POST',
              body: JSON.stringify(values),
              credentials: 'include',
              headers: { 'Content-Type': 'application/json' },
            });

            if (ok) {
              push(`/r/${values.name}`);
            } else {
              setFieldError('name', 'A recipe with this name already exists');
            }
          }}
        >
          {({ isSubmitting, errors }) => (
            <Form>
              <InputField
                name="name"
                label="Recipe name"
                placeholder="recipe name"
                autoFocus
                errors={errors}
              />
              <Box mt="4" />
              <InputField
                name="description"
                label="Description"
                placeholder="recipe description"
                errors={errors}
              />
              <Center mt="4">
                <Button
                  type="submit"
                  isLoading={isSubmitting}
                  fontWeight={600}
                  color={'white'}
                  bg={'orange.400'}
                  _hover={{
                    bg: 'orange.300',
                  }}
                >
                  Create
                </Button>
              </Center>
            </Form>
          )}
        </Formik>
      </Center>
    </Layout>
  );
};

export default dynamic(() => Promise.resolve(NewRecipe), { ssr: false });
