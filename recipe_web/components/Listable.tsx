import { DeleteIcon, PlusSquareIcon } from '@chakra-ui/icons';
import {
  Button,
  Flex,
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
        <>
          <Textarea
            value={content}
            onChange={(e) => {
              setContent(e.target.value);
            }}
          ></Textarea>
          <IconButton
            aria-label={'add'}
            icon={<PlusSquareIcon />}
            onClick={() => {
              // TODO: Do not permit empty strings
              setContent('');
              pushState(content);
            }}
          ></IconButton>
        </>
      ) : (
        <Button
          onClick={() => {
            setInputFieldOpen(true);
          }}
        >
          Add step
        </Button>
      )}
    </>
  );
};
