import { DeleteIcon, PlusSquareIcon } from '@chakra-ui/icons';
import {
  Box,
  Button,
  Center,
  Flex,
  FormControl,
  FormErrorMessage,
  HStack,
  IconButton,
  ListItem,
  OrderedList,
  Textarea,
} from '@chakra-ui/react';
import { useState } from 'react';

export const Listable = ({ state, pushState, removeStateByIndex, removeState }: any) => {
  // TODO: Add variant attribute, to be usable with ingredient and steps as well.
  const [inputFieldOpen, setInputFieldOpen] = useState(false);
  const [content, setContent] = useState('');
  const removeFunction =
    (removeStateByIndex && ((i: number, _: any) => removeStateByIndex(i))) ||
    (removeState && ((_: number, v: any) => removeState(v))) ||
    (() => {});

  return (
    <>
      <OrderedList>
        {state &&
          state.map((item: any, i: number) => (
            <HStack key={i} mb="2">
              <ListItem>{item}</ListItem>
              <IconButton
                size={'xs'}
                aria-label={'delete'}
                icon={<DeleteIcon />}
                onClick={() => removeFunction(i, item)}
              ></IconButton>
            </HStack>
          ))}
      </OrderedList>
      {inputFieldOpen ? (
        <FormControl isInvalid={content.trim().length === 0}>
          <Textarea
            value={content}
            onChange={(e) => {
              setContent(e.target.value);
            }}
          ></Textarea>
          <FormErrorMessage>{'This field cannot be empty!'}</FormErrorMessage>
          <Box>
            <Center>
              <Button
                m="1"
                aria-label={'add'}
                onClick={() => {
                  if (content.trim().length === 0) {
                    return;
                  }
                  setContent('');
                  pushState(content);
                }}
              >
                Add step
              </Button>
              <Button aria-label={'cancel'} onClick={() => setInputFieldOpen(false)}>
                Cancel
              </Button>
            </Center>
          </Box>
        </FormControl>
      ) : (
        <Center>
          <IconButton
            aria-label={'add'}
            icon={<PlusSquareIcon />}
            onClick={() => {
              setInputFieldOpen(true);
            }}
          ></IconButton>
        </Center>
      )}
    </>
  );
};
