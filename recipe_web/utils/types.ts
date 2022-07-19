export const mealTypes = ['breakfast', 'lunch', 'dinner', 'other'] as const;

export type MealType = typeof mealTypes[number];

export const difficultyLevels = [
  'easy',
  'moderate',
  'medium',
  'challenging',
  'hard',
  'extreme',
  'do_not_attempt',
] as const;

export type DifficultyLevel = typeof difficultyLevels[number];
