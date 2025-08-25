export interface PaginationOptions {
  page: number;
  pageSize: number | null;
}

export const createPaginationUrl = (paginationOptions: PaginationOptions): string => {
  let url = `&page=${paginationOptions.page}`;

  if (paginationOptions.pageSize !== null) {
    url += `&page_size=${paginationOptions.pageSize}`;
  }

  return url;
};
