import create from 'zustand';
import { combine } from 'zustand/middleware';

export const useAddIngredient = create(
  combine(
    {
      addIngredientOpen: false,
    },
    (set) => ({
      setAddIngredientOpen: (value: boolean) => set(() => ({ addIngredientOpen: value })),
    })
  )
);
