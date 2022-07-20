import { FormControl, FormErrorMessage, FormLabel, Input } from '@chakra-ui/react';
import { useField } from 'formik';
import React from 'react';

export const InputField: React.FC<
  React.DetailedHTMLProps<React.InputHTMLAttributes<HTMLInputElement>, HTMLInputElement> & {
    name: string;
    errors?: Record<string, string>;
    label?: string;
    textarea?: boolean;
    altErrorMsg?: string;
    rows?: number;
    size?: 'lg' | 'sm' | 'md' | 'xs' | undefined;
    onChange?: (...args: any) => any;
  }
> = ({ label, errors, ref: _, className, onChange, ...props }) => {
  const [field, meta] = useField(props);
  return (
    <FormControl id={props.name} isInvalid={!!meta.error}>
      <FormLabel htmlFor={props.name}>{label}</FormLabel>
      <Input {...field} {...props} onBlur={onChange} />
      <FormErrorMessage>{meta.touched && meta.error ? meta.error : null}</FormErrorMessage>
    </FormControl>
  );
};
