import { CheckIcon, CloseIcon, EditIcon } from '@chakra-ui/icons';
import { useEditableControls, ButtonGroup, IconButton, Flex } from '@chakra-ui/react';

export function EditableControls() {
  const { isEditing, getSubmitButtonProps, getCancelButtonProps, getEditButtonProps } =
    useEditableControls();

  return isEditing ? (
    <Flex justifyContent="center" mt="2">
      <ButtonGroup size="sm">
        <IconButton icon={<CheckIcon />} {...(getSubmitButtonProps() as any)} />
        <IconButton icon={<CloseIcon />} {...(getCancelButtonProps() as any)} />
      </ButtonGroup>
    </Flex>
  ) : (
    <Flex justifyContent="center">
      <IconButton
        size="xs"
        icon={<EditIcon />}
        {...(getEditButtonProps() as any)}
      />
    </Flex>
  );
}
