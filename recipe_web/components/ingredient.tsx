import {
  Box,
  Center,
  Editable,
  EditableInput,
  EditablePreview,
  Heading,
  Stack,
  Text,
  useColorModeValue,
} from '@chakra-ui/react';
import { Fragment } from 'react';
import { useIngredientEditMode } from '../stores/useIngredientEditMode';
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
  isNew?: boolean;
  isDeleteVote?: boolean;
  shouldShowEditControls?: boolean;
  editableMapping: any;
}

export default function Ingredient({
  iProps,
  withModifiedAttributes = [],
  isNew,
  isDeleteVote,
  shouldShowEditControls = false,
  editableMapping,
}: { iProps: IngredientProps } & ModifiedAttributes) {
  const coloring = isNew ? 'green.400' : 'red.400';
  const editModeOpen = useIngredientEditMode((state) => state.editModeOpen);
  const editedValues = useIngredientEditMode((state) => state.editedValues);
  const updateEditedValues = useIngredientEditMode((state) => state.updateEditedValues);
  const { name, category, contains_alcohol } = iProps;

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
          {shouldShowEditControls ? (
            <IngredientEditControls name={name} originals={iProps} />
          ) : null}
          <Text
            color={'orange.400'}
            fontSize={'xl'}
            textTransform={'uppercase'}
            textColor={isDeleteVote ? 'red.400' : undefined}
            as={isDeleteVote ? 'del' : undefined}
          >
            {name}
          </Text>
          {Object.entries(editableMapping).map(([key, value]: [string, any], i: number) => (
            <Fragment key={i}>
              <Text fontWeight={400} fontSize={'xs'}>
                {value.namePretty}
              </Text>
              {editModeOpen ? (
                <Editable
                  textAlign="center"
                  defaultValue={iProps[key as keyof IngredientProps].toString()}
                  fontSize={'2xl'}
                  fontFamily={'body'}
                  fontWeight={500}
                  textColor={
                    Object.prototype.hasOwnProperty.call(editedValues, key as keyof IngredientProps)
                      ? 'green.400'
                      : undefined
                  }
                >
                  <EditablePreview />
                  <EditableInput
                    onChange={(e) => updateEditedValues({ [key]: parseFloat(e.target.value) })}
                    onBlur={(e) => updateEditedValues({ [key]: parseFloat(e.target.value) })}
                  />
                  {!!value.unitSuffix ? (
                    <Text as={'span'} fontSize={'2xl'} fontFamily={'body'} fontWeight={500}>
                      {value.unitSuffix}
                    </Text>
                  ) : null}
                </Editable>
              ) : (
                <Heading
                  fontSize={'2xl'}
                  fontFamily={'body'}
                  fontWeight={500}
                  textColor={
                    withModifiedAttributes.includes(key as keyof IngredientProps)
                      ? coloring
                      : undefined
                  }
                >
                  {iProps[key as keyof IngredientProps] + (value.unitSuffix || '')}
                </Heading>
              )}
            </Fragment>
          ))}
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
