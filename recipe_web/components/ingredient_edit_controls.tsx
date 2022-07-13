import { HamburgerIcon, EditIcon, DeleteIcon, CheckIcon, CloseIcon } from '@chakra-ui/icons';
import { IconButton, Menu, MenuButton, MenuItem, MenuList, useToast } from '@chakra-ui/react';
import { useRouter } from 'next/router';
import { useIngredientEditMode } from '../stores/useIngredientEditMode';
import { diffObjects } from '../utils/diff';
import { IngredientProps } from './ingredient';

export const IngredientEditControls = ({
  name,
  originals,
}: {
  name: string;
  originals: IngredientProps;
}) => {
  const router = useRouter();
  const editModeOpen = useIngredientEditMode((state) => state.editModeOpen);
  const setEditModeOpen = useIngredientEditMode((state) => state.setEditModeOpen);
  const editedValues = useIngredientEditMode((state) => state.editedValues);
  const data = { is_delete_vote: false, update_ingredient: editedValues };
  const toast = useToast();

  const handleSubmitSuggestion = async (data: any) => {
    if (diffObjects(originals, editedValues).length === 0) {
      toast({
        title: 'There is nothing to update',
        status: 'warning',
        duration: 9000,
        isClosable: true,
      });
      return;
    }
    const { ok, status } = await fetch(`http://localhost:3000/i/${name}/suggestion`, {
      method: 'POST',
      credentials: 'include',
      body: JSON.stringify(data),
      headers: {
        'Content-Type': 'application/json',
      },
    });
    if (ok) {
      toast({
        title: 'Suggestion submitted',
        status: 'success',
        duration: 9000,
        isClosable: true,
      });

      // TODO: if we conflict, should we update the row?
    } else if (status === 409) {
      toast({
        title: 'You have already submitted a suggestion',
        status: 'error',
        duration: 9000,
        isClosable: true,
      });
    }
  };

  return (
    <Menu>
      <MenuButton
        as={IconButton}
        aria-label="Suggestion"
        icon={<HamburgerIcon />}
        variant="outline"
      />
      <MenuList>
        <MenuItem
          onClick={() => setEditModeOpen(!editModeOpen)}
          icon={editModeOpen ? <CloseIcon /> : <EditIcon />}
        >
          {editModeOpen ? 'Cancel edit' : 'Suggest edit'}
        </MenuItem>
        {editModeOpen ? (
          <MenuItem onClick={() => handleSubmitSuggestion(data)} icon={<CheckIcon />}>
            Submit edit
          </MenuItem>
        ) : null}
        <MenuItem icon={<DeleteIcon />}>Suggest delete</MenuItem>
      </MenuList>
    </Menu>
  );
};
