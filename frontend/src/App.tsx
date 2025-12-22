import { Routes, Route } from 'react-router'
import './App.css'
import { HomePage, ApiConfigPage } from './pages'

function App() {
  return (
    <Routes>
      <Route path="/" element={<HomePage />} />
      <Route path="/settings/api" element={<ApiConfigPage />} />
    </Routes>
  )
}

export default App
