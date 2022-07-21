import { Center, Flex, FormLabel, Input } from '@chakra-ui/react';
import { useSearchBox } from 'react-instantsearch-hooks-web';

export function CustomSearchBox(props: any) {
  const { query, refine } = useSearchBox(props);
  const { passRef } = props;
  return (
    <>
      <FormLabel htmlFor={props.name}>{props.label}</FormLabel>
      <Input
        ref={passRef}
        maxW={'240px'}
        type="text"
        placeholder="Search"
        value={query}
        onChange={(e) => {
          refine(e.target.value);
        }}
        {...props}
      />
    </>
  );
}
