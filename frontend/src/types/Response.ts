import { Pagination } from "./Pagination";

export interface Response<T> {
  response_data: ResponseData<T> | null
  error: any | null;
}

export interface ResponseData<T> {
  data: T;
  pagination?: Pagination | null;
}
