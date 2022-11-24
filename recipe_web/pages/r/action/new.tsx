import { Box, Button, Center, Divider, Heading } from '@chakra-ui/react';
import { Form, Formik } from 'formik';
import dynamic from 'next/dynamic';
import { useRouter } from 'next/router';
import { EnumSelector } from '../../../components/EnumSelect';
import { InputField } from '../../../components/form/field';
import { Layout } from '../../../components/layout';
import { Listable } from '../../../components/Listable';
import { DurationSlider } from '../../../components/slider';
import { useAddRecipe } from '../../../stores/useAddRecipe';
import { difficultyLevels, mealTypes } from '../../../utils/types';
import { useAuth } from '../../../utils/useAuth';
import type { DifficultyLevel, MealType } from '../../../utils/types';

const NewRecipe = () => {
  const { push } = useRouter();
  useAuth();

  const [
    name,
    description,
    prep_time,
    cook_time,
    difficulty,
    steps,
    cuisine,
    meal_type,
    ingredients,
    setPrepTime,
    setDifficulty,
    pushStep,
    removeStepByIndex,
    setCookTime,
    setName,
    setDescription,
    setCuisine,
    setMealType,
    resetState,
  ] = useAddRecipe((state) => [
    state.name,
    state.description,
    state.prep_time,
    state.cook_time,
    state.difficulty,
    state.steps,
    state.cuisine,
    state.meal_type,
    state.ingredients,
    state.setPrepTime,
    state.setDifficulty,
    state.pushStep,
    state.removeStepByIndex,
    state.setCookTime,
    state.setName,
    state.setDescription,
    state.setCuisine,
    state.setMealType,
    state.resetState,
  ]);

  return (
    <Layout>
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
            name,
            description,
            prep_time,
            cook_time,
            difficulty,
            steps,
            cuisine,
            meal_type,
            ingredients,
          }}
          validateOnChange={false}
          validateOnBlur={false}
          validate={({ name, description }) => {
            const errors: Record<string, string> = {};

            if (name.length < 3) {
              errors.name = 'name should be longer than 2 characters';
            }
            if (description.length < 2 || description.length > 250) {
              errors.description =
                'description should be longer than 2 characters, but no more than 250';
            }
            return errors;
          }}
          onSubmit={async (values, { setFieldError }) => {
            // TODO: we do not use Formik's values anyway..
            const response = await fetch(`${process.env.NEXT_PUBLIC_BASE_URL}/r`, {
              method: 'POST',
              body: JSON.stringify({
                name,
                description,
                prep_time,
                cook_time,
                difficulty,
                steps,
                cuisine,
                meal_type,
                ingredients,
              }),
              credentials: 'include',
              headers: { 'Content-Type': 'application/json' },
            });

            if (response.ok) {
              resetState();
              push(`/r/${values.name}`);
            } else if (response.status === 422) {
              const { errors } = await response.json();
              Object.entries(errors).forEach(([name, value]: [string, any]) =>
                setFieldError(name, value)
              );
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
                required
                errors={errors as any}
                onBlur={(e) => setName(e.target.value)}
              />
              <Box mt="4" />
              <InputField
                name="description"
                label="Description"
                placeholder="recipe description"
                errors={errors as any}
                required
                onBlur={(e) => setDescription(e.target.value)}
              />
              <Box mt="4" />
              <InputField
                name="cuisine"
                label="Cuisine"
                placeholder="cuisine"
                required
                errors={errors as any}
                onBlur={(e) => setCuisine(e.target.value)}
              />
              <Divider m="4"></Divider>
              <Heading fontSize={'lg'} m="2">
                {'Meal type'}
              </Heading>
              <EnumSelector
                options={mealTypes}
                defaultValue={'breakfast'}
                name={'mealtype'}
                onChange={(v) => setMealType(v)}
              ></EnumSelector>
              <Divider m="4" />
              <Heading fontSize={'lg'} m="2">
                {'Difficulty'}
              </Heading>
              <EnumSelector
                options={difficultyLevels}
                defaultValue={'easy'}
                name={'diff'}
                onChange={(v) => setDifficulty(v)}
              ></EnumSelector>
              <Divider m="4" />
              <Heading fontSize={'lg'} m="2">
                {'Preparation time'}
              </Heading>
              <DurationSlider onChangeEnd={(value) => setPrepTime(value)}></DurationSlider>
              <Divider m="4" />
              <Heading fontSize={'lg'} m="2">
                {'Cook time'}
              </Heading>
              <DurationSlider onChangeEnd={(value) => setCookTime(value)}></DurationSlider>
              <Divider m="4" />
              <Heading fontSize={'lg'} m="2">
                {'Steps'}
              </Heading>
              <Listable
                state={steps}
                pushState={pushStep}
                removeStateByIndex={removeStepByIndex}
              ></Listable>
              <Center mt="4">
                <Button
                  type="submit"
                  isLoading={isSubmitting}
                  fontWeight={600}
                  autoFocus={isSubmitting} // This'll trigger an onBlur on every input field
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
