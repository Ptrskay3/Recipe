export const diffObjects = (
  original: Record<string, any>,
  suggested: Record<string, any>
): any[] => {
  if (!original || !suggested) {
    return [];
  }
  const originalKeys = Object.keys(original);
  return Object.entries(suggested)
    .map(([key, value]) => {
      if (originalKeys.includes(key)) {
        if (Array.isArray(value) && !arraysEqual(original[key], value)) {
          return key;
        } else if (!Array.isArray(value) && original[key] !== value) {
          return key;
        }
      }
    })
    .filter(Boolean);
};

function arraysEqual(a: any[], b: any[]): boolean {
  a = Array.isArray(a) ? a : [];
  b = Array.isArray(b) ? b : [];
  return a.length === b.length && a.every((el, ix) => el === b[ix]);
}
