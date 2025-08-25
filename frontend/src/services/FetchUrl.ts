import { Pagination } from "../types/Pagination"

export interface FetchUrlProps {
  route: string;
  dataName: string;
  data: string;
  pagination: Pagination | null;
  channel?: string | null;
  additional?: string | null;
}

export const buildFetchUrl = (props: FetchUrlProps): string => {
  const route = props.channel ? `/${props.channel}${props.route}` : props.route
  const fetchUrl = `${import.meta.env.VITE_BACKEND_URL}${route}?${props.dataName}=${props.data}`

  const page = props.pagination ? `&page=${props.pagination.page}` : "";
  const additional_data = props.additional ? `&${props.additional}` : "";

  return `${fetchUrl}${page}${additional_data}`;
}
