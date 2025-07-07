import { useState } from 'react'
import './App.css'

export default function App() {
  const [count, setCount] = useState(0)

  const environment = process.env.NODE_ENV ?? "Unknown";
  const backendUrl = import.meta.env.VITE_BACKEND_URL ?? 'Unknown';

  function updateOnClick() {
    setCount(count + 1);
  }

  console.log('VITE_BACKEND_URL = ' + import.meta.env.VITE_BACKEND_URL);

  return (
    <>
      <p>
        Waow {count} in {environment} mode.
      </p>

      <p>
        Data from {backendUrl}
      </p>

      <button onClick={updateOnClick}> Click {count} </button>
    </>
  )
}
