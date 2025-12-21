/**
 * 用户测试数据工厂
 * 
 * 自动创建和清理测试用户
 */

const API_URL = process.env.API_URL || 'http://localhost:3000/api/v1';

export interface TestUser {
  id: string;
  username: string;
  email: string;
  password: string;
}

export class UserFactory {
  private createdUsers: string[] = [];

  /**
   * 创建测试用户
   */
  async createUser(overrides: Partial<TestUser> = {}): Promise<TestUser> {
    const timestamp = Date.now();
    const user = {
      username: `test_user_${timestamp}`,
      email: `test_${timestamp}@example.com`,
      password: `TestPass123!`,
      ...overrides,
    };

    try {
      const response = await fetch(`${API_URL}/users`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(user),
      });

      if (!response.ok) {
        throw new Error(`创建用户失败: ${response.statusText}`);
      }

      const created = await response.json();
      this.createdUsers.push(created.data?.id || created.id);
      
      return {
        ...user,
        id: created.data?.id || created.id,
      };
    } catch (error) {
      // 如果 API 尚未实现，返回模拟数据
      console.warn('用户 API 尚未实现，使用模拟数据');
      const mockUser = {
        id: `mock_${timestamp}`,
        ...user,
      };
      this.createdUsers.push(mockUser.id);
      return mockUser;
    }
  }

  /**
   * 清理所有创建的用户
   */
  async cleanup(): Promise<void> {
    for (const userId of this.createdUsers) {
      try {
        await fetch(`${API_URL}/users/${userId}`, {
          method: 'DELETE',
        });
      } catch {
        // 忽略清理错误
      }
    }
    this.createdUsers = [];
  }
}
