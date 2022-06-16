export const AXUM_SERVER = process.env.NEXT_PUBLIC_API_URL;

export const commonGetOptions: Partial<RequestInit> = {
  credentials: 'include',
  method: 'GET',
};
