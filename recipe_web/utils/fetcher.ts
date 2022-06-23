// @ts-nocheck
export const fetcher = (...args) =>
  fetch(...args, { credentials: 'include' }).then((res) => res.json());
