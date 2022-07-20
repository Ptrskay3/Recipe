import { Box, HStack, useRadio, useRadioGroup, Wrap, WrapItem } from '@chakra-ui/react';

function RadioCard(props: any) {
  const { getInputProps, getCheckboxProps } = useRadio(props);

  const input = getInputProps();
  const checkbox = getCheckboxProps();

  return (
    <Box as="label">
      <input {...input} />
      <Box
        {...checkbox}
        cursor="pointer"
        borderWidth="1px"
        borderRadius="md"
        boxShadow="md"
        _checked={{
          bg: 'orange.400',
          color: 'white',
          borderColor: 'orange.400',
        }}
        _focus={{
          boxShadow: 'outline',
        }}
        px={5}
        py={3}
      >
        {props.children}
      </Box>
    </Box>
  );
}

export function EnumSelector<T>({
  options,
  name,
  defaultValue,
  onChange,
}: {
  options: readonly T[];
  name: string;
  defaultValue: T;
  onChange?: (...args: any) => any;
}) {
  const { getRootProps, getRadioProps } = useRadioGroup({
    name,
    defaultValue: defaultValue as any,
    onChange,
  });

  const group = getRootProps();

  return (
    <HStack {...group}>
      <Wrap justify="center">
        {options.map((value) => {
          const radio = getRadioProps({ value } as any);
          return (
            <WrapItem key={value as any}>
              <RadioCard {...radio}>{toPretty(value as unknown as string)}</RadioCard>
            </WrapItem>
          );
        })}
      </Wrap>
    </HStack>
  );
}

function toPretty(s: string): string {
  return s.replaceAll('_', ' ');
}
