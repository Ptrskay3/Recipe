import { useRouter } from 'next/router';
import { FormControl, Text } from '@chakra-ui/react';
import { AsyncSelect, chakraComponents, GroupBase } from 'chakra-react-select';

const asyncComponents = {
  LoadingIndicator: (props: any) => (
    <chakraComponents.LoadingIndicator
      color="currentColor"
      emptyColor="transparent"
      spinnerSize="md"
      speed="0.45s"
      thickness="2px"
      {...props}
    />
  ),
  Option: ({ children, ...props }: any) => (
    <chakraComponents.Option {...props}>
      {props.data.name} {' | description: '} {props.data.description}
      {/* TODO: format description */}
    </chakraComponents.Option>
  ),
  DropdownIndicator: () => null,
  IndicatorSeparator: () => null,
};

interface RecipeOption {
  name: string;
  description: string;
}

export default function RecipeSearch() {
  const router = useRouter();
  return (
    <FormControl p={4}>
      <AsyncSelect<RecipeOption, false, GroupBase<RecipeOption>>
        name="recipes"
        placeholder="Search for a recipe"
        components={asyncComponents}
        isClearable={true}
        variant={'filled'}
        getOptionLabel={(option) => option.name}
        getOptionValue={(option) => option.name}
        loadOptions={(inputValue, callback) => {
          fetch('http://localhost:7700/indexes/recipes/search', {
            method: 'POST',
            headers: {
              'Content-Type': 'application/json',
            },
            body: JSON.stringify({ limit: 25, q: inputValue }),
          })
            .then((resp) => resp.json())
            .then((body) => callback(body.hits));
        }}
        onChange={(e) => {
          if (e !== null) {
            router.push(`/r/${e.name}`);
          }
        }}
      />
    </FormControl>
  );
}
