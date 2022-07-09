import { Button, Center } from '@chakra-ui/react';
import { Form, Formik } from 'formik';
import dynamic from 'next/dynamic';
import { useRouter } from 'next/router';
import { InputField } from '../../components/form/field';
import { Layout } from '../../components/layout';
import { intoFormBody } from '../../utils/form';
import { useAuth } from '../../utils/useAuth';

const NewRecipe = () => {
  const { push } = useRouter();
  useAuth();
  return (
    <Layout>
      <Center>
        <Formik<{
          name: string;
          description: string;
        }>
          initialValues={{ name: '', description: '' }}
          validateOnChange={false}
          validateOnBlur={false}
          validate={({ name, description }) => {
            const errors: Record<string, string> = {};

            if (name.length < 4) {
              return {
                name: 'should be longer than 3 characters',
              };
            }
            if (description.length < 4) {
              return {
                description: 'should be longer than 3 characters',
              };
            }
            return errors;
          }}
          onSubmit={async (values) => {
            const { ok } = await fetch(`http://localhost:3000/r/${values.name}`, {
              method: 'POST',
              body: intoFormBody(values),
              credentials: 'include',
              headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
            });

            if (ok) {
              push(`/r/${values.name}`);
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
              <InputField
                name="description"
                label="Description"
                placeholder="recipe description"
                autoFocus
                errors={errors}
              />
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
            </Form>
          )}
        </Formik>
      </Center>
    </Layout>
  );
};

export default dynamic(() => Promise.resolve(NewRecipe), { ssr: false });
