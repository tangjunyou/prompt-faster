import { ApiConfigPanel } from '@/features/api-config';

/**
 * API 配置页面
 */
export function ApiConfigPage() {
  return (
    <div className="min-h-screen flex items-center justify-center p-4 bg-background">
      <ApiConfigPanel />
    </div>
  );
}
