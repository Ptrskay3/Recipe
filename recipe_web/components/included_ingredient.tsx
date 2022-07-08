import { EditIcon } from '@chakra-ui/icons';
import {
  Box,
  Center,
  useColorModeValue,
  Heading,
  Text,
  Stack,
  Button,
  Input,
  Editable,
  EditablePreview,
  EditableTextarea,
  EditableInput,
  useEditableControls,
  Tooltip,
} from '@chakra-ui/react';
import { useState } from 'react';

interface IncludedIngredientProps {
  name: string;
  quantity: string;
  quantity_unit: string;
}

export default function IncludedIngredient({
  name,
  quantity,
  quantity_unit,
}: IncludedIngredientProps) {
  return (
    <Center py={12}>
      <Box
        role={'group'}
        p={6}
        maxW={'330px'}
        w={'330px'}
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
          <Tooltip
            label="Click to edit"
            fontSize="md"
            placement="right-end"
            hasArrow
            bg={useColorModeValue('gray.800', 'orange.400')}
          >
            <Editable
              defaultValue={name}
              fontSize={'xl'}
              textTransform={'uppercase'}
              color={'orange.400'}
            >
              <EditablePreview />
              <EditableInput textAlign={'center'} />
            </Editable>
          </Tooltip>
          <Editable
            submitOnBlur={true}
            defaultValue={quantity}
            fontSize={'2xl'}
            textTransform={'uppercase'}
          >
            <EditablePreview />
            <EditableInput textAlign={'center'} />
          </Editable>
          <Heading fontSize={'2xl'} fontFamily={'body'} fontWeight={500}>
            {quantity_unit}
          </Heading>
        </Stack>
      </Box>
    </Center>
  );
}
