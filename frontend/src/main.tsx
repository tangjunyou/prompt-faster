import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { BrowserRouter } from 'react-router'
import { QueryClientProvider } from '@tanstack/react-query'
import './index.css'
import '@xyflow/react/dist/style.css'
import App from './App.tsx'
import { queryClient } from './lib/query-client'

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    {/* QueryClientProvider 统一管理请求缓存与重试策略 */}
    <QueryClientProvider client={queryClient}>
      {/* BrowserRouter 提供全局路由上下文 */}
      <BrowserRouter>
        <App />
      </BrowserRouter>
    </QueryClientProvider>
  </StrictMode>,
)
