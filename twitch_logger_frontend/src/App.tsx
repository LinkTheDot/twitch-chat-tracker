import { useState } from 'react'
import './App.css'
import { CategoryState } from './types/CategoryState';
import QueryForm, { QueryFormData } from './components/QueryForm';
import { UserResults } from './components/UserResults';
import { NameChangeResults } from './components/NameChangesResults';
import { SubscriptionResults } from './components/SubscriptionResults';
import { FollowingResults } from './components/FollowingResults';

export default function App() {
  const [currentCategory, setCurrentCategory] = useState<CategoryState>(CategoryState.Users);
  const [queryFormData, setQueryForm] = useState<QueryFormData>({
    category: CategoryState.Users,
    channelSearchQuery: "",
    userSearchQuery: "",
  });

  return (
    <>
      <QueryForm onSubmitQuery={setQueryForm} />


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
    </>
  )
}
