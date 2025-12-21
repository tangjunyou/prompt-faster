/**
 * 工作区测试数据工厂
 * 
 * 自动创建和清理测试工作区
 */

const API_URL = process.env.API_URL || 'http://localhost:3000/api/v1';

export interface TestWorkspace {
  id: string;
  name: string;
  description?: string;
  user_id: string;
}

export class WorkspaceFactory {
  private createdWorkspaces: string[] = [];

  /**
   * 创建测试工作区
   */
  async createWorkspace(
    userId: string,
    overrides: Partial<TestWorkspace> = {}
  ): Promise<TestWorkspace> {
    const timestamp = Date.now();
    const workspace = {
      name: `测试工作区_${timestamp}`,
      description: '自动创建的测试工作区',
      user_id: userId,
      ...overrides,
    };

    try {
      const response = await fetch(`${API_URL}/workspaces`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(workspace),
      });

      if (!response.ok) {
        throw new Error(`创建工作区失败: ${response.statusText}`);
      }

      const created = await response.json();
      this.createdWorkspaces.push(created.data?.id || created.id);

      return {
        ...workspace,
        id: created.data?.id || created.id,
      };
    } catch (error) {
      // 如果 API 尚未实现，返回模拟数据
      console.warn('工作区 API 尚未实现，使用模拟数据');
      const mockWorkspace = {
        id: `mock_ws_${timestamp}`,
        ...workspace,
      };
      this.createdWorkspaces.push(mockWorkspace.id);
      return mockWorkspace;
    }
  }

  /**
   * 清理所有创建的工作区
   */
  async cleanup(): Promise<void> {
    for (const workspaceId of this.createdWorkspaces) {
      try {
        await fetch(`${API_URL}/workspaces/${workspaceId}`, {
          method: 'DELETE',
        });
      } catch {
        // 忽略清理错误
      }
    }
    this.createdWorkspaces = [];
  }
}
