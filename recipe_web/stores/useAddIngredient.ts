import create from 'zustand';
import { combine } from 'zustand/middleware';

interface AddIngredient {
  addIngredientOpen: boolean;
  selected: string | undefined;
  setAddIngredientOpen: (value: boolean) => void;
  setSelected: (value: string | undefined) => void;
}

export const useAddIngredient = create<AddIngredient>(
  combine(
    {
      addIngredientOpen: false,
      selected: undefined as string | undefined,
    },
    (set) => ({
      setAddIngredientOpen: (value: boolean) =>
        set((state) => ({ ...state, addIngredientOpen: value })),
      setSelected: (value: string | undefined) => set((state) => ({ ...state, selected: value })),
    })
  )
);
