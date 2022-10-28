export function prettyNotification(ty: string): string {
  switch (ty) {
    case 'NewRecipe':
      return 'New recipe';
    default:
      return 'Unknown';
  }
}
