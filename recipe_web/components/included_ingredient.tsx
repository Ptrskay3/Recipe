import { DeleteIcon } from '@chakra-ui/icons';
import {
  Box,
  Center,
  Editable,
  EditableInput,
  EditablePreview,
  IconButton,
  Stack,
  Text,
  useColorModeValue,
} from '@chakra-ui/react';
import { useRouter } from 'next/router';
import { useState } from 'react';
import { intoFormBody } from '../utils/form';
import { EditableControls } from './editable_custom_controls';

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
  const [deleted, setDeleted] = useState(false);
  const router = useRouter();
  const { name: rName } = router.query;
  const deleteIngredient = async (rName: string, iName: string) => {
    const body = intoFormBody({ name: iName });
    const { ok } = await fetch(`${process.env.NEXT_PUBLIC_BASE_URL}/r/${rName}/ingredient`, {
      method: 'DELETE',
      credentials: 'include',
      body,
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
      },
    });

    if (ok) {
      setDeleted(true);
    }
  };
  const colorValue = useColorModeValue('white', 'gray.800');
  if (deleted) {
    return null;
  }

  return (
    <Center py={12}>
      <Box
        role={'group'}
        p={6}
        maxW={'330px'}
        w={'330px'}
        bg={colorValue}
        boxShadow={'2xl'}
        rounded={'lg'}
        pos={'relative'}
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
          <Text
            as={'a'}
            href={`/i/${name}`}
            fontSize={'xl'}
            textTransform={'uppercase'}
            color={'orange.400'}
          >
            {name}
          </Text>
          <Editable
            submitOnBlur={true}
            defaultValue={quantity}
            fontSize={'2xl'}
            textTransform={'uppercase'}
          >
            <EditablePreview />
            <EditableInput textAlign={'center'} />
            <EditableControls />
          </Editable>
          {!!quantity_unit.trim() ? (
            <Editable submitOnBlur={true} defaultValue={quantity_unit} fontSize={'2xl'}>
              <EditablePreview />
              <EditableInput textAlign={'center'} />
              <EditableControls />
            </Editable>
          ) : null}
          <IconButton
            aria-label="delete ingredient"
            size="xs"
            icon={<DeleteIcon />}
            onClick={() => deleteIngredient(rName as string, name)}
          />
        </Stack>
      </Box>
    </Center>
  );
}
