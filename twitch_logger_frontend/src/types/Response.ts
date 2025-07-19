export interface Response<T> {
  value: T | null,
  error: any | null,
  isLoading: boolean,
}
