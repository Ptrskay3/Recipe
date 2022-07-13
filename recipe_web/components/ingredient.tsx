import {
  Box,
  Center,
  Editable,
  EditableInput,
  EditablePreview,
  Flex,
  Heading,
  Stack,
  Text,
  useColorModeValue,
} from '@chakra-ui/react';
import { useRef, useState } from 'react';
import { useIngredientEditMode } from '../stores/useIngredientEditMode';
import { EditableControls } from './editable_custom_controls';
import { IngredientEditControls } from './ingredient_edit_controls';

export interface IngredientProps {
  name: string;
  calories_per_100g: number;
  category: string[];
  protein: number;
  water: number;
  fat: number;
  sugar: number;
  carbohydrate: number;
  fiber: number;
  caffeine: number;
  contains_alcohol: boolean;
}
interface ModifiedAttributes {
  withModifiedAttributes?: (keyof IngredientProps)[];
  isNew: boolean;
  isDeleteVote: boolean;
}

export default function Ingredient({
  name,
  calories_per_100g,
  category,
  protein,
  water,
  fat,
  sugar,
  carbohydrate,
  fiber,
  caffeine,
  contains_alcohol,
  withModifiedAttributes = [],
  isNew,
  isDeleteVote,
}: IngredientProps & ModifiedAttributes) {
  const coloring = isNew ? 'green.400' : 'red.400';
  const editModeOpen = useIngredientEditMode((state) => state.editModeOpen);
  const editedValues = useIngredientEditMode((state) => state.editedValues);
  const updateEditedValues = useIngredientEditMode((state) => state.updateEditedValues);
  const originals = {
    name,
    calories_per_100g,
    category,
    protein,
    water,
    fat,
    sugar,
    carbohydrate,
    fiber,
    caffeine,
    contains_alcohol,
  };

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
          <IngredientEditControls name={name} originals={originals} />
          <Text
            color={'orange.400'}
            fontSize={'xl'}
            textTransform={'uppercase'}
            textColor={isDeleteVote ? 'red.400' : undefined}
            as={isDeleteVote ? 'del' : undefined}
          >
            {name}
          </Text>
          <Text fontWeight={400} fontSize={'xs'}>
            {'Calories per 100g'}
          </Text>
          {editModeOpen ? (
            <Editable
              defaultValue={'' + calories_per_100g}
              fontSize={'2xl'}
              fontFamily={'body'}
              fontWeight={500}
              textColor={!!editedValues.calories_per_100g ? 'green.400' : undefined}
            >
              <EditablePreview />
              <EditableInput
                onChange={(e) =>
                  updateEditedValues({ calories_per_100g: parseFloat(e.target.value) })
                }
                onBlur={(e) =>
                  updateEditedValues({ calories_per_100g: parseFloat(e.target.value) })
                }
              />
            </Editable>
          ) : (
            <Heading
              fontSize={'2xl'}
              fontFamily={'body'}
              fontWeight={500}
              textColor={
                withModifiedAttributes.includes('calories_per_100g') ? coloring : undefined
              }
            >
              {calories_per_100g}
            </Heading>
          )}
          {/* TODO: Make this a mapping of some sort */}
          <Text fontWeight={400} fontSize={'xl'}>
            {'💪💯🔥🚀Protein🚀🔥💯💪'}
          </Text>
          <Heading
            fontSize={'2xl'}
            fontFamily={'body'}
            fontWeight={500}
            textColor={withModifiedAttributes.includes('protein') ? coloring : undefined}
          >
            {protein + ' g'}
          </Heading>
          <Text fontWeight={400} fontSize={'xs'}>
            {'Carbohydrate'}
          </Text>
          <Heading
            fontSize={'2xl'}
            fontFamily={'body'}
            fontWeight={500}
            textColor={withModifiedAttributes.includes('carbohydrate') ? coloring : undefined}
          >
            {carbohydrate + ' g'}
          </Heading>
          <Text fontWeight={400} fontSize={'xs'}>
            {'Fat'}
          </Text>
          <Heading
            fontSize={'2xl'}
            fontFamily={'body'}
            fontWeight={500}
            textColor={withModifiedAttributes.includes('fat') ? coloring : undefined}
          >
            {fat + ' g'}
          </Heading>
          <Text fontWeight={400} fontSize={'xs'}>
            {'Sugar'}
          </Text>
          <Heading
            fontSize={'2xl'}
            fontFamily={'body'}
            fontWeight={500}
            textColor={withModifiedAttributes.includes('sugar') ? coloring : undefined}
          >
            {sugar + ' g'}
          </Heading>
          <Text fontWeight={400} fontSize={'xs'}>
            {'Fiber'}
          </Text>
          <Heading
            fontSize={'2xl'}
            fontFamily={'body'}
            fontWeight={500}
            textColor={withModifiedAttributes.includes('fiber') ? coloring : undefined}
          >
            {fiber + ' g'}
          </Heading>
          <Text fontWeight={400} fontSize={'xs'}>
            {'Water'}
          </Text>
          <Heading
            fontSize={'2xl'}
            fontFamily={'body'}
            fontWeight={500}
            textColor={withModifiedAttributes.includes('water') ? coloring : undefined}
          >
            {water + ' g'}
          </Heading>
          <Text fontWeight={400} fontSize={'xs'}>
            {'caffeine'}
          </Text>
          <Heading
            fontSize={'2xl'}
            fontFamily={'body'}
            fontWeight={500}
            textColor={withModifiedAttributes.includes('caffeine') ? coloring : undefined}
          >
            {caffeine + ' mg'}
          </Heading>
          <Text fontWeight={400} fontSize={'xs'}>
            {'Contains alcohol'}
          </Text>
          <Heading
            fontSize={'2xl'}
            fontFamily={'body'}
            fontWeight={500}
            textColor={withModifiedAttributes.includes('contains_alcohol') ? coloring : undefined}
          >
            {contains_alcohol ? 'Yes' : 'No'}
          </Heading>
          <Text fontWeight={400} fontSize={'xs'}>
            {'Category'}
          </Text>
          <Stack direction={'row'} align={'center'}>
            <Text
              fontSize={'2xl'}
              fontFamily={'body'}
              fontWeight={500}
              textColor={withModifiedAttributes.includes('category') ? coloring : undefined}
            >
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
