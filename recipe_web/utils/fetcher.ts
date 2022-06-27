// @ts-nocheck
export const fetcher = (...args) =>
  fetch(...args, { credentials: 'include' }).then((res) => res.json());

export const fetcherOk = (...args) =>
  fetch(...args, { credentials: 'include' }).then((res) => res.status === 200);
