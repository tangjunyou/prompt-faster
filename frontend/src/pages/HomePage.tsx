import { useState } from 'react'
import { Link } from 'react-router'
import reactLogo from '../assets/react.svg'
import viteLogo from '/vite.svg'
import { HealthCheck } from '../components/HealthCheck'

/**
 * 首页组件
 */
export function HomePage() {
  const [count, setCount] = useState(0)

  return (
    <>
      <div>
        <a href="https://vite.dev" target="_blank">
          <img src={viteLogo} className="logo" alt="Vite logo" />
        </a>
        <a href="https://react.dev" target="_blank">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
      </div>
      <h1>Prompt Faster</h1>
      <HealthCheck />
      <div className="card">
        <button onClick={() => setCount((count) => count + 1)}>
          count is {count}
        </button>
        <p>
          Edit <code>src/pages/HomePage.tsx</code> and save to test HMR
        </p>
      </div>
      <p className="read-the-docs">
        Click on the Vite and React logos to learn more
      </p>
      <div className="card">
        <Link to="/settings/api" className="text-primary hover:underline">
          ⚙️ API 配置
        </Link>
      </div>
    </>
  )
}
