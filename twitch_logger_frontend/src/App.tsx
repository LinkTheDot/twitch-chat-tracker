import { useState } from 'react'
import './App.css'
import { CategoryState } from './types/CategoryState';
import QueryForm, { QueryFormData } from './components/QueryForm';
import { UserResults } from './components/UserResults';

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


      <UserResults queryResults={queryFormData} />
    </>
  )
}
