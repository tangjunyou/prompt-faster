/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        // 自定义颜色（来自 UX Design）
        primary: {
          50: '#f0f9ff',
          500: '#0ea5e9',
          600: '#0284c7',
          700: '#0369a1',
        },
        // 节点状态颜色
        node: {
          idle: '#9ca3af',     // 灰色 - 未开始
          running: '#3b82f6',  // 蓝色 - 运行中
          success: '#22c55e',  // 绿色 - 成功
          failed: '#ef4444',   // 红色 - 失败
          warning: '#eab308',  // 黄色 - 警告
        },
      },
      animation: {
        'pulse-slow': 'pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite',
        'flow': 'flow 2s linear infinite',
      },
      keyframes: {
        flow: {
          '0%': { strokeDashoffset: '100' },
          '100%': { strokeDashoffset: '0' },
        },
      },
    },
  },
  plugins: [],
}
