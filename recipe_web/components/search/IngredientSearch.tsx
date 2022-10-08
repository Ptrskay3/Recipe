import { FormControl, FormLabel, Container } from '@chakra-ui/react';
import { AsyncSelect, chakraComponents, GroupBase } from 'chakra-react-select';
import { useAddIngredient } from '../../stores/useAddIngredient';

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
      {props.data.name} {' | calories: '} {props.data.calories_per_100g}
    </chakraComponents.Option>
  ),
  DropdownIndicator: () => null,
  IndicatorSeparator: () => null,
};

interface IngredientOption {
  name: string;
}

export default function IngredientSearch() {
  const setSelected = useAddIngredient((state) => state.setSelected);
  return (
    <Container mb={16}>
      <FormControl p={4}>
        <FormLabel>Select an ingredient</FormLabel>
        <AsyncSelect<IngredientOption, false, GroupBase<IngredientOption>>
          name="ingredients"
          placeholder="Start typing"
          components={asyncComponents}
          isClearable={true}
          variant={'filled'}
          getOptionLabel={(option) => option.name}
          getOptionValue={(option) => option.name}
          loadOptions={(inputValue, callback) => {
            fetch('http://localhost:7700/indexes/ingredients/search', {
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
            setSelected(e?.name);
          }}
        />
      </FormControl>
    </Container>
  );
}
