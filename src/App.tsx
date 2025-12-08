import { useState } from 'react'
import './App.css'
import { useTheme } from './hooks/useTheme'
import Home from './pages/Home'
import Settings from './pages/Settings'

type Page = 'home' | 'settings'

function App() {
  const [currentPage, setCurrentPage] = useState<Page>('home')

  // Initialize theme on app load
  useTheme()

  return (
    <>
      {currentPage === 'home' && <Home onOpenSettings={() => setCurrentPage('settings')} />}
      {currentPage === 'settings' && <Settings onBack={() => setCurrentPage('home')} />}
    </>
  )
}

export default App
