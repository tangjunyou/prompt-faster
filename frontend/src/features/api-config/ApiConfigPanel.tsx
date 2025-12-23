import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { DifyCredentialForm } from './DifyCredentialForm';
import { GenericLlmCredentialForm } from './GenericLlmCredentialForm';

/**
 * API 配置面板组件
 * 作为 api-config feature 的对外入口
 */
export function ApiConfigPanel() {
  return (
    <Card className="w-full max-w-md">
      <CardHeader>
        <CardTitle>API 配置</CardTitle>
        <CardDescription>
          配置用于优化测试的 API 凭证
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-8">
        <DifyCredentialForm />
        <div className="border-t pt-6">
          <GenericLlmCredentialForm />
        </div>
      </CardContent>
    </Card>
  );
}
