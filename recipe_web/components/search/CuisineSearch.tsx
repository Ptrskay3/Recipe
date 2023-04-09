import { FormControl, FormLabel, FormErrorMessage } from '@chakra-ui/react';
import { AsyncSelect, chakraComponents, GroupBase } from 'chakra-react-select';
import { useField } from 'formik';
import { useAddRecipe } from '../../stores/useAddRecipe';

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
  DropdownIndicator: () => null,
  IndicatorSeparator: () => null,
};

interface Cuisine {
  name: string;
}

export default function CuisineSearch({ label, ...props }: any) {
  const [cuisine, setCuisine] = useAddRecipe((state) => [state.cuisine, state.setCuisine]);
  const [_field, meta] = useField(props);

  return (
    <FormControl id={props.name} isInvalid={meta.touched && cuisine === ''}>
      <FormLabel htmlFor={props.name}>{label}</FormLabel>
      <AsyncSelect<Cuisine, false, GroupBase<Cuisine>>
        name="cuisines"
        placeholder="Start searching"
        components={asyncComponents}
        isClearable={true}
        variant={'filled'}
        getOptionLabel={(option) => option.name}
        getOptionValue={(option) => option.name}
        loadOptions={(inputValue, callback) => {
          fetch('http://localhost:7700/indexes/cuisines/search', {
            method: 'POST',
            headers: {
              'Content-Type': 'application/json',
            },
            body: JSON.stringify({ limit: 5, q: inputValue }),
          })
            .then((resp) => resp.json())
            .then((body) => callback(body.hits));
        }}
        onChange={(e) => {
          if (!!e) {
            setCuisine(e.name);
          } else {
            setCuisine('');
          }
        }}
      />
      <FormErrorMessage>{meta.touched && meta.error ? meta.error : null}</FormErrorMessage>
    </FormControl>
  );
}
