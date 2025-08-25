import { useState } from 'react'
import './App.css'
import { CategoryState } from './types/CategoryState';
import QueryForm from './components/QueryForm';
import { UserResults } from './components/UserResults';
import { NameChangeResults } from './components/NameChangesResults';
import { SubscriptionResults } from './components/SubscriptionResults';
import { FollowingResults } from './components/FollowingResults';
import { ReturnToTopButton } from './components/ReturnToTopButton';
import { PageSelect } from './components/PageSelect';
import { QueryFormData } from './types/QueryFormData';
import { Pagination } from './types/Pagination';
import { MessageResults } from './components/UserMessageResults';

export default function App() {
  const [queryFormData, setQueryForm] = useState<QueryFormData>({
    category: CategoryState.Users,
    channelSearchQuery: "",
    userSearchQuery: "",
  });
  const [pagination, setPagination] = useState<Pagination | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(false);

  const updatePagination = (paginationChange: Pagination | null): void => {
    console.log(`Updating pagination: totalItems=${paginationChange?.totalItems} totalPages=${paginationChange?.totalPages} page=${paginationChange?.page} totalSize=${paginationChange?.totalSize}`);

    setPagination(paginationChange);
  };

  const onQueryFormSubmit = (data: QueryFormData): void => {
    setPagination(null);
    setQueryForm(data);
  };


  return (
    <div className="min-h-screen bg-gray-950 text-gray-100">
      <div className="container mx-auto px-4 py-8 lg:px-8 xl:px-12 2xl:px-16">
        <header className="mb-12 text-center">
          <h1 className="text-4xl md:text-5xl font-bold mb-2 bg-gradient-to-r from-purple-400 to-pink-400 bg-clip-text text-transparent">
            Twitch Chat Tracker
          </h1>
          <p className="text-gray-400 text-lg">Search and analyze Twitch user data</p>
        </header>

        <div className="mb-8">
          <QueryForm onSubmitQuery={onQueryFormSubmit} />
        </div>

        {pagination && pagination.totalItems > 0 && (queryFormData.userSearchQuery || queryFormData.channelSearchQuery) &&
          <div className="flex justify-end items-center mb-8 gap-4">
            <PageSelect
              pagination={pagination}
              onPageChange={updatePagination}
              isLoading={isLoading}
            />
          </div>
        }

        <main className="mx-auto">
          {queryFormData.category == CategoryState.Users && (
            <UserResults
              queryResults={queryFormData}
              pagination={pagination}
              updatePagination={updatePagination}
            />
          )}

          {queryFormData.category == CategoryState.NameChanges && (
            <NameChangeResults
              queryResults={queryFormData}
              pagination={pagination}
              updatePagination={updatePagination}
            />
          )}

          {queryFormData.category == CategoryState.Subscriptions && (
            <SubscriptionResults
              queryResults={queryFormData}
              pagination={pagination}
              updatePagination={updatePagination}
            />
          )}

          {queryFormData.category == CategoryState.Following && (
            <FollowingResults
              queryResults={queryFormData}
              pagination={pagination}
              updatePagination={updatePagination}
            />
          )}

          {queryFormData.category == CategoryState.Messages && (
            <MessageResults
              queryResults={queryFormData}
              pagination={pagination}
              updatePagination={updatePagination}
              setIsLoading={setIsLoading}
            />
          )}
        </main>

        <div>
          <ReturnToTopButton />
        </div>
      </div>
    </div>
  )
}
