import { useState } from 'react'
import './App.css'
import { CategoryState } from './types/CategoryState';
import QueryForm, { QueryFormData } from './components/QueryForm';
import { UserResults } from './components/UserResults';
import { NameChangeResults } from './components/NameChangesResults';
import { SubscriptionResults } from './components/SubscriptionResults';
import { FollowingResults } from './components/FollowingResults';
import { ReturnToTopButton } from './components/ReturnToTopButton';

export default function App() {
  const [currentCategory, setCurrentCategory] = useState<CategoryState>(CategoryState.Users);
  const [queryFormData, setQueryForm] = useState<QueryFormData>({
    category: CategoryState.Users,
    channelSearchQuery: "",
    userSearchQuery: "",
  });

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
          <QueryForm onSubmitQuery={setQueryForm} />
        </div>

        <div>
          <ReturnToTopButton />
        </div>

        <main className="mx-auto">
          {queryFormData.category == CategoryState.Users && (
            <UserResults queryResults={queryFormData} />
          )}

          {queryFormData.category == CategoryState.NameChanges && (
            <NameChangeResults queryResults={queryFormData} />
          )}

          {queryFormData.category == CategoryState.Subscriptions && (
            <SubscriptionResults queryResults={queryFormData} />
          )}

          {queryFormData.category == CategoryState.Following && (
            <FollowingResults queryResults={queryFormData} />
          )}
        </main>
      </div>
    </div>
  )
}
