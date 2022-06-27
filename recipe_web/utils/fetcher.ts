// @ts-nocheck
export const fetcher = (...args) =>
  fetch(...args, { credentials: 'include' }).then((res) => res.json());

export const fetcherWithoutJson = (...args) => fetch(...args, { credentials: 'include' });
