import create from 'zustand';
import { combine } from 'zustand/middleware';
import { IngredientProps } from '../components/ingredient';

interface IngredientEditMode {
  editModeOpen: boolean;
  editedValues: Partial<IngredientProps>;
  setEditModeOpen: (value: boolean) => void;
  updateEditedValues: (value: Partial<Record<keyof IngredientProps, any>>) => void;
  resetEditedValues: () => void;
}

export const useIngredientEditMode = create<IngredientEditMode>(
  combine(
    {
      editModeOpen: false,
      editedValues: {},
    },
    (set) => ({
      setEditModeOpen: (value) => set(() => ({ editModeOpen: value })),
      updateEditedValues: (edited) =>
        set((state) => {
          const st = { ...state, editedValues: { ...state.editedValues, ...edited } };
          console.log(st.editedValues);
          return st;
        }),
      resetEditedValues: () => set((state) => ({ ...state, editedValues: {} })),
    })
  )
);
