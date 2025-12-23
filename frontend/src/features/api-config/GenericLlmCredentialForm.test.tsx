import { describe, it, expect, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { GenericLlmCredentialForm } from './GenericLlmCredentialForm';
import { useCredentialStore } from '@/stores/useCredentialStore';

/**
 * GenericLlmCredentialForm 组件测试
 * 覆盖组件渲染、用户交互和状态显示
 */
describe('GenericLlmCredentialForm 组件', () => {
  // 每个测试前重置 Store
  beforeEach(() => {
    useCredentialStore.getState().clearGenericLlmCredential();
  });

  describe('初始渲染', () => {
    it('应正确渲染标题和 Provider 选择按钮', () => {
      render(<GenericLlmCredentialForm />);
      
      expect(screen.getByText('通用大模型凭证')).toBeInTheDocument();
      expect(screen.getByRole('button', { name: '硅基流动' })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: '魔搭社区' })).toBeInTheDocument();
    });

    it('初始状态应显示"未配置"徽章', () => {
      render(<GenericLlmCredentialForm />);
      
      expect(screen.getByText('未配置')).toBeInTheDocument();
    });

    it('Provider 未选择时不应显示表单字段', () => {
      render(<GenericLlmCredentialForm />);
      
      expect(screen.queryByLabelText('API 地址')).not.toBeInTheDocument();
      expect(screen.queryByLabelText('API Key')).not.toBeInTheDocument();
    });
  });

  describe('Provider 选择交互', () => {
    it('点击硅基流动后应显示表单字段', async () => {
      const user = userEvent.setup();
      render(<GenericLlmCredentialForm />);
      
      await user.click(screen.getByRole('button', { name: '硅基流动' }));
      
      expect(screen.getByLabelText('API 地址')).toBeInTheDocument();
      expect(screen.getByLabelText('API Key')).toBeInTheDocument();
      expect(screen.getByRole('button', { name: '保存' })).toBeInTheDocument();
    });

    it('选择 Provider 后徽章应变为"待填写"', async () => {
      const user = userEvent.setup();
      render(<GenericLlmCredentialForm />);
      
      await user.click(screen.getByRole('button', { name: '魔搭社区' }));
      
      expect(screen.getByText('待填写')).toBeInTheDocument();
    });

    it('切换 Provider 应清空已填写的字段', async () => {
      const user = userEvent.setup();
      render(<GenericLlmCredentialForm />);
      
      // 选择硅基流动并填写
      await user.click(screen.getByRole('button', { name: '硅基流动' }));
      await user.type(screen.getByLabelText('API 地址'), 'https://api.siliconflow.cn');
      await user.type(screen.getByLabelText('API Key'), 'sk-test');
      
      // 切换到魔搭社区
      await user.click(screen.getByRole('button', { name: '魔搭社区' }));
      
      // 验证字段已清空
      expect(screen.getByLabelText('API 地址')).toHaveValue('');
      expect(screen.getByLabelText('API Key')).toHaveValue('');
    });
  });

  describe('表单验证', () => {
    it('空字段失焦时应显示错误提示', async () => {
      const user = userEvent.setup();
      render(<GenericLlmCredentialForm />);
      
      await user.click(screen.getByRole('button', { name: '硅基流动' }));
      
      // 聚焦再失焦 baseUrl
      const baseUrlInput = screen.getByLabelText('API 地址');
      await user.click(baseUrlInput);
      await user.tab();
      
      expect(screen.getByText('API 地址不能为空')).toBeInTheDocument();
    });

    it('无效 URL 应显示格式错误', async () => {
      const user = userEvent.setup();
      render(<GenericLlmCredentialForm />);
      
      await user.click(screen.getByRole('button', { name: '硅基流动' }));
      await user.type(screen.getByLabelText('API 地址'), 'invalid-url');
      await user.tab();
      
      expect(screen.getByText('请输入有效的 HTTP/HTTPS 地址')).toBeInTheDocument();
    });

    it('提交无效表单应显示顶部错误反馈', async () => {
      const user = userEvent.setup();
      render(<GenericLlmCredentialForm />);
      
      await user.click(screen.getByRole('button', { name: '硅基流动' }));
      await user.type(screen.getByLabelText('API 地址'), 'invalid');
      await user.click(screen.getByRole('button', { name: '保存' }));
      
      // 等待异步操作完成后检查反馈消息
      await waitFor(() => {
        const feedbackMessage = screen.getByText('请修正下方标红字段后重试');
        expect(feedbackMessage).toBeInTheDocument();
        expect(feedbackMessage.closest('[aria-live="assertive"]')).toBeInTheDocument();
      });
    });
  });

  describe('成功提交', () => {
    it('有效凭证提交后应显示成功反馈', async () => {
      const user = userEvent.setup();
      render(<GenericLlmCredentialForm />);
      
      await user.click(screen.getByRole('button', { name: '硅基流动' }));
      await user.type(screen.getByLabelText('API 地址'), 'https://api.siliconflow.cn');
      await user.type(screen.getByLabelText('API Key'), 'sk-test-key');
      await user.click(screen.getByRole('button', { name: '保存' }));
      
      await waitFor(() => {
        expect(screen.getByRole('status')).toHaveTextContent('已保存，待测试连接');
      });
    });

    it('提交后徽章应变为"已填写，待测试"', async () => {
      const user = userEvent.setup();
      render(<GenericLlmCredentialForm />);
      
      await user.click(screen.getByRole('button', { name: '硅基流动' }));
      await user.type(screen.getByLabelText('API 地址'), 'https://api.siliconflow.cn');
      await user.type(screen.getByLabelText('API Key'), 'sk-test-key');
      await user.click(screen.getByRole('button', { name: '保存' }));
      
      await waitFor(() => {
        expect(screen.getByText('已填写，待测试')).toBeInTheDocument();
      });
    });

    it('清空字段后提交应显示清空成功反馈', async () => {
      const user = userEvent.setup();
      render(<GenericLlmCredentialForm />);
      
      // 先填写并保存
      await user.click(screen.getByRole('button', { name: '硅基流动' }));
      await user.type(screen.getByLabelText('API 地址'), 'https://api.siliconflow.cn');
      await user.type(screen.getByLabelText('API Key'), 'sk-test-key');
      await user.click(screen.getByRole('button', { name: '保存' }));
      
      // 等待提交完成（按钮恢复为"保存"）
      await waitFor(() => {
        expect(screen.getByRole('button', { name: '保存' })).toBeInTheDocument();
      });
      
      // 清空字段
      await user.clear(screen.getByLabelText('API 地址'));
      await user.clear(screen.getByLabelText('API Key'));
      await user.click(screen.getByRole('button', { name: '保存' }));
      
      await waitFor(() => {
        expect(screen.getByRole('status')).toHaveTextContent('凭证已清空');
      });
    });
  });

  describe('Store 同步', () => {
    it('输入时 Store 应实时更新', async () => {
      // 重置 Store 以避免测试间状态污染
      useCredentialStore.getState().clearGenericLlmCredential();
      
      const user = userEvent.setup();
      render(<GenericLlmCredentialForm />);
      
      await user.click(screen.getByRole('button', { name: '硅基流动' }));
      await user.type(screen.getByLabelText('API 地址'), 'https://test.com');
      
      const store = useCredentialStore.getState();
      expect(store.genericLlm.baseUrl).toBe('https://test.com');
      expect(store.genericLlm.provider).toBe('siliconflow');
    });
  });

  describe('无障碍性', () => {
    it('错误字段应有正确的 aria-invalid 属性', async () => {
      const user = userEvent.setup();
      render(<GenericLlmCredentialForm />);
      
      await user.click(screen.getByRole('button', { name: '硅基流动' }));
      await user.click(screen.getByLabelText('API 地址'));
      await user.tab();
      
      expect(screen.getByLabelText('API 地址')).toHaveAttribute('aria-invalid', 'true');
    });

    it('错误消息应关联到对应输入框', async () => {
      const user = userEvent.setup();
      render(<GenericLlmCredentialForm />);
      
      await user.click(screen.getByRole('button', { name: '硅基流动' }));
      await user.click(screen.getByLabelText('API 地址'));
      await user.tab();
      
      const input = screen.getByLabelText('API 地址');
      const errorId = input.getAttribute('aria-describedby');
      expect(errorId).toBe('generic-llm-base-url-error');
      expect(document.getElementById(errorId!)).toHaveTextContent('API 地址不能为空');
    });
  });
});
