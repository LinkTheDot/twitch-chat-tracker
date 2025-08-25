import { useState, useEffect } from "react";
import { Response, ResponseData } from "../types/Response";
import { Pagination } from "../types/Pagination";

export interface GetDataProps {
  requestUrl: string;
  updatePagination: (paginationResponse: Pagination | null) => void;
}

export const useGetData = <T,>({ requestUrl, updatePagination }: GetDataProps): Response<T> => {
  const [response_data, setResponseData] = useState<ResponseData<T> | null>(null);
  const [error, setError] = useState<any | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(true);

  useEffect(() => {
    const fetchData = async () => {
      try {
        const response = await fetch(requestUrl);
        if (!response.ok) {
          throw new Error(`HTTP error! ${await response.text()}`);
        } else {
          setError(null)
        }

        const jsonResponse = await response.json();

        if ('data' in jsonResponse) {
          updatePagination(jsonResponse.pagination);
          setResponseData(jsonResponse);
        } else {
          updatePagination(null);
          setResponseData({ data: jsonResponse, pagination: null });
        }
      } catch (err: any) {
        setError(err);
      } finally {
        setIsLoading(false);
      }
    };

    fetchData();
  }, [requestUrl]);

  return {
    response_data,
    error,
    isLoading,
  };
};
