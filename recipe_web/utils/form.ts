export const intoFormBody = (state: Record<string, string | number | boolean>): string => {
  return Object.entries(state)
    .map(([key, value]) => encodeURIComponent(key) + '=' + encodeURIComponent(value))
    .join('&');
};
