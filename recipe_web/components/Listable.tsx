import { DeleteIcon } from '@chakra-ui/icons';
import { Button, HStack, IconButton, ListItem, OrderedList } from '@chakra-ui/react';

export const Listable = ({ state, pushState, removeStateByIndex, removeState }: any) => {
  const removeFunction =
    (removeStateByIndex && ((i: number, _: any) => removeStateByIndex(i))) ||
    (removeState && ((_: number, v: any) => removeState(v))) ||
    (() => {});

  return (
    <>
      <OrderedList>
        {state &&
          state.map((item: any, i: number) => (
            <HStack key={i}>
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
      <Button onClick={() => pushState((Math.random() + 1).toString(36).substring(7))}>
        Add a random string
      </Button>
    </>
  );
};
