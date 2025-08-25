import React, { useState } from 'react';
import { Pagination } from '../types/Pagination';

interface PaginationProps {
  pagination: Pagination;
  onPageChange: (paginationResponse: Pagination) => void;
  isLoading: boolean,
}

export function PageSelect({ pagination: paginationResponse, onPageChange, isLoading }: PaginationProps) {
  const [showInput, setShowInput] = useState<boolean>(false);
  const [inputValue, setInputValue] = useState<string>('');

  const { page: backendPage, totalPages } = paginationResponse;
  // Convert 0-based backend page to 1-based display page
  const currentPage = backendPage + 1;

  if (totalPages && totalPages <= 1) {
    return null;
  }

  const handlePageChange = (displayPage: number): void => {
    console.log(`handlePageChange: Is loading exists? ${isLoading !== null}`)
    console.log(`handlePageChange: Is loading? ${isLoading}`)

    if (isLoading) {
      return;
    }

    // Convert 1-based display page to 0-based backend page
    const backendPageNumber = displayPage - 1;
    if (backendPageNumber >= 0 && backendPageNumber < totalPages) {
      const updatedPagination: Pagination = {
        ...paginationResponse,
        page: backendPageNumber
      };
      onPageChange(updatedPagination);
    }
  };

  const handleInputSubmit = (): void => {
    console.log(`handleInputSubmit: Is loading exists? ${isLoading !== null}`)
    console.log(`handleInputSubmit: Is loading? ${isLoading}`)

    if (isLoading) {
      return;
    }

    const displayPage: number = parseInt(inputValue);
    // Convert 1-based display page to 0-based backend page
    const backendPageNumber = displayPage - 1;
    if (backendPageNumber >= 0 && backendPageNumber < totalPages) {
      const updatedPagination: Pagination = {
        ...paginationResponse,
        page: backendPageNumber
      };
      onPageChange(updatedPagination);
      setShowInput(false);
      setInputValue('');
    }
  };

  const handleDotsClick = (): void => {
    console.log(`handleDotsClick: Is loading exists? ${isLoading !== null}`)
    console.log(`handleDotsClick: Is loading? ${isLoading}`)

    if (isLoading) {
      return;
    }


    setShowInput(true);
  };

  return (
    <div className="flex items-center gap-1 bg-gray-900 p-2 rounded">
      <button
        onClick={() => handlePageChange(currentPage - 1)}
        disabled={currentPage === 1}
        className="flex items-center gap-1 px-3 py-2 text-gray-300 hover:text-white disabled:opacity-50 disabled:cursor-not-allowed"
      >
        ←
        Back
      </button>

      <button
        onClick={() => handlePageChange(1)}
        className={`px-3 py-2 rounded ${currentPage === 1
          ? 'bg-blue-600 text-white'
          : 'text-gray-300 hover:text-white hover:bg-gray-700'
          }`}
      >
        1
      </button>

      {currentPage > 2 && (
        <button
          onClick={() => handlePageChange(currentPage - 1)}
          className="px-3 py-2 text-gray-300 hover:text-white hover:bg-gray-700 rounded"
        >
          {currentPage - 1}
        </button>
      )}

      {currentPage > 1 && (
        <button
          className="px-3 py-2 bg-blue-600 text-white rounded"
        >
          {currentPage}
        </button>
      )}

      {currentPage < totalPages && (
        <button
          onClick={() => handlePageChange(currentPage + 1)}
          className="px-3 py-2 text-gray-300 hover:text-white hover:bg-gray-700 rounded"
        >
          {currentPage + 1}
        </button>
      )}

      {showInput ? (
        <div className="flex items-center gap-1">
          <input
            type="number"
            value={inputValue}
            onChange={(e: React.ChangeEvent<HTMLInputElement>) => setInputValue(e.target.value)}
            onKeyDown={(e: React.KeyboardEvent<HTMLInputElement>) => {
              if (e.key === 'Enter') {
                handleInputSubmit();
              }
            }}
            onBlur={() => {
              setTimeout(() => setShowInput(false), 150);
            }}
            className="w-16 px-2 py-1 bg-gray-800 text-white border border-gray-600 rounded text-sm"
            placeholder="Page"
            min="1"
            max={totalPages}
            autoFocus
          />
        </div>
      ) : (
        <button
          onClick={handleDotsClick}
          className="px-2 py-2 text-gray-300 hover:text-white hover:bg-gray-700 rounded"
        >
          ...
        </button>
      )}

      <button
        onClick={() => handlePageChange(totalPages)}
        className={`px-3 py-2 rounded ${currentPage === totalPages
          ? 'bg-blue-600 text-white'
          : 'text-gray-300 hover:text-white hover:bg-gray-700'
          }`}
      >
        {totalPages}
      </button>

      <button
        onClick={() => handlePageChange(currentPage + 1)}
        disabled={currentPage === totalPages}
        className="flex items-center gap-1 px-3 py-2 text-gray-300 hover:text-white disabled:opacity-50 disabled:cursor-not-allowed"
      >
        Next
        →
      </button>
    </div>
  );
}
